use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSClipView, NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, ns_string};

use super::panel_constants::{
    ACTIVE_COUNT_LABEL_HEIGHT, ACTIVE_COUNT_SCROLL_TRAVEL, ACTIVE_COUNT_SLOT_NUDGE_X,
    ACTIVE_COUNT_SLOT_WIDTH, ACTIVE_COUNT_TEXT_OFFSET_X, ACTIVE_COUNT_TEXT_WIDTH,
    COMPACT_HEADLINE_LABEL_HEIGHT, COMPACT_HEADLINE_VERTICAL_NUDGE_Y,
};
use crate::native_panel_renderer::facade::presentation::NativePanelCompactBarCommand;

pub(super) struct CompactBarViews {
    pub(super) headline: Retained<NSTextField>,
    pub(super) active_count_clip: Retained<NSClipView>,
    pub(super) active_count: Retained<NSTextField>,
    pub(super) active_count_next: Retained<NSTextField>,
    pub(super) slash: Retained<NSTextField>,
    pub(super) total_count: Retained<NSTextField>,
}

#[derive(Clone, Copy)]
pub(super) struct NativeCompactStyle {
    pub(super) headline_color: [f64; 4],
    pub(super) active_count_color: [f64; 4],
    pub(super) total_count_color: [f64; 4],
}

pub(super) fn compact_style_for_command(
    command: &NativePanelCompactBarCommand,
) -> NativeCompactStyle {
    let active_count = command.active_count.parse::<usize>().unwrap_or_default();

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

pub(super) fn create_compact_bar_views(
    mtm: MainThreadMarker,
    pill_size: NSSize,
    hide_headline: bool,
    text_primary: &NSColor,
    accent_active: &NSColor,
) -> CompactBarViews {
    let metrics_trailing = 2.0;
    let metrics_gap = 0.0;
    let active_width = ACTIVE_COUNT_SLOT_WIDTH;
    let slash_width = 10.0;
    let total_width = 24.0;
    let total_x = pill_size.width - metrics_trailing - total_width;
    let slash_x = total_x - metrics_gap - slash_width;
    let active_x = slash_x - metrics_gap - active_width + ACTIVE_COUNT_SLOT_NUDGE_X;
    let digit_y = ((pill_size.height - ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 0.5;

    let headline = NSTextField::labelWithString(ns_string!("No active tasks"), mtm);
    headline.setFrame(NSRect::new(
        NSPoint::new(44.0, compact_headline_y(pill_size.height)),
        NSSize::new(136.0, COMPACT_HEADLINE_LABEL_HEIGHT),
    ));
    headline.setAlignment(NSTextAlignment::Center);
    headline.setTextColor(Some(&text_primary));
    headline.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    configure_label_text_field(&headline);
    headline.setHidden(hide_headline);

    let active_count_clip = NSClipView::initWithFrame(
        NSClipView::alloc(mtm),
        NSRect::new(
            NSPoint::new(active_x, digit_y),
            NSSize::new(active_width, ACTIVE_COUNT_LABEL_HEIGHT),
        ),
    );
    active_count_clip.setDrawsBackground(false);
    active_count_clip.setBackgroundColor(&NSColor::clearColor());

    let active_count_doc = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(
                ACTIVE_COUNT_SCROLL_TRAVEL + ACTIVE_COUNT_TEXT_WIDTH,
                ACTIVE_COUNT_LABEL_HEIGHT,
            ),
        ),
    );

    let active_count = create_active_count_label(mtm, "1", 0.0, accent_active);
    active_count_doc.addSubview(&active_count);

    let active_count_next =
        create_active_count_label(mtm, "2", ACTIVE_COUNT_SCROLL_TRAVEL, accent_active);
    active_count_doc.addSubview(&active_count_next);
    active_count_clip.setDocumentView(Some(&active_count_doc));

    let slash = NSTextField::labelWithString(ns_string!("/"), mtm);
    slash.setFrame(NSRect::new(
        NSPoint::new(slash_x, digit_y),
        NSSize::new(slash_width, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    slash.setAlignment(NSTextAlignment::Center);
    slash.setTextColor(Some(&text_primary));
    slash.setFont(Some(&NSFont::systemFontOfSize_weight(15.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    configure_label_text_field(&slash);

    let total_count = NSTextField::labelWithString(ns_string!("99"), mtm);
    total_count.setFrame(NSRect::new(
        NSPoint::new(total_x, digit_y),
        NSSize::new(total_width, 24.0),
    ));
    total_count.setAlignment(NSTextAlignment::Left);
    total_count.setTextColor(Some(&text_primary));
    total_count.setFont(Some(&NSFont::systemFontOfSize_weight(15.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    configure_label_text_field(&total_count);

    CompactBarViews {
        headline,
        active_count_clip,
        active_count,
        active_count_next,
        slash,
        total_count,
    }
}

fn create_active_count_label(
    mtm: MainThreadMarker,
    text: &str,
    travel_offset: f64,
    accent_active: &NSColor,
) -> Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    label.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_TEXT_OFFSET_X + travel_offset, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    label.setAlignment(NSTextAlignment::Right);
    label.setTextColor(Some(&accent_active));
    label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        15.0,
        unsafe { objc2_app_kit::NSFontWeightSemibold },
    )));
    configure_label_text_field(&label);
    label.setWantsLayer(true);
    label
}

fn configure_label_text_field(label: &NSTextField) {
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
}
