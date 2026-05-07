use crate::notification_sound::play_message_card_sound;
use chrono::Utc;
use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use super::card_animation::apply_card_stack_transition;
use super::panel_entry::native_ui_enabled;
use super::panel_refs::native_panel_state;
use super::panel_render::apply_panel_geometry;
use super::panel_runtime_dispatch::{
    dispatch_native_panel_render_payload,
    dispatch_native_panel_transition_request_or_apply_render_payload,
};
use super::panel_runtime_input::native_panel_runtime_input_descriptor;
use super::panel_scene_adapter::resolve_snapshot_render_plan;
use super::panel_types::{
    NativePanelHandles, NativePanelRenderPayload, NativePanelState, NativePanelTransitionFrame,
};
use super::panel_view_updates::apply_snapshot_values_to_panel;

#[cfg(test)]
use super::panel_types::NativeStatusQueueSyncResult;
use super::transition_ui::{
    render_transition_cards_with_plan, reset_collapsed_cards, resolve_native_transition_context,
    resolved_snapshot_panel_height_for_plan,
};
use crate::native_panel_renderer::facade::{
    host::sync_runtime_host_shared_body_height_in_state,
    presentation::NativePanelSnapshotRenderPlan,
    renderer::{cache_runtime_scene_sync_result, sync_runtime_scene_bundle_from_state_input},
    runtime::{NativePanelRuntimeRenderPayloadState, NativePanelRuntimeRenderPayloadStateBridge},
    transition::{NativePanelTransitionRequest, native_panel_transition_request_for_snapshot_sync},
};

struct NativeSnapshotUpdate {
    snapshot: RuntimeSnapshot,
    play_message_sound: bool,
    transition_request: Option<NativePanelTransitionRequest>,
    apply_state: NativePanelRuntimeRenderPayloadState,
}

struct NativeSharedBodyHeightUpdate {
    rerender_payload: Option<NativePanelRenderPayload>,
}

#[derive(Clone, Copy)]
struct NativeCardTransitionState {
    progress: f64,
    entering: bool,
}

enum NativeSnapshotApplyMode {
    TransitioningExpanded(NativeCardTransitionState),
    TransitioningCollapsed,
    Static { expanded: bool, total_height: f64 },
}

pub(crate) fn update_native_island_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let Some(update) = sync_native_snapshot_update(snapshot)? else {
        return Ok(());
    };
    play_message_sound_if_needed(update.play_message_sound);
    dispatch_snapshot_update(app, update)
}

pub(crate) fn set_shared_expanded_body_height<R: tauri::Runtime>(
    app: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let Some(update) = sync_shared_expanded_body_height_update(body_height)? else {
        return Ok(());
    };
    dispatch_shared_expanded_body_height_update(app, update)
}

pub(super) fn native_panel_render_payload(
    state: &NativePanelState,
) -> Option<NativePanelRenderPayload> {
    state.render_payload()
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_native_panel_render_payload(
    handles: NativePanelHandles,
    payload: NativePanelRenderPayload,
) {
    apply_snapshot_to_panel(handles, &payload);
}

fn sync_shared_expanded_body_height_update(
    body_height: f64,
) -> Result<Option<NativeSharedBodyHeightUpdate>, String> {
    let Some(state_mutex) = native_panel_state() else {
        return Ok(None);
    };
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
        return Ok(None);
    }

    let next_height = Some(decision.next_height);
    state.shared_body_height = next_height;
    sync_runtime_host_shared_body_height_in_state(&mut *state, next_height);
    Ok(Some(NativeSharedBodyHeightUpdate {
        rerender_payload: decision
            .should_rerender
            .then(|| native_panel_render_payload(&state))
            .flatten(),
    }))
}

fn dispatch_shared_expanded_body_height_update<R: tauri::Runtime>(
    app: &AppHandle<R>,
    update: NativeSharedBodyHeightUpdate,
) -> Result<(), String> {
    if let Some(payload) = update.rerender_payload {
        dispatch_native_panel_render_payload(app, payload)?;
    }

    Ok(())
}

#[cfg(test)]
pub(super) type NativeStatusSurfaceTransition = crate::native_panel_core::StatusSurfaceTransition;

#[cfg(test)]
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

fn sync_native_snapshot_update(
    snapshot: &RuntimeSnapshot,
) -> Result<Option<NativeSnapshotUpdate>, String> {
    let Some(state) = native_panel_state() else {
        return Ok(None);
    };
    let mut state = state
        .lock()
        .map_err(|_| "native panel state poisoned".to_string())?;
    let input = native_panel_runtime_input_descriptor();
    let sync_result =
        sync_runtime_scene_bundle_from_state_input(&mut *state, snapshot, &input, Utc::now());
    let snapshot = sync_result.snapshot_sync.displayed_snapshot.clone();
    let update = NativeSnapshotUpdate {
        transition_request: native_panel_transition_request_for_snapshot_sync(
            &sync_result.snapshot_sync,
        ),
        play_message_sound: sync_result.snapshot_sync.reminder.play_sound,
        apply_state: state.runtime_render_payload_state(),
        snapshot: snapshot.clone(),
    };
    state.last_snapshot = Some(snapshot);
    cache_runtime_scene_sync_result(&mut state.scene_cache, sync_result);
    Ok(Some(update))
}

fn play_message_sound_if_needed(enabled: bool) {
    if enabled {
        play_message_card_sound();
    }
}

fn dispatch_snapshot_update<R: tauri::Runtime>(
    app: &AppHandle<R>,
    update: NativeSnapshotUpdate,
) -> Result<(), String> {
    let apply_state = update.apply_state;
    dispatch_native_panel_transition_request_or_apply_render_payload(
        app,
        update.transition_request,
        Some(update.snapshot.clone()),
        NativePanelRenderPayload {
            snapshot: update.snapshot,
            expanded: apply_state.expanded,
            shared_body_height: apply_state.shared_body_height,
            transitioning: apply_state.transitioning,
            transition_cards_progress: apply_state.transition_cards_progress,
            transition_cards_entering: apply_state.transition_cards_entering,
        },
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_snapshot_to_panel(
    handles: NativePanelHandles,
    payload: &NativePanelRenderPayload,
) {
    apply_snapshot_values_to_panel(handles, &payload.snapshot);
    let context = resolve_native_transition_context(handles);
    let render_plan = resolve_snapshot_render_plan(&payload.snapshot);
    let mode = resolve_snapshot_apply_mode(context, &render_plan, payload);
    apply_snapshot_mode(handles, context, &render_plan, mode);
}

fn resolve_snapshot_apply_mode(
    context: super::transition_ui::NativeTransitionContext,
    render_plan: &NativePanelSnapshotRenderPlan,
    payload: &NativePanelRenderPayload,
) -> NativeSnapshotApplyMode {
    if let Some(mode) = resolve_transitioning_snapshot_apply_mode(payload) {
        return mode;
    }

    NativeSnapshotApplyMode::Static {
        expanded: payload.expanded,
        total_height: resolved_snapshot_panel_height_for_plan(
            context,
            render_plan,
            payload.expanded,
            payload.shared_body_height,
        ),
    }
}

fn resolve_transitioning_snapshot_apply_mode(
    payload: &NativePanelRenderPayload,
) -> Option<NativeSnapshotApplyMode> {
    if !payload.transitioning {
        return None;
    }

    Some(if payload.expanded {
        NativeSnapshotApplyMode::TransitioningExpanded(NativeCardTransitionState {
            progress: payload.transition_cards_progress,
            entering: payload.transition_cards_entering,
        })
    } else {
        NativeSnapshotApplyMode::TransitioningCollapsed
    })
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_snapshot_mode(
    handles: NativePanelHandles,
    context: super::transition_ui::NativeTransitionContext,
    render_plan: &NativePanelSnapshotRenderPlan,
    mode: NativeSnapshotApplyMode,
) {
    match mode {
        NativeSnapshotApplyMode::TransitioningExpanded(cards) => {
            if context.refs.cards_container.subviews().is_empty() {
                render_transition_cards_with_plan(context, render_plan);
            }
            apply_card_stack_transition(
                context.refs.cards_container,
                cards.progress,
                cards.entering,
            );
        }
        NativeSnapshotApplyMode::TransitioningCollapsed => {}
        NativeSnapshotApplyMode::Static {
            expanded,
            total_height,
        } => {
            apply_snapshot_panel_geometry(handles, expanded, total_height);
            if expanded {
                render_transition_cards_with_plan(context, render_plan);
            } else {
                reset_collapsed_cards(context);
            }
        }
    }

    context.refs.panel.displayIfNeeded();
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_snapshot_panel_geometry(
    handles: NativePanelHandles,
    expanded: bool,
    total_height: f64,
) {
    if expanded {
        apply_panel_geometry(handles, NativePanelTransitionFrame::expanded(total_height));
    } else {
        apply_panel_geometry(handles, NativePanelTransitionFrame::collapsed(total_height));
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeSnapshotApplyMode, resolve_transitioning_snapshot_apply_mode};
    use echoisland_runtime::RuntimeSnapshot;

    fn snapshot(status: &str) -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: status.to_string(),
            primary_source: "claude".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: Vec::new(),
            pending_questions: Vec::new(),
            sessions: Vec::new(),
        }
    }

    #[test]
    fn transitioning_snapshot_apply_mode_expanded_carries_card_transition_state() {
        let payload = crate::macos_native_panel::panel_types::NativePanelRenderPayload {
            snapshot: snapshot("running"),
            expanded: true,
            shared_body_height: Some(180.0),
            transitioning: true,
            transition_cards_progress: 0.42,
            transition_cards_entering: true,
        };
        let mode = resolve_transitioning_snapshot_apply_mode(&payload).expect("transitioning mode");

        match mode {
            NativeSnapshotApplyMode::TransitioningExpanded(cards) => {
                assert_eq!(cards.progress, 0.42);
                assert!(cards.entering);
            }
            NativeSnapshotApplyMode::TransitioningCollapsed
            | NativeSnapshotApplyMode::Static { .. } => {
                panic!("expected transitioning expanded mode")
            }
        }
    }

    #[test]
    fn transitioning_snapshot_apply_mode_collapsed_uses_collapsing_variant() {
        let payload = crate::macos_native_panel::panel_types::NativePanelRenderPayload {
            snapshot: snapshot("idle"),
            expanded: false,
            shared_body_height: Some(180.0),
            transitioning: true,
            transition_cards_progress: 0.18,
            transition_cards_entering: false,
        };
        let mode = resolve_transitioning_snapshot_apply_mode(&payload).expect("transitioning mode");

        match mode {
            NativeSnapshotApplyMode::TransitioningCollapsed => {}
            NativeSnapshotApplyMode::TransitioningExpanded(_)
            | NativeSnapshotApplyMode::Static { .. } => {
                panic!("expected transitioning collapsed mode")
            }
        }
    }
}
