use tauri::AppHandle;
use tracing::warn;

use crate::native_panel_core::PanelInteractionCommand;

use super::panel_refs::{native_panel_handles, native_panel_state};
use super::panel_snapshot::{apply_native_panel_render_payload, native_panel_render_payload};
use super::panel_types::NativePanelHitAction;
use super::panel_window_control::{
    refresh_native_panel_from_last_snapshot, reposition_native_panel_to_selected_display,
};

pub(super) fn handle_native_click_command<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    click_command: PanelInteractionCommand,
) {
    match click_command {
        PanelInteractionCommand::HitTarget(target) => match target.action {
            NativePanelHitAction::FocusSession => {
                spawn_native_focus_session(app, target.value);
            }
            NativePanelHitAction::CycleDisplay => {
                spawn_native_cycle_display(app);
            }
            NativePanelHitAction::ToggleCompletionSound => {
                spawn_native_toggle_completion_sound(app);
            }
            NativePanelHitAction::ToggleMascot => {
                spawn_native_toggle_mascot(app);
            }
            NativePanelHitAction::OpenReleasePage => {
                spawn_native_open_release_page();
            }
        },
        PanelInteractionCommand::ToggleSettingsSurface => {}
        PanelInteractionCommand::QuitApplication => app.exit(0),
        PanelInteractionCommand::None => {}
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

        let Some(handles) = native_panel_handles() else {
            return;
        };
        let rerender = native_panel_state().and_then(|state| {
            state
                .lock()
                .ok()
                .and_then(|guard| native_panel_render_payload(&guard))
        });

        if let Some(payload) = rerender {
            let _ = app.run_on_main_thread(move || unsafe {
                apply_native_panel_render_payload(handles, payload);
            });
        }
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
