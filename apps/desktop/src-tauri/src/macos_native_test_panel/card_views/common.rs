use objc2::MainThreadMarker;
use objc2_app_kit::{NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_foundation::{NSRect, NSSize, NSString};

use super::super::panel_constants::CARD_RADIUS;
use super::super::panel_helpers::ns_color;

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
pub(crate) unsafe fn clear_subviews(view: &NSView) {
    let subviews = view.subviews();
    for index in 0..subviews.len() {
        subviews.objectAtIndex(index).removeFromSuperview();
    }
}
