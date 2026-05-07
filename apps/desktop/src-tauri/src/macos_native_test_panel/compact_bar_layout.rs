use std::time::Instant;

use objc2_app_kit::NSTextAlignment;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::compact_bar::compact_headline_y;
use super::completion_glow_view::update_completion_glow_layout;
use super::panel_constants::{
    ACTIVE_COUNT_LABEL_HEIGHT, ACTIVE_COUNT_SCROLL_HOLD_MS, ACTIVE_COUNT_SCROLL_MOVE_MS,
    ACTIVE_COUNT_SCROLL_STEP_MS, ACTIVE_COUNT_SCROLL_TRAVEL, ACTIVE_COUNT_SLOT_NUDGE_X,
    ACTIVE_COUNT_SLOT_WIDTH, ACTIVE_COUNT_TEXT_OFFSET_X, ACTIVE_COUNT_TEXT_WIDTH,
    COMPACT_HEADLINE_LABEL_HEIGHT, MASCOT_VERTICAL_NUDGE_Y,
};
use super::panel_globals::{ACTIVE_COUNT_SCROLL_STARTED, ACTIVE_COUNT_SCROLL_TEXT};
use super::panel_refs::NativePanelRefs;
use super::panel_screen_geometry::screen_has_camera_housing;
use crate::native_panel_core::{PanelRect, resolve_compact_action_button_layout};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn compact_headline_should_hide(refs: &NativePanelRefs) -> bool {
    refs.panel
        .screen()
        .as_deref()
        .is_some_and(screen_has_camera_housing)
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn relayout_compact_content(
    refs: &NativePanelRefs,
    bar_size: NSSize,
    actions_active: bool,
) {
    let mascot_enabled = crate::app_settings::current_app_settings().mascot_enabled;
    let top_highlight = refs.top_highlight;
    let completion_glow = refs.completion_glow;
    let settings_button = refs.settings_button;
    let quit_button = refs.quit_button;
    let mascot_shell = refs.mascot_shell;
    let headline = refs.headline;
    let active_count_clip = refs.active_count_clip;
    let slash = refs.slash;
    let total_count = refs.total_count;
    let mascot_size = if mascot_enabled {
        (bar_size.height - 9.0).clamp(24.0, 28.0)
    } else {
        0.0
    };
    let left_inset = ((bar_size.height - mascot_size) / 2.0).clamp(8.0, 12.0);
    let compact_frame = PanelRect {
        x: 0.0,
        y: 0.0,
        width: bar_size.width,
        height: bar_size.height,
    };
    let action_layout = resolve_compact_action_button_layout(compact_frame);
    let settings_x = action_layout.settings.x;
    let title_x = if actions_active {
        settings_x + action_layout.settings.width + 8.0
    } else {
        left_inset + mascot_size + 8.0
    };
    let right_padding = 4.0;
    let active_width = ACTIVE_COUNT_SLOT_WIDTH;
    let slash_width = 10.0;
    let total_width = 24.0;
    let metrics_gap = 0.0;
    let group_right = (bar_size.width - right_padding).max(208.0);
    let total_x = group_right - total_width;
    let slash_x = total_x - metrics_gap - slash_width;
    let right_start = (slash_x - metrics_gap - active_width + ACTIVE_COUNT_SLOT_NUDGE_X).max(168.0);
    let headline_width = (right_start - title_x - 8.0).max(96.0);
    let digit_y = ((bar_size.height - ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 1.5;
    let slash_y = digit_y;

    top_highlight.setFrame(NSRect::new(
        NSPoint::new(12.0, bar_size.height - 1.0),
        NSSize::new((bar_size.width - 24.0).max(0.0), 1.0),
    ));
    completion_glow.setFrame(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(bar_size.width.max(0.0), bar_size.height.max(0.0)),
    ));
    update_completion_glow_layout(completion_glow, bar_size);
    settings_button.setFrame(NSRect::new(
        NSPoint::new(action_layout.settings.x, action_layout.settings.y),
        NSSize::new(action_layout.settings.width, action_layout.settings.height),
    ));
    quit_button.setFrame(NSRect::new(
        NSPoint::new(action_layout.quit.x, action_layout.quit.y),
        NSSize::new(action_layout.quit.width, action_layout.quit.height),
    ));
    mascot_shell.setFrame(NSRect::new(
        NSPoint::new(
            left_inset,
            ((bar_size.height - mascot_size) / 2.0).round() + MASCOT_VERTICAL_NUDGE_Y,
        ),
        NSSize::new(mascot_size, mascot_size),
    ));
    mascot_shell.setHidden(!mascot_enabled);
    headline.setFrame(NSRect::new(
        NSPoint::new(title_x, compact_headline_y(bar_size.height)),
        NSSize::new(headline_width, COMPACT_HEADLINE_LABEL_HEIGHT),
    ));
    headline.setHidden(compact_headline_should_hide(refs));
    active_count_clip.setFrame(NSRect::new(
        NSPoint::new(right_start, digit_y),
        NSSize::new(active_width, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    slash.setFrame(NSRect::new(
        NSPoint::new(slash_x, slash_y),
        NSSize::new(slash_width, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    total_count.setFrame(NSRect::new(
        NSPoint::new(total_x, digit_y),
        NSSize::new(total_width, 24.0),
    ));
    sync_active_count_marquee(refs);
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn sync_active_count_marquee(refs: &NativePanelRefs) {
    let active_count_clip = refs.active_count_clip;
    let active_count = refs.active_count;
    let active_count_next = refs.active_count_next;
    let Some(source) = ACTIVE_COUNT_SCROLL_TEXT.get() else {
        active_count.setStringValue(&NSString::from_str("9"));
        active_count_next.setStringValue(&NSString::from_str("9"));
        return;
    };
    let value = source
        .lock()
        .ok()
        .map(|text| text.clone())
        .unwrap_or_else(|| "9".to_string());
    let chars = value.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        active_count.setHidden(false);
        active_count_next.setHidden(true);
        active_count.setStringValue(&NSString::from_str("0"));
        active_count_next.setStringValue(&NSString::from_str("0"));
        active_count_clip.scrollToPoint(NSPoint::new(0.0, 0.0));
        return;
    }

    let started = ACTIVE_COUNT_SCROLL_STARTED.get_or_init(Instant::now);
    let elapsed = started.elapsed().as_millis();
    let current = chars[0].to_string();
    let next = chars.get(1).copied().unwrap_or(chars[0]).to_string();

    let phase = if chars.len() < 2 {
        0.0
    } else {
        let step_elapsed = elapsed % ACTIVE_COUNT_SCROLL_STEP_MS;
        if step_elapsed < ACTIVE_COUNT_SCROLL_HOLD_MS {
            0.0
        } else if step_elapsed < ACTIVE_COUNT_SCROLL_HOLD_MS + ACTIVE_COUNT_SCROLL_MOVE_MS {
            ((step_elapsed - ACTIVE_COUNT_SCROLL_HOLD_MS) as f64
                / ACTIVE_COUNT_SCROLL_MOVE_MS as f64)
                .clamp(0.0, 1.0)
        } else {
            1.0
        }
    };
    let current_x = -(ACTIVE_COUNT_SCROLL_TRAVEL * phase).round();

    active_count.setAlignment(NSTextAlignment::Right);
    active_count_next.setAlignment(NSTextAlignment::Right);
    active_count.setHidden(false);
    active_count_next.setHidden(chars.len() < 2);
    active_count.setStringValue(&NSString::from_str(&current));
    active_count_next.setStringValue(&NSString::from_str(&next));
    active_count.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    active_count_next.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_SCROLL_TRAVEL + ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    active_count_clip.scrollToPoint(NSPoint::new(-current_x, 0.0));
}
