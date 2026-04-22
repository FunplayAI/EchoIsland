use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

pub(super) fn text_primary_color() -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(0.96, 0.97, 0.99, 0.88)
}

pub(super) fn close_action_color() -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.32, 0.32, 0.95)
}

pub(super) fn create_edge_action_button(
    mtm: MainThreadMarker,
    label: &str,
    text_color: Retained<NSColor>,
    font_size: f64,
    label_y: f64,
) -> Retained<NSView> {
    let button = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(26.0, 26.0)),
    );
    button.setWantsLayer(true);
    if let Some(layer) = button.layer() {
        layer.setCornerRadius(0.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
        layer.setBorderWidth(0.0);
    }
    button.setHidden(true);
    button.setAlphaValue(0.0);

    let label = NSTextField::labelWithString(&NSString::from_str(label), mtm);
    label.setFrame(NSRect::new(
        NSPoint::new(0.0, label_y),
        NSSize::new(26.0, 20.0),
    ));
    label.setAlignment(NSTextAlignment::Center);
    label.setTextColor(Some(&text_color));
    label.setFont(Some(&NSFont::boldSystemFontOfSize(font_size)));
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
    button.addSubview(&label);

    button
}
