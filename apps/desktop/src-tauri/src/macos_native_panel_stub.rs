#![cfg(not(target_os = "macos"))]
#![allow(dead_code)]

use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

pub fn native_ui_enabled() -> bool {
    false
}

pub fn create_native_island_panel() -> Result<(), String> {
    Ok(())
}

pub fn hide_native_island_panel<R: tauri::Runtime>(_: &AppHandle<R>) -> Result<(), String> {
    Ok(())
}

pub fn update_native_island_snapshot<R: tauri::Runtime>(
    _: &AppHandle<R>,
    _: &RuntimeSnapshot,
) -> Result<(), String> {
    Ok(())
}

pub fn set_shared_expanded_body_height<R: tauri::Runtime>(
    _: &AppHandle<R>,
    _: f64,
) -> Result<(), String> {
    Ok(())
}

pub fn spawn_native_hover_loop<R: tauri::Runtime + 'static>(_: AppHandle<R>) {}

pub fn spawn_native_count_marquee_loop<R: tauri::Runtime + 'static>(_: AppHandle<R>) {}

pub fn spawn_native_status_queue_loop<R: tauri::Runtime + 'static>(_: AppHandle<R>) {}

pub fn hide_main_webview_window<R: tauri::Runtime>(_: &AppHandle<R>) -> Result<(), String> {
    Ok(())
}

pub fn reposition_native_panel_to_selected_display<R: tauri::Runtime>(
    _: &AppHandle<R>,
) -> Result<(), String> {
    Ok(())
}

pub fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    _: &AppHandle<R>,
) -> Result<(), String> {
    Ok(())
}
