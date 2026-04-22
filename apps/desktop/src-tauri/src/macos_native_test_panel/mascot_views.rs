use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, ns_string};
use objc2_quartz_core::CALayer;

use super::panel_constants::MASCOT_VERTICAL_NUDGE_Y;

pub(super) struct MascotViews {
    pub(super) shell: Retained<NSView>,
    pub(super) body: Retained<NSView>,
    pub(super) left_eye: Retained<NSView>,
    pub(super) right_eye: Retained<NSView>,
    pub(super) mouth: Retained<NSView>,
    pub(super) bubble: Retained<NSView>,
    pub(super) sleep_label: Retained<NSTextField>,
    pub(super) completion_badge: Retained<NSView>,
    pub(super) completion_badge_label: Retained<NSTextField>,
}

pub(super) fn create_mascot_views(
    mtm: MainThreadMarker,
    shell_border: &NSColor,
    body_fill: &NSColor,
    stroke: &NSColor,
    face: &NSColor,
) -> MascotViews {
    let shell = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(10.0, 6.0 + MASCOT_VERTICAL_NUDGE_Y),
            NSSize::new(28.0, 28.0),
        ),
    );
    shell.setWantsLayer(true);
    let shell_layer = CALayer::layer();
    shell_layer.setCornerRadius(7.0);
    shell_layer.setMasksToBounds(false);
    shell_layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    shell_layer.setBorderWidth(0.0);
    shell_layer.setBorderColor(Some(&shell_border.CGColor()));
    shell.setLayer(Some(&shell_layer));

    let body = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(2.0, 4.0 + MASCOT_VERTICAL_NUDGE_Y),
            NSSize::new(24.0, 20.0),
        ),
    );
    body.setWantsLayer(true);
    let body_layer = CALayer::layer();
    body_layer.setCornerRadius(6.0);
    body_layer.setMasksToBounds(false);
    body_layer.setBackgroundColor(Some(&body_fill.CGColor()));
    body_layer.setBorderWidth(2.2);
    body_layer.setBorderColor(Some(&stroke.CGColor()));
    body_layer.setShadowColor(Some(&stroke.CGColor()));
    body_layer.setShadowOpacity(0.18);
    body_layer.setShadowRadius(4.0);
    body.setLayer(Some(&body_layer));
    shell.addSubview(&body);

    let left_eye = create_eye_view(mtm, face, 7.0);
    shell.addSubview(&left_eye);

    let right_eye = create_eye_view(mtm, face, 15.3);
    shell.addSubview(&right_eye);

    let mouth = create_mouth_view(mtm, face);
    shell.addSubview(&mouth);

    let bubble = create_bubble_view(mtm, face);
    shell.addSubview(&bubble);

    let sleep_label = create_sleep_label(mtm, face);
    shell.addSubview(&sleep_label);

    let (completion_badge, completion_badge_label) = create_completion_badge(mtm);
    shell.addSubview(&completion_badge);

    MascotViews {
        shell,
        body,
        left_eye,
        right_eye,
        mouth,
        bubble,
        sleep_label,
        completion_badge,
        completion_badge_label,
    }
}

fn create_eye_view(mtm: MainThreadMarker, face: &NSColor, x: f64) -> Retained<NSView> {
    let eye = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(x, 14.1), NSSize::new(5.7, 4.8)),
    );
    eye.setWantsLayer(true);
    let eye_layer = CALayer::layer();
    eye_layer.setCornerRadius(2.4);
    eye_layer.setMasksToBounds(true);
    eye_layer.setBackgroundColor(Some(&face.CGColor()));
    eye_layer.setShadowColor(Some(&face.CGColor()));
    eye_layer.setShadowOpacity(0.22);
    eye_layer.setShadowRadius(6.0);
    eye.setLayer(Some(&eye_layer));
    eye
}

fn create_mouth_view(mtm: MainThreadMarker, face: &NSColor) -> Retained<NSView> {
    let mouth = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(10.5, 9.0), NSSize::new(7.0, 2.2)),
    );
    mouth.setWantsLayer(true);
    let mouth_layer = CALayer::layer();
    mouth_layer.setCornerRadius(1.1);
    mouth_layer.setMasksToBounds(true);
    mouth_layer.setBackgroundColor(Some(&face.CGColor()));
    mouth_layer.setShadowColor(Some(&face.CGColor()));
    mouth_layer.setShadowOpacity(0.20);
    mouth_layer.setShadowRadius(6.0);
    mouth.setLayer(Some(&mouth_layer));
    mouth
}

fn create_bubble_view(mtm: MainThreadMarker, face: &NSColor) -> Retained<NSView> {
    let bubble = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(18.0, 19.5), NSSize::new(14.0, 7.5)),
    );
    bubble.setWantsLayer(true);
    let bubble_layer = CALayer::layer();
    bubble_layer.setCornerRadius(3.7);
    bubble_layer.setMasksToBounds(true);
    bubble_layer.setBackgroundColor(Some(&face.CGColor()));
    bubble_layer.setShadowColor(Some(&face.CGColor()));
    bubble_layer.setShadowOpacity(0.24);
    bubble_layer.setShadowRadius(7.0);
    bubble.setLayer(Some(&bubble_layer));
    bubble.setHidden(true);
    for x in [3.6, 6.8, 10.0] {
        let dot = NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(NSPoint::new(x, 3.0), NSSize::new(1.6, 1.6)),
        );
        dot.setWantsLayer(true);
        let dot_layer = CALayer::layer();
        dot_layer.setCornerRadius(0.8);
        dot_layer.setMasksToBounds(true);
        dot_layer.setBackgroundColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.02, 0.02, 0.02, 0.72).CGColor(),
        ));
        dot.setLayer(Some(&dot_layer));
        bubble.addSubview(&dot);
    }
    bubble
}

fn create_sleep_label(mtm: MainThreadMarker, face: &NSColor) -> Retained<NSTextField> {
    let sleep_label = NSTextField::labelWithString(ns_string!("Z"), mtm);
    sleep_label.setFrame(NSRect::new(
        NSPoint::new(20.0, 18.0),
        NSSize::new(10.0, 10.0),
    ));
    sleep_label.setAlignment(NSTextAlignment::Center);
    sleep_label.setTextColor(Some(&face));
    sleep_label.setFont(Some(&NSFont::boldSystemFontOfSize(8.0)));
    sleep_label.setDrawsBackground(false);
    sleep_label.setBezeled(false);
    sleep_label.setBordered(false);
    sleep_label.setEditable(false);
    sleep_label.setSelectable(false);
    sleep_label.setHidden(true);
    sleep_label
}

fn create_completion_badge(mtm: MainThreadMarker) -> (Retained<NSView>, Retained<NSTextField>) {
    let badge = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(21.0, 17.0), NSSize::new(13.0, 13.0)),
    );
    badge.setWantsLayer(true);
    if let Some(layer) = badge.layer() {
        layer.setCornerRadius(6.5);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.40, 0.87, 0.57, 1.0).CGColor(),
        ));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.94, 1.0, 0.96, 0.92).CGColor(),
        ));
        layer.setShadowColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.40, 0.87, 0.57, 1.0).CGColor(),
        ));
        layer.setShadowOpacity(0.30);
        layer.setShadowRadius(4.0);
    }
    badge.setHidden(true);

    let label = NSTextField::labelWithString(ns_string!("1"), mtm);
    label.setFrame(NSRect::new(NSPoint::new(0.0, 1.0), NSSize::new(13.0, 11.0)));
    label.setAlignment(NSTextAlignment::Center);
    label.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
        0.02, 0.02, 0.02, 0.90,
    )));
    label.setFont(Some(&NSFont::boldSystemFontOfSize(8.0)));
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
    badge.addSubview(&label);

    (badge, label)
}
