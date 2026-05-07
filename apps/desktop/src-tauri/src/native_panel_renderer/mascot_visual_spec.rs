use crate::{
    native_panel_core::{
        MascotVisualFrame, MascotVisualFrameInput, PanelMascotBaseState, PanelPoint, PanelRect,
        lerp, resolve_mascot_visual_frame,
    },
    native_panel_scene::SceneMascotPose,
};

use super::visual_primitives::{NativePanelVisualColor, NativePanelVisualTextWeight};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MascotVisualSpecInput {
    pub(crate) body_center: PanelPoint,
    pub(crate) completion_badge_anchor: PanelPoint,
    pub(crate) radius: f64,
    pub(crate) pose: SceneMascotPose,
    pub(crate) debug_mode_enabled: bool,
    pub(crate) completion_count: usize,
    pub(crate) elapsed_ms: u128,
    pub(crate) motion_frame: Option<MascotVisualFrame>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MascotVisualSpec {
    pub(crate) pose: SceneMascotPose,
    pub(crate) motion: MascotVisualFrame,
    pub(crate) body: MascotBodyVisualSpec,
    pub(crate) eyes: Vec<MascotEllipseVisualSpec>,
    pub(crate) mouth: MascotRoundRectVisualSpec,
    pub(crate) message_bubble: Option<MascotMessageBubbleVisualSpec>,
    pub(crate) sleep_label: Option<MascotTextVisualSpec>,
    pub(crate) completion_badge: Option<MascotCompletionBadgeVisualSpec>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MascotBodyVisualSpec {
    pub(crate) center: PanelPoint,
    pub(crate) frame: PanelRect,
    pub(crate) radius: f64,
    pub(crate) corner_radius: f64,
    pub(crate) scale_x: f64,
    pub(crate) scale_y: f64,
    pub(crate) color: NativePanelVisualColor,
    pub(crate) fill_color: NativePanelVisualColor,
    pub(crate) stroke_color: NativePanelVisualColor,
    pub(crate) stroke_width: f64,
    pub(crate) shadow_opacity: f64,
    pub(crate) shadow_radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MascotEllipseVisualSpec {
    pub(crate) frame: PanelRect,
    pub(crate) color: NativePanelVisualColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MascotRoundRectVisualSpec {
    pub(crate) frame: PanelRect,
    pub(crate) radius: f64,
    pub(crate) color: NativePanelVisualColor,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MascotTextVisualSpec {
    pub(crate) origin: PanelPoint,
    pub(crate) max_width: f64,
    pub(crate) text: String,
    pub(crate) color: NativePanelVisualColor,
    pub(crate) size: i32,
    pub(crate) weight: NativePanelVisualTextWeight,
    pub(crate) alpha: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MascotMessageBubbleVisualSpec {
    pub(crate) bubble: MascotRoundRectVisualSpec,
    pub(crate) dots: Vec<MascotEllipseVisualSpec>,
    pub(crate) alpha: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MascotCompletionBadgeVisualSpec {
    pub(crate) outline: MascotRoundRectVisualSpec,
    pub(crate) fill: MascotRoundRectVisualSpec,
    pub(crate) label: MascotTextVisualSpec,
    pub(crate) text: String,
}

pub(crate) fn resolve_mascot_visual_spec(input: MascotVisualSpecInput) -> MascotVisualSpec {
    let frame = input.motion_frame.unwrap_or_else(|| {
        resolve_mascot_visual_frame(MascotVisualFrameInput {
            state: panel_mascot_state_from_scene_pose(input.pose),
            elapsed_ms: input.elapsed_ms,
        })
    });
    let center = PanelPoint {
        x: input.body_center.x + frame.offset_x,
        y: input.body_center.y + frame.offset_y,
    };
    let body_width = input.radius * (24.0 / 11.0) * frame.scale_x;
    let body_height = input.radius * (20.0 / 11.0) * frame.scale_y;
    let body_frame = centered_rect(center, body_width / 2.0, body_height / 2.0);
    let body = MascotBodyVisualSpec {
        center,
        frame: body_frame,
        radius: input.radius,
        corner_radius: (body_width.min(body_height) * 0.28).max(4.0),
        scale_x: frame.scale_x,
        scale_y: frame.scale_y,
        color: if input.debug_mode_enabled {
            NativePanelVisualColor::rgb(255, 255, 255)
        } else {
            NativePanelVisualColor::rgb(255, 122, 36)
        },
        fill_color: if input.debug_mode_enabled {
            NativePanelVisualColor::rgb(255, 255, 255)
        } else if input.pose == SceneMascotPose::Sleepy {
            NativePanelVisualColor::rgb(3, 3, 3)
        } else {
            NativePanelVisualColor::rgb(5, 5, 5)
        },
        stroke_color: if input.debug_mode_enabled {
            NativePanelVisualColor::rgb(255, 255, 255)
        } else {
            NativePanelVisualColor::rgb(255, 122, 36)
        },
        stroke_width: 2.2,
        shadow_opacity: frame.shadow_opacity,
        shadow_radius: frame.shadow_radius,
    };
    let open_pct = if input.pose == SceneMascotPose::Running {
        ((frame.offset_y - 0.4) / 5.2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let face_color = if input.debug_mode_enabled {
        NativePanelVisualColor::rgb(24, 24, 28)
    } else {
        NativePanelVisualColor::rgb(255, 255, 255)
    };
    let eyes = mascot_eyes(
        input.pose,
        body_frame,
        body_width,
        body_height,
        frame.eye_open,
        open_pct,
        face_color,
    );
    let mouth = mascot_mouth(
        input.pose,
        body_frame,
        body_width,
        body_height,
        open_pct,
        face_color,
    );

    MascotVisualSpec {
        pose: input.pose,
        motion: frame,
        body,
        eyes,
        mouth,
        message_bubble: (input.pose == SceneMascotPose::MessageBubble)
            .then(|| mascot_message_bubble(body_frame, input.elapsed_ms))
            .flatten(),
        sleep_label: (input.pose == SceneMascotPose::Sleepy)
            .then(|| mascot_sleep_label(body_frame, input.elapsed_ms))
            .flatten(),
        completion_badge: mascot_completion_badge(
            input.completion_badge_anchor,
            input.completion_count,
        ),
    }
}

fn mascot_eyes(
    pose: SceneMascotPose,
    body_frame: PanelRect,
    body_width: f64,
    body_height: f64,
    eye_open: f64,
    open_pct: f64,
    color: NativePanelVisualColor,
) -> Vec<MascotEllipseVisualSpec> {
    let (eye_width_factor, eye_height_factor, eye_offset_factor) =
        mascot_eye_metrics(pose, open_pct);
    let eye_width = (body_width * eye_width_factor).max(2.4);
    let eye_height = (body_height * eye_height_factor * eye_open.clamp(0.12, 1.0)).max(
        if matches!(pose, SceneMascotPose::Question | SceneMascotPose::Sleepy) {
            1.6
        } else {
            2.4
        },
    );
    let eye_center_y = body_frame.y + body_frame.height * 0.58;
    let eye_offset_x = body_width * eye_offset_factor;
    [-eye_offset_x, eye_offset_x]
        .into_iter()
        .map(|x_offset| MascotEllipseVisualSpec {
            frame: centered_rect(
                PanelPoint {
                    x: body_frame.x + body_width / 2.0 + x_offset,
                    y: eye_center_y,
                },
                eye_width / 2.0,
                eye_height / 2.0,
            ),
            color,
        })
        .collect()
}

fn mascot_mouth(
    pose: SceneMascotPose,
    body_frame: PanelRect,
    body_width: f64,
    body_height: f64,
    open_pct: f64,
    color: NativePanelVisualColor,
) -> MascotRoundRectVisualSpec {
    let (mouth_width, mouth_height) = mascot_mouth_metrics(pose, body_width, body_height, open_pct);
    MascotRoundRectVisualSpec {
        frame: centered_rect(
            PanelPoint {
                x: body_frame.x + body_width / 2.0,
                y: body_frame.y + body_height * 0.32,
            },
            mouth_width / 2.0,
            mouth_height / 2.0,
        ),
        radius: mouth_height / 2.0,
        color,
    }
}

fn mascot_eye_metrics(pose: SceneMascotPose, open_pct: f64) -> (f64, f64, f64) {
    match pose {
        SceneMascotPose::Running => (lerp(0.24, 0.20, open_pct), lerp(0.24, 0.20, open_pct), 0.18),
        SceneMascotPose::Approval => (0.22, 0.22, 0.18),
        SceneMascotPose::Question => (0.26, 0.055, 0.20),
        SceneMascotPose::Sleepy => (0.22, 0.085, 0.20),
        SceneMascotPose::WakeAngry => (0.20, 0.12, 0.18),
        SceneMascotPose::MessageBubble => (0.14, 0.16, 0.20),
        SceneMascotPose::Complete => (0.22, 0.18, 0.19),
        SceneMascotPose::Idle | SceneMascotPose::Hidden => (0.24, 0.24, 0.21),
    }
}

fn mascot_mouth_metrics(
    pose: SceneMascotPose,
    body_width: f64,
    body_height: f64,
    open_pct: f64,
) -> (f64, f64) {
    match pose {
        SceneMascotPose::Approval => (body_width * 0.34, body_height * 0.11),
        SceneMascotPose::Question => (body_width * 0.18, body_height * 0.10),
        SceneMascotPose::Sleepy => (body_width * 0.16, body_height * 0.095),
        SceneMascotPose::WakeAngry => (body_width * 0.34, body_height * 0.105),
        SceneMascotPose::MessageBubble => (body_width * 0.16, body_height * 0.085),
        SceneMascotPose::Complete => (body_width * 0.38, body_height * 0.085),
        SceneMascotPose::Running => (
            lerp(body_width * 0.21, body_width * 0.28, open_pct),
            lerp(body_height * 0.08, body_height * 0.30, open_pct),
        ),
        SceneMascotPose::Idle | SceneMascotPose::Hidden => (
            lerp(body_width * 0.20, body_width * 0.32, open_pct),
            lerp(body_height * 0.09, body_height * 0.13, open_pct),
        ),
    }
}

fn mascot_message_bubble(
    body_frame: PanelRect,
    elapsed_ms: u128,
) -> Option<MascotMessageBubbleVisualSpec> {
    let phase = ((elapsed_ms as f64 / 1000.0) % 1.8) / 1.8;
    let pop = smoothstep_range(0.0, 0.28, phase) * (1.0 - smoothstep_range(0.78, 1.0, phase));
    if pop <= 0.06 {
        return None;
    }
    let frame = PanelRect {
        x: body_frame.x + body_frame.width * 0.58,
        y: body_frame.y + body_frame.height * 0.86 + pop * 1.4,
        width: body_frame.width * 0.54,
        height: body_frame.height * 0.30,
    };
    let dots = (0..3)
        .map(|index| MascotEllipseVisualSpec {
            frame: centered_rect(
                PanelPoint {
                    x: frame.x + frame.width * (0.30 + index as f64 * 0.20),
                    y: frame.y + frame.height * 0.52,
                },
                1.1,
                1.1,
            ),
            color: NativePanelVisualColor::rgb(5, 5, 5),
        })
        .collect();
    Some(MascotMessageBubbleVisualSpec {
        bubble: MascotRoundRectVisualSpec {
            frame,
            radius: frame.height / 2.0,
            color: NativePanelVisualColor::rgb(102, 222, 145),
        },
        dots,
        alpha: pop,
    })
}

fn mascot_sleep_label(body_frame: PanelRect, elapsed_ms: u128) -> Option<MascotTextVisualSpec> {
    let phase = ((elapsed_ms as f64 / 1000.0) % 2.5) / 2.5;
    let rise = smoothstep_range(0.0, 0.66, phase);
    let fade = 1.0 - smoothstep_range(0.58, 1.0, phase);
    let alpha = rise * fade;
    if alpha <= 0.03 {
        return None;
    }
    Some(MascotTextVisualSpec {
        origin: PanelPoint {
            x: body_frame.x + body_frame.width * 0.66 + rise * body_frame.width * 0.16,
            y: body_frame.y + body_frame.height * 0.78 + rise * body_frame.width * 0.16,
        },
        max_width: 10.0,
        text: "Z".to_string(),
        color: NativePanelVisualColor::rgb(255, 122, 36),
        size: 9,
        weight: NativePanelVisualTextWeight::Bold,
        alpha,
    })
}

fn mascot_completion_badge(
    center: PanelPoint,
    completion_count: usize,
) -> Option<MascotCompletionBadgeVisualSpec> {
    if completion_count == 0 {
        return None;
    }
    let count = completion_count.min(99);
    let width = if count >= 10 { 16.0 } else { 13.0 };
    let badge = PanelRect {
        x: center.x + 6.0,
        y: center.y + 2.0,
        width,
        height: 13.0,
    };
    let text_inset_x = if count >= 10 { 2.0 } else { 4.0 };
    Some(MascotCompletionBadgeVisualSpec {
        outline: MascotRoundRectVisualSpec {
            frame: PanelRect {
                x: badge.x - 1.0,
                y: badge.y - 1.0,
                width: badge.width + 2.0,
                height: badge.height + 2.0,
            },
            radius: 7.5,
            color: NativePanelVisualColor::rgb(240, 255, 246),
        },
        fill: MascotRoundRectVisualSpec {
            frame: badge,
            radius: 6.5,
            color: NativePanelVisualColor::rgb(102, 222, 145),
        },
        label: MascotTextVisualSpec {
            origin: PanelPoint {
                x: badge.x + text_inset_x,
                y: badge.y - 1.5,
            },
            max_width: (badge.width - text_inset_x * 2.0).max(1.0),
            text: count.to_string(),
            color: NativePanelVisualColor::rgb(5, 5, 5),
            size: 8,
            weight: NativePanelVisualTextWeight::Bold,
            alpha: 1.0,
        },
        text: count.to_string(),
    })
}

fn panel_mascot_state_from_scene_pose(pose: SceneMascotPose) -> PanelMascotBaseState {
    match pose {
        SceneMascotPose::Running => PanelMascotBaseState::Running,
        SceneMascotPose::Approval => PanelMascotBaseState::Approval,
        SceneMascotPose::Question => PanelMascotBaseState::Question,
        SceneMascotPose::MessageBubble => PanelMascotBaseState::MessageBubble,
        SceneMascotPose::Complete => PanelMascotBaseState::Complete,
        SceneMascotPose::Sleepy => PanelMascotBaseState::Sleepy,
        SceneMascotPose::WakeAngry => PanelMascotBaseState::WakeAngry,
        SceneMascotPose::Idle | SceneMascotPose::Hidden => PanelMascotBaseState::Idle,
    }
}

fn smoothstep_range(edge0: f64, edge1: f64, value: f64) -> f64 {
    if edge1 <= edge0 {
        return if value >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn centered_rect(center: PanelPoint, radius_x: f64, radius_y: f64) -> PanelRect {
    PanelRect {
        x: center.x - radius_x,
        y: center.y - radius_y,
        width: radius_x * 2.0,
        height: radius_y * 2.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        native_panel_core::{MascotVisualFrame, PanelPoint},
        native_panel_renderer::facade::visual::NativePanelVisualColor,
        native_panel_scene::SceneMascotPose,
    };

    use super::{MascotVisualSpecInput, resolve_mascot_visual_spec};

    #[test]
    fn mascot_visual_spec_resolves_body_face_and_completion_badge() {
        let spec = resolve_mascot_visual_spec(MascotVisualSpecInput {
            body_center: PanelPoint { x: 22.0, y: 18.0 },
            completion_badge_anchor: PanelPoint { x: 22.0, y: 18.0 },
            radius: 11.0,
            pose: SceneMascotPose::Complete,
            debug_mode_enabled: false,
            completion_count: 12,
            elapsed_ms: 0,
            motion_frame: None,
        });

        assert_eq!(spec.pose, SceneMascotPose::Complete);
        assert_eq!(spec.eyes.len(), 2);
        assert!(spec.body.frame.width > 24.0);
        assert!(spec.body.frame.height < 20.0);
        assert_eq!(spec.completion_badge.as_ref().unwrap().text, "12");
        assert!(spec.message_bubble.is_none());
        assert!(spec.sleep_label.is_none());
    }

    #[test]
    fn mascot_visual_spec_exposes_body_paint_style() {
        let spec = resolve_mascot_visual_spec(MascotVisualSpecInput {
            body_center: PanelPoint { x: 22.0, y: 18.0 },
            completion_badge_anchor: PanelPoint { x: 22.0, y: 18.0 },
            radius: 11.0,
            pose: SceneMascotPose::Idle,
            debug_mode_enabled: false,
            completion_count: 0,
            elapsed_ms: 0,
            motion_frame: Some(MascotVisualFrame {
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shell_alpha: 1.0,
                shadow_opacity: 0.42,
                shadow_radius: 7.0,
                eye_open: 1.0,
            }),
        });
        let sleepy = resolve_mascot_visual_spec(MascotVisualSpecInput {
            pose: SceneMascotPose::Sleepy,
            ..MascotVisualSpecInput {
                body_center: PanelPoint { x: 22.0, y: 18.0 },
                completion_badge_anchor: PanelPoint { x: 22.0, y: 18.0 },
                radius: 11.0,
                pose: SceneMascotPose::Idle,
                debug_mode_enabled: false,
                completion_count: 0,
                elapsed_ms: 0,
                motion_frame: None,
            }
        });
        let debug = resolve_mascot_visual_spec(MascotVisualSpecInput {
            debug_mode_enabled: true,
            ..MascotVisualSpecInput {
                body_center: PanelPoint { x: 22.0, y: 18.0 },
                completion_badge_anchor: PanelPoint { x: 22.0, y: 18.0 },
                radius: 11.0,
                pose: SceneMascotPose::Idle,
                debug_mode_enabled: false,
                completion_count: 0,
                elapsed_ms: 0,
                motion_frame: None,
            }
        });

        assert_eq!(spec.body.fill_color, NativePanelVisualColor::rgb(5, 5, 5));
        assert_eq!(
            spec.body.stroke_color,
            NativePanelVisualColor::rgb(255, 122, 36)
        );
        assert!((spec.body.corner_radius - 5.6).abs() < 0.001);
        assert_eq!(spec.body.stroke_width, 2.2);
        assert_eq!(spec.body.shadow_opacity, 0.42);
        assert_eq!(spec.body.shadow_radius, 7.0);
        assert_eq!(sleepy.body.fill_color, NativePanelVisualColor::rgb(3, 3, 3));
        assert_eq!(
            debug.body.fill_color,
            NativePanelVisualColor::rgb(255, 255, 255)
        );
        assert_eq!(
            debug.body.stroke_color,
            NativePanelVisualColor::rgb(255, 255, 255)
        );
    }

    #[test]
    fn mascot_visual_spec_resolves_message_bubble_and_sleep_label_phases() {
        let message = resolve_mascot_visual_spec(MascotVisualSpecInput {
            body_center: PanelPoint { x: 22.0, y: 18.0 },
            completion_badge_anchor: PanelPoint { x: 22.0, y: 18.0 },
            radius: 11.0,
            pose: SceneMascotPose::MessageBubble,
            debug_mode_enabled: false,
            completion_count: 0,
            elapsed_ms: 500,
            motion_frame: None,
        });
        let sleep = resolve_mascot_visual_spec(MascotVisualSpecInput {
            body_center: PanelPoint { x: 22.0, y: 18.0 },
            completion_badge_anchor: PanelPoint { x: 22.0, y: 18.0 },
            radius: 11.0,
            pose: SceneMascotPose::Sleepy,
            debug_mode_enabled: false,
            completion_count: 0,
            elapsed_ms: 1500,
            motion_frame: None,
        });

        assert_eq!(message.message_bubble.as_ref().unwrap().dots.len(), 3);
        assert!(message.completion_badge.is_none());
        assert_eq!(sleep.sleep_label.as_ref().unwrap().text, "Z");
    }

    #[test]
    fn mascot_visual_spec_keeps_completion_badge_on_fixed_anchor() {
        let spec = resolve_mascot_visual_spec(MascotVisualSpecInput {
            body_center: PanelPoint { x: 22.0, y: 18.0 },
            completion_badge_anchor: PanelPoint { x: 40.0, y: 12.0 },
            radius: 11.0,
            pose: SceneMascotPose::Complete,
            debug_mode_enabled: false,
            completion_count: 7,
            elapsed_ms: 0,
            motion_frame: None,
        });
        let badge = spec.completion_badge.as_ref().expect("completion badge");

        assert_eq!(badge.fill.frame.x, 46.0);
        assert_eq!(badge.fill.frame.y, 14.0);
        assert_ne!(badge.fill.frame.y, spec.body.center.y + 2.0);
    }
}
