use tauri::AppHandle;
use tracing::warn;

use crate::native_panel_core::PanelInteractionCommand;
use crate::native_panel_renderer::{
    NativePanelPlatformEventHandler, dispatch_native_panel_platform_event,
    native_panel_platform_event_for_interaction_command,
};

use super::panel_host_runtime::rerender_runtime_panel_from_last_snapshot;
use super::panel_window_control::{
    refresh_native_panel_from_last_snapshot, reposition_native_panel_to_selected_display,
};

pub(super) fn handle_native_click_command<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    click_command: PanelInteractionCommand,
) {
    let Some(event) = native_panel_platform_event_for_interaction_command(&click_command) else {
        return;
    };
    let mut handler = MacosNativePanelPlatformEventHandler { app };
    let _ = dispatch_native_panel_platform_event(&mut handler, event);
}

struct MacosNativePanelPlatformEventHandler<R: tauri::Runtime + 'static> {
    app: AppHandle<R>,
}

impl<R: tauri::Runtime + 'static> NativePanelPlatformEventHandler
    for MacosNativePanelPlatformEventHandler<R>
{
    type Error = ();

    fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error> {
        spawn_native_focus_session(self.app.clone(), session_id);
        Ok(())
    }

    fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn quit_application(&mut self) -> Result<(), Self::Error> {
        self.app.exit(0);
        Ok(())
    }

    fn cycle_display(&mut self) -> Result<(), Self::Error> {
        spawn_native_cycle_display(self.app.clone());
        Ok(())
    }

    fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
        spawn_native_toggle_completion_sound(self.app.clone());
        Ok(())
    }

    fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
        spawn_native_toggle_mascot(self.app.clone());
        Ok(())
    }

    fn open_release_page(&mut self) -> Result<(), Self::Error> {
        spawn_native_open_release_page();
        Ok(())
    }
}

fn spawn_native_focus_session<R: tauri::Runtime + 'static>(app: AppHandle<R>, session_id: String) {
    crate::native_panel_runtime::spawn_native_focus_session(app, session_id);
}

fn spawn_native_open_release_page() {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = crate::commands::open_release_page() {
            warn!(error = %error, "native settings button failed to open release page");
        }
    });
}

fn spawn_native_toggle_completion_sound<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let next_enabled = !crate::app_settings::current_app_settings().completion_sound_enabled;
        if let Err(error) = crate::app_settings::update_completion_sound_enabled(next_enabled) {
            warn!(error = %error, "failed to update completion sound setting");
            return;
        }

        let _ = rerender_runtime_panel_from_last_snapshot(&app);
    });
}

fn spawn_native_toggle_mascot<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let next_enabled = !crate::app_settings::current_app_settings().mascot_enabled;
        if let Err(error) = crate::app_settings::update_mascot_enabled(next_enabled) {
            warn!(error = %error, "failed to update mascot setting");
            return;
        }
        let _ = refresh_native_panel_from_last_snapshot(&app);
    });
}

fn spawn_native_cycle_display<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let Ok(displays) = crate::display_settings::list_available_displays(&app) else {
            return;
        };
        let total = displays.len().max(1);
        let settings = crate::app_settings::current_app_settings();
        let current = crate::display_settings::resolve_preferred_display_index(
            &displays,
            settings.preferred_display_key.as_deref(),
        );
        let next = (current + 1) % total;
        let selected = &displays[next];
        if let Err(error) = crate::app_settings::update_preferred_display_selection(
            next,
            Some(selected.key.clone()),
        ) {
            warn!(error = %error, "failed to update preferred display");
            return;
        }
        let _ = reposition_native_panel_to_selected_display(&app);
    });
}
