use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::native_panel_renderer::NativePanelPlatformRuntimeBackend;

use super::{
    create_native_panel, hide_main_webview_window, hide_native_panel, native_ui_enabled,
    refresh_native_panel_from_last_snapshot, reposition_native_panel_to_selected_display,
    set_shared_expanded_body_height, spawn_platform_loops, update_native_panel_snapshot,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WindowsNativePanelRuntimeBackendFacade;

impl NativePanelPlatformRuntimeBackend for WindowsNativePanelRuntimeBackendFacade {
    fn native_ui_enabled(&self) -> bool {
        native_ui_enabled()
    }

    fn create_panel(&self) -> Result<(), String> {
        create_native_panel()
    }

    fn hide_main_webview_window<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        hide_main_webview_window(app)
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, app: AppHandle<R>) {
        spawn_platform_loops(app);
    }

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        update_native_panel_snapshot(app, snapshot)
    }

    fn hide_panel<R: tauri::Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        hide_native_panel(app)
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

pub(crate) fn current_windows_native_panel_runtime_backend()
-> WindowsNativePanelRuntimeBackendFacade {
    WindowsNativePanelRuntimeBackendFacade
}
