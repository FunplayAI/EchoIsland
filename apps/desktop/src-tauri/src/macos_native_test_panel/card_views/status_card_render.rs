use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSFont, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::super::card_animation::register_card_animation_layout;
use super::super::panel_constants::{
    CARD_CHAT_PREFIX_WIDTH, CARD_CONTENT_BOTTOM_INSET, CARD_INSET_X, CARD_PENDING_ACTION_GAP,
    CARD_PENDING_ACTION_HEIGHT, CARD_PENDING_ACTION_Y,
};
use super::super::panel_helpers::{card_chat_body_width, estimated_chat_body_height, ns_color};
use super::common::{
    add_chat_line_with_max_lines_from_bottom, apply_card_layer, badge_width, make_badge_view,
    make_label,
};
use super::status_card_specs::{CompletionCardSpec, PendingCardSpec, StatusCardHeaderSpec};

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_card_view(
    mtm: MainThreadMarker,
    frame: NSRect,
    background: [f64; 4],
    border: [f64; 4],
    collapsed_height: f64,
) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, background, border);
    register_card_animation_layout(&view, frame, collapsed_height);
    view
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_status_card_header(
    view: &NSView,
    mtm: MainThreadMarker,
    frame: NSRect,
    spec: &StatusCardHeaderSpec,
) {
    let status_badge = make_badge_view(
        mtm,
        &spec.status_badge.text,
        badge_width(&spec.status_badge.text, 10.0, 16.0),
        spec.status_badge.background,
        spec.status_badge.foreground,
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let source_badge = make_badge_view(
        mtm,
        &spec.source_badge.text,
        badge_width(&spec.source_badge.text, 10.0, 16.0),
        spec.source_badge.background,
        spec.source_badge.foreground,
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let title_label = make_label(
        mtm,
        &spec.title,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let meta_label = make_label(
        mtm,
        &spec.meta,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_pending_card_spec(
    frame: NSRect,
    spec: PendingCardSpec,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = create_card_view(
        mtm,
        frame,
        spec.background,
        spec.border,
        spec.collapsed_height,
    );
    add_status_card_header(&view, mtm, frame, &spec.header);

    let body_width = card_chat_body_width(frame.size.width);
    let body_height = estimated_chat_body_height(&spec.body, body_width, 2);
    let header_bottom = frame.size.height - 40.0;
    let body_gap_from_header = 6.0;
    let body_origin_y = (header_bottom - body_gap_from_header - body_height)
        .max(CARD_PENDING_ACTION_Y + CARD_PENDING_ACTION_HEIGHT + CARD_PENDING_ACTION_GAP);
    let prefix_y = body_origin_y + body_height - 12.0;
    let prefix_label = make_label(
        mtm,
        spec.body_prefix,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, prefix_y),
            NSSize::new(10.0, 12.0),
        ),
        10.0,
        spec.body_prefix_color,
        true,
        true,
    );
    prefix_label.setFont(Some(&NSFont::boldSystemFontOfSize(10.0)));
    view.addSubview(&prefix_label);

    let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(&spec.body), mtm);
    body_label.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X + CARD_CHAT_PREFIX_WIDTH, body_origin_y),
        NSSize::new(body_width, body_height),
    ));
    body_label.setTextColor(Some(&ns_color([0.86, 0.88, 0.92, 0.78])));
    body_label.setFont(Some(&NSFont::systemFontOfSize(10.0)));
    body_label.setDrawsBackground(false);
    body_label.setBezeled(false);
    body_label.setBordered(false);
    body_label.setEditable(false);
    body_label.setSelectable(false);
    body_label.setMaximumNumberOfLines(2);
    view.addSubview(&body_label);

    let action_badge = make_badge_view(
        mtm,
        &spec.action_hint,
        badge_width(&spec.action_hint, 10.0, 18.0).min(frame.size.width - (CARD_INSET_X * 2.0)),
        [1.0, 1.0, 1.0, 0.07],
        [0.90, 0.92, 0.96, 0.86],
    );
    action_badge.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X, CARD_PENDING_ACTION_Y),
        action_badge.frame().size,
    ));
    view.addSubview(&action_badge);

    view
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_completion_card_spec(
    frame: NSRect,
    spec: CompletionCardSpec,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = create_card_view(
        mtm,
        frame,
        spec.background,
        spec.border,
        spec.collapsed_height,
    );
    add_status_card_header(&view, mtm, frame, &spec.header);
    add_chat_line_with_max_lines_from_bottom(
        &view,
        mtm,
        CARD_CONTENT_BOTTOM_INSET,
        spec.preview_prefix,
        &spec.preview,
        spec.preview_prefix_color,
        [0.86, 0.88, 0.92, 0.78],
        frame.size.width,
        2,
        0.0,
    );
    view
}
