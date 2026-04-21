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
    let pill_frame = refs.pill_view.frame();
    let hover_pill_frame = native_hover_pill_rect(panel_frame, pill_frame);
    let expanded_container = view_from_ptr(handles.expanded_container);
    let cards_container = view_from_ptr(handles.cards_container);
    let inside = if panel_frame.size.height > COLLAPSED_PANEL_HEIGHT + 0.5 {
        point_in_rect(
            mouse,
            absolute_rect(panel_frame, expanded_container.frame()),
        ) || point_in_rect(mouse, hover_pill_frame)
    } else {
        point_in_rect(mouse, hover_pill_frame)
    };
    panel.setIgnoresMouseEvents(!inside);

    let now = Instant::now();
    let mut transition_snapshot: Option<(RuntimeSnapshot, bool)> = None;
    let mut surface_transition_snapshot: Option<RuntimeSnapshot> = None;
    let mut clicked_session_id: Option<String> = None;
    let mut settings_clicked = false;
    let mut quit_clicked = false;

    {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let primary_click_started = primary_mouse_down && !state.primary_mouse_down && inside;
        if primary_click_started && state.expanded && !state.transitioning {
            settings_clicked = point_in_rect(
                mouse,
                native_edge_action_button_rect(
                    panel_frame,
                    refs.pill_view.frame(),
                    refs.settings_button.frame(),
                ),
            );
            quit_clicked = !settings_clicked
                && point_in_rect(
                    mouse,
                    native_edge_action_button_rect(
                        panel_frame,
                        refs.pill_view.frame(),
                        refs.quit_button.frame(),
                    ),
                );
        }
        if primary_click_started
            && state.expanded
            && !state.transitioning
            && !settings_clicked
            && !quit_clicked
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
    } else if settings_clicked {
        spawn_native_open_settings_location();
    } else if quit_clicked {
        app.exit(0);
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
            state.completion_badge_items.clear();
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

fn native_edge_action_button_rect(
    panel_frame: NSRect,
    pill_frame: NSRect,
    button_frame: NSRect,
) -> NSRect {
    absolute_rect(panel_frame, compose_local_rect(pill_frame, button_frame))
}

fn native_hover_pill_rect(panel_frame: NSRect, pill_frame: NSRect) -> NSRect {
    let top_gap =
        (panel_frame.size.height - (pill_frame.origin.y + pill_frame.size.height)).max(0.0);
    absolute_rect(
        panel_frame,
        NSRect::new(
            pill_frame.origin,
            NSSize::new(pill_frame.size.width, pill_frame.size.height + top_gap),
        ),
    )
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

fn spawn_native_open_settings_location() {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = crate::commands::open_settings_location() {
            warn!(error = %error, "native settings button failed to open settings location");
        }
    });
}
