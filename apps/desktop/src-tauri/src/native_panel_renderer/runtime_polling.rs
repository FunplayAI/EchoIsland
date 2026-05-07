use echoisland_runtime::RuntimeSnapshot;
use std::time::Instant;

use crate::native_panel_core::{PanelPoint, PanelRect, point_in_rect};

use super::descriptors::{
    NativePanelInteractionPlan, NativePanelPlatformEvent, NativePanelPointerPointState,
};
use super::runtime_click::resolve_native_panel_click_command_for_pointer_state;
use super::runtime_hover::sync_native_panel_hover_interaction_for_state;
use super::runtime_interaction::{
    NativePanelClickStateBridge, NativePanelCoreStateBridge, NativePanelHostBehaviorCommand,
    NativePanelHostBehaviorPlan, NativePanelHostInteractionState,
    NativePanelHostInteractionStateBridge, NativePanelHostPollingInteractionResult,
    NativePanelHoverFallbackFrames, NativePanelHoverFallbackState, NativePanelPollingHostFacts,
    NativePanelPollingInteractionInput, NativePanelPollingInteractionResult,
    NativePanelPrimaryPointerStateBridge,
};
use super::transition_controller::native_panel_transition_request_for_surface_change;
use super::visual_plan::{NativePanelVisualDisplayMode, NativePanelVisualPlanInput};

pub(crate) fn resolve_native_panel_hover_fallback_frames(
    input: &NativePanelVisualPlanInput,
) -> NativePanelHoverFallbackFrames {
    let interactive_pill_frame =
        non_zero_rect(input.compact_bar_frame).unwrap_or(input.panel_frame);
    let hover_pill_frame = non_zero_rect(input.panel_frame).unwrap_or(interactive_pill_frame);
    let interactive_expanded_frame = (input.display_mode == NativePanelVisualDisplayMode::Expanded)
        .then(|| non_zero_rect(input.shell_frame))
        .flatten();

    NativePanelHoverFallbackFrames {
        interactive_pill_frame,
        hover_pill_frame,
        interactive_expanded_frame,
    }
}

pub(crate) fn resolve_native_panel_hover_fallback_state(
    point: PanelPoint,
    frames: NativePanelHoverFallbackFrames,
) -> NativePanelHoverFallbackState {
    let interactive_expanded_inside = frames
        .interactive_expanded_frame
        .is_some_and(|frame| point_in_rect(point, frame));
    let interactive_inside =
        interactive_expanded_inside || point_in_rect(point, frames.interactive_pill_frame);
    let hover_inside = interactive_expanded_inside || point_in_rect(point, frames.hover_pill_frame);

    NativePanelHoverFallbackState {
        interactive_inside,
        hover_inside,
    }
}

pub(crate) fn resolve_native_panel_host_interaction_state(
    interactive_inside: bool,
) -> NativePanelHostInteractionState {
    NativePanelHostInteractionState {
        interactive_inside,
        ignores_mouse_events: !interactive_inside,
    }
}

pub(crate) fn resolve_native_panel_host_behavior_plan(
    current_ignores_mouse_events: bool,
    interactive_inside: bool,
) -> NativePanelHostBehaviorPlan {
    let state = resolve_native_panel_host_interaction_state(interactive_inside);
    let commands = (current_ignores_mouse_events != state.ignores_mouse_events)
        .then_some(NativePanelHostBehaviorCommand::SetMouseEventPassthrough {
            ignores_mouse_events: state.ignores_mouse_events,
        })
        .into_iter()
        .collect();

    NativePanelHostBehaviorPlan {
        interactive_inside: state.interactive_inside,
        ignores_mouse_events: state.ignores_mouse_events,
        commands,
    }
}

fn non_zero_rect(rect: PanelRect) -> Option<PanelRect> {
    (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

pub(crate) fn native_panel_interactive_inside_for_polling_input(
    input: &NativePanelPollingInteractionInput,
) -> bool {
    if input.pointer_regions_available {
        input.pointer_state.inside
    } else {
        input.fallback_hover.interactive_inside
    }
}

pub(crate) fn sync_native_panel_mouse_passthrough_for_interactive_inside<S>(
    state: &mut S,
    interactive_inside: bool,
) -> Option<bool>
where
    S: NativePanelHostInteractionStateBridge,
{
    let plan = sync_native_panel_host_behavior_for_interactive_inside(state, interactive_inside);
    plan.mouse_event_passthrough_target()
}

pub(crate) fn sync_native_panel_host_behavior_for_interactive_inside<S>(
    state: &mut S,
    interactive_inside: bool,
) -> NativePanelHostBehaviorPlan
where
    S: NativePanelHostInteractionStateBridge,
{
    let plan = resolve_native_panel_host_behavior_plan(
        state.host_ignores_mouse_events(),
        interactive_inside,
    );
    state.set_host_ignores_mouse_events(plan.ignores_mouse_events);
    plan
}

pub(crate) fn native_panel_polling_interaction_input(
    pointer_state: NativePanelPointerPointState,
    pointer_regions_available: bool,
    fallback_hover: NativePanelHoverFallbackState,
    primary_mouse_down: bool,
    cards_visible: bool,
    snapshot: Option<RuntimeSnapshot>,
) -> NativePanelPollingInteractionInput {
    NativePanelPollingInteractionInput {
        pointer_state,
        pointer_regions_available,
        fallback_hover,
        primary_mouse_down,
        cards_visible,
        snapshot,
    }
}

pub(crate) fn native_panel_polling_interaction_input_from_host_facts(
    facts: NativePanelPollingHostFacts<'_>,
) -> NativePanelPollingInteractionInput {
    let interaction_plan = NativePanelInteractionPlan::from_pointer_regions(facts.pointer_regions);
    native_panel_polling_interaction_input(
        interaction_plan.pointer_state_at_point(facts.pointer),
        !facts.pointer_regions.is_empty(),
        resolve_native_panel_hover_fallback_state(facts.pointer, facts.hover_frames),
        facts.primary_mouse_down,
        facts.cards_visible,
        facts.snapshot,
    )
}

pub(crate) fn native_panel_interactive_inside_from_host_facts(
    facts: NativePanelPollingHostFacts<'_>,
) -> bool {
    let input = native_panel_polling_interaction_input_from_host_facts(facts);
    native_panel_interactive_inside_for_polling_input(&input)
}

pub(crate) fn sync_native_panel_polling_interaction_for_state<S>(
    state: &mut S,
    input: NativePanelPollingInteractionInput,
    now: Instant,
    hover_delay_ms: u64,
    focus_debounce_ms: u128,
) -> NativePanelPollingInteractionResult
where
    S: NativePanelCoreStateBridge
        + NativePanelClickStateBridge
        + NativePanelPrimaryPointerStateBridge,
{
    let interactive_inside = if input.pointer_regions_available {
        input.pointer_state.inside
    } else {
        input.fallback_hover.interactive_inside
    };
    let hover_inside = interactive_inside || input.fallback_hover.hover_inside;
    let primary_click_started =
        input.primary_mouse_down && !state.primary_pointer_down() && interactive_inside;

    let click_event = primary_click_started
        .then(|| input.pointer_state.platform_event.clone())
        .flatten();
    let settings_clicked = matches!(
        click_event,
        Some(NativePanelPlatformEvent::ToggleSettingsSurface)
    );
    let quit_clicked = matches!(click_event, Some(NativePanelPlatformEvent::QuitApplication));
    let click_pointer_state = if primary_click_started
        && state.click_expanded()
        && !state.click_transitioning()
        && !settings_clicked
        && !quit_clicked
    {
        input.pointer_state.clone()
    } else {
        NativePanelPointerPointState {
            inside: input.pointer_state.inside,
            platform_event: input.pointer_state.platform_event.clone(),
            hit_target: None,
        }
    };
    let click_command = resolve_native_panel_click_command_for_pointer_state(
        state,
        &click_pointer_state,
        primary_click_started,
        input.cards_visible,
        now,
        focus_debounce_ms,
    );
    state.set_primary_pointer_down(input.primary_mouse_down);

    let previous_core = state.snapshot_core_panel_state();
    let hover_sync =
        sync_native_panel_hover_interaction_for_state(state, hover_inside, now, hover_delay_ms);

    let (transition_request, transition_snapshot) = if let Some(request) = hover_sync.request {
        if let Some(snapshot) = input.snapshot.clone() {
            (Some(request), Some(snapshot))
        } else {
            state.apply_core_panel_state(previous_core);
            (None, None)
        }
    } else {
        let current_core = state.snapshot_core_panel_state();
        let was_status_surface = previous_core.surface_mode
            == crate::native_panel_core::ExpandedSurface::Status
            && !previous_core.status_queue.is_empty();
        let is_status_surface = current_core.surface_mode
            == crate::native_panel_core::ExpandedSurface::Status
            && !current_core.status_queue.is_empty();
        let surface_changed = was_status_surface != is_status_surface
            || previous_core.surface_mode != current_core.surface_mode;
        let request = native_panel_transition_request_for_surface_change(
            surface_changed,
            current_core.expanded,
            current_core.transitioning,
        );
        let snapshot = request.and(input.snapshot);
        (request, snapshot)
    };

    NativePanelPollingInteractionResult {
        interactive_inside,
        click_platform_event: click_event,
        click_command,
        transition_request,
        transition_snapshot,
    }
}

pub(crate) fn sync_native_panel_host_polling_interaction_for_state<S>(
    state: &mut S,
    input: NativePanelPollingInteractionInput,
    now: Instant,
    hover_delay_ms: u64,
    focus_debounce_ms: u128,
) -> NativePanelHostPollingInteractionResult
where
    S: NativePanelCoreStateBridge
        + NativePanelClickStateBridge
        + NativePanelPrimaryPointerStateBridge
        + NativePanelHostInteractionStateBridge,
{
    let interaction = sync_native_panel_polling_interaction_for_state(
        state,
        input,
        now,
        hover_delay_ms,
        focus_debounce_ms,
    );
    let host_behavior = sync_native_panel_host_behavior_for_interactive_inside(
        state,
        interaction.interactive_inside,
    );
    let next_ignores_mouse_events = host_behavior.ignores_mouse_events;
    let sync_mouse_event_passthrough = host_behavior.sync_mouse_event_passthrough();

    NativePanelHostPollingInteractionResult {
        interactive_inside: interaction.interactive_inside,
        click_platform_event: interaction.click_platform_event,
        click_command: interaction.click_command,
        transition_request: interaction.transition_request,
        transition_snapshot: interaction.transition_snapshot,
        host_behavior,
        next_ignores_mouse_events,
        sync_mouse_event_passthrough,
    }
}

pub(crate) fn sync_native_panel_host_polling_interaction_from_host_facts_for_state<S>(
    state: &mut S,
    facts: NativePanelPollingHostFacts<'_>,
    now: Instant,
    hover_delay_ms: u64,
    focus_debounce_ms: u128,
) -> NativePanelHostPollingInteractionResult
where
    S: NativePanelCoreStateBridge
        + NativePanelClickStateBridge
        + NativePanelPrimaryPointerStateBridge
        + NativePanelHostInteractionStateBridge,
{
    let input = native_panel_polling_interaction_input_from_host_facts(facts);
    sync_native_panel_host_polling_interaction_for_state(
        state,
        input,
        now,
        hover_delay_ms,
        focus_debounce_ms,
    )
}
