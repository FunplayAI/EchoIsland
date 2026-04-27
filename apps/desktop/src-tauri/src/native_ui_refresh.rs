use std::sync::Arc;

use echoisland_runtime::SharedRuntime;
use tauri::AppHandle;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::native_panel_renderer::facade::runtime::{
    NativePanelRuntimeBackend, current_native_panel_runtime_backend,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tokio::time::{Duration, sleep};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tracing::warn;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn maybe_refresh_native_ui_for_event<R: tauri::Runtime + 'static>(
    app_handle: AppHandle<R>,
    runtime: Arc<SharedRuntime>,
    event_name: &str,
) {
    if !should_refresh_native_ui_for_event(event_name) {
        return;
    }
    let native_panel_backend = current_native_panel_runtime_backend();
    if !native_panel_backend.native_ui_enabled() {
        return;
    }
    let event_name = event_name.to_string();

    tauri::async_runtime::spawn(async move {
        for delay_ms in [0, 100, 320, 900] {
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
            let snapshot = runtime.snapshot().await;
            warn!(
                event_name,
                delay_ms,
                active_session_count = snapshot.active_session_count,
                pending_permission_count = snapshot.pending_permission_count,
                pending_question_count = snapshot.pending_question_count,
                "native pending-lifecycle snapshot refresh"
            );
            if let Err(error) = native_panel_backend.update_snapshot(&app_handle, &snapshot) {
                warn!(error = %error, "failed to refresh native island after pending event");
            }
        }
    });
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn should_refresh_native_ui_for_event(event_name: &str) -> bool {
    matches!(
        event_name,
        "PermissionRequest"
            | "AskUserQuestion"
            | "PermissionResponse"
            | "QuestionResponse"
            | "PeerDisconnect"
    )
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn maybe_refresh_native_ui_for_event<R: tauri::Runtime + 'static>(
    _app_handle: AppHandle<R>,
    _runtime: Arc<SharedRuntime>,
    _event_name: &str,
) {
}

#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod tests {
    use super::should_refresh_native_ui_for_event;

    #[test]
    fn refreshes_native_ui_for_pending_lifecycle_events() {
        for event_name in [
            "PermissionRequest",
            "AskUserQuestion",
            "PermissionResponse",
            "QuestionResponse",
            "PeerDisconnect",
        ] {
            assert!(should_refresh_native_ui_for_event(event_name));
        }
    }

    #[test]
    fn skips_native_ui_refresh_for_unrelated_events() {
        assert!(!should_refresh_native_ui_for_event("Notification"));
        assert!(!should_refresh_native_ui_for_event("SessionStart"));
    }
}
