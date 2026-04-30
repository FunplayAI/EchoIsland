use std::time::Instant;

use crate::native_panel_core::{HoverTransition, PanelPoint};

use super::descriptors::{NativePanelPointerInput, NativePanelRuntimeInputDescriptor};
use super::runtime_interaction::{
    NativePanelCoreStateBridge, NativePanelHoverInteractionHost, NativePanelHoverSyncResult,
};
use super::runtime_scene_cache::NativePanelRuntimeSceneCache;
use super::runtime_scene_sync::rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor;
use super::traits::NativePanelSceneHost;
use super::transition_controller::{
    NativePanelTransitionRequest, native_panel_transition_request_for_hover_transition,
};

pub(crate) fn sync_native_panel_hover_expansion_state_for_state<S>(
    state: &mut S,
    inside: bool,
    now: Instant,
    hover_delay_ms: u64,
) -> Option<HoverTransition>
where
    S: NativePanelCoreStateBridge,
{
    let mut core = state.snapshot_core_panel_state();
    let transition = crate::native_panel_core::sync_hover_expansion_state(
        &mut core,
        inside,
        now,
        hover_delay_ms,
    );
    state.apply_core_panel_state(core);
    transition
}

pub(crate) fn sync_native_panel_hover_interaction_for_state<S>(
    state: &mut S,
    inside: bool,
    now: Instant,
    hover_delay_ms: u64,
) -> NativePanelHoverSyncResult
where
    S: NativePanelCoreStateBridge,
{
    let transition =
        sync_native_panel_hover_expansion_state_for_state(state, inside, now, hover_delay_ms);
    NativePanelHoverSyncResult {
        transition,
        request: native_panel_transition_request_for_hover_transition(transition),
    }
}

pub(crate) fn sync_native_panel_hover_interaction_at_point_for_state<S, H>(
    state: &mut S,
    host: &H,
    point: PanelPoint,
    now: Instant,
    hover_delay_ms: u64,
) -> Option<HoverTransition>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelHoverInteractionHost,
{
    let inside = host.hover_inside_at_point(point);
    sync_native_panel_hover_interaction_for_state(state, inside, now, hover_delay_ms).transition
}

pub(crate) fn sync_native_panel_hover_interaction_for_pointer_input_for_state<S, H>(
    state: &mut S,
    host: &H,
    pointer_input: NativePanelPointerInput,
    now: Instant,
    hover_delay_ms: u64,
) -> Option<HoverTransition>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelHoverInteractionHost,
{
    let inside = host.hover_inside_for_input(pointer_input)?;
    sync_native_panel_hover_interaction_for_state(state, inside, now, hover_delay_ms).transition
}

pub(crate) fn sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor<
    S,
    H,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    inside: bool,
    now: Instant,
    hover_delay_ms: u64,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<NativePanelHoverSyncResult, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost,
{
    let original_core = state.snapshot_core_panel_state();
    let hover_sync =
        sync_native_panel_hover_interaction_for_state(state, inside, now, hover_delay_ms);
    if hover_sync.request == Some(NativePanelTransitionRequest::Close) {
        let mut core = state.snapshot_core_panel_state();
        core.transitioning = true;
        state.apply_core_panel_state(core);
    }
    if hover_sync.transition.is_some() {
        match rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor(
            host, cache, state, input,
        ) {
            Ok(true) => {}
            Ok(false) => {
                state.apply_core_panel_state(original_core);
                return Ok(NativePanelHoverSyncResult {
                    transition: None,
                    request: None,
                });
            }
            Err(error) => {
                state.apply_core_panel_state(original_core);
                return Err(error);
            }
        }
    }
    Ok(hover_sync)
}

pub(crate) fn sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor<
    S,
    H,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    point: PanelPoint,
    now: Instant,
    hover_delay_ms: u64,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<NativePanelHoverSyncResult, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost + NativePanelHoverInteractionHost,
{
    let inside = host.hover_inside_at_point(point);
    sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor(
        host,
        cache,
        state,
        inside,
        now,
        hover_delay_ms,
        input,
    )
}

pub(crate) fn sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor<
    S,
    H,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    pointer_input: NativePanelPointerInput,
    now: Instant,
    hover_delay_ms: u64,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<Option<NativePanelHoverSyncResult>, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost + NativePanelHoverInteractionHost,
{
    let Some(inside) = host.hover_inside_for_input(pointer_input) else {
        return Ok(None);
    };
    sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor(
        host,
        cache,
        state,
        inside,
        now,
        hover_delay_ms,
        input,
    )
    .map(Some)
}
