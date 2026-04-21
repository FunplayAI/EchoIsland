use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use crate::{app_runtime::AppRuntime, terminal_focus_service::TerminalFocusService};
#[cfg(target_os = "macos")]
use tokio::time::{Duration, MissedTickBehavior};
#[cfg(target_os = "macos")]
use tracing::{info, warn};

#[cfg(target_os = "macos")]
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

#[cfg(not(target_os = "macos"))]
pub(crate) fn spawn_native_snapshot_loop<R: tauri::Runtime + 'static>(
    _: AppHandle<R>,
    _: crate::app_runtime::AppRuntime,
) {
}

#[cfg(target_os = "macos")]
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

    if let Err(error) =
        crate::macos_native_test_panel::update_native_island_snapshot(app, &raw_snapshot)
    {
        warn!(error = %error, "failed to update native macOS island panel");
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn spawn_native_focus_session<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    session_id: String,
) {
    let runtime = app.state::<AppRuntime>().inner().clone();
    tauri::async_runtime::spawn(async move {
        match TerminalFocusService::new(&runtime)
            .focus_session(&session_id)
            .await
        {
            Ok(true) => {
                info!(session_id = %session_id, "native card click focused terminal session");
            }
            Ok(false) => {
                warn!(
                    session_id = %session_id,
                    "native card click did not find a focusable terminal target"
                );
            }
            Err(error) => {
                warn!(
                    session_id = %session_id,
                    error = %error,
                    "native card click failed to focus terminal session"
                );
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn spawn_native_focus_session<R: tauri::Runtime + 'static>(_: AppHandle<R>, _: String) {}
