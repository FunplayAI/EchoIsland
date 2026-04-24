use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::native_panel_renderer::NativePanelPlatformRuntimeBackend;

use super::panel_entry::{create_native_island_panel, native_ui_enabled};
use super::panel_loops::{
    spawn_native_count_marquee_loop, spawn_native_hover_loop, spawn_native_status_queue_loop,
};
use super::panel_snapshot::{set_shared_expanded_body_height, update_native_island_snapshot};
use super::panel_window_control::{
    hide_main_webview_window, hide_native_island_panel, refresh_native_panel_from_last_snapshot,
    reposition_native_panel_to_selected_display,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MacosNativePanelRuntimeBackendFacade;

impl NativePanelPlatformRuntimeBackend for MacosNativePanelRuntimeBackendFacade {
    fn native_ui_enabled(&self) -> bool {
        native_ui_enabled()
    }

    fn create_panel(&self) -> Result<(), String> {
        create_native_island_panel()
    }

    fn hide_main_webview_window<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        hide_main_webview_window(app)
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, app: AppHandle<R>) {
        spawn_native_hover_loop(app.clone());
        spawn_native_count_marquee_loop(app.clone());
        spawn_native_status_queue_loop(app);
    }

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        update_native_island_snapshot(app, snapshot)
    }

    fn hide_panel<R: tauri::Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        hide_native_island_panel(app)
    }

    fn refresh_from_last_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        refresh_native_panel_from_last_snapshot(app)
    }

    fn reposition_to_selected_display<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        reposition_native_panel_to_selected_display(app)
    }

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String> {
        set_shared_expanded_body_height(app, body_height)
    }
}

pub(crate) fn current_macos_native_panel_runtime_backend() -> MacosNativePanelRuntimeBackendFacade {
    MacosNativePanelRuntimeBackendFacade
}
