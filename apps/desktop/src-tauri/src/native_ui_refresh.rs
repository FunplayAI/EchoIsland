use std::sync::Arc;

use echoisland_runtime::SharedRuntime;
use tauri::AppHandle;

#[cfg(target_os = "macos")]
use crate::native_panel_renderer::{
    NativePanelRuntimeBackend, current_native_panel_runtime_backend,
};
#[cfg(target_os = "macos")]
use tokio::time::{Duration, sleep};
#[cfg(target_os = "macos")]
use tracing::warn;

#[cfg(target_os = "macos")]
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
                "native macOS pending-lifecycle snapshot refresh"
            );
            if let Err(error) = native_panel_backend.update_snapshot(&app_handle, &snapshot) {
                warn!(error = %error, "failed to refresh native macOS island after pending event");
            }
        }
    });
}

#[cfg(target_os = "macos")]
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

#[cfg(not(target_os = "macos"))]
pub fn maybe_refresh_native_ui_for_event<R: tauri::Runtime + 'static>(
    _app_handle: AppHandle<R>,
    _runtime: Arc<SharedRuntime>,
    _event_name: &str,
) {
}

#[cfg(all(test, target_os = "macos"))]
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
