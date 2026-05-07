use objc2_app_kit::NSView;
use objc2_foundation::{NSPoint, NSRect, NSSize};

use crate::native_panel_core::{
    CompactBarContentLayoutInput, ExpandedSurface, MascotVisualFrame, PanelPoint, PanelRect,
    resolve_compact_bar_content_layout,
};
use crate::native_panel_renderer::facade::{
    descriptor::NativePanelHostWindowState,
    presentation::{NativePanelVisualDisplayMode, NativePanelVisualPlanInput},
    visual::{
        NativePanelVisualColor, NativePanelVisualMascotEllipseRole,
        NativePanelVisualMascotRoundRectRole, NativePanelVisualMascotTextRole,
        NativePanelVisualPlan, NativePanelVisualPrimitive, NativePanelVisualTextWeight,
    },
};
use crate::native_panel_scene::SceneMascotPose;

use super::super::mascot::{NativeMascotFrame, NativeMascotState};
use super::super::panel_constants::MASCOT_VERTICAL_NUDGE_Y;
use super::super::panel_helpers::ns_color;

const MACOS_MASCOT_COMPACT_FRAME_WIDTH: f64 = 240.0;
const MACOS_MASCOT_COMPACT_FRAME_HEIGHT: f64 = 36.0;
const MACOS_MASCOT_BODY_CENTER: PanelPoint = PanelPoint {
    x: 14.0,
    y: 14.0 + MASCOT_VERTICAL_NUDGE_Y,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosMascotBodyPrimitive {
    pub(in crate::macos_native_panel) frame: PanelRect,
    pub(in crate::macos_native_panel) corner_radius: f64,
    pub(in crate::macos_native_panel) fill: NativePanelVisualColor,
    pub(in crate::macos_native_panel) stroke: NativePanelVisualColor,
    pub(in crate::macos_native_panel) stroke_width: f64,
    pub(in crate::macos_native_panel) shadow_opacity: f64,
    pub(in crate::macos_native_panel) shadow_radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosMascotEllipsePrimitive {
    pub(in crate::macos_native_panel) frame: PanelRect,
    pub(in crate::macos_native_panel) color: NativePanelVisualColor,
    pub(in crate::macos_native_panel) alpha: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosMascotRoundRectPrimitive {
    pub(in crate::macos_native_panel) frame: PanelRect,
    pub(in crate::macos_native_panel) radius: f64,
    pub(in crate::macos_native_panel) color: NativePanelVisualColor,
    pub(in crate::macos_native_panel) alpha: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosMascotTextPrimitive {
    pub(in crate::macos_native_panel) origin: PanelPoint,
    pub(in crate::macos_native_panel) max_width: f64,
    pub(in crate::macos_native_panel) text: String,
    pub(in crate::macos_native_panel) color: NativePanelVisualColor,
    pub(in crate::macos_native_panel) size: i32,
    pub(in crate::macos_native_panel) weight: NativePanelVisualTextWeight,
    pub(in crate::macos_native_panel) alpha: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosMascotMessageBubblePrimitive {
    pub(in crate::macos_native_panel) bubble: MacosMascotRoundRectPrimitive,
    pub(in crate::macos_native_panel) dots: Vec<MacosMascotEllipsePrimitive>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosMascotCompletionBadgePrimitive {
    pub(in crate::macos_native_panel) outline: MacosMascotRoundRectPrimitive,
    pub(in crate::macos_native_panel) fill: MacosMascotRoundRectPrimitive,
    pub(in crate::macos_native_panel) label: MacosMascotTextPrimitive,
}

pub(in crate::macos_native_panel) fn resolve_macos_mascot_visual_plan(
    frame: NativeMascotFrame,
) -> NativePanelVisualPlan {
    let compact_frame = macos_mascot_compact_frame();
    let zero = PanelRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    crate::native_panel_renderer::facade::visual::resolve_native_panel_visual_plan(
        &NativePanelVisualPlanInput {
            window_state: NativePanelHostWindowState {
                frame: Some(compact_frame),
                visible: true,
                preferred_display_index: 0,
            },
            display_mode: NativePanelVisualDisplayMode::Compact,
            surface: ExpandedSurface::Default,
            panel_frame: compact_frame,
            compact_bar_frame: compact_frame,
            left_shoulder_frame: zero,
            right_shoulder_frame: zero,
            shoulder_progress: 0.0,
            content_frame: compact_frame,
            card_stack_frame: zero,
            card_stack_content_height: 0.0,
            shell_frame: compact_frame,
            headline_text: String::new(),
            headline_emphasized: false,
            active_count: String::new(),
            active_count_elapsed_ms: 0,
            total_count: String::new(),
            separator_visibility: 0.0,
            cards_visible: false,
            card_count: 0,
            cards: Vec::new(),
            glow_visible: false,
            glow_opacity: 0.0,
            action_buttons_visible: false,
            action_buttons: Vec::new(),
            completion_count: frame.completion_count,
            mascot_elapsed_ms: (frame.t * 1000.0).max(0.0).round() as u128,
            mascot_motion_frame: Some(macos_mascot_visual_frame(frame)),
            mascot_pose: scene_pose_from_native_state(frame.state),
            mascot_debug_mode_enabled: frame.debug_mode_enabled,
        },
    )
}

pub(in crate::macos_native_panel) fn mascot_body_primitive(
    plan: &NativePanelVisualPlan,
) -> Option<MacosMascotBodyPrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::MascotDot {
            frame,
            corner_radius,
            fill,
            stroke,
            stroke_width,
            shadow_opacity,
            shadow_radius,
            ..
        } = primitive
        else {
            return None;
        };
        Some(MacosMascotBodyPrimitive {
            frame: *frame,
            corner_radius: *corner_radius,
            fill: *fill,
            stroke: *stroke,
            stroke_width: *stroke_width,
            shadow_opacity: *shadow_opacity,
            shadow_radius: *shadow_radius,
        })
    })
}

pub(in crate::macos_native_panel) fn mascot_eye_primitive(
    plan: &NativePanelVisualPlan,
    left: bool,
) -> Option<MacosMascotEllipsePrimitive> {
    let role = if left {
        NativePanelVisualMascotEllipseRole::LeftEye
    } else {
        NativePanelVisualMascotEllipseRole::RightEye
    };
    mascot_ellipse_primitive(plan, role)
}

pub(in crate::macos_native_panel) fn mascot_mouth_primitive(
    plan: &NativePanelVisualPlan,
) -> Option<MacosMascotRoundRectPrimitive> {
    mascot_round_rect_primitive(plan, NativePanelVisualMascotRoundRectRole::Mouth)
}

pub(in crate::macos_native_panel) fn mascot_message_bubble_primitive(
    plan: &NativePanelVisualPlan,
) -> Option<MacosMascotMessageBubblePrimitive> {
    let bubble =
        mascot_round_rect_primitive(plan, NativePanelVisualMascotRoundRectRole::MessageBubble)?;
    let dots = plan
        .primitives
        .iter()
        .filter_map(|primitive| {
            let NativePanelVisualPrimitive::MascotEllipse {
                role,
                frame,
                color,
                alpha,
            } = primitive
            else {
                return None;
            };
            (*role == NativePanelVisualMascotEllipseRole::MessageBubbleDot).then_some(
                MacosMascotEllipsePrimitive {
                    frame: *frame,
                    color: *color,
                    alpha: *alpha,
                },
            )
        })
        .collect();
    Some(MacosMascotMessageBubblePrimitive { bubble, dots })
}

pub(in crate::macos_native_panel) fn mascot_sleep_label_primitive(
    plan: &NativePanelVisualPlan,
) -> Option<MacosMascotTextPrimitive> {
    mascot_text_primitive(plan, NativePanelVisualMascotTextRole::SleepLabel)
}

pub(in crate::macos_native_panel) fn mascot_completion_badge_primitive(
    plan: &NativePanelVisualPlan,
) -> Option<MacosMascotCompletionBadgePrimitive> {
    Some(MacosMascotCompletionBadgePrimitive {
        outline: mascot_round_rect_primitive(
            plan,
            NativePanelVisualMascotRoundRectRole::CompletionBadgeOutline,
        )?,
        fill: mascot_round_rect_primitive(
            plan,
            NativePanelVisualMascotRoundRectRole::CompletionBadgeFill,
        )?,
        label: mascot_text_primitive(plan, NativePanelVisualMascotTextRole::CompletionBadgeLabel)?,
    })
}

fn mascot_round_rect_primitive(
    plan: &NativePanelVisualPlan,
    expected_role: NativePanelVisualMascotRoundRectRole,
) -> Option<MacosMascotRoundRectPrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::MascotRoundRect {
            role,
            frame,
            radius,
            color,
            alpha,
        } = primitive
        else {
            return None;
        };
        (*role == expected_role).then_some(MacosMascotRoundRectPrimitive {
            frame: *frame,
            radius: *radius,
            color: *color,
            alpha: *alpha,
        })
    })
}

fn mascot_ellipse_primitive(
    plan: &NativePanelVisualPlan,
    expected_role: NativePanelVisualMascotEllipseRole,
) -> Option<MacosMascotEllipsePrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::MascotEllipse {
            role,
            frame,
            color,
            alpha,
        } = primitive
        else {
            return None;
        };
        (*role == expected_role).then_some(MacosMascotEllipsePrimitive {
            frame: *frame,
            color: *color,
            alpha: *alpha,
        })
    })
}

fn mascot_text_primitive(
    plan: &NativePanelVisualPlan,
    expected_role: NativePanelVisualMascotTextRole,
) -> Option<MacosMascotTextPrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::MascotText {
            role,
            origin,
            max_width,
            text,
            color,
            size,
            weight,
            alpha,
            ..
        } = primitive
        else {
            return None;
        };
        (*role == expected_role).then_some(MacosMascotTextPrimitive {
            origin: *origin,
            max_width: *max_width,
            text: text.clone(),
            color: *color,
            size: *size,
            weight: *weight,
            alpha: *alpha,
        })
    })
}

pub(in crate::macos_native_panel) fn apply_macos_mascot_body_primitive(
    mascot_body: &NSView,
    primitive: MacosMascotBodyPrimitive,
    stroke_override: [f64; 4],
) {
    mascot_body.setFrame(ns_rect_from_panel_rect(primitive.frame));
    let fill = ns_color(visual_color(primitive.fill));
    let stroke = ns_color(stroke_override);
    if let Some(layer) = mascot_body.layer() {
        layer.setCornerRadius(primitive.corner_radius.max(4.0));
        layer.setBackgroundColor(Some(&fill.CGColor()));
        layer.setBorderColor(Some(&stroke.CGColor()));
        layer.setShadowColor(Some(&stroke.CGColor()));
        layer.setShadowOpacity(primitive.shadow_opacity.clamp(0.0, 1.0) as f32);
        layer.setShadowRadius(primitive.shadow_radius);
    }
}

fn macos_mascot_compact_frame() -> PanelRect {
    let layout = resolve_compact_bar_content_layout(CompactBarContentLayoutInput {
        bar_width: MACOS_MASCOT_COMPACT_FRAME_WIDTH,
        bar_height: MACOS_MASCOT_COMPACT_FRAME_HEIGHT,
    });
    PanelRect {
        x: MACOS_MASCOT_BODY_CENTER.x - layout.mascot_center_x,
        y: MACOS_MASCOT_BODY_CENTER.y - MACOS_MASCOT_COMPACT_FRAME_HEIGHT / 2.0,
        width: MACOS_MASCOT_COMPACT_FRAME_WIDTH,
        height: MACOS_MASCOT_COMPACT_FRAME_HEIGHT,
    }
}

fn macos_mascot_visual_frame(frame: NativeMascotFrame) -> MascotVisualFrame {
    MascotVisualFrame {
        offset_x: frame.motion.offset_x,
        offset_y: frame.motion.offset_y,
        scale_x: frame.motion.scale_x,
        scale_y: frame.motion.scale_y,
        shell_alpha: frame.motion.shell_alpha,
        shadow_opacity: frame.motion.shadow_opacity as f64,
        shadow_radius: frame.motion.shadow_radius,
        eye_open: frame.motion.eye_open,
    }
}

fn scene_pose_from_native_state(state: NativeMascotState) -> SceneMascotPose {
    match state {
        NativeMascotState::Idle => SceneMascotPose::Idle,
        NativeMascotState::Bouncing => SceneMascotPose::Running,
        NativeMascotState::Approval => SceneMascotPose::Approval,
        NativeMascotState::Question => SceneMascotPose::Question,
        NativeMascotState::MessageBubble => SceneMascotPose::MessageBubble,
        NativeMascotState::Complete => SceneMascotPose::Complete,
        NativeMascotState::Sleepy => SceneMascotPose::Sleepy,
        NativeMascotState::WakeAngry => SceneMascotPose::WakeAngry,
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

#[cfg(test)]
mod tests {
    use super::{
        MACOS_MASCOT_BODY_CENTER, mascot_body_primitive, mascot_completion_badge_primitive,
        mascot_eye_primitive, mascot_message_bubble_primitive, mascot_mouth_primitive,
        mascot_sleep_label_primitive, resolve_macos_mascot_visual_plan,
    };
    use crate::macos_native_panel::mascot::{
        NativeMascotFrame, NativeMascotMotion, NativeMascotState,
    };
    use crate::native_panel_renderer::facade::visual::NativePanelVisualColor;

    fn frame(state: NativeMascotState) -> NativeMascotFrame {
        frame_at(state, 0.0, 0)
    }

    fn frame_at(state: NativeMascotState, t: f64, completion_count: usize) -> NativeMascotFrame {
        NativeMascotFrame {
            state,
            t,
            motion: NativeMascotMotion {
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shell_alpha: 1.0,
                shadow_opacity: 0.34,
                shadow_radius: 4.0,
                eye_open: 1.0,
            },
            color: [1.0, 0.48, 0.14, 1.0],
            completion_count,
            mascot_hidden: false,
            debug_mode_enabled: false,
            completion_glow_opacity: 0.0,
        }
    }

    #[test]
    fn macos_mascot_visual_plan_places_body_at_native_local_center() {
        let plan = resolve_macos_mascot_visual_plan(frame(NativeMascotState::Idle));
        let primitive = mascot_body_primitive(&plan).expect("mascot body");

        assert!(
            (primitive.frame.x + primitive.frame.width / 2.0 - MACOS_MASCOT_BODY_CENTER.x).abs()
                < 0.001
        );
        assert!(
            (primitive.frame.y + primitive.frame.height / 2.0 - MACOS_MASCOT_BODY_CENTER.y).abs()
                < 0.001
        );
        assert_eq!(primitive.fill, NativePanelVisualColor::rgb(5, 5, 5));
        assert!((primitive.corner_radius - 5.6).abs() < 0.001);
    }

    #[test]
    fn macos_mascot_visual_plan_uses_shared_sleepy_body_fill() {
        let plan = resolve_macos_mascot_visual_plan(frame(NativeMascotState::Sleepy));
        let primitive = mascot_body_primitive(&plan).expect("mascot body");

        assert_eq!(primitive.fill, NativePanelVisualColor::rgb(3, 3, 3));
    }

    #[test]
    fn macos_mascot_visual_plan_exposes_face_primitives_by_role() {
        let plan = resolve_macos_mascot_visual_plan(frame(NativeMascotState::Idle));
        let left_eye = mascot_eye_primitive(&plan, true).expect("left eye");
        let right_eye = mascot_eye_primitive(&plan, false).expect("right eye");
        let mouth = mascot_mouth_primitive(&plan).expect("mouth");

        assert!(left_eye.frame.x < right_eye.frame.x);
        assert_eq!(left_eye.color, NativePanelVisualColor::rgb(255, 255, 255));
        assert_eq!(right_eye.color, NativePanelVisualColor::rgb(255, 255, 255));
        assert_eq!(mouth.color, NativePanelVisualColor::rgb(255, 255, 255));
        assert!((mouth.alpha - 1.0).abs() < 0.001);
    }

    #[test]
    fn macos_mascot_visual_plan_exposes_bubble_sleep_and_completion_primitives() {
        let bubble_plan =
            resolve_macos_mascot_visual_plan(frame_at(NativeMascotState::MessageBubble, 0.5, 0));
        let bubble = mascot_message_bubble_primitive(&bubble_plan).expect("message bubble");

        assert_eq!(
            bubble.bubble.color,
            NativePanelVisualColor::rgb(102, 222, 145)
        );
        assert_eq!(bubble.dots.len(), 3);
        assert!(bubble.bubble.alpha > 0.0);

        let sleep_plan =
            resolve_macos_mascot_visual_plan(frame_at(NativeMascotState::Sleepy, 0.2, 0));
        let sleep = mascot_sleep_label_primitive(&sleep_plan).expect("sleep label");
        assert_eq!(sleep.text, "Z");
        assert_eq!(sleep.color, NativePanelVisualColor::rgb(255, 122, 36));
        assert!(sleep.alpha > 0.0);

        let badge_plan =
            resolve_macos_mascot_visual_plan(frame_at(NativeMascotState::Complete, 0.0, 12));
        let badge = mascot_completion_badge_primitive(&badge_plan).expect("completion badge");
        assert_eq!(badge.fill.color, NativePanelVisualColor::rgb(102, 222, 145));
        assert_eq!(badge.label.text, "12");

        let message_with_badge_plan =
            resolve_macos_mascot_visual_plan(frame_at(NativeMascotState::MessageBubble, 0.5, 7));
        let message_badge =
            mascot_completion_badge_primitive(&message_with_badge_plan).expect("message badge");
        assert_eq!(message_badge.label.text, "7");
    }
}
