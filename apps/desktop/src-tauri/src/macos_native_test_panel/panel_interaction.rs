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
    let mut clicked_action: Option<NativeCardHitTarget> = None;
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
            clicked_action = find_clicked_card_target(
                &state.card_hit_targets,
                panel_frame,
                expanded_container.frame(),
                cards_container.frame(),
                mouse,
            );
            if let Some(target) = clicked_action.as_ref().filter(|target| {
                target.action == NativePanelHitAction::FocusSession
            }) {
                let session_id = &target.value;
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
                    clicked_action = None;
                } else {
                    state.last_focus_click = Some((session_id.clone(), now));
                }
            }
        }
        state.primary_mouse_down = primary_mouse_down;
        let was_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        let was_surface_mode = state.surface_mode;

        if let Some(hover_transition) = sync_native_hover_expansion_state(&mut state, inside, now) {
            if let Some(snapshot) = state.last_snapshot.clone() {
                transition_snapshot =
                    Some((snapshot, hover_transition == NativeHoverTransition::Expand));
            }
        }

        if primary_click_started
            && state.expanded
            && !state.transitioning
            && settings_clicked
        {
            state.status_auto_expanded = false;
            state.surface_mode = if state.surface_mode == NativeExpandedSurface::Settings {
                NativeExpandedSurface::Default
            } else {
                NativeExpandedSurface::Settings
            };
            surface_transition_snapshot = state.last_snapshot.clone();
        }

        let is_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        if surface_transition_snapshot.is_none()
            && was_status_surface != is_status_surface
            && state.expanded
            && !state.transitioning
        {
            if let Some(snapshot) = state.last_snapshot.clone() {
                surface_transition_snapshot = Some(snapshot);
            }
        } else if surface_transition_snapshot.is_none()
            && was_surface_mode != state.surface_mode
            && state.expanded
            && !state.transitioning
        {
            surface_transition_snapshot = state.last_snapshot.clone();
        }
    }

    if let Some((snapshot, expanded)) = transition_snapshot {
        begin_native_panel_transition(app.clone(), handles, snapshot, expanded);
    } else if let Some(snapshot) = surface_transition_snapshot {
        begin_native_panel_surface_transition(app.clone(), handles, snapshot);
    }

    if let Some(target) = clicked_action {
        match target.action {
            NativePanelHitAction::FocusSession => {
                spawn_native_focus_session(app, target.value);
            }
            NativePanelHitAction::CycleDisplay => {
                spawn_native_cycle_display(app);
            }
            NativePanelHitAction::ToggleCompletionSound => {
                spawn_native_toggle_completion_sound(app);
            }
            NativePanelHitAction::ToggleMascot => {
                spawn_native_toggle_mascot(app);
            }
            NativePanelHitAction::OpenReleasePage => {
                spawn_native_open_release_page();
            }
        }
    } else if settings_clicked {
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

pub(super) fn native_settings_surface_active() -> bool {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                guard.surface_mode == NativeExpandedSurface::Settings && guard.expanded
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

fn find_clicked_card_target(
    targets: &[NativeCardHitTarget],
    panel_frame: NSRect,
    expanded_frame: NSRect,
    cards_frame: NSRect,
    mouse: NSPoint,
) -> Option<NativeCardHitTarget> {
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
        .cloned()
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
    crate::native_panel_runtime::spawn_native_focus_session(app, session_id);
}

fn spawn_native_open_release_page() {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = crate::commands::open_release_page() {
            warn!(error = %error, "native settings button failed to open release page");
        }
    });
}

fn spawn_native_toggle_completion_sound<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let next_enabled = !crate::app_settings::current_app_settings().completion_sound_enabled;
        if let Err(error) = crate::app_settings::update_completion_sound_enabled(next_enabled) {
            warn!(error = %error, "failed to update completion sound setting");
            return;
        }

        let Some(handles) = native_panel_handles() else {
            return;
        };
        let rerender = native_panel_state().and_then(|state| {
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
        });

        if let Some((snapshot, expanded, shared_body_height, transitioning, cards_progress, cards_entering)) =
            rerender
        {
            let _ = app.run_on_main_thread(move || unsafe {
                apply_snapshot_to_panel(
                    handles,
                    &snapshot,
                    expanded,
                    shared_body_height,
                    transitioning,
                    cards_progress,
                    cards_entering,
                );
            });
        }
    });
}

fn spawn_native_toggle_mascot<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let next_enabled = !crate::app_settings::current_app_settings().mascot_enabled;
        if let Err(error) = crate::app_settings::update_mascot_enabled(next_enabled) {
            warn!(error = %error, "failed to update mascot setting");
            return;
        }
        let _ = crate::macos_native_test_panel::refresh_native_panel_from_last_snapshot(&app);
    });
}

fn spawn_native_cycle_display<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let Ok(displays) = crate::display_settings::list_available_displays(&app) else {
            return;
        };
        let total = displays.len().max(1);
        let settings = crate::app_settings::current_app_settings();
        let current = crate::display_settings::resolve_preferred_display_index(
            &displays,
            settings.preferred_display_key.as_deref(),
        );
        let next = (current + 1) % total;
        let selected = &displays[next];
        if let Err(error) = crate::app_settings::update_preferred_display_selection(
            next,
            Some(selected.key.clone()),
        ) {
            warn!(error = %error, "failed to update preferred display");
            return;
        }
        let _ = crate::macos_native_test_panel::reposition_native_panel_to_selected_display(&app);
    });
}
