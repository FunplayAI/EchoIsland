use tauri::{AppHandle, Manager};

use crate::{constants::MAIN_WINDOW_LABEL, macos_shared_expanded_window};
use objc2::MainThreadMarker;

use super::panel_entry::native_ui_enabled;
use super::panel_geometry::centered_top_frame;
use super::panel_refs::{native_panel_handles, native_panel_state, panel_from_ptr};
use super::panel_setup::resolve_preferred_native_screen;
use super::panel_snapshot::{apply_native_panel_render_payload, native_panel_render_payload};

pub(crate) fn hide_native_island_panel<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let _ = macos_shared_expanded_window::hide_shared_expanded_window(app);

    app.run_on_main_thread(move || {
        if let Some(handles) = native_panel_handles() {
            unsafe {
                panel_from_ptr(handles.panel).orderOut(None);
            }
        }
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
        let Some(handles) = native_panel_handles() else {
            return;
        };
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(screen) = resolve_preferred_native_screen(mtm) else {
            return;
        };
        let panel = panel_from_ptr(handles.panel);
        let frame = centered_top_frame(screen.frame(), panel.frame().size);
        panel.setFrame_display(frame, true);

        if let Some(payload) = native_panel_state().and_then(|state| {
            state
                .lock()
                .ok()
                .and_then(|guard| native_panel_render_payload(&guard))
        }) {
            apply_native_panel_render_payload(handles, payload);
        } else {
            panel.displayIfNeeded();
        }
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    app.run_on_main_thread(move || unsafe {
        let Some(handles) = native_panel_handles() else {
            return;
        };
        if let Some(payload) = native_panel_state().and_then(|state| {
            state
                .lock()
                .ok()
                .and_then(|guard| native_panel_render_payload(&guard))
        }) {
            apply_native_panel_render_payload(handles, payload);
        }
    })
    .map_err(|error| error.to_string())
}
