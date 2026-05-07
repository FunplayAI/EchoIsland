use std::time::Duration;

use objc2_app_kit::NSColor;

use super::card_animation::card_content_visibility_phase;
use super::panel_constants::{PANEL_CARD_EXIT_MS, STATUS_QUEUE_EXIT_EXTRA_MS};
use super::panel_refs::native_panel_state;

pub(super) fn native_panel_content_visibility() -> f64 {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                if guard.transitioning {
                    card_content_visibility_phase(
                        guard.transition_cards_progress,
                        guard.transition_cards_entering,
                    )
                } else if guard.expanded {
                    1.0
                } else {
                    0.0
                }
            })
        })
        .unwrap_or(0.0)
}

pub(super) fn estimated_chat_body_height(body: &str, width: f64, max_lines: isize) -> f64 {
    crate::native_panel_core::resolve_estimated_chat_body_height(
        body,
        width,
        max_lines,
        crate::native_panel_core::default_panel_card_metric_constants(),
    )
}

pub(super) fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}

pub(super) fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

pub(super) fn status_queue_exit_duration() -> Duration {
    Duration::from_millis(PANEL_CARD_EXIT_MS.max(220) + STATUS_QUEUE_EXIT_EXTRA_MS)
}

pub(super) fn ns_color(rgba: [f64; 4]) -> objc2::rc::Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(rgba[0], rgba[1], rgba[2], rgba[3])
}
