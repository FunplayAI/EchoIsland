use super::*;

#[derive(Clone, Copy)]
pub(super) struct NativeCompactStyle {
    pub(super) headline_color: [f64; 4],
    pub(super) active_count_color: [f64; 4],
    pub(super) total_count_color: [f64; 4],
}

pub(super) fn compact_active_count_value(snapshot: &RuntimeSnapshot) -> usize {
    snapshot
        .sessions
        .iter()
        .filter(|session| !should_hide_legacy_opencode_session(session))
        .filter(|session| normalize_status(&session.status) != "idle")
        .count()
}

pub(super) fn compact_active_count_text(snapshot: &RuntimeSnapshot) -> String {
    compact_active_count_value(snapshot).to_string()
}

pub(super) fn compact_style(snapshot: &RuntimeSnapshot) -> NativeCompactStyle {
    let active_count = compact_active_count_value(snapshot);

    NativeCompactStyle {
        headline_color: if active_count > 0 {
            [0.96, 0.97, 0.99, 1.0]
        } else {
            [0.90, 0.92, 0.96, 0.92]
        },
        active_count_color: if active_count > 0 {
            [0.40, 0.87, 0.57, 1.0]
        } else {
            [0.61, 0.65, 0.72, 1.0]
        },
        total_count_color: [0.96, 0.97, 0.99, 1.0],
    }
}

pub(super) fn compact_headline_y(bar_height: f64) -> f64 {
    ((bar_height - COMPACT_HEADLINE_LABEL_HEIGHT) / 2.0).round() + COMPACT_HEADLINE_VERTICAL_NUDGE_Y
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn compact_headline_should_hide(refs: &NativePanelRefs) -> bool {
    refs.panel
        .screen()
        .as_deref()
        .is_some_and(screen_has_camera_housing)
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn relayout_compact_content(refs: &NativePanelRefs, bar_size: NSSize) {
    let top_highlight = refs.top_highlight;
    let mascot_shell = refs.mascot_shell;
    let headline = refs.headline;
    let active_count_clip = refs.active_count_clip;
    let slash = refs.slash;
    let total_count = refs.total_count;

    let mascot_size = (bar_size.height - 9.0).clamp(24.0, 28.0);
    let left_inset = ((bar_size.height - mascot_size) / 2.0).clamp(8.0, 12.0);
    let title_x = left_inset + mascot_size + 8.0;
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
    let digit_y = ((bar_size.height - ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 0.5;
    let slash_y = digit_y;

    top_highlight.setFrame(NSRect::new(
        NSPoint::new(12.0, bar_size.height - 1.0),
        NSSize::new((bar_size.width - 24.0).max(0.0), 1.0),
    ));
    mascot_shell.setFrame(NSRect::new(
        NSPoint::new(
            left_inset,
            ((bar_size.height - mascot_size) / 2.0).round() + MASCOT_VERTICAL_NUDGE_Y,
        ),
        NSSize::new(mascot_size, mascot_size),
    ));
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
