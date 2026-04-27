use tauri::AppHandle;

use crate::native_panel_core::PanelInteractionCommand;
use crate::native_panel_renderer::facade::{
    command::{
        NativePanelPlatformEvent, NativePanelRuntimeDispatchMode,
        dispatch_native_panel_click_command_with_app_handle,
        execute_native_panel_settings_surface_command,
    },
    interaction::{
        NativePanelSettingsSurfaceSnapshotUpdate,
        resolve_native_panel_settings_surface_snapshot_update_for_state,
    },
};

use super::panel_host_runtime::with_native_runtime_panel_state_mut;
use super::panel_runtime_dispatch::dispatch_native_panel_transition_request_or_rerender;

pub(super) fn handle_native_click_command<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    click_command: PanelInteractionCommand,
) -> Result<Option<NativePanelPlatformEvent>, String> {
    dispatch_native_panel_click_command_with_app_handle(
        app,
        click_command,
        toggle_native_panel_settings_surface,
        NativePanelRuntimeDispatchMode::Scheduled,
    )
}

fn toggle_native_panel_settings_surface<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let _ = execute_native_panel_settings_surface_command(
        sync_native_settings_surface_update,
        |request, snapshot| {
            dispatch_native_panel_transition_request_or_rerender(app, request, snapshot)
        },
    )?;
    Ok(())
}

fn sync_native_settings_surface_update()
-> Result<Option<NativePanelSettingsSurfaceSnapshotUpdate>, String> {
    with_native_runtime_panel_state_mut(|state| {
        resolve_native_panel_settings_surface_snapshot_update_for_state(
            state,
            state.last_snapshot.clone(),
        )
    })
    .ok_or_else(|| "native panel state unavailable".to_string())
}
