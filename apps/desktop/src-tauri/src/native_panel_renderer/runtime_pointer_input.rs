use std::time::Instant;

use super::descriptors::{
    NativePanelPointerInput, NativePanelPointerInputOutcome, NativePanelRuntimeCommandHandler,
    NativePanelRuntimeInputDescriptor,
};
use super::runtime_interaction::NativePanelPointerInputRuntimeBridge;

pub(crate) fn handle_native_panel_pointer_input_with_handler<S, H>(
    state: &mut S,
    input_event: NativePanelPointerInput,
    now: Instant,
    input: &NativePanelRuntimeInputDescriptor,
    handler: &mut H,
) -> Result<NativePanelPointerInputOutcome, S::Error>
where
    S: NativePanelPointerInputRuntimeBridge,
    H: NativePanelRuntimeCommandHandler<Error = S::Error>,
{
    state.sync_mouse_passthrough_for_pointer_input(input_event);
    state.record_pointer_input(input_event);

    match input_event {
        NativePanelPointerInput::Move(_) | NativePanelPointerInput::Leave => {
            Ok(NativePanelPointerInputOutcome::Hover(
                state.sync_hover_and_refresh_for_pointer_input(input_event, now, input)?,
            ))
        }
        NativePanelPointerInput::Click(point) => Ok(NativePanelPointerInputOutcome::Click(
            state.dispatch_click_command_for_pointer_point(point, now, handler)?,
        )),
    }
}

pub(crate) fn handle_optional_native_panel_pointer_input_with_handler<S, H>(
    state: &mut S,
    input_event: Option<NativePanelPointerInput>,
    now: Instant,
    input: &NativePanelRuntimeInputDescriptor,
    handler: &mut H,
) -> Result<Option<NativePanelPointerInputOutcome>, S::Error>
where
    S: NativePanelPointerInputRuntimeBridge,
    H: NativePanelRuntimeCommandHandler<Error = S::Error>,
{
    let Some(input_event) = input_event else {
        return Ok(None);
    };

    handle_native_panel_pointer_input_with_handler(state, input_event, now, input, handler)
        .map(Some)
}
