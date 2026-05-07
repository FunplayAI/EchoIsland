use objc2_app_kit::{NSFont, NSTextAlignment, NSTextField, NSView};
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

use super::super::panel_helpers::ns_color;
use super::super::panel_refs::NativePanelRefs;
use super::super::panel_shoulder::apply_shoulder_path_scale_x;
use super::super::panel_types::NativePanelLayout;

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
        layout,
        macos_action_icon_primitive(plan, NativePanelEdgeAction::Settings),
    );
    apply_action_button_icon_primitive(
        refs.quit_button,
        refs.quit_button_label,
        layout,
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
    label.setTextColor(Some(&ns_color(visual_color(primitive.color))));
    label.setFont(Some(&font_for_visual_weight(
        primitive.weight,
        primitive.size as f64,
    )));
    label.setFrame(NSRect::new(
        NSPoint::new(
            primitive.origin.x - layout.pill_frame.origin.x,
            primitive.origin.y - layout.pill_frame.origin.y,
        ),
        NSSize::new(
            primitive.max_width.max(1.0),
            action_icon_text_height(primitive.size),
        ),
    ));
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
    refs.active_count.setFont(Some(&font_for_visual_weight(
        primitive.weight,
        primitive.size as f64,
    )));
    refs.active_count_next.setFont(Some(&font_for_visual_weight(
        primitive.weight,
        primitive.size as f64,
    )));
    refs.active_count_clip.setFrame(NSRect::new(
        NSPoint::new(
            primitive.origin.x
                - layout.pill_frame.origin.x
                - super::super::panel_constants::ACTIVE_COUNT_TEXT_OFFSET_X,
            primitive.origin.y - layout.pill_frame.origin.y,
        ),
        NSSize::new(
            super::super::panel_constants::ACTIVE_COUNT_SLOT_WIDTH,
            action_icon_text_height(primitive.size),
        ),
    ));
}

fn apply_action_button_icon_primitive(
    button: &NSView,
    label: &NSTextField,
    layout: &NativePanelLayout,
    primitive: Option<MacosActionIconPrimitive>,
) {
    let Some(primitive) = primitive else {
        return;
    };

    let button_frame = button.frame();
    let local_origin = action_icon_local_origin(&primitive, layout, button_frame);
    label.setStringValue(&NSString::from_str(&primitive.text));
    label.setTextColor(Some(&ns_color(visual_color(primitive.color))));
    label.setFont(Some(&font_for_visual_weight(
        primitive.weight,
        primitive.size as f64,
    )));
    label.setAlignment(NSTextAlignment::Center);
    label.setFrame(NSRect::new(
        local_origin,
        NSSize::new(
            primitive.max_width.max(1.0),
            action_icon_text_height(primitive.size),
        ),
    ));
}

fn action_icon_local_origin(
    primitive: &MacosActionIconPrimitive,
    layout: &NativePanelLayout,
    button_frame: NSRect,
) -> NSPoint {
    NSPoint::new(
        primitive.origin.x - layout.pill_frame.origin.x - button_frame.origin.x,
        primitive.origin.y - layout.pill_frame.origin.y - button_frame.origin.y
            + macos_action_icon_baseline_offset_y(primitive.role),
    )
}

fn macos_action_icon_baseline_offset_y(role: NativePanelVisualTextRole) -> f64 {
    match role {
        NativePanelVisualTextRole::ActionButtonSettings
        | NativePanelVisualTextRole::ActionButtonQuit => -2.0,
        _ => 0.0,
    }
}

fn action_icon_text_height(size: i32) -> f64 {
    if size >= 13 { 24.0 } else { size as f64 + 8.0 }
}

fn font_for_visual_weight(
    weight: NativePanelVisualTextWeight,
    size: f64,
) -> objc2::rc::Retained<NSFont> {
    match weight {
        NativePanelVisualTextWeight::Bold | NativePanelVisualTextWeight::Semibold => {
            NSFont::boldSystemFontOfSize(size)
        }
        NativePanelVisualTextWeight::Normal => NSFont::systemFontOfSize(size),
    }
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
    };
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use crate::{
        macos_native_panel::panel_types::NativePanelLayout,
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
    fn macos_action_icon_local_origin_applies_glyph_baseline_correction_only_to_label() {
        let layout = NativePanelLayout {
            panel_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 80.0)),
            content_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 80.0)),
            pill_frame: NSRect::new(NSPoint::new(40.0, 20.0), NSSize::new(240.0, 38.0)),
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
            role: NativePanelVisualTextRole::ActionButtonQuit,
            origin: PanelPoint { x: 210.0, y: 32.0 },
            max_width: 26.0,
            text: "\u{23FB}".to_string(),
            color: NativePanelVisualColor::rgb(255, 82, 82),
            size: 16,
            weight: NativePanelVisualTextWeight::Bold,
        };
        let button_frame = NSRect::new(NSPoint::new(160.0, 10.0), NSSize::new(26.0, 26.0));

        let local = action_icon_local_origin(&primitive, &layout, button_frame);

        assert_eq!(local.x, 10.0);
        assert_eq!(local.y, 0.0);
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
