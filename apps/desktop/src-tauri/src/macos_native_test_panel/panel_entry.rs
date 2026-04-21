use super::*;

use tauri::{AppHandle, Manager};
use tokio::time::{Duration, MissedTickBehavior};
use tracing::{info, warn};

use crate::{constants::MAIN_WINDOW_LABEL, macos_shared_expanded_window};

pub(crate) fn native_ui_enabled() -> bool {
    !matches!(
        std::env::var("CODEISLAND_USE_WEBVIEW").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

pub(crate) fn create_native_island_panel() -> Result<(), String> {
    if NATIVE_TEST_PANEL_CREATED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let Some(mtm) = MainThreadMarker::new() else {
        return Err("native test panel must be created on the main thread".to_string());
    };

    let setup = resolve_native_panel_setup(mtm)?;
    let colors = native_panel_colors();
    let NativePanelSetup {
        screen,
        compact_height,
        compact_width,
        expanded_width,
        size,
        pill_size,
        screen_frame,
        frame,
        pill_frame,
    } = setup;
    let panel = panel_window::create_native_panel_window(mtm, frame);
    let NativePanelColors {
        pill_background,
        pill_border,
        pill_highlight,
        mascot_shell_border,
        mascot_body_fill,
        mascot_stroke,
        mascot_face,
        expanded_background,
        expanded_border,
        text_primary,
        accent_active,
        separator_color,
    } = colors;

    let PanelBaseViews {
        content_view,
        left_shoulder,
        right_shoulder,
        pill_view,
        expanded_container,
        cards_container,
        completion_glow,
        top_highlight,
        body_separator,
        settings_button,
        quit_button,
    } = create_panel_base_views(
        mtm,
        size,
        pill_frame,
        pill_size,
        compact_width,
        expanded_width,
        compact_height,
        &pill_background,
        &pill_border,
        &pill_highlight,
        &expanded_background,
        &expanded_border,
        &separator_color,
    );
    let MascotViews {
        shell: mascot_shell,
        body: mascot_body,
        left_eye: mascot_left_eye,
        right_eye: mascot_right_eye,
        mouth: mascot_mouth,
        bubble: mascot_bubble,
        sleep_label: mascot_sleep_label,
        completion_badge: mascot_completion_badge,
        completion_badge_label: mascot_completion_badge_label,
    } = create_mascot_views(
        mtm,
        &mascot_shell_border,
        &mascot_body_fill,
        &mascot_stroke,
        &mascot_face,
    );

    let CompactBarViews {
        headline,
        active_count_clip,
        active_count,
        active_count_next,
        slash,
        total_count,
    } = create_compact_bar_views(
        mtm,
        pill_size,
        screen_has_camera_housing(&screen),
        &text_primary,
        &accent_active,
    );

    assemble_native_panel_views(NativePanelAssemblyViews {
        content_view: &content_view,
        left_shoulder: &left_shoulder,
        right_shoulder: &right_shoulder,
        pill_view: &pill_view,
        expanded_container: &expanded_container,
        completion_glow: &completion_glow,
        top_highlight: &top_highlight,
        body_separator: &body_separator,
        settings_button: &settings_button,
        quit_button: &quit_button,
        mascot_shell: &mascot_shell,
        headline: &headline,
        active_count_clip: &active_count_clip,
        slash: &slash,
        total_count: &total_count,
    });

    unsafe {
        panel_window::configure_native_panel_window(&panel, &content_view, frame);
    }

    initialize_native_panel_handles(NativePanelHandleViews {
        panel: &panel,
        content_view: &content_view,
        left_shoulder: &left_shoulder,
        right_shoulder: &right_shoulder,
        pill_view: &pill_view,
        expanded_container: &expanded_container,
        cards_container: &cards_container,
        completion_glow: &completion_glow,
        top_highlight: &top_highlight,
        body_separator: &body_separator,
        settings_button: &settings_button,
        quit_button: &quit_button,
        mascot_shell: &mascot_shell,
        mascot_body: &mascot_body,
        mascot_left_eye: &mascot_left_eye,
        mascot_right_eye: &mascot_right_eye,
        mascot_mouth: &mascot_mouth,
        mascot_bubble: &mascot_bubble,
        mascot_sleep_label: &mascot_sleep_label,
        mascot_completion_badge: &mascot_completion_badge,
        mascot_completion_badge_label: &mascot_completion_badge_label,
        headline: &headline,
        active_count_clip: &active_count_clip,
        active_count: &active_count,
        active_count_next: &active_count_next,
        slash: &slash,
        total_count: &total_count,
    });
    initialize_native_panel_state();
    initialize_active_count_scroll_text();

    info!(
        panel_x = frame.origin.x,
        panel_y = frame.origin.y,
        panel_width = frame.size.width,
        panel_height = frame.size.height,
        screen_height = screen_frame.size.height,
        "created native macOS island panel"
    );

    let _: &'static mut _ = Box::leak(Box::new(panel));

    Ok(())
}

pub(crate) fn hide_native_island_panel<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let _ = macos_shared_expanded_window::hide_shared_expanded_window(app);

    app.run_on_main_thread(move || {
        if let Some(handles) = native_panel_handles() {
            unsafe {
                panel_from_ptr(handles.panel).orderOut(None);
            }
        }
    })
    .map_err(|error| error.to_string())
}

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

pub(crate) fn hide_main_webview_window<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.hide().map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(crate) fn reposition_native_panel_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    app.run_on_main_thread(move || unsafe {
        let Some(handles) = native_panel_handles() else {
            return;
        };
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let Some(screen) = resolve_preferred_native_screen(mtm) else {
            return;
        };
        let panel = panel_from_ptr(handles.panel);
        let frame = centered_top_frame(screen.frame(), panel.frame().size);
        panel.setFrame_display(frame, true);

        if let Some(state) = native_panel_state().and_then(|state| {
            state.lock().ok().and_then(|guard| {
                guard.last_snapshot.clone().map(|snapshot| {
                    (
                        snapshot,
                        guard.expanded,
                        guard.shared_body_height,
                        guard.transitioning,
                        guard.transition_cards_progress,
                        guard.transition_cards_entering,
                    )
                })
            })
        }) {
            apply_snapshot_to_panel(
                handles,
                &state.0,
                state.1,
                state.2,
                state.3,
                state.4,
                state.5,
            );
        } else {
            panel.displayIfNeeded();
        }
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    app.run_on_main_thread(move || unsafe {
        let Some(handles) = native_panel_handles() else {
            return;
        };
        if let Some(state) = native_panel_state().and_then(|state| {
            state.lock().ok().and_then(|guard| {
                guard.last_snapshot.clone().map(|snapshot| {
                    (
                        snapshot,
                        guard.expanded,
                        guard.shared_body_height,
                        guard.transitioning,
                        guard.transition_cards_progress,
                        guard.transition_cards_entering,
                    )
                })
            })
        }) {
            apply_snapshot_to_panel(
                handles,
                &state.0,
                state.1,
                state.2,
                state.3,
                state.4,
                state.5,
            );
        }
    })
    .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn native_ui_enabled() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn create_native_island_panel() -> Result<(), String> {
    Ok(())
}
