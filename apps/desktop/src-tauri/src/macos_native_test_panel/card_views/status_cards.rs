use std::time::Instant;

use echoisland_runtime::{PendingPermissionView, PendingQuestionView, SessionSnapshotView};
use objc2_app_kit::NSView;
use objc2_foundation::NSRect;

use super::super::card_animation::apply_card_exit_phase;
use super::super::panel_helpers::status_queue_exit_duration;
use super::super::panel_types::{NativeStatusQueueItem, NativeStatusQueuePayload};
use super::status_card_render::{render_completion_card_spec, render_pending_card_spec};
use super::status_card_specs::{
    build_pending_permission_card_spec, build_pending_question_card_spec,
    build_prompt_assist_card_spec, build_status_queue_completion_card_spec,
    build_status_queue_pending_card_spec,
};

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_pending_permission_card(
    frame: NSRect,
    pending: &PendingPermissionView,
    _waiting_count: usize,
) -> objc2::rc::Retained<NSView> {
    render_pending_card_spec(frame, build_pending_permission_card_spec(pending))
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_pending_question_card(
    frame: NSRect,
    pending: &PendingQuestionView,
    _waiting_count: usize,
) -> objc2::rc::Retained<NSView> {
    render_pending_card_spec(frame, build_pending_question_card_spec(pending))
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_status_queue_card(
    frame: NSRect,
    item: &NativeStatusQueueItem,
) -> objc2::rc::Retained<NSView> {
    match &item.payload {
        NativeStatusQueuePayload::Approval(_) => {
            render_pending_card_spec(frame, build_status_queue_pending_card_spec(item))
        }
        NativeStatusQueuePayload::Question(_) => {
            render_pending_card_spec(frame, build_status_queue_pending_card_spec(item))
        }
        NativeStatusQueuePayload::Completion(_) => {
            render_completion_card_spec(frame, build_status_queue_completion_card_spec(item))
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn apply_status_queue_item_visual_state(
    card: &NSView,
    item: &NativeStatusQueueItem,
) {
    if !item.is_removing {
        return;
    }

    let Some(remove_after) = item.remove_after else {
        return;
    };
    let exit_duration = status_queue_exit_duration();
    let elapsed =
        exit_duration.saturating_sub(remove_after.saturating_duration_since(Instant::now()));
    let progress = (elapsed.as_secs_f64() / exit_duration.as_secs_f64()).clamp(0.0, 1.0);
    apply_card_exit_phase(card, progress);
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_prompt_assist_card(
    frame: NSRect,
    session: &SessionSnapshotView,
) -> objc2::rc::Retained<NSView> {
    render_pending_card_spec(frame, build_prompt_assist_card_spec(session))
}
