use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::CALayer;

use super::panel_constants::{COLLAPSED_PANEL_HEIGHT, EXPANDED_PANEL_RADIUS};
use super::panel_geometry::{expanded_background_frame, expanded_cards_frame};

pub(super) fn create_content_view(mtm: MainThreadMarker, size: NSSize) -> Retained<NSView> {
    let content_view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), size),
    );
    content_view.setWantsLayer(true);
    let content_layer = CALayer::layer();
    content_layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    content_layer.setMasksToBounds(false);
    content_view.setLayer(Some(&content_layer));
    content_view
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_expanded_container(
    mtm: MainThreadMarker,
    size: NSSize,
    compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    pill_height: f64,
    expanded_background: &NSColor,
    expanded_border: &NSColor,
) -> (Retained<NSView>, Retained<NSView>) {
    let expanded_container = NSView::initWithFrame(
        NSView::alloc(mtm),
        expanded_background_frame(
            size,
            COLLAPSED_PANEL_HEIGHT,
            0.0,
            0.0,
            compact_width,
            expanded_width,
            compact_height,
            0.0,
        ),
    );
    expanded_container.setHidden(true);
    expanded_container.setWantsLayer(true);
    let expanded_layer = CALayer::layer();
    expanded_layer.setCornerRadius(EXPANDED_PANEL_RADIUS);
    expanded_layer.setMasksToBounds(true);
    expanded_layer.setBackgroundColor(Some(&expanded_background.CGColor()));
    expanded_layer.setBorderWidth(0.0);
    expanded_layer.setBorderColor(Some(&expanded_border.CGColor()));
    expanded_container.setLayer(Some(&expanded_layer));

    let cards_container = NSView::initWithFrame(
        NSView::alloc(mtm),
        expanded_cards_frame(expanded_container.frame(), pill_height),
    );
    expanded_container.addSubview(&cards_container);

    (expanded_container, cards_container)
}

pub(super) fn create_top_highlight(
    mtm: MainThreadMarker,
    pill_size: NSSize,
    pill_highlight: &NSColor,
) -> Retained<NSView> {
    let top_highlight = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(12.0, pill_size.height - 1.0),
            NSSize::new(pill_size.width - 24.0, 1.0),
        ),
    );
    top_highlight.setWantsLayer(true);
    let top_highlight_layer = CALayer::layer();
    top_highlight_layer.setCornerRadius(0.5);
    top_highlight_layer.setBackgroundColor(Some(&pill_highlight.CGColor()));
    top_highlight_layer.setOpacity(0.0);
    top_highlight.setLayer(Some(&top_highlight_layer));
    top_highlight.setHidden(true);
    top_highlight
}

pub(super) fn create_body_separator(
    mtm: MainThreadMarker,
    expanded_width: f64,
    separator_color: &NSColor,
) -> Retained<NSView> {
    let body_separator = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(14.0, 0.0),
            NSSize::new(expanded_width - 28.0, 1.0),
        ),
    );
    body_separator.setWantsLayer(true);
    let body_separator_layer = CALayer::layer();
    body_separator_layer.setCornerRadius(0.5);
    body_separator_layer.setBackgroundColor(Some(&separator_color.CGColor()));
    body_separator_layer.setOpacity(0.0);
    body_separator.setLayer(Some(&body_separator_layer));
    body_separator
}
