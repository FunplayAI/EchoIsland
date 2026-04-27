use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::native_panel_renderer::facade::runtime::{
    NativePanelPlatformRuntimeBackendFacade, NativePanelPlatformRuntimeFacadeApi,
};

use super::{
    create_native_panel, hide_native_panel, native_ui_enabled,
    refresh_native_panel_from_last_snapshot, reposition_native_panel_to_selected_display,
    set_shared_expanded_body_height, spawn_platform_loops, update_native_panel_snapshot,
};

pub(crate) struct WindowsNativePanelRuntimeBackendApi;

impl NativePanelPlatformRuntimeFacadeApi for WindowsNativePanelRuntimeBackendApi {
    fn native_ui_enabled() -> bool {
        native_ui_enabled()
    }

    fn create_panel() -> Result<(), String> {
        create_native_panel()
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
        spawn_platform_loops(app);
    }

    fn update_snapshot<R: tauri::Runtime>(
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        update_native_panel_snapshot(app, snapshot)
    }

    fn hide_panel<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        hide_native_panel(app)
    }

    fn refresh_from_last_snapshot<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        refresh_native_panel_from_last_snapshot(app)
    }

    fn reposition_to_selected_display<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        reposition_native_panel_to_selected_display(app)
    }

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String> {
        set_shared_expanded_body_height(app, body_height)
    }
}

pub(crate) type WindowsNativePanelRuntimeBackendFacade =
    NativePanelPlatformRuntimeBackendFacade<WindowsNativePanelRuntimeBackendApi>;

pub(crate) fn current_windows_native_panel_runtime_backend()
-> WindowsNativePanelRuntimeBackendFacade {
    NativePanelPlatformRuntimeBackendFacade::new()
}
