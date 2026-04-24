use super::mascot::NativeMascotState;
use super::panel_runtime_input::native_panel_runtime_input_descriptor;
use super::panel_scene_adapter::build_native_panel_scene_for_state_with_input;
use super::panel_types::{NativeExpandedSurface, NativePanelState, NativeStatusQueuePayload};

pub(super) struct NativeMascotFrameInput {
    pub(super) base_state: NativeMascotState,
    pub(super) expanded: bool,
    pub(super) completion_count: usize,
    pub(super) mascot_hidden: bool,
    pub(super) completion_glow_opacity: f64,
}

pub(super) fn resolve_native_mascot_frame_input(
    state: &NativePanelState,
) -> NativeMascotFrameInput {
    let snapshot = state.last_snapshot.clone();
    let input = native_panel_runtime_input_descriptor();
    let scene = snapshot
        .as_ref()
        .map(|snapshot| build_native_panel_scene_for_state_with_input(state, snapshot, &input));
    let has_status_completion = state.expanded
        && state.surface_mode == NativeExpandedSurface::Status
        && state
            .status_queue
            .iter()
            .any(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)));
    let has_completion_badge = !state.completion_badge_items.is_empty();
    let base_state =
        native_mascot_state_from_core(crate::native_panel_core::resolve_mascot_base_state(
            snapshot.as_ref(),
            has_status_completion,
            has_completion_badge,
        ));
    let completion_count = scene
        .as_ref()
        .map(|scene| scene.compact_bar.completion_count)
        .unwrap_or_else(|| state.completion_badge_items.len());

    NativeMascotFrameInput {
        base_state,
        expanded: state.expanded,
        completion_count,
        mascot_hidden: scene.as_ref().is_some_and(|scene| {
            scene.mascot_pose == crate::native_panel_scene::SceneMascotPose::Hidden
        }),
        completion_glow_opacity: scene
            .and_then(|scene| scene.glow.map(|glow| glow.opacity))
            .unwrap_or(0.0),
    }
}

fn native_mascot_state_from_core(
    state: crate::native_panel_core::PanelMascotBaseState,
) -> NativeMascotState {
    match state {
        crate::native_panel_core::PanelMascotBaseState::Idle => NativeMascotState::Idle,
        crate::native_panel_core::PanelMascotBaseState::Running => NativeMascotState::Bouncing,
        crate::native_panel_core::PanelMascotBaseState::Approval => NativeMascotState::Approval,
        crate::native_panel_core::PanelMascotBaseState::Question => NativeMascotState::Question,
        crate::native_panel_core::PanelMascotBaseState::MessageBubble => {
            NativeMascotState::MessageBubble
        }
        crate::native_panel_core::PanelMascotBaseState::Complete => NativeMascotState::Complete,
    }
}
