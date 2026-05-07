use tauri::AppHandle;
use tokio::time::{Duration, MissedTickBehavior};
use tracing::warn;

use super::compact_bar_layout::sync_active_count_marquee;
use super::mascot::sync_native_mascot;
use super::panel_constants::{
    ACTIVE_COUNT_SCROLL_REFRESH_MS, HOVER_POLL_MS, MASCOT_ANIMATION_REFRESH_MS,
    STATUS_QUEUE_REFRESH_MS,
};
use super::panel_entry::native_ui_enabled;
use super::panel_interaction::sync_hover_state_on_main_thread;
use super::panel_refs::{
    native_panel_handles, native_panel_state, panel_from_ptr, resolve_native_panel_refs,
};
use super::panel_snapshot::update_native_island_snapshot;

pub(crate) fn spawn_native_hover_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(HOVER_POLL_MS));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if !native_ui_enabled() {
                continue;
            }

            let app_for_hover = app.clone();
            let _ = app.run_on_main_thread(move || unsafe {
                sync_hover_state_on_main_thread(app_for_hover);
            });
        }
    });
}

pub(crate) fn spawn_native_count_marquee_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(
            ACTIVE_COUNT_SCROLL_REFRESH_MS.min(MASCOT_ANIMATION_REFRESH_MS),
        ));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if !native_ui_enabled() {
                continue;
            }

            let _ = app.run_on_main_thread(move || unsafe {
                let Some(handles) = native_panel_handles() else {
                    return;
                };
                let refs = resolve_native_panel_refs(handles);
                sync_active_count_marquee(&refs);
                sync_native_mascot(handles);
                panel_from_ptr(handles.panel).displayIfNeeded();
            });
        }
    });
}

pub(crate) fn spawn_native_status_queue_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(STATUS_QUEUE_REFRESH_MS));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if !native_ui_enabled() {
                continue;
            }

            let snapshot = native_panel_state().and_then(|state| {
                state.lock().ok().and_then(|guard| {
                    if guard.status_queue.is_empty()
                        && guard.pending_permission_card.is_none()
                        && guard.pending_question_card.is_none()
                    {
                        None
                    } else {
                        guard.last_raw_snapshot.clone()
                    }
                })
            });
            let Some(snapshot) = snapshot else {
                continue;
            };

            if let Err(error) = update_native_island_snapshot(&app, &snapshot) {
                warn!(error = %error, "failed to refresh native macOS status queue animation");
            }
        }
    });
}
