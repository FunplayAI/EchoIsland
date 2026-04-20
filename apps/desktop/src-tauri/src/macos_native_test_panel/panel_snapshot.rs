use super::*;

pub(crate) fn update_native_island_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let Some(handles) = native_panel_handles() else {
        return Ok(());
    };

    let raw_snapshot = snapshot.clone();
    let snapshot = {
        let Some(state) = native_panel_state() else {
            return Ok(());
        };
        let mut state = state
            .lock()
            .map_err(|_| "native panel state poisoned".to_string())?;
        sync_native_pending_card_visibility(&mut state, &raw_snapshot)
    };
    if macos_shared_expanded_window::shared_expanded_enabled() {
        if let Err(error) =
            macos_shared_expanded_window::sync_shared_expanded_snapshot(app, &snapshot)
        {
            warn!(error = %error, "failed to sync shared expanded snapshot");
        }
    }
    let mut transition_snapshot: Option<(RuntimeSnapshot, bool)> = None;
    let mut surface_transition_snapshot: Option<RuntimeSnapshot> = None;
    let (
        expanded,
        shared_body_height,
        transitioning,
        transition_cards_progress,
        transition_cards_entering,
    ) = {
        let Some(state) = native_panel_state() else {
            return Ok(());
        };
        let mut state = state
            .lock()
            .map_err(|_| "native panel state poisoned".to_string())?;
        let completed_session_ids = state
            .last_raw_snapshot
            .as_ref()
            .map_or_else(Vec::new, |previous| {
                detect_completed_sessions(previous, &raw_snapshot, Utc::now())
            });
        sync_native_completion_badge(&mut state, &raw_snapshot, &completed_session_ids);
        let status_queue_sync = sync_native_status_queue(&mut state, &raw_snapshot);
        let status_surface_transition =
            sync_native_status_surface_policy(&mut state, status_queue_sync);
        if let Some(expanded) = status_surface_transition.panel_transition {
            transition_snapshot = Some((snapshot.clone(), expanded));
        }
        if status_surface_transition.surface_transition {
            surface_transition_snapshot = Some(snapshot.clone());
        }
        state.last_raw_snapshot = Some(raw_snapshot.clone());
        state.last_snapshot = Some(snapshot.clone());
        (
            state.expanded,
            state.shared_body_height,
            state.transitioning,
            state.transition_cards_progress,
            state.transition_cards_entering,
        )
    };

    if let Some((transition_snapshot, expanded)) = transition_snapshot {
        let app_for_transition = app.clone();
        return app
            .run_on_main_thread(move || unsafe {
                begin_native_panel_transition(
                    app_for_transition,
                    handles,
                    transition_snapshot,
                    expanded,
                );
            })
            .map_err(|error| error.to_string());
    }

    if let Some(snapshot) = surface_transition_snapshot {
        let app_for_transition = app.clone();
        return app
            .run_on_main_thread(move || unsafe {
                begin_native_panel_surface_transition(app_for_transition, handles, snapshot);
            })
            .map_err(|error| error.to_string());
    }

    app.run_on_main_thread(move || unsafe {
        apply_snapshot_to_panel(
            handles,
            &snapshot,
            expanded,
            shared_body_height,
            transitioning,
            transition_cards_progress,
            transition_cards_entering,
        );
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn set_shared_expanded_body_height<R: tauri::Runtime>(
    app: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let Some(handles) = native_panel_handles() else {
        return Ok(());
    };
    let Some(state_mutex) = native_panel_state() else {
        return Ok(());
    };

    let rerender_payload = {
        let mut state = state_mutex
            .lock()
            .map_err(|_| "native panel state poisoned".to_string())?;
        let next_height = body_height.max(0.0);
        if state
            .shared_body_height
            .is_some_and(|current| (current - next_height).abs() < 1.0)
        {
            return Ok(());
        }
        state.shared_body_height = Some(next_height);
        state.last_snapshot.clone().map(|snapshot| {
            (
                snapshot,
                state.expanded,
                state.shared_body_height,
                state.transitioning,
                state.transition_cards_progress,
                state.transition_cards_entering,
            )
        })
    };

    if let Some((
        snapshot,
        expanded,
        shared_body_height,
        transitioning,
        transition_cards_progress,
        transition_cards_entering,
    )) = rerender_payload
    {
        app.run_on_main_thread(move || unsafe {
            apply_snapshot_to_panel(
                handles,
                &snapshot,
                expanded,
                shared_body_height,
                transitioning,
                transition_cards_progress,
                transition_cards_entering,
            );
        })
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(super) struct NativeStatusSurfaceTransition {
    pub(super) panel_transition: Option<bool>,
    pub(super) surface_transition: bool,
}

pub(super) fn sync_native_status_surface_policy(
    state: &mut NativePanelState,
    status_queue_sync: NativeStatusQueueSyncResult,
) -> NativeStatusSurfaceTransition {
    let was_status_surface =
        state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
    let added_status_items =
        status_queue_sync.added_approvals + status_queue_sync.added_completions;
    let mut panel_transition = None;

    if added_status_items > 0 && !state.expanded && !state.transitioning {
        state.expanded = true;
        state.status_auto_expanded = true;
        state.surface_mode = NativeExpandedSurface::Status;
        panel_transition = Some(true);
    } else if added_status_items > 0
        && state.expanded
        && !state.transitioning
        && state.pointer_inside_since.is_none()
    {
        state.status_auto_expanded = true;
        state.surface_mode = NativeExpandedSurface::Status;
    } else if state.status_auto_expanded
        && state.status_queue.is_empty()
        && state.expanded
        && !state.transitioning
        && state.pointer_inside_since.is_none()
    {
        state.expanded = false;
        state.status_auto_expanded = false;
        state.surface_mode = NativeExpandedSurface::Default;
        state.skip_next_close_card_exit = true;
        panel_transition = Some(false);
    } else if state.status_queue.is_empty() && state.surface_mode == NativeExpandedSurface::Status {
        state.surface_mode = NativeExpandedSurface::Default;
        state.status_auto_expanded = false;
    }

    let is_status_surface =
        state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
    NativeStatusSurfaceTransition {
        panel_transition,
        surface_transition: was_status_surface != is_status_surface
            && panel_transition.is_none()
            && state.expanded
            && !state.transitioning,
    }
}

pub(super) async fn sync_native_snapshot_once<R: tauri::Runtime>(
    app: &AppHandle<R>,
    runtime: &AppRuntime,
) {
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

    if let Err(error) = update_native_island_snapshot(app, &raw_snapshot) {
        warn!(error = %error, "failed to update native macOS island panel");
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_snapshot_to_panel(
    handles: NativePanelHandles,
    snapshot: &RuntimeSnapshot,
    expanded: bool,
    shared_body_height: Option<f64>,
    transitioning: bool,
    transition_cards_progress: f64,
    transition_cards_entering: bool,
) {
    apply_snapshot_values_to_panel(handles, snapshot);
    let context = resolve_native_transition_context(handles);
    let panel = context.refs.panel;

    if transitioning {
        if expanded {
            if context.refs.cards_container.subviews().is_empty() {
                render_transition_cards(context, snapshot);
            }
            apply_card_stack_transition(
                context.refs.cards_container,
                transition_cards_progress,
                transition_cards_entering,
            );
            context.refs.panel.displayIfNeeded();
        }
        return;
    }

    let total_height = if expanded {
        let shared_body_height = if macos_shared_expanded_window::shared_expanded_enabled()
            && !native_status_surface_active()
        {
            shared_body_height
        } else {
            None
        };
        expanded_total_height(
            snapshot,
            compact_pill_height_for_screen_rect(panel.screen().as_deref(), context.screen_frame),
            shared_body_height,
        )
    } else {
        COLLAPSED_PANEL_HEIGHT
    };
    if expanded {
        apply_panel_geometry(handles, NativePanelTransitionFrame::expanded(total_height));
    } else {
        apply_panel_geometry(handles, NativePanelTransitionFrame::collapsed(total_height));
    }

    if expanded {
        render_transition_cards(context, snapshot);
    } else {
        reset_collapsed_cards(context);
    }

    context.refs.panel.displayIfNeeded();
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_snapshot_values_to_panel(
    handles: NativePanelHandles,
    snapshot: &RuntimeSnapshot,
) {
    let refs = resolve_native_panel_refs(handles);
    let headline = refs.headline;
    let active_count = refs.active_count;
    let active_count_next = refs.active_count_next;
    let total_count = refs.total_count;

    let headline_value = NSString::from_str(&summarize_headline(snapshot));
    let active_count_text = compact_active_count_text(snapshot);
    let total_count_text = snapshot.total_session_count.to_string();
    let active_count_value = NSString::from_str(&active_count_text);
    let total_count_value = NSString::from_str(&total_count_text);
    let style = compact_style(snapshot);
    let headline_color = ns_color(style.headline_color);
    let active_count_color = ns_color(style.active_count_color);
    let total_count_color = ns_color(style.total_count_color);

    headline.setStringValue(&headline_value);
    headline.setTextColor(Some(&headline_color));
    headline.setHidden(compact_headline_should_hide(&refs));
    active_count.setTextColor(Some(&active_count_color));
    active_count_next.setTextColor(Some(&active_count_color));
    total_count.setStringValue(&total_count_value);
    total_count.setTextColor(Some(&total_count_color));
    if let Some(source) = ACTIVE_COUNT_SCROLL_TEXT.get() {
        if let Ok(mut value) = source.lock() {
            *value = active_count_text;
        }
    }
    active_count.setStringValue(&active_count_value);
    sync_active_count_marquee(&refs);

    headline.displayIfNeeded();
    refs.active_count_clip.displayIfNeeded();
    active_count.displayIfNeeded();
    active_count_next.displayIfNeeded();
    total_count.displayIfNeeded();
}

pub(super) fn with_disabled_layer_actions<T>(f: impl FnOnce() -> T) -> T {
    CATransaction::begin();
    CATransaction::setDisableActions(true);
    let result = f();
    CATransaction::commit();
    result
}
