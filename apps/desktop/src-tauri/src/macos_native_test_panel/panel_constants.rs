pub(super) const DEFAULT_PANEL_CANVAS_WIDTH: f64 = 420.0;
pub(super) const COLLAPSED_PANEL_HEIGHT: f64 = 80.0;
pub(super) const DEFAULT_COMPACT_PILL_WIDTH: f64 = 253.0;
pub(super) const EXPANDED_PILL_WIDTH_DELTA: f64 = 30.0;
pub(super) const DEFAULT_EXPANDED_PILL_WIDTH: f64 =
    DEFAULT_COMPACT_PILL_WIDTH + EXPANDED_PILL_WIDTH_DELTA;
pub(super) const DEFAULT_COMPACT_PILL_HEIGHT: f64 = 37.0;
pub(super) const COMPACT_SHOULDER_SIZE: f64 = 6.0;
pub(super) const COMPACT_PILL_RADIUS: f64 = 12.5;
pub(super) const COMPACT_HEADLINE_LABEL_HEIGHT: f64 = 24.0;
pub(super) const COMPACT_HEADLINE_VERTICAL_NUDGE_Y: f64 = -1.5;
pub(super) const EXPANDED_PANEL_RADIUS: f64 = 12.0;
pub(super) const CARD_RADIUS: f64 = 9.0;
pub(super) const SHOULDER_CURVE_FACTOR: f64 = 0.62;
pub(super) const EXPANDED_MAX_BODY_HEIGHT: f64 = 420.0;
pub(super) const EXPANDED_CARD_GAP: f64 = 12.0;
pub(super) const EXPANDED_CONTENT_TOP_GAP: f64 = 9.0;
pub(super) const EXPANDED_CONTENT_BOTTOM_INSET: f64 = 10.0;
pub(super) const EXPANDED_CARDS_SIDE_INSET: f64 = 10.0;
pub(super) const EXPANDED_CARD_OVERHANG: f64 = 0.0;
pub(super) const CARD_INSET_X: f64 = 10.0;
pub(super) const CARD_HEADER_HEIGHT: f64 = 52.0;
pub(super) const CARD_CHAT_PREFIX_WIDTH: f64 = 15.0;
pub(super) const CARD_CHAT_LINE_HEIGHT: f64 = 14.0;
pub(super) const CARD_CONTENT_BOTTOM_INSET: f64 = 6.0;
pub(super) const CARD_CHAT_GAP: f64 = 4.0;
pub(super) const CARD_TOOL_GAP: f64 = 7.0;
pub(super) const CARD_PENDING_ACTION_Y: f64 = 9.0;
pub(super) const CARD_PENDING_ACTION_HEIGHT: f64 = 18.0;
pub(super) const CARD_PENDING_ACTION_GAP: f64 = 6.0;
pub(super) const PENDING_QUESTION_CARD_MIN_HEIGHT: f64 = 108.0;
pub(super) const PENDING_QUESTION_CARD_MAX_HEIGHT: f64 = 132.0;
#[cfg(test)]
pub(super) const STATUS_COMPLETION_VISIBLE_SECONDS: u64 = 10;
#[cfg(test)]
pub(super) const STATUS_APPROVAL_VISIBLE_SECONDS: u64 = 30;
pub(super) const STATUS_QUEUE_EXIT_EXTRA_MS: u64 = 80;
pub(super) const STATUS_QUEUE_REFRESH_MS: u64 = 33;
pub(super) const HOVER_POLL_MS: u64 = 120;
pub(super) const HOVER_DELAY_MS: u64 = 500;
#[cfg(test)]
pub(super) const PENDING_CARD_MIN_VISIBLE_MS: u64 = 2200;
#[cfg(test)]
pub(super) const PENDING_CARD_RELEASE_GRACE_MS: u64 = 1600;
pub(super) const PANEL_OPEN_TOTAL_MS: u64 = 660;
pub(super) const PANEL_CLOSE_TOTAL_MS: u64 = 660;
#[cfg(test)]
pub(super) const PANEL_MORPH_DELAY_MS: u64 = 120;
#[cfg(test)]
pub(super) const PANEL_MORPH_MS: u64 = 270;
#[cfg(test)]
pub(super) const PANEL_SHOULDER_HIDE_MS: u64 = 120;
#[cfg(test)]
pub(super) const PANEL_HEIGHT_MS: u64 = 270;
#[cfg(test)]
pub(super) const PANEL_CLOSE_MORPH_DELAY_MS: u64 = 270;
#[cfg(test)]
pub(super) const PANEL_CLOSE_SHOULDER_DELAY_MS: u64 = 540;
pub(super) const PANEL_DROP_DISTANCE: f64 = 4.5;
pub(super) const PANEL_MORPH_PILL_RADIUS: f64 = 14.0;
pub(super) const PANEL_ANIMATION_FRAME_MS: u64 = 16;
pub(super) const PANEL_CARD_REVEAL_MS: u64 = 320;
pub(super) const PANEL_CARD_REVEAL_STAGGER_MS: u64 = 70;
pub(super) const PANEL_CARD_EXIT_MS: u64 = 220;
pub(super) const PANEL_CARD_EXIT_STAGGER_MS: u64 = 70;
pub(super) const PANEL_CARD_EXIT_SETTLE_MS: u64 = 40;
pub(super) const PANEL_SURFACE_SWITCH_HEIGHT_MS: u64 = 220;
pub(super) const PANEL_SURFACE_SWITCH_CARD_REVEAL_MS: u64 = 220;
pub(super) const PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS: u64 = 42;
pub(super) const PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS: f64 = 0.24;
pub(super) const PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS: f64 = 0.18;
pub(super) const PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS: f64 = 0.30;
pub(super) const PANEL_CARD_REVEAL_Y: f64 = -8.0;
pub(super) const ACTIVE_COUNT_SLOT_WIDTH: f64 = 11.0;
pub(super) const ACTIVE_COUNT_SLOT_NUDGE_X: f64 = 2.0;
pub(super) const ACTIVE_COUNT_SCROLL_TRAVEL: f64 = 10.0;
pub(super) const ACTIVE_COUNT_TEXT_WIDTH: f64 = 22.0;
pub(super) const ACTIVE_COUNT_TEXT_OFFSET_X: f64 =
    ACTIVE_COUNT_SLOT_WIDTH - ACTIVE_COUNT_TEXT_WIDTH;
pub(super) const ACTIVE_COUNT_LABEL_HEIGHT: f64 = 24.0;
pub(super) const ACTIVE_COUNT_SCROLL_STEP_MS: u128 = 2300;
pub(super) const ACTIVE_COUNT_SCROLL_HOLD_MS: u128 = 1000;
pub(super) const ACTIVE_COUNT_SCROLL_MOVE_MS: u128 = 300;
pub(super) const ACTIVE_COUNT_SCROLL_REFRESH_MS: u64 = 33;
pub(super) const MASCOT_ANIMATION_REFRESH_MS: u64 = 33;
pub(super) const MASCOT_STATE_TRANSITION_SECONDS: f64 = 0.24;
pub(super) const MASCOT_IDLE_LONG_SECONDS: u64 = 120;
pub(super) const MASCOT_WAKE_ANGRY_SECONDS: f64 = 0.82;
pub(super) const MASCOT_VERTICAL_NUDGE_Y: f64 = -2.0;
pub(super) const CARD_FOCUS_CLICK_DEBOUNCE_MS: u128 = 600;
