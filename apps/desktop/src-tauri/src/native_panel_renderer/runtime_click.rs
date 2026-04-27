use std::time::Instant;

use crate::native_panel_core::{
    PanelClickInput, PanelInteractionCommand, PanelPoint, resolve_panel_click_action,
};

use super::descriptors::{
    NativePanelPlatformEvent, NativePanelPointerPointState, NativePanelRuntimeCommandHandler,
    dispatch_native_panel_platform_event, dispatch_native_panel_platform_events,
    native_panel_platform_event_for_interaction_command,
};
use super::runtime_interaction::{
    NativePanelClickInteractionHost, NativePanelClickStateBridge,
    NativePanelQueuedPlatformEventSource,
};

pub(crate) fn resolve_native_panel_click_command_for_pointer_state<S>(
    state: &mut S,
    pointer_state: &NativePanelPointerPointState,
    primary_click_started: bool,
    cards_visible: bool,
    now: Instant,
    focus_debounce_ms: u128,
) -> PanelInteractionCommand
where
    S: NativePanelClickStateBridge,
{
    let settings_button_hit = matches!(
        pointer_state.platform_event,
        Some(NativePanelPlatformEvent::ToggleSettingsSurface)
    );
    let quit_button_hit = matches!(
        pointer_state.platform_event,
        Some(NativePanelPlatformEvent::QuitApplication)
    );
    let resolution = resolve_panel_click_action(PanelClickInput {
        primary_click_started,
        expanded: state.click_expanded(),
        transitioning: state.click_transitioning(),
        settings_button_hit,
        quit_button_hit,
        cards_visible,
        card_target: pointer_state.hit_target.clone(),
        last_focus_click: state.click_last_focus_click(),
        now,
        focus_debounce_ms,
    });
    if let Some(session_id) = resolution.focus_click_to_record {
        state.record_click_focus_session(session_id, now);
    }
    resolution.command
}

pub(crate) fn dispatch_native_panel_click_command_with_handler<H>(
    handler: &mut H,
    command: PanelInteractionCommand,
) -> Result<Option<NativePanelPlatformEvent>, H::Error>
where
    H: NativePanelRuntimeCommandHandler,
{
    let event = native_panel_platform_event_for_interaction_command(&command);
    if let Some(event) = event.clone() {
        dispatch_native_panel_platform_event(handler, event)?;
    }
    Ok(event)
}

pub(crate) fn dispatch_queued_native_panel_platform_events_with_handler<S, H>(
    source: &mut S,
    handler: &mut H,
) -> Result<(), H::Error>
where
    S: NativePanelQueuedPlatformEventSource,
    H: NativePanelRuntimeCommandHandler,
{
    dispatch_native_panel_platform_events(handler, source.take_queued_platform_events())
}

pub(crate) fn dispatch_native_panel_click_command_at_point_with_handler<S, P, H>(
    state: &mut S,
    host: &P,
    point: PanelPoint,
    now: Instant,
    focus_debounce_ms: u128,
    handler: &mut H,
) -> Result<Option<NativePanelPlatformEvent>, H::Error>
where
    S: NativePanelClickStateBridge,
    P: NativePanelClickInteractionHost,
    H: NativePanelRuntimeCommandHandler,
{
    let pointer_state = host.click_pointer_state_at_point(point);
    let cards_visible = host.click_cards_visible() || pointer_state.hit_target.is_some();
    let command = resolve_native_panel_click_command_for_pointer_state(
        state,
        &pointer_state,
        true,
        cards_visible,
        now,
        focus_debounce_ms,
    );
    dispatch_native_panel_click_command_with_handler(handler, command)
}
