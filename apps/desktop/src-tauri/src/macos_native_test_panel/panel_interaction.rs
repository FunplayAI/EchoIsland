use super::*;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn sync_hover_state_on_main_thread<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
) {
    let Some(handles) = native_panel_handles() else {
        return;
    };
    let Some(state_mutex) = native_panel_state() else {
        return;
    };

    let refs = resolve_native_panel_refs(handles);
    sync_active_count_marquee(&refs);

    let panel = panel_from_ptr(handles.panel);
    let mouse = NSEvent::mouseLocation();
    let primary_mouse_down = (NSEvent::pressedMouseButtons() & 1) != 0;
    let panel_frame = panel.frame();
    let pill_frame = absolute_rect(panel_frame, compact_pill_frame(panel, panel_frame.size));
    let expanded_container = view_from_ptr(handles.expanded_container);
    let cards_container = view_from_ptr(handles.cards_container);
    let inside = if panel_frame.size.height > COLLAPSED_PANEL_HEIGHT + 0.5 {
        point_in_rect(
            mouse,
            absolute_rect(panel_frame, expanded_container.frame()),
        )
    } else {
        point_in_rect(mouse, pill_frame)
    };
    panel.setIgnoresMouseEvents(!inside);

    let now = Instant::now();
    let mut transition_snapshot: Option<(RuntimeSnapshot, bool)> = None;
    let mut surface_transition_snapshot: Option<RuntimeSnapshot> = None;
    let mut clicked_session_id: Option<String> = None;

    {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if primary_mouse_down
            && !state.primary_mouse_down
            && inside
            && state.expanded
            && !state.transitioning
            && !cards_container.isHidden()
        {
            clicked_session_id = find_clicked_card_session_id(
                &state.card_hit_targets,
                panel_frame,
                expanded_container.frame(),
                cards_container.frame(),
                mouse,
            );
            if let Some(session_id) = clicked_session_id.as_ref() {
                let suppressed =
                    state
                        .last_focus_click
                        .as_ref()
                        .is_some_and(|(last_session_id, last_at)| {
                            last_session_id == session_id
                                && now.duration_since(*last_at).as_millis()
                                    < CARD_FOCUS_CLICK_DEBOUNCE_MS
                        });
                if suppressed {
                    clicked_session_id = None;
                } else {
                    state.last_focus_click = Some((session_id.clone(), now));
                }
            }
        }
        state.primary_mouse_down = primary_mouse_down;
        let was_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();

        if let Some(hover_transition) = sync_native_hover_expansion_state(&mut state, inside, now) {
            if let Some(snapshot) = state.last_snapshot.clone() {
                transition_snapshot =
                    Some((snapshot, hover_transition == NativeHoverTransition::Expand));
            }
        }

        let is_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        if was_status_surface != is_status_surface && state.expanded && !state.transitioning {
            if let Some(snapshot) = state.last_snapshot.clone() {
                surface_transition_snapshot = Some(snapshot);
            }
        }
    }

    if let Some((snapshot, expanded)) = transition_snapshot {
        begin_native_panel_transition(app.clone(), handles, snapshot, expanded);
    } else if let Some(snapshot) = surface_transition_snapshot {
        begin_native_panel_surface_transition(app.clone(), handles, snapshot);
    }

    if let Some(session_id) = clicked_session_id {
        spawn_native_focus_session(app, session_id);
    }
}

pub(super) fn sync_native_hover_expansion_state(
    state: &mut NativePanelState,
    inside: bool,
    now: Instant,
) -> Option<NativeHoverTransition> {
    if inside {
        state.pointer_outside_since = None;
        state.pointer_inside_since.get_or_insert(now);
        if !state.expanded
            && !state.transitioning
            && state.pointer_inside_since.is_some_and(|entered_at| {
                now.duration_since(entered_at).as_millis() >= HOVER_DELAY_MS as u128
            })
        {
            state.expanded = true;
            state.status_auto_expanded = false;
            state.surface_mode = NativeExpandedSurface::Default;
            return Some(NativeHoverTransition::Expand);
        }
    } else {
        state.pointer_inside_since = None;
        state.pointer_outside_since.get_or_insert(now);
        let keep_open_for_status = state.status_auto_expanded
            && state.surface_mode == NativeExpandedSurface::Status
            && !state.status_queue.is_empty();
        if state.expanded
            && !state.transitioning
            && !keep_open_for_status
            && state.pointer_outside_since.is_some_and(|left_at| {
                now.duration_since(left_at).as_millis() >= HOVER_DELAY_MS as u128
            })
        {
            state.expanded = false;
            state.status_auto_expanded = false;
            state.surface_mode = NativeExpandedSurface::Default;
            return Some(NativeHoverTransition::Collapse);
        }
    }

    None
}

pub(super) fn native_status_surface_active() -> bool {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                guard.surface_mode == NativeExpandedSurface::Status
                    && !guard.status_queue.is_empty()
            })
        })
        .unwrap_or(false)
}

pub(super) fn replace_native_card_hit_targets(targets: Vec<NativeCardHitTarget>) {
    if let Some(state) = native_panel_state() {
        if let Ok(mut guard) = state.lock() {
            guard.card_hit_targets = targets;
        }
    }
}

fn find_clicked_card_session_id(
    targets: &[NativeCardHitTarget],
    panel_frame: NSRect,
    expanded_frame: NSRect,
    cards_frame: NSRect,
    mouse: NSPoint,
) -> Option<String> {
    targets
        .iter()
        .find(|target| {
            point_in_rect(
                mouse,
                absolute_rect(
                    panel_frame,
                    compose_local_rect(
                        expanded_frame,
                        compose_local_rect(cards_frame, target.frame),
                    ),
                ),
            )
        })
        .map(|target| target.session_id.clone())
}

fn spawn_native_focus_session<R: tauri::Runtime + 'static>(app: AppHandle<R>, session_id: String) {
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
