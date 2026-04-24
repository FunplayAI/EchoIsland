use objc2_app_kit::{NSPanel, NSScreen};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::panel_constants::{
    COLLAPSED_PANEL_HEIGHT, COMPACT_SHOULDER_SIZE, EXPANDED_CARDS_SIDE_INSET,
    EXPANDED_CONTENT_BOTTOM_INSET, EXPANDED_CONTENT_TOP_GAP, EXPANDED_MAX_BODY_HEIGHT,
    PANEL_DROP_DISTANCE,
};
use super::panel_screen_geometry::{
    compact_pill_height_for_screen_rect, compact_pill_width_for_screen_rect,
    expanded_panel_width_for_screen_rect, panel_canvas_width_for_screen_rect,
};
use super::panel_types::{
    NativePanelAnimationDescriptor, NativePanelGeometryMetrics, NativePanelLayout,
};

pub(super) fn native_panel_geometry_metrics(
    screen: Option<&NSScreen>,
    screen_frame: NSRect,
) -> NativePanelGeometryMetrics {
    let compact_height = compact_pill_height_for_screen_rect(screen, screen_frame);
    NativePanelGeometryMetrics {
        compact_height,
        compact_width: compact_pill_width_for_screen_rect(screen, compact_height),
        expanded_width: expanded_panel_width_for_screen_rect(screen, screen_frame),
        panel_width: panel_canvas_width_for_screen_rect(screen, compact_height, screen_frame),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_native_panel_layout(
    screen_frame: NSRect,
    metrics: NativePanelGeometryMetrics,
    canvas_height: f64,
    visible_height: f64,
    bar_progress: f64,
    height_progress: f64,
    drop_progress: f64,
    content_visibility: f64,
) -> NativePanelLayout {
    let layout = crate::native_panel_core::resolve_panel_layout(
        crate::native_panel_core::PanelLayoutInput {
            screen_frame: panel_rect(screen_frame),
            metrics: crate::native_panel_core::PanelGeometryMetrics {
                compact_height: metrics.compact_height,
                compact_width: metrics.compact_width,
                expanded_width: metrics.expanded_width,
                panel_width: metrics.panel_width,
            },
            canvas_height,
            visible_height,
            bar_progress,
            height_progress,
            drop_progress,
            content_visibility,
            collapsed_height: COLLAPSED_PANEL_HEIGHT,
            drop_distance: PANEL_DROP_DISTANCE,
            content_top_gap: EXPANDED_CONTENT_TOP_GAP,
            content_bottom_inset: EXPANDED_CONTENT_BOTTOM_INSET,
            cards_side_inset: EXPANDED_CARDS_SIDE_INSET,
            shoulder_size: COMPACT_SHOULDER_SIZE,
            separator_side_inset: 14.0,
        },
    );
    NativePanelLayout {
        panel_frame: ns_rect(layout.panel_frame),
        content_frame: ns_rect(layout.content_frame),
        pill_frame: ns_rect(layout.pill_frame),
        left_shoulder_frame: ns_rect(layout.left_shoulder_frame),
        right_shoulder_frame: ns_rect(layout.right_shoulder_frame),
        expanded_frame: ns_rect(layout.expanded_frame),
        cards_frame: ns_rect(layout.cards_frame),
        separator_frame: ns_rect(layout.separator_frame),
        shared_content_frame: ns_rect(layout.shared_content_frame),
        shell_visible: layout.shell_visible,
        separator_visibility: layout.separator_visibility,
    }
}

#[cfg(test)]
pub(super) fn shared_expanded_content_state(
    shared_expanded_enabled: bool,
    shell_visible: bool,
    height_progress: f64,
    bar_progress: f64,
    cards_height: f64,
    status_surface_active: bool,
    content_visibility: f64,
) -> (bool, bool) {
    let state = crate::native_panel_core::resolve_shared_expanded_content_state(
        crate::native_panel_core::SharedExpandedContentInput {
            enabled: shared_expanded_enabled,
            shell_visible,
            height_progress,
            bar_progress,
            cards_height,
            status_surface_active,
            content_visibility,
        },
    );
    (state.visible, state.interactive)
}

pub(super) fn rects_nearly_equal(a: NSRect, b: NSRect) -> bool {
    crate::native_panel_core::rects_nearly_equal(panel_rect(a), panel_rect(b), 0.5)
}

pub(super) fn native_panel_core_layout(
    layout: &NativePanelLayout,
) -> crate::native_panel_core::PanelLayout {
    crate::native_panel_core::PanelLayout {
        panel_frame: panel_rect(layout.panel_frame),
        content_frame: panel_rect(layout.content_frame),
        pill_frame: panel_rect(layout.pill_frame),
        left_shoulder_frame: panel_rect(layout.left_shoulder_frame),
        right_shoulder_frame: panel_rect(layout.right_shoulder_frame),
        expanded_frame: panel_rect(layout.expanded_frame),
        cards_frame: panel_rect(layout.cards_frame),
        separator_frame: panel_rect(layout.separator_frame),
        shared_content_frame: panel_rect(layout.shared_content_frame),
        shell_visible: layout.shell_visible,
        separator_visibility: layout.separator_visibility,
    }
}

pub(super) fn apply_panel_frame(panel: &NSPanel, frame: NSRect) {
    let current = panel.frame();
    if rects_nearly_equal(current, frame) {
        return;
    }
    let top_left = NSPoint::new(frame.origin.x, frame.origin.y + frame.size.height);
    panel.setContentSize(frame.size);
    panel.setFrameTopLeftPoint(top_left);
}

pub(super) fn centered_top_frame(screen_frame: NSRect, size: NSSize) -> NSRect {
    let frame = crate::native_panel_core::resolve_centered_top_frame(
        crate::native_panel_core::PanelRect {
            x: screen_frame.origin.x,
            y: screen_frame.origin.y,
            width: screen_frame.size.width,
            height: screen_frame.size.height,
        },
        crate::native_panel_core::PanelSize {
            width: size.width,
            height: size.height,
        },
    );
    NSRect::new(
        NSPoint::new(frame.x, frame.y),
        NSSize::new(frame.width, frame.height),
    )
}

#[allow(dead_code)]
pub(super) fn resolve_native_panel_host_frame(
    screen_frame: NSRect,
    metrics: NativePanelGeometryMetrics,
    animation: NativePanelAnimationDescriptor,
) -> NSRect {
    ns_rect(crate::native_panel_core::resolve_native_panel_host_frame(
        animation,
        panel_rect(screen_frame),
        metrics.compact_width,
        metrics.expanded_width,
    ))
}

pub(super) fn island_bar_frame(
    content_size: NSSize,
    progress: f64,
    compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> NSRect {
    ns_rect(crate::native_panel_core::resolve_island_bar_frame(
        crate::native_panel_core::PanelSize {
            width: content_size.width,
            height: content_size.height,
        },
        progress,
        compact_width,
        expanded_width,
        compact_height,
        drop_offset,
    ))
}

pub(super) fn left_shoulder_frame(pill_frame: NSRect) -> NSRect {
    ns_rect(crate::native_panel_core::resolve_left_shoulder_frame(
        panel_rect(pill_frame),
        COMPACT_SHOULDER_SIZE,
    ))
}

pub(super) fn right_shoulder_frame(pill_frame: NSRect) -> NSRect {
    ns_rect(crate::native_panel_core::resolve_right_shoulder_frame(
        panel_rect(pill_frame),
        COMPACT_SHOULDER_SIZE,
    ))
}

pub(super) fn expanded_background_frame(
    content_size: NSSize,
    visible_height: f64,
    bar_progress: f64,
    height_progress: f64,
    compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> NSRect {
    ns_rect(crate::native_panel_core::resolve_expanded_background_frame(
        crate::native_panel_core::PanelSize {
            width: content_size.width,
            height: content_size.height,
        },
        visible_height,
        bar_progress,
        height_progress,
        compact_width,
        expanded_width,
        compact_height,
        drop_offset,
        COLLAPSED_PANEL_HEIGHT,
    ))
}

pub(super) fn expanded_cards_frame(container_frame: NSRect, compact_height: f64) -> NSRect {
    ns_rect(crate::native_panel_core::resolve_expanded_cards_frame(
        panel_rect(container_frame),
        compact_height,
        EXPANDED_CONTENT_TOP_GAP,
        EXPANDED_CONTENT_BOTTOM_INSET,
        EXPANDED_CARDS_SIDE_INSET,
    ))
}

pub(super) fn expanded_total_height_for_body_height(
    estimated_height: f64,
    compact_height: f64,
    shared_body_height: Option<f64>,
) -> f64 {
    crate::native_panel_core::resolve_expanded_total_height(
        estimated_height,
        shared_body_height,
        compact_height,
        EXPANDED_CONTENT_TOP_GAP,
        EXPANDED_CONTENT_BOTTOM_INSET,
        EXPANDED_MAX_BODY_HEIGHT,
    )
}

#[cfg(test)]
pub(super) fn panel_transition_canvas_height(start_height: f64, target_height: f64) -> f64 {
    crate::native_panel_core::resolve_panel_transition_canvas_height(
        start_height,
        target_height,
        COLLAPSED_PANEL_HEIGHT,
    )
}

pub(super) fn expanded_cards_width(container_width: f64) -> f64 {
    crate::native_panel_core::resolve_expanded_cards_width(
        container_width,
        EXPANDED_CARDS_SIDE_INSET,
    )
}

pub(super) fn absolute_rect(panel_frame: NSRect, local_frame: NSRect) -> NSRect {
    ns_rect(crate::native_panel_core::absolute_rect(
        panel_rect(panel_frame),
        panel_rect(local_frame),
    ))
}

#[allow(dead_code)]
pub(super) fn compose_local_rect(parent_frame: NSRect, child_frame: NSRect) -> NSRect {
    ns_rect(crate::native_panel_core::compose_local_rect(
        panel_rect(parent_frame),
        panel_rect(child_frame),
    ))
}

pub(super) fn point_in_rect(point: NSPoint, rect: NSRect) -> bool {
    crate::native_panel_core::point_in_rect(
        crate::native_panel_core::PanelPoint {
            x: point.x,
            y: point.y,
        },
        panel_rect(rect),
    )
}

fn panel_rect(rect: NSRect) -> crate::native_panel_core::PanelRect {
    crate::native_panel_core::PanelRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

fn ns_rect(rect: crate::native_panel_core::PanelRect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, rect.y),
        NSSize::new(rect.width, rect.height),
    )
}
