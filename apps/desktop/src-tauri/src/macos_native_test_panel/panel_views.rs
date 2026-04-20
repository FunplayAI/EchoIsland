use super::panel_shoulder;
use super::panel_style;
use super::*;

use objc2::rc::Retained;

pub(super) struct PanelBaseViews {
    pub(super) content_view: Retained<NSView>,
    pub(super) left_shoulder: Retained<NSView>,
    pub(super) right_shoulder: Retained<NSView>,
    pub(super) pill_view: Retained<NSView>,
    pub(super) expanded_container: Retained<NSView>,
    pub(super) cards_container: Retained<NSView>,
    pub(super) top_highlight: Retained<NSView>,
    pub(super) body_separator: Retained<NSView>,
    pub(super) settings_button: Retained<NSView>,
    pub(super) quit_button: Retained<NSView>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_panel_base_views(
    mtm: MainThreadMarker,
    size: NSSize,
    pill_frame: NSRect,
    pill_size: NSSize,
    compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    pill_background: &NSColor,
    pill_border: &NSColor,
    pill_highlight: &NSColor,
    expanded_background: &NSColor,
    expanded_border: &NSColor,
    separator_color: &NSColor,
) -> PanelBaseViews {
    let content_view = create_content_view(mtm, size);
    let (left_shoulder, right_shoulder) = create_shoulder_views(mtm, pill_frame, pill_background);
    let pill_view = create_pill_view(mtm, pill_frame, pill_background, pill_border);
    let (expanded_container, cards_container) = create_expanded_container(
        mtm,
        size,
        compact_width,
        expanded_width,
        compact_height,
        pill_size.height,
        expanded_background,
        expanded_border,
    );
    let top_highlight = create_top_highlight(mtm, pill_size, pill_highlight);
    let body_separator = create_body_separator(mtm, expanded_width, separator_color);
    let settings_button = create_edge_action_button(mtm, "⚙", text_primary_color(), 20.0, 5.0);
    let quit_button = create_edge_action_button(mtm, "⏻", close_action_color(), 16.0, 2.0);

    PanelBaseViews {
        content_view,
        left_shoulder,
        right_shoulder,
        pill_view,
        expanded_container,
        cards_container,
        top_highlight,
        body_separator,
        settings_button,
        quit_button,
    }
}

fn create_content_view(mtm: MainThreadMarker, size: NSSize) -> Retained<NSView> {
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

fn create_shoulder_views(
    mtm: MainThreadMarker,
    pill_frame: NSRect,
    pill_background: &NSColor,
) -> (Retained<NSView>, Retained<NSView>) {
    let left_shoulder = NSView::initWithFrame(NSView::alloc(mtm), left_shoulder_frame(pill_frame));
    let right_shoulder =
        NSView::initWithFrame(NSView::alloc(mtm), right_shoulder_frame(pill_frame));
    panel_shoulder::apply_shoulder_layer(&left_shoulder, pill_background, true);
    panel_shoulder::apply_shoulder_layer(&right_shoulder, pill_background, false);
    (left_shoulder, right_shoulder)
}

fn create_pill_view(
    mtm: MainThreadMarker,
    pill_frame: NSRect,
    pill_background: &NSColor,
    pill_border: &NSColor,
) -> Retained<NSView> {
    let pill_view = NSView::initWithFrame(NSView::alloc(mtm), pill_frame);
    pill_view.setWantsLayer(true);
    let pill_layer = CALayer::layer();
    pill_layer.setCornerRadius(COMPACT_PILL_RADIUS);
    pill_layer.setMasksToBounds(true);
    pill_layer.setMaskedCorners(panel_style::compact_pill_corner_mask());
    pill_layer.setBackgroundColor(Some(&pill_background.CGColor()));
    pill_layer.setBorderWidth(1.0);
    pill_layer.setBorderColor(Some(&pill_border.CGColor()));
    pill_view.setLayer(Some(&pill_layer));
    pill_view
}

#[allow(clippy::too_many_arguments)]
fn create_expanded_container(
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

fn create_top_highlight(
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

fn create_body_separator(
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

fn text_primary_color() -> objc2::rc::Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(0.96, 0.97, 0.99, 0.88)
}

fn close_action_color() -> objc2::rc::Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.32, 0.32, 0.95)
}

fn create_edge_action_button(
    mtm: MainThreadMarker,
    label: &str,
    text_color: objc2::rc::Retained<NSColor>,
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
