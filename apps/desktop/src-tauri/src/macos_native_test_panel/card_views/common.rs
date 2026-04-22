use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::super::card_animation::register_card_animation_layout;
use super::super::panel_constants::{CARD_CHAT_PREFIX_WIDTH, CARD_INSET_X, CARD_RADIUS};
use super::super::panel_helpers::{card_chat_body_width, estimated_chat_body_height, ns_color};

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_empty_card(frame: NSRect) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 1.0, 1.0, 0.055], [1.0, 1.0, 1.0, 0.08]);
    register_card_animation_layout(&view, frame, 34.0);

    let label = make_label(
        mtm,
        "No sessions yet.",
        NSRect::new(NSPoint::new(0.0, 31.0), NSSize::new(frame.size.width, 20.0)),
        12.0,
        [0.67, 0.70, 0.76, 1.0],
        true,
        false,
    );
    view.addSubview(&label);
    view
}

#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn add_chat_line_with_max_lines_from_bottom(
    parent: &NSView,
    mtm: MainThreadMarker,
    bottom_y: f64,
    prefix: &str,
    body: &str,
    prefix_color: [f64; 4],
    body_color: [f64; 4],
    width: f64,
    max_lines: isize,
    gap_after: f64,
) -> f64 {
    let body_width = card_chat_body_width(width);
    let body_height = estimated_chat_body_height(body, body_width, max_lines);
    let prefix_y = bottom_y + body_height - 12.0;
    let prefix_label = make_label(
        mtm,
        prefix,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, prefix_y),
            NSSize::new(10.0, 12.0),
        ),
        10.0,
        prefix_color,
        true,
        true,
    );
    prefix_label.setFont(Some(&NSFont::boldSystemFontOfSize(10.0)));
    parent.addSubview(&prefix_label);

    let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(body), mtm);
    body_label.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X + CARD_CHAT_PREFIX_WIDTH, bottom_y),
        NSSize::new(body_width, body_height),
    ));
    body_label.setTextColor(Some(&ns_color(body_color)));
    body_label.setFont(Some(&NSFont::systemFontOfSize(10.0)));
    body_label.setDrawsBackground(false);
    body_label.setBezeled(false);
    body_label.setBordered(false);
    body_label.setEditable(false);
    body_label.setSelectable(false);
    body_label.setMaximumNumberOfLines(max_lines);
    parent.addSubview(&body_label);

    bottom_y + body_height + gap_after.max(0.0)
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn apply_card_layer(view: &NSView, background: [f64; 4], border: [f64; 4]) {
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        let background = ns_color(background);
        let border = ns_color(border);
        layer.setCornerRadius(CARD_RADIUS);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&background.CGColor()));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&border.CGColor()));
        layer.setShadowColor(Some(&NSColor::blackColor().CGColor()));
        layer.setShadowOpacity(0.0);
        layer.setShadowRadius(0.0);
        layer.setShadowOffset(NSSize::new(0.0, 0.0));
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn make_label(
    mtm: MainThreadMarker,
    text: &str,
    frame: NSRect,
    font_size: f64,
    color: [f64; 4],
    centered: bool,
    single_line: bool,
) -> objc2::rc::Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    label.setFrame(frame);
    if centered {
        label.setAlignment(NSTextAlignment::Center);
    }
    label.setTextColor(Some(&ns_color(color)));
    label.setFont(Some(&NSFont::systemFontOfSize(font_size)));
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
    if single_line {
        label.setUsesSingleLineMode(true);
    }
    label
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn make_badge_view(
    mtm: MainThreadMarker,
    text: &str,
    width: f64,
    background: [f64; 4],
    foreground: [f64; 4],
) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width.max(24.0), 22.0)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        let background = ns_color(background);
        layer.setCornerRadius(11.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&background.CGColor()));
    }

    let label = make_label(
        mtm,
        text,
        NSRect::new(
            NSPoint::new(7.0, 4.0),
            NSSize::new(width.max(24.0) - 14.0, 13.0),
        ),
        10.0,
        foreground,
        true,
        true,
    );
    label.setFont(Some(&NSFont::systemFontOfSize_weight(10.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    view.addSubview(&label);
    view
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn make_live_tool_view(
    mtm: MainThreadMarker,
    tool_name: &str,
    description: Option<&str>,
    width: f64,
) -> objc2::rc::Retained<NSView> {
    let width = width.max(36.0);
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, 22.0)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(5.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&ns_color([1.0, 1.0, 1.0, 0.04]).CGColor()));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&ns_color([1.0, 1.0, 1.0, 0.06]).CGColor()));
    }

    let name_width = badge_width(tool_name, 9.0, 0.0).min((width - 14.0).max(0.0));
    let name_label = make_label(
        mtm,
        tool_name,
        NSRect::new(NSPoint::new(7.0, 5.0), NSSize::new(name_width, 11.0)),
        9.0,
        tool_tone_color(tool_name),
        false,
        true,
    );
    name_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightBold },
    )));
    view.addSubview(&name_label);

    if let Some(description) = description.filter(|value| !value.trim().is_empty()) {
        let desc_x = 7.0 + name_width + 6.0;
        let desc_width = (width - desc_x - 7.0).max(0.0);
        let desc_label = make_label(
            mtm,
            description,
            NSRect::new(NSPoint::new(desc_x, 5.0), NSSize::new(desc_width, 11.0)),
            9.0,
            [1.0, 1.0, 1.0, 0.70],
            false,
            true,
        );
        desc_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            9.0,
            unsafe { objc2_app_kit::NSFontWeightRegular },
        )));
        view.addSubview(&desc_label);
    }

    view
}

pub(crate) fn tool_tone_color(tool: &str) -> [f64; 4] {
    match tool.to_ascii_lowercase().as_str() {
        "bash" => [0.49, 0.95, 0.64, 1.0],
        "edit" | "write" => [0.53, 0.67, 1.0, 1.0],
        "read" => [0.94, 0.82, 0.49, 1.0],
        "grep" | "glob" => [0.76, 0.63, 1.0, 1.0],
        "agent" => [1.0, 0.61, 0.40, 1.0],
        _ => [0.96, 0.97, 0.99, 0.86],
    }
}

pub(crate) fn badge_width(text: &str, font_size: f64, horizontal_padding: f64) -> f64 {
    (text.chars().count() as f64 * font_size * 0.58) + horizontal_padding
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn clear_subviews(view: &NSView) {
    let subviews = view.subviews();
    for index in 0..subviews.len() {
        subviews.objectAtIndex(index).removeFromSuperview();
    }
}
