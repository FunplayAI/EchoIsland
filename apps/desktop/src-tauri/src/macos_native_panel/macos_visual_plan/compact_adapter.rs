use objc2_app_kit::{NSColor, NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use crate::native_panel_core::{PanelPoint, PanelRect};
use crate::native_panel_renderer::facade::{
    descriptor::NativePanelEdgeAction,
    presentation::NativePanelPresentationModel,
    visual::{
        NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
        NativePanelVisualShoulderSide, NativePanelVisualTextRole, NativePanelVisualTextWeight,
    },
};

use super::super::panel_constants::{
    ACTIVE_COUNT_SCROLL_TRAVEL, ACTIVE_COUNT_TEXT_OFFSET_X, ACTIVE_COUNT_TEXT_WIDTH,
};
use super::super::panel_helpers::ns_color;
use super::super::panel_refs::NativePanelRefs;
use super::super::panel_shoulder::apply_shoulder_path_scale_x;
use super::super::panel_types::NativePanelLayout;
use super::super::text_metrics::{
    centered_child_origin, font_for_visual_weight, macos_action_icon_font_size,
    macos_action_icon_glyph_offset_y, macos_text_frame_height,
    macos_text_frame_origin_for_visual_center,
};

const SETTINGS_VISUAL_ICON_TEXT: &str = "\u{E713}";
const SETTINGS_MACOS_ICON_TEXT: &str = "\u{2699}";

#[derive(Clone, Debug, PartialEq)]
struct MacosActionIconPrimitive {
    role: NativePanelVisualTextRole,
    origin: PanelPoint,
    max_width: f64,
    text: String,
    color: NativePanelVisualColor,
    size: i32,
    weight: NativePanelVisualTextWeight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MacosRoundRectPrimitive {
    frame: PanelRect,
    radius: f64,
    color: NativePanelVisualColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MacosShoulderPrimitive {
    frame: PanelRect,
    side: NativePanelVisualShoulderSide,
    progress: f64,
    fill: NativePanelVisualColor,
}

pub(in crate::macos_native_panel) fn apply_macos_visual_plan_compact_primitives(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    presentation: &NativePanelPresentationModel,
    plan: &NativePanelVisualPlan,
) {
    apply_compact_background_primitives(refs, layout, plan);
    apply_compact_bar_text_primitives(refs, layout, presentation, plan);
    apply_action_button_icon_primitive(
        refs.settings_button,
        refs.settings_button_label,
        macos_action_icon_primitive(plan, NativePanelEdgeAction::Settings),
    );
    apply_action_button_icon_primitive(
        refs.quit_button,
        refs.quit_button_label,
        macos_action_icon_primitive(plan, NativePanelEdgeAction::Quit),
    );
}

fn apply_compact_background_primitives(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    plan: &NativePanelVisualPlan,
) {
    if let Some(pill) = compact_pill_primitive(plan, panel_rect_from_ns_rect(layout.pill_frame)) {
        if let Some(layer) = refs.pill_view.layer() {
            layer.setBackgroundColor(Some(&ns_color(visual_color(pill.color)).CGColor()));
        }
    }
    apply_compact_shoulder_primitive(refs.left_shoulder, compact_shoulder_primitive(plan, true));
    apply_compact_shoulder_primitive(refs.right_shoulder, compact_shoulder_primitive(plan, false));
}

fn compact_pill_primitive(
    plan: &NativePanelVisualPlan,
    pill_frame: PanelRect,
) -> Option<MacosRoundRectPrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::RoundRect {
            frame,
            radius,
            color,
        } = primitive
        else {
            return None;
        };
        rects_nearly_equal(*frame, pill_frame).then_some(MacosRoundRectPrimitive {
            frame: *frame,
            radius: *radius,
            color: *color,
        })
    })
}

fn compact_shoulder_primitive(
    plan: &NativePanelVisualPlan,
    left: bool,
) -> Option<MacosShoulderPrimitive> {
    let expected_side = if left {
        NativePanelVisualShoulderSide::Left
    } else {
        NativePanelVisualShoulderSide::Right
    };
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::CompactShoulder {
            frame,
            side,
            progress,
            fill,
            ..
        } = primitive
        else {
            return None;
        };
        (*side == expected_side).then_some(MacosShoulderPrimitive {
            frame: *frame,
            side: *side,
            progress: *progress,
            fill: *fill,
        })
    })
}

fn apply_compact_shoulder_primitive(shoulder: &NSView, primitive: Option<MacosShoulderPrimitive>) {
    let Some(primitive) = primitive else {
        return;
    };
    let anchor_on_right = primitive.side == NativePanelVisualShoulderSide::Left;
    apply_shoulder_path_scale_x(
        shoulder,
        ns_rect_from_panel_rect(primitive.frame),
        primitive.progress,
        anchor_on_right,
    );
    let Some(layer) = shoulder.layer() else {
        return;
    };
    let Ok(shape_layer) = layer.downcast::<objc2_quartz_core::CAShapeLayer>() else {
        return;
    };
    shape_layer.setFillColor(Some(&ns_color(visual_color(primitive.fill)).CGColor()));
}

fn apply_compact_bar_text_primitives(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    presentation: &NativePanelPresentationModel,
    plan: &NativePanelVisualPlan,
) {
    if let Some(headline) = compact_headline_primitive(plan) {
        apply_text_primitive_to_label(refs.headline, layout, &headline);
    }
    if let Some(slash) = compact_slash_primitive(plan) {
        apply_text_primitive_to_label(refs.slash, layout, &slash);
    }
    if let Some(total) = compact_total_count_primitive(plan, &presentation.compact_bar.total_count)
    {
        apply_text_primitive_to_label(refs.total_count, layout, &total);
    }
    if let Some(active) =
        compact_active_count_primitive(plan, &presentation.compact_bar.active_count)
    {
        apply_active_count_text_primitive(refs, layout, &active);
    }
}

fn compact_headline_primitive(plan: &NativePanelVisualPlan) -> Option<MacosActionIconPrimitive> {
    text_primitive_by_role(plan, NativePanelVisualTextRole::CompactHeadline)
}

fn compact_slash_primitive(plan: &NativePanelVisualPlan) -> Option<MacosActionIconPrimitive> {
    text_primitive_by_role(plan, NativePanelVisualTextRole::CompactSlash)
}

fn compact_total_count_primitive(
    plan: &NativePanelVisualPlan,
    total_count: &str,
) -> Option<MacosActionIconPrimitive> {
    (!total_count.is_empty())
        .then(|| text_primitive_by_role(plan, NativePanelVisualTextRole::CompactTotalCount))
        .flatten()
}

fn compact_active_count_primitive(
    plan: &NativePanelVisualPlan,
    active_count: &str,
) -> Option<MacosActionIconPrimitive> {
    (!active_count.is_empty())
        .then(|| text_primitive_by_role(plan, NativePanelVisualTextRole::CompactActiveCount))
        .flatten()
}

fn macos_action_icon_primitive(
    plan: &NativePanelVisualPlan,
    action: NativePanelEdgeAction,
) -> Option<MacosActionIconPrimitive> {
    let role = match action {
        NativePanelEdgeAction::Settings => NativePanelVisualTextRole::ActionButtonSettings,
        NativePanelEdgeAction::Quit => NativePanelVisualTextRole::ActionButtonQuit,
    };

    text_primitive_by_role(plan, role).map(|text| MacosActionIconPrimitive {
        text: macos_action_icon_text(&text.text),
        ..text
    })
}

fn text_primitive_by_role(
    plan: &NativePanelVisualPlan,
    role: NativePanelVisualTextRole,
) -> Option<MacosActionIconPrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let text = text_primitive(primitive)?;
        (text.role == role).then_some(text)
    })
}

fn text_primitive(primitive: &NativePanelVisualPrimitive) -> Option<MacosActionIconPrimitive> {
    let NativePanelVisualPrimitive::Text {
        role,
        origin,
        max_width,
        text,
        color,
        size,
        weight,
        ..
    } = primitive
    else {
        return None;
    };

    Some(MacosActionIconPrimitive {
        role: *role,
        origin: *origin,
        max_width: *max_width,
        text: text.clone(),
        color: *color,
        size: *size,
        weight: *weight,
    })
}

fn macos_action_icon_text(text: &str) -> String {
    if text == SETTINGS_VISUAL_ICON_TEXT {
        // AppKit does not have Windows' Segoe MDL2 glyph; keep the shared semantic icon
        // while drawing with a native macOS fallback glyph.
        SETTINGS_MACOS_ICON_TEXT.to_string()
    } else {
        text.to_string()
    }
}

fn apply_text_primitive_to_label(
    label: &NSTextField,
    layout: &NativePanelLayout,
    primitive: &MacosActionIconPrimitive,
) {
    let font = font_for_visual_weight(primitive.weight, primitive.size as f64);
    label.setTextColor(Some(&ns_color(visual_color(primitive.color))));
    label.setFont(Some(&font));
    label.setFrame(text_primitive_label_frame(primitive, layout, &font));
}

fn apply_active_count_text_primitive(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    primitive: &MacosActionIconPrimitive,
) {
    refs.active_count
        .setTextColor(Some(&ns_color(visual_color(primitive.color))));
    refs.active_count_next
        .setTextColor(Some(&ns_color(visual_color(primitive.color))));
    let font = font_for_visual_weight(primitive.weight, primitive.size as f64);
    refs.active_count.setFont(Some(&font));
    refs.active_count_next.setFont(Some(&font));
    let label_frame = text_primitive_label_frame(primitive, layout, &font);
    refs.active_count_clip.setFrame(NSRect::new(
        NSPoint::new(
            label_frame.origin.x - ACTIVE_COUNT_TEXT_OFFSET_X,
            label_frame.origin.y,
        ),
        NSSize::new(
            super::super::panel_constants::ACTIVE_COUNT_SLOT_WIDTH,
            label_frame.size.height,
        ),
    ));
    refs.active_count.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, label_frame.size.height),
    ));
    refs.active_count_next.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_SCROLL_TRAVEL + ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, label_frame.size.height),
    ));
}

fn apply_action_button_icon_primitive(
    button: &NSView,
    label: &NSTextField,
    primitive: Option<MacosActionIconPrimitive>,
) {
    let Some(primitive) = primitive else {
        return;
    };

    clear_action_button_chrome(button);
    let font = font_for_visual_weight(
        primitive.weight,
        macos_action_icon_font_size(primitive.role, primitive.size),
    );
    let button_frame = button.frame();
    let local_origin = action_icon_local_origin(&primitive, button_frame, &font);
    label.setStringValue(&NSString::from_str(&primitive.text));
    label.setTextColor(Some(&ns_color(visual_color(primitive.color))));
    label.setFont(Some(&font));
    label.setAlignment(NSTextAlignment::Center);
    label.setFrame(NSRect::new(
        local_origin,
        NSSize::new(primitive.max_width.max(1.0), macos_text_frame_height(&font)),
    ));
}

fn clear_action_button_chrome(button: &NSView) {
    let Some(layer) = button.layer() else {
        return;
    };
    layer.setCornerRadius(0.0);
    layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    layer.setBorderWidth(0.0);
    layer.setBorderColor(Some(&NSColor::clearColor().CGColor()));
}

fn action_icon_local_origin(
    primitive: &MacosActionIconPrimitive,
    button_frame: NSRect,
    font: &NSFont,
) -> NSPoint {
    let label_width = primitive.max_width.max(1.0);
    NSPoint::new(
        centered_child_origin(0.0, button_frame.size.width, label_width),
        macos_text_frame_origin_for_visual_center(0.0, button_frame.size.height, font)
            + macos_action_icon_glyph_offset_y(primitive.role),
    )
}

fn text_primitive_label_frame(
    primitive: &MacosActionIconPrimitive,
    layout: &NativePanelLayout,
    font: &NSFont,
) -> NSRect {
    let shared_height = action_icon_text_height(primitive.size);
    let label_height = macos_text_frame_height(font);
    let local_shared_y = primitive.origin.y - layout.pill_frame.origin.y;
    let label_y = macos_text_frame_origin_for_visual_center(local_shared_y, shared_height, font);

    NSRect::new(
        NSPoint::new(primitive.origin.x - layout.pill_frame.origin.x, label_y),
        NSSize::new(primitive.max_width.max(1.0), label_height),
    )
}

fn action_icon_text_height(size: i32) -> f64 {
    if size >= 13 { 24.0 } else { size as f64 + 8.0 }
}

fn visual_color(color: NativePanelVisualColor) -> [f64; 4] {
    [
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        1.0,
    ]
}

fn ns_rect_from_panel_rect(rect: PanelRect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, rect.y),
        NSSize::new(rect.width, rect.height),
    )
}

fn panel_rect_from_ns_rect(rect: NSRect) -> PanelRect {
    PanelRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

fn rects_nearly_equal(left: PanelRect, right: PanelRect) -> bool {
    (left.x - right.x).abs() < 0.001
        && (left.y - right.y).abs() < 0.001
        && (left.width - right.width).abs() < 0.001
        && (left.height - right.height).abs() < 0.001
}

#[cfg(test)]
mod tests {
    use super::{
        SETTINGS_MACOS_ICON_TEXT, action_icon_local_origin, action_icon_text_height,
        compact_pill_primitive, compact_shoulder_primitive, macos_action_icon_primitive,
        text_primitive_label_frame,
    };
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use crate::{
        macos_native_panel::panel_types::NativePanelLayout,
        macos_native_panel::text_metrics::{
            font_for_visual_weight, macos_action_icon_font_size, macos_action_icon_glyph_offset_y,
            macos_font_vertical_metrics, macos_glyph_center_from_frame_origin,
        },
        native_panel_core::{PanelPoint, PanelRect},
        native_panel_renderer::facade::{
            descriptor::NativePanelEdgeAction,
            visual::{
                NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
                NativePanelVisualShoulderSide, NativePanelVisualTextAlignment,
                NativePanelVisualTextRole, NativePanelVisualTextWeight,
            },
        },
    };

    #[test]
    fn macos_visual_plan_maps_shared_settings_icon_to_macos_fallback_glyph() {
        let plan = NativePanelVisualPlan {
            hidden: false,
            primitives: vec![NativePanelVisualPrimitive::Text {
                role: NativePanelVisualTextRole::ActionButtonSettings,
                origin: PanelPoint { x: 1.0, y: 2.0 },
                max_width: 18.0,
                text: "\u{E713}".to_string(),
                color: NativePanelVisualColor::rgb(245, 247, 252),
                size: 16,
                weight: NativePanelVisualTextWeight::Normal,
                alignment: NativePanelVisualTextAlignment::Center,
            }],
        };

        let primitive =
            macos_action_icon_primitive(&plan, NativePanelEdgeAction::Settings).expect("settings");

        assert_eq!(primitive.text, SETTINGS_MACOS_ICON_TEXT);
        assert_eq!(primitive.origin, PanelPoint { x: 1.0, y: 2.0 });
    }

    #[test]
    fn macos_visual_plan_extracts_quit_icon_from_shared_text_primitive() {
        let plan = NativePanelVisualPlan {
            hidden: false,
            primitives: vec![NativePanelVisualPrimitive::Text {
                role: NativePanelVisualTextRole::ActionButtonQuit,
                origin: PanelPoint { x: 10.0, y: 12.0 },
                max_width: 18.0,
                text: "\u{23FB}".to_string(),
                color: NativePanelVisualColor::rgb(255, 82, 82),
                size: 16,
                weight: NativePanelVisualTextWeight::Bold,
                alignment: NativePanelVisualTextAlignment::Center,
            }],
        };

        let primitive =
            macos_action_icon_primitive(&plan, NativePanelEdgeAction::Quit).expect("quit");

        assert_eq!(primitive.text, "\u{23FB}");
        assert_eq!(primitive.color, NativePanelVisualColor::rgb(255, 82, 82));
        assert_eq!(primitive.max_width, 18.0);
    }

    #[test]
    fn macos_visual_plan_uses_shared_compact_text_box_height() {
        assert_eq!(action_icon_text_height(15), 24.0);
        assert_eq!(action_icon_text_height(16), 24.0);
        assert_eq!(action_icon_text_height(10), 18.0);
    }

    #[test]
    fn macos_compact_headline_uses_appkit_centered_label_frame() {
        let layout = NativePanelLayout {
            panel_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 80.0)),
            content_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 80.0)),
            pill_frame: NSRect::new(NSPoint::new(40.0, 20.0), NSSize::new(253.0, 37.0)),
            left_shoulder_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            right_shoulder_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            expanded_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            cards_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            separator_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            shared_content_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            shell_visible: true,
            separator_visibility: 1.0,
        };
        let primitive = super::MacosActionIconPrimitive {
            role: NativePanelVisualTextRole::CompactHeadline,
            origin: PanelPoint { x: 96.0, y: 25.5 },
            max_width: 92.0,
            text: "1 active task".to_string(),
            color: NativePanelVisualColor::rgb(245, 247, 252),
            size: 13,
            weight: NativePanelVisualTextWeight::Semibold,
        };
        let font = font_for_visual_weight(primitive.weight, primitive.size as f64);

        let frame = text_primitive_label_frame(&primitive, &layout, &font);
        let metrics = macos_font_vertical_metrics(&font);
        let glyph_center_y = frame.origin.y + macos_glyph_center_from_frame_origin(metrics);
        let expected_center_y = primitive.origin.y - layout.pill_frame.origin.y
            + action_icon_text_height(primitive.size) / 2.0;

        assert_eq!(frame.origin.x, 56.0);
        assert_eq!(frame.size.width, 92.0);
        assert!((glyph_center_y - expected_center_y).abs() < 0.001);
    }

    #[test]
    fn macos_action_icon_local_origin_centers_label_in_button_frame() {
        let primitive = super::MacosActionIconPrimitive {
            role: NativePanelVisualTextRole::ActionButtonQuit,
            origin: PanelPoint { x: 210.0, y: 32.0 },
            max_width: 26.0,
            text: "\u{23FB}".to_string(),
            color: NativePanelVisualColor::rgb(255, 82, 82),
            size: 16,
            weight: NativePanelVisualTextWeight::Bold,
        };
        let button_frame = NSRect::new(NSPoint::new(160.0, 10.0), NSSize::new(26.0, 26.0));
        let font = font_for_visual_weight(primitive.weight, primitive.size as f64);

        let local = action_icon_local_origin(&primitive, button_frame, &font);
        let metrics = macos_font_vertical_metrics(&font);
        let glyph_center_y = local.y + macos_glyph_center_from_frame_origin(metrics)
            - macos_action_icon_glyph_offset_y(primitive.role);

        assert_eq!(local.x, 0.0);
        assert!((glyph_center_y - button_frame.size.height / 2.0).abs() < 0.001);
    }

    #[test]
    fn macos_settings_action_icon_local_origin_centers_larger_label_in_button() {
        let primitive = super::MacosActionIconPrimitive {
            role: NativePanelVisualTextRole::ActionButtonSettings,
            origin: PanelPoint { x: 200.0, y: 29.5 },
            max_width: 26.0,
            text: SETTINGS_MACOS_ICON_TEXT.to_string(),
            color: NativePanelVisualColor::rgb(245, 247, 252),
            size: 16,
            weight: NativePanelVisualTextWeight::Normal,
        };
        let button_frame = NSRect::new(NSPoint::new(160.0, 10.0), NSSize::new(26.0, 26.0));
        let font = font_for_visual_weight(
            primitive.weight,
            macos_action_icon_font_size(primitive.role, primitive.size),
        );

        let local = action_icon_local_origin(&primitive, button_frame, &font);
        let metrics = macos_font_vertical_metrics(&font);
        let glyph_center_y = local.y + macos_glyph_center_from_frame_origin(metrics);

        assert_eq!(local.x, 0.0);
        assert!((glyph_center_y - button_frame.size.height / 2.0).abs() < 0.001);
    }

    #[test]
    fn macos_visual_plan_extracts_compact_background_primitives() {
        let pill_frame = PanelRect {
            x: 40.0,
            y: 12.0,
            width: 240.0,
            height: 36.0,
        };
        let plan = NativePanelVisualPlan {
            hidden: false,
            primitives: vec![
                NativePanelVisualPrimitive::CompactShoulder {
                    frame: PanelRect {
                        x: 34.0,
                        y: 42.0,
                        width: 6.0,
                        height: 6.0,
                    },
                    side: NativePanelVisualShoulderSide::Left,
                    progress: 0.25,
                    fill: NativePanelVisualColor::rgb(12, 12, 15),
                    border: NativePanelVisualColor::rgb(44, 44, 50),
                },
                NativePanelVisualPrimitive::RoundRect {
                    frame: pill_frame,
                    radius: 18.0,
                    color: NativePanelVisualColor::rgb(12, 12, 15),
                },
            ],
        };

        let pill = compact_pill_primitive(&plan, pill_frame).expect("pill");
        let left_shoulder = compact_shoulder_primitive(&plan, true).expect("left shoulder");

        assert_eq!(pill.radius, 18.0);
        assert_eq!(pill.color, NativePanelVisualColor::rgb(12, 12, 15));
        assert_eq!(left_shoulder.side, NativePanelVisualShoulderSide::Left);
        assert_eq!(left_shoulder.progress, 0.25);
    }
}
