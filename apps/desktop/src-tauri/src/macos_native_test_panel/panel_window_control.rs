use tauri::{AppHandle, Manager};

use crate::{constants::MAIN_WINDOW_LABEL, macos_shared_expanded_window};
use objc2::MainThreadMarker;

use super::panel_entry::native_ui_enabled;
use super::panel_geometry::centered_top_frame;
use super::panel_host_runtime::{
    MacosNativePanelRuntimeHost, rerender_runtime_panel_from_last_snapshot,
};
use super::panel_setup::resolve_preferred_native_screen;

pub(crate) fn hide_native_island_panel<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let _ = macos_shared_expanded_window::hide_shared_expanded_window(app);

    app.run_on_main_thread(move || unsafe {
        let _ = MacosNativePanelRuntimeHost::order_out();
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn hide_main_webview_window<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.hide().map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(crate) fn reposition_native_panel_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    app.run_on_main_thread(move || unsafe {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(screen) = resolve_preferred_native_screen(mtm) else {
            return;
        };
        let Some(panel_frame) = MacosNativePanelRuntimeHost::current_window_frame() else {
            return;
        };
        let preferred_display_index =
            crate::app_settings::current_app_settings().preferred_display_index;
        let frame = centered_top_frame(screen.frame(), panel_frame.size);
        let _ = MacosNativePanelRuntimeHost::reposition_to_screen(
            preferred_display_index,
            screen.frame(),
            frame,
        );
    })
    .map_err(|error| error.to_string())?;

    rerender_runtime_panel_from_last_snapshot(app)
}

pub(crate) fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    rerender_runtime_panel_from_last_snapshot(app)
}
