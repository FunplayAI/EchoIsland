pub(super) const DEFAULT_PANEL_CANVAS_WIDTH: f64 =
    crate::native_panel_core::DEFAULT_PANEL_CANVAS_WIDTH;
pub(super) const COLLAPSED_PANEL_HEIGHT: f64 = crate::native_panel_core::COLLAPSED_PANEL_HEIGHT;
pub(super) const DEFAULT_COMPACT_PILL_WIDTH: f64 =
    crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH;
pub(super) const EXPANDED_PILL_WIDTH_DELTA: f64 =
    crate::native_panel_core::EXPANDED_PILL_WIDTH_DELTA;
pub(super) const DEFAULT_EXPANDED_PILL_WIDTH: f64 =
    crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH;
pub(super) const DEFAULT_COMPACT_PILL_HEIGHT: f64 =
    crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT;
pub(super) const COMPACT_SHOULDER_SIZE: f64 = crate::native_panel_core::COMPACT_SHOULDER_SIZE;
pub(super) const COMPACT_PILL_RADIUS: f64 = crate::native_panel_core::COMPACT_PILL_RADIUS;
pub(super) const COMPACT_HEADLINE_LABEL_HEIGHT: f64 = 24.0;
pub(super) const COMPACT_HEADLINE_VERTICAL_NUDGE_Y: f64 = -1.5;
pub(super) const EXPANDED_PANEL_RADIUS: f64 = crate::native_panel_core::EXPANDED_PANEL_RADIUS;
pub(super) const CARD_RADIUS: f64 = crate::native_panel_core::CARD_RADIUS;
pub(super) const SHOULDER_CURVE_FACTOR: f64 =
    crate::native_panel_core::COMPACT_SHOULDER_CURVE_FACTOR;
pub(super) const EXPANDED_MAX_BODY_HEIGHT: f64 = crate::native_panel_core::EXPANDED_MAX_BODY_HEIGHT;
pub(super) const EXPANDED_CARD_GAP: f64 = crate::native_panel_core::EXPANDED_CARD_GAP;
pub(super) const EXPANDED_CONTENT_TOP_GAP: f64 = crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP;
pub(super) const EXPANDED_CONTENT_BOTTOM_INSET: f64 =
    crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET;
pub(super) const EXPANDED_CARDS_SIDE_INSET: f64 =
    crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET;
pub(super) const EXPANDED_CARD_OVERHANG: f64 = crate::native_panel_core::EXPANDED_CARD_OVERHANG;
pub(super) const CARD_CHAT_GAP: f64 = crate::native_panel_core::CARD_CHAT_GAP;
pub(super) const CARD_TOOL_GAP: f64 = crate::native_panel_core::CARD_TOOL_GAP;
#[cfg(test)]
pub(super) const STATUS_COMPLETION_VISIBLE_SECONDS: u64 =
    crate::native_panel_core::STATUS_COMPLETION_VISIBLE_SECONDS;
#[cfg(test)]
pub(super) const STATUS_APPROVAL_VISIBLE_SECONDS: u64 =
    crate::native_panel_core::STATUS_APPROVAL_VISIBLE_SECONDS;
pub(super) const STATUS_QUEUE_EXIT_EXTRA_MS: u64 =
    crate::native_panel_core::STATUS_QUEUE_EXIT_EXTRA_MS;
pub(super) const STATUS_QUEUE_REFRESH_MS: u64 = crate::native_panel_core::STATUS_QUEUE_REFRESH_MS;
pub(super) const HOVER_POLL_MS: u64 = 32;
pub(super) const HOVER_DELAY_MS: u64 = crate::native_panel_core::HOVER_DELAY_MS;
#[cfg(test)]
pub(super) const PENDING_CARD_MIN_VISIBLE_MS: u64 =
    crate::native_panel_core::PENDING_CARD_MIN_VISIBLE_MS;
#[cfg(test)]
pub(super) const PENDING_CARD_RELEASE_GRACE_MS: u64 =
    crate::native_panel_core::PENDING_CARD_RELEASE_GRACE_MS;
#[cfg(test)]
pub(super) const PANEL_MORPH_DELAY_MS: u64 = crate::native_panel_core::PANEL_MORPH_DELAY_MS;
#[cfg(test)]
pub(super) const PANEL_MORPH_MS: u64 = crate::native_panel_core::PANEL_MORPH_MS;
#[cfg(test)]
pub(super) const PANEL_SHOULDER_HIDE_MS: u64 = crate::native_panel_core::PANEL_SHOULDER_HIDE_MS;
#[cfg(test)]
pub(super) const PANEL_HEIGHT_MS: u64 = crate::native_panel_core::PANEL_HEIGHT_MS;
#[cfg(test)]
pub(super) const PANEL_CLOSE_MORPH_DELAY_MS: u64 =
    crate::native_panel_core::PANEL_CLOSE_MORPH_DELAY_MS;
#[cfg(test)]
pub(super) const PANEL_CLOSE_SHOULDER_DELAY_MS: u64 =
    crate::native_panel_core::PANEL_CLOSE_SHOULDER_DELAY_MS;
pub(super) const PANEL_DROP_DISTANCE: f64 = crate::native_panel_core::PANEL_DROP_DISTANCE;
pub(super) const PANEL_MORPH_PILL_RADIUS: f64 = crate::native_panel_core::PANEL_MORPH_PILL_RADIUS;
pub(super) const PANEL_CARD_EXIT_MS: u64 = crate::native_panel_core::PANEL_CARD_EXIT_MS;
#[cfg(test)]
pub(super) const PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS: f64 =
    crate::native_panel_core::PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS;
pub(super) const PANEL_CARD_REVEAL_Y: f64 = crate::native_panel_core::PANEL_CARD_REVEAL_Y;
pub(super) const ACTIVE_COUNT_SLOT_WIDTH: f64 = crate::native_panel_core::ACTIVE_COUNT_SLOT_WIDTH;
pub(super) const ACTIVE_COUNT_SLOT_NUDGE_X: f64 =
    crate::native_panel_core::ACTIVE_COUNT_SLOT_NUDGE_X;
pub(super) const ACTIVE_COUNT_SCROLL_TRAVEL: f64 =
    crate::native_panel_core::ACTIVE_COUNT_SCROLL_TRAVEL;
pub(super) const ACTIVE_COUNT_TEXT_WIDTH: f64 = crate::native_panel_core::ACTIVE_COUNT_TEXT_WIDTH;
pub(super) const ACTIVE_COUNT_TEXT_OFFSET_X: f64 =
    crate::native_panel_core::ACTIVE_COUNT_TEXT_OFFSET_X;
pub(super) const ACTIVE_COUNT_LABEL_HEIGHT: f64 =
    crate::native_panel_core::ACTIVE_COUNT_LABEL_HEIGHT;
pub(super) const ACTIVE_COUNT_SCROLL_STEP_MS: u128 =
    crate::native_panel_core::ACTIVE_COUNT_SCROLL_STEP_MS;
pub(super) const ACTIVE_COUNT_SCROLL_HOLD_MS: u128 =
    crate::native_panel_core::ACTIVE_COUNT_SCROLL_HOLD_MS;
pub(super) const ACTIVE_COUNT_SCROLL_MOVE_MS: u128 =
    crate::native_panel_core::ACTIVE_COUNT_SCROLL_MOVE_MS;
pub(super) const ACTIVE_COUNT_SCROLL_REFRESH_MS: u64 =
    crate::native_panel_core::ACTIVE_COUNT_SCROLL_REFRESH_MS;
pub(super) const MASCOT_ANIMATION_REFRESH_MS: u64 =
    crate::native_panel_core::MASCOT_ANIMATION_REFRESH_MS;
pub(super) const MASCOT_STATE_TRANSITION_SECONDS: f64 =
    crate::native_panel_core::MASCOT_STATE_TRANSITION_SECONDS;
pub(super) const MASCOT_VERTICAL_NUDGE_Y: f64 = crate::native_panel_core::MASCOT_VERTICAL_NUDGE_Y;
pub(super) const CARD_FOCUS_CLICK_DEBOUNCE_MS: u128 =
    crate::native_panel_core::CARD_FOCUS_CLICK_DEBOUNCE_MS;
