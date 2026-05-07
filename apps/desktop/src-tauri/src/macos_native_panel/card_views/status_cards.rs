use std::time::Instant;

use objc2_app_kit::NSView;

use super::super::card_animation::apply_card_exit_phase;
use super::super::panel_helpers::status_queue_exit_duration;
use super::super::panel_types::NativeStatusQueueItem;

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
