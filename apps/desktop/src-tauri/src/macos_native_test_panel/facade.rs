use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::native_panel_renderer::facade::runtime::reposition_native_panel_to_selected_display_then_refresh;

use super::{
    panel_entry::create_native_island_panel,
    panel_host_controller::{
        hide_native_panel_with_host_controller,
        reposition_native_panel_to_selected_display_with_host_controller,
        set_shared_expanded_body_height_with_host_controller,
    },
    panel_loops::{
        spawn_native_count_marquee_loop, spawn_native_hover_loop, spawn_native_status_queue_loop,
    },
    panel_runtime_dispatch::refresh_native_panel_render_payload_from_runtime_state,
    panel_snapshot::update_native_island_snapshot,
};

pub(crate) fn native_ui_enabled() -> bool {
    super::panel_entry::native_ui_enabled()
}

pub(crate) fn create_native_panel() -> Result<(), String> {
    create_native_island_panel()
}

pub(crate) fn spawn_platform_loops<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    spawn_native_hover_loop(app.clone());
    spawn_native_count_marquee_loop(app.clone());
    spawn_native_status_queue_loop(app);
}

pub(crate) fn update_native_panel_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    update_native_island_snapshot(app, snapshot)
}

pub(crate) fn hide_native_panel<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    hide_native_panel_with_host_controller(app)
}

pub(crate) fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let _ = refresh_native_panel_render_payload_from_runtime_state(app)?;
    Ok(())
}

pub(crate) fn reposition_native_panel_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    reposition_native_panel_to_selected_display_then_refresh(
        app,
        reposition_native_panel_to_selected_display_with_host_controller,
        refresh_native_panel_from_last_snapshot,
    )
}

pub(crate) fn set_shared_expanded_body_height<R: tauri::Runtime>(
    app: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    set_shared_expanded_body_height_with_host_controller(app, body_height)
}
