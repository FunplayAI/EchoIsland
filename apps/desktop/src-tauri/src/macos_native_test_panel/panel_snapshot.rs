use super::*;
use crate::notification_sound::play_message_card_sound;

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
    let (
        snapshot,
        play_message_sound,
        transition_snapshot,
        surface_transition_snapshot,
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
        let mut core = state.to_core_panel_state();
        let sync_result = crate::native_panel_core::sync_panel_snapshot_state(
            &mut core,
            &raw_snapshot,
            Utc::now(),
        );
        state.apply_core_panel_state(core);

        let snapshot = sync_result.displayed_snapshot.clone();
        let transition_snapshot = sync_result
            .panel_transition
            .map(|expanded| (snapshot.clone(), expanded));
        let surface_transition_snapshot = sync_result.surface_transition.then(|| snapshot.clone());
        state.last_snapshot = Some(snapshot.clone());
        (
            snapshot,
            sync_result.play_message_card_sound,
            transition_snapshot,
            surface_transition_snapshot,
            state.expanded,
            state.shared_body_height,
            state.transitioning,
            state.transition_cards_progress,
            state.transition_cards_entering,
        )
    };
    if macos_shared_expanded_window::shared_expanded_enabled() {
        if let Err(error) =
            macos_shared_expanded_window::sync_shared_expanded_snapshot(app, &snapshot)
        {
            warn!(error = %error, "failed to sync shared expanded snapshot");
        }
    }
    if play_message_sound {
        play_message_card_sound();
    }

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
        let decision = crate::native_panel_core::resolve_shared_body_height_decision(
            crate::native_panel_core::SharedBodyHeightDecisionInput {
                current_height: state.shared_body_height,
                requested_height: body_height,
                has_snapshot: state.last_snapshot.is_some(),
                update_threshold: 1.0,
            },
        );
        if !decision.should_update {
            return Ok(());
        }
        state.shared_body_height = Some(decision.next_height);
        if decision.should_rerender {
            native_panel_render_payload(&state)
        } else {
            None
        }
    };

    if let Some(payload) = rerender_payload {
        app.run_on_main_thread(move || unsafe {
            apply_native_panel_render_payload(handles, payload);
        })
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(super) fn native_panel_render_payload(
    state: &NativePanelState,
) -> Option<NativePanelRenderPayload> {
    state
        .last_snapshot
        .clone()
        .map(|snapshot| NativePanelRenderPayload {
            snapshot,
            expanded: state.expanded,
            shared_body_height: state.shared_body_height,
            transitioning: state.transitioning,
            transition_cards_progress: state.transition_cards_progress,
            transition_cards_entering: state.transition_cards_entering,
        })
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_native_panel_render_payload(
    handles: NativePanelHandles,
    payload: NativePanelRenderPayload,
) {
    apply_snapshot_to_panel(
        handles,
        &payload.snapshot,
        payload.expanded,
        payload.shared_body_height,
        payload.transitioning,
        payload.transition_cards_progress,
        payload.transition_cards_entering,
    );
}

pub(super) type NativeStatusSurfaceTransition = crate::native_panel_core::StatusSurfaceTransition;

pub(super) fn sync_native_status_surface_policy(
    state: &mut NativePanelState,
    status_queue_sync: NativeStatusQueueSyncResult,
) -> NativeStatusSurfaceTransition {
    let mut core = state.to_core_panel_state();
    let transition =
        crate::native_panel_core::sync_status_surface_policy(&mut core, status_queue_sync);
    state.apply_core_panel_state(core);
    transition
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
            && !native_settings_surface_active()
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
    let scene = build_native_panel_scene(snapshot);
    let headline = refs.headline;
    let active_count = refs.active_count;
    let active_count_next = refs.active_count_next;
    let total_count = refs.total_count;

    let headline_value = NSString::from_str(&scene.compact_bar.headline.text);
    let active_count_text = scene.compact_bar.active_count.clone();
    let total_count_text = scene.compact_bar.total_count.clone();
    let active_count_value = NSString::from_str(&active_count_text);
    let total_count_value = NSString::from_str(&total_count_text);
    let style = compact_style_for_scene(&scene);
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
