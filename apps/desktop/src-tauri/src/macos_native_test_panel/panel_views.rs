use super::panel_shoulder;
use super::panel_style;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSTextField, NSView};
use objc2_foundation::{NSRect, NSSize};
use objc2_quartz_core::CALayer;

use super::completion_glow_view::create_completion_glow;
use super::panel_action_buttons::{
    EdgeActionButtonViews, close_action_color, create_edge_action_button, text_primary_color,
};
use super::panel_base_container_views::{
    create_body_separator, create_content_view, create_expanded_container, create_top_highlight,
};
use super::panel_constants::COMPACT_PILL_RADIUS;
use super::panel_geometry::{left_shoulder_frame, right_shoulder_frame};

pub(super) struct PanelBaseViews {
    pub(super) content_view: Retained<NSView>,
    pub(super) left_shoulder: Retained<NSView>,
    pub(super) right_shoulder: Retained<NSView>,
    pub(super) pill_view: Retained<NSView>,
    pub(super) expanded_container: Retained<NSView>,
    pub(super) cards_container: Retained<NSView>,
    pub(super) completion_glow: Retained<NSView>,
    pub(super) top_highlight: Retained<NSView>,
    pub(super) body_separator: Retained<NSView>,
    pub(super) settings_button: Retained<NSView>,
    pub(super) settings_button_label: Retained<NSTextField>,
    pub(super) quit_button: Retained<NSView>,
    pub(super) quit_button_label: Retained<NSTextField>,
}
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
    let completion_glow = create_completion_glow(mtm, pill_size);
    let body_separator = create_body_separator(mtm, expanded_width, separator_color);
    let EdgeActionButtonViews {
        button: settings_button,
        label: settings_button_label,
    } = create_edge_action_button(mtm, "⚙", text_primary_color(), 20.0, 5.0);
    let EdgeActionButtonViews {
        button: quit_button,
        label: quit_button_label,
    } = create_edge_action_button(mtm, "⏻", close_action_color(), 16.0, 2.0);

    PanelBaseViews {
        content_view,
        left_shoulder,
        right_shoulder,
        pill_view,
        expanded_container,
        cards_container,
        completion_glow,
        top_highlight,
        body_separator,
        settings_button,
        settings_button_label,
        quit_button,
        quit_button_label,
    }
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
