use tauri::AppHandle;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::{
    app_runtime::AppRuntime,
    native_panel_renderer::facade::runtime::{
        NativePanelRuntimeBackend, current_native_panel_runtime_backend,
    },
    terminal_focus_service::TerminalFocusService,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tokio::time::{Duration, MissedTickBehavior};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tracing::warn;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn spawn_native_snapshot_loop<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    runtime: AppRuntime,
) {
    tauri::async_runtime::spawn(async move {
        sync_native_snapshot_once(&app, &runtime).await;

        let mut interval = tokio::time::interval(Duration::from_millis(1500));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            sync_native_snapshot_once(&app, &runtime).await;
        }
    });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn spawn_native_snapshot_loop<R: tauri::Runtime + 'static>(
    _: AppHandle<R>,
    _: crate::app_runtime::AppRuntime,
) {
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
async fn sync_native_snapshot_once<R: tauri::Runtime>(app: &AppHandle<R>, runtime: &AppRuntime) {
    let raw_snapshot = runtime.runtime.snapshot().await;
    if raw_snapshot.pending_permission_count > 0 || raw_snapshot.pending_question_count > 0 {
        warn!(
            active_session_count = raw_snapshot.active_session_count,
            pending_permission_count = raw_snapshot.pending_permission_count,
            pending_question_count = raw_snapshot.pending_question_count,
            "native snapshot loop observed pending items"
        );
    }
    if raw_snapshot.active_session_count > 0 {
        if let Err(error) = TerminalFocusService::new(runtime)
            .sync_snapshot_focus_bindings(&raw_snapshot)
            .await
        {
            warn!(error = %error, "failed to sync focus bindings during native snapshot refresh");
        }
    }

    let native_panel_backend = current_native_panel_runtime_backend();
    if let Err(error) = native_panel_backend.update_snapshot(app, &raw_snapshot) {
        warn!(error = %error, "failed to update native island panel");
    }
}
