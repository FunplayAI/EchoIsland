use crate::macos_shared_expanded_window;
use crate::notification_sound::play_message_card_sound;
use chrono::Utc;
use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;
use tracing::warn;

use super::card_animation::apply_card_stack_transition;
use super::panel_constants::COLLAPSED_PANEL_HEIGHT;
use super::panel_entry::native_ui_enabled;
use super::panel_geometry::expanded_total_height;
use super::panel_interaction::{native_settings_surface_active, native_status_surface_active};
use super::panel_refs::{native_panel_handles, native_panel_state};
use super::panel_render::apply_panel_geometry;
use super::panel_screen_geometry::compact_pill_height_for_screen_rect;
use super::panel_transition_entry::{
    begin_native_panel_surface_transition, begin_native_panel_transition,
};
use super::panel_types::{
    NativePanelHandles, NativePanelRenderPayload, NativePanelState, NativePanelTransitionFrame,
};
use super::panel_view_updates::apply_snapshot_values_to_panel;

#[cfg(test)]
use super::panel_types::NativeStatusQueueSyncResult;
use super::transition_ui::{
    render_transition_cards, reset_collapsed_cards, resolve_native_transition_context,
};

struct NativeSnapshotUpdate {
    snapshot: RuntimeSnapshot,
    play_message_sound: bool,
    transition_snapshot: Option<(RuntimeSnapshot, bool)>,
    surface_transition_snapshot: Option<RuntimeSnapshot>,
    apply_state: NativeSnapshotApplyState,
}

#[derive(Clone)]
struct NativeSnapshotApplyState {
    expanded: bool,
    shared_body_height: Option<f64>,
    transitioning: bool,
    transition_cards_progress: f64,
    transition_cards_entering: bool,
}

struct NativeSharedBodyHeightUpdate {
    rerender_payload: Option<NativePanelRenderPayload>,
}

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

    let Some(update) = sync_native_snapshot_update(snapshot)? else {
        return Ok(());
    };
    sync_shared_expanded_snapshot_if_needed(app, &update.snapshot);
    play_message_sound_if_needed(update.play_message_sound);
    dispatch_snapshot_update(app, handles, update)
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

    let Some(update) = sync_shared_expanded_body_height_update(body_height)? else {
        return Ok(());
    };
    dispatch_shared_expanded_body_height_update(app, handles, update)
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

    state.shared_body_height = Some(decision.next_height);
    Ok(Some(NativeSharedBodyHeightUpdate {
        rerender_payload: decision
            .should_rerender
            .then(|| native_panel_render_payload(&state))
            .flatten(),
    }))
}

fn dispatch_shared_expanded_body_height_update<R: tauri::Runtime>(
    app: &AppHandle<R>,
    handles: NativePanelHandles,
    update: NativeSharedBodyHeightUpdate,
) -> Result<(), String> {
    if let Some(payload) = update.rerender_payload {
        app.run_on_main_thread(move || unsafe {
            apply_native_panel_render_payload(handles, payload);
        })
        .map_err(|error| error.to_string())?;
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
    let mut core = state.to_core_panel_state();
    let sync_result =
        crate::native_panel_core::sync_panel_snapshot_state(&mut core, snapshot, Utc::now());
    state.apply_core_panel_state(core);

    let snapshot = sync_result.displayed_snapshot.clone();
    let update = NativeSnapshotUpdate {
        transition_snapshot: sync_result
            .panel_transition
            .map(|expanded| (snapshot.clone(), expanded)),
        surface_transition_snapshot: sync_result.surface_transition.then(|| snapshot.clone()),
        play_message_sound: sync_result.play_message_card_sound,
        apply_state: NativeSnapshotApplyState {
            expanded: state.expanded,
            shared_body_height: state.shared_body_height,
            transitioning: state.transitioning,
            transition_cards_progress: state.transition_cards_progress,
            transition_cards_entering: state.transition_cards_entering,
        },
        snapshot: snapshot.clone(),
    };
    state.last_snapshot = Some(snapshot);
    Ok(Some(update))
}

fn sync_shared_expanded_snapshot_if_needed<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) {
    if !macos_shared_expanded_window::shared_expanded_enabled() {
        return;
    }
    if let Err(error) = macos_shared_expanded_window::sync_shared_expanded_snapshot(app, snapshot) {
        warn!(error = %error, "failed to sync shared expanded snapshot");
    }
}

fn play_message_sound_if_needed(enabled: bool) {
    if enabled {
        play_message_card_sound();
    }
}

fn dispatch_snapshot_update<R: tauri::Runtime>(
    app: &AppHandle<R>,
    handles: NativePanelHandles,
    update: NativeSnapshotUpdate,
) -> Result<(), String> {
    if let Some((snapshot, expanded)) = update.transition_snapshot {
        let app_for_transition = app.clone();
        return app
            .run_on_main_thread(move || unsafe {
                begin_native_panel_transition(app_for_transition, handles, snapshot, expanded);
            })
            .map_err(|error| error.to_string());
    }

    if let Some(snapshot) = update.surface_transition_snapshot {
        let app_for_transition = app.clone();
        return app
            .run_on_main_thread(move || unsafe {
                begin_native_panel_surface_transition(app_for_transition, handles, snapshot);
            })
            .map_err(|error| error.to_string());
    }

    let apply_state = update.apply_state;
    let snapshot = update.snapshot;
    app.run_on_main_thread(move || unsafe {
        apply_snapshot_to_panel(
            handles,
            &snapshot,
            apply_state.expanded,
            apply_state.shared_body_height,
            apply_state.transitioning,
            apply_state.transition_cards_progress,
            apply_state.transition_cards_entering,
        );
    })
    .map_err(|error| error.to_string())
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

    if apply_transitioning_snapshot_to_panel(
        context,
        snapshot,
        expanded,
        transitioning,
        transition_cards_progress,
        transition_cards_entering,
    ) {
        return;
    }

    apply_static_snapshot_to_panel(handles, context, snapshot, expanded, shared_body_height);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_transitioning_snapshot_to_panel(
    context: super::transition_ui::NativeTransitionContext,
    snapshot: &RuntimeSnapshot,
    expanded: bool,
    transitioning: bool,
    transition_cards_progress: f64,
    transition_cards_entering: bool,
) -> bool {
    if !transitioning {
        return false;
    }

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

    true
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_static_snapshot_to_panel(
    handles: NativePanelHandles,
    context: super::transition_ui::NativeTransitionContext,
    snapshot: &RuntimeSnapshot,
    expanded: bool,
    shared_body_height: Option<f64>,
) {
    let total_height =
        resolved_snapshot_panel_height(context, snapshot, expanded, shared_body_height);
    apply_snapshot_panel_geometry(handles, expanded, total_height);

    if expanded {
        render_transition_cards(context, snapshot);
    } else {
        reset_collapsed_cards(context);
    }

    context.refs.panel.displayIfNeeded();
}

fn resolved_snapshot_panel_height(
    context: super::transition_ui::NativeTransitionContext,
    snapshot: &RuntimeSnapshot,
    expanded: bool,
    shared_body_height: Option<f64>,
) -> f64 {
    if !expanded {
        return COLLAPSED_PANEL_HEIGHT;
    }

    let shared_body_height = shared_expanded_body_height_for_snapshot(shared_body_height);
    expanded_total_height(
        snapshot,
        compact_pill_height_for_screen_rect(
            context.refs.panel.screen().as_deref(),
            context.screen_frame,
        ),
        shared_body_height,
    )
}

fn shared_expanded_body_height_for_snapshot(shared_body_height: Option<f64>) -> Option<f64> {
    if macos_shared_expanded_window::shared_expanded_enabled()
        && !native_status_surface_active()
        && !native_settings_surface_active()
    {
        shared_body_height
    } else {
        None
    }
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
