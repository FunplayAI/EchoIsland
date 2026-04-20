use super::*;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_native_mascot_frame(
    handles: NativePanelHandles,
    frame: NativeMascotFrame,
) {
    let refs = resolve_native_panel_refs(handles);
    let mascot_shell = refs.mascot_shell;
    let mascot_body = refs.mascot_body;
    let mascot_left_eye = refs.mascot_left_eye;
    let mascot_right_eye = refs.mascot_right_eye;
    let mascot_mouth = refs.mascot_mouth;
    let mascot_bubble = refs.mascot_bubble;
    let mascot_sleep_label = refs.mascot_sleep_label;
    let motion = frame.motion;

    mascot_shell.setAlphaValue(motion.shell_alpha.clamp(0.0, 1.0));
    let body_width = 24.0 * motion.scale_x;
    let body_height = 20.0 * motion.scale_y;
    let body_x = 14.0 - (body_width / 2.0) + motion.offset_x;
    let body_y = 4.0 + MASCOT_VERTICAL_NUDGE_Y + motion.offset_y;
    mascot_body.setFrame(NSRect::new(
        NSPoint::new(body_x, body_y),
        NSSize::new(body_width, body_height),
    ));
    let stroke_color = ns_color(frame.color);
    let body_fill = if frame.state == NativeMascotState::Sleepy {
        ns_color([0.012, 0.012, 0.012, 1.0])
    } else {
        ns_color([0.02, 0.02, 0.02, 1.0])
    };
    if let Some(layer) = mascot_body.layer() {
        layer.setCornerRadius((body_width.min(body_height) * 0.28).max(4.0));
        layer.setBackgroundColor(Some(&body_fill.CGColor()));
        layer.setBorderColor(Some(&stroke_color.CGColor()));
        layer.setShadowColor(Some(&stroke_color.CGColor()));
        layer.setShadowOpacity(motion.shadow_opacity.clamp(0.0, 1.0));
        layer.setShadowRadius(motion.shadow_radius);
    }

    let blink_scale = native_mascot_blink_scale(frame.t, frame.state);
    let open_pct = if frame.state == NativeMascotState::Bouncing {
        ((motion.offset_y - 0.4) / 5.2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (eye_width_factor, eye_height_factor, eye_offset_factor) =
        native_mascot_eye_metrics(frame.state, open_pct);
    let eye_width = (body_width * eye_width_factor).max(2.4);
    let eye_height = (body_height * eye_height_factor * blink_scale).max(
        if matches!(
            frame.state,
            NativeMascotState::Question | NativeMascotState::Sleepy
        ) {
            1.6
        } else {
            2.4
        },
    );
    let eye_center_y = body_y + body_height * 0.58;
    let eye_offset_x = body_width * eye_offset_factor;
    set_mascot_face_part_frame(
        mascot_left_eye,
        body_x + (body_width / 2.0) - eye_offset_x,
        eye_center_y,
        eye_width,
        eye_height,
    );
    set_mascot_face_part_frame(
        mascot_right_eye,
        body_x + (body_width / 2.0) + eye_offset_x,
        eye_center_y,
        eye_width,
        eye_height,
    );

    let mouth_center_y = body_y + body_height * 0.32;
    let (mouth_width, mouth_height, mouth_alpha) =
        native_mascot_mouth_metrics(frame.state, body_width, body_height, open_pct);
    set_mascot_face_part_frame(
        mascot_mouth,
        body_x + (body_width / 2.0),
        mouth_center_y,
        mouth_width,
        mouth_height,
    );
    mascot_mouth.setAlphaValue(mouth_alpha);
    if let Some(layer) = mascot_mouth.layer() {
        layer.setCornerRadius((mouth_height / 2.0).max(0.8));
    }

    let bubble_visible = frame.state == NativeMascotState::MessageBubble;
    let bubble_phase = (frame.t % 1.8) / 1.8;
    let bubble_pop = smoothstep_range(0.0, 0.28, bubble_phase)
        * (1.0 - smoothstep_range(0.78, 1.0, bubble_phase));
    mascot_bubble.setHidden(!bubble_visible || bubble_pop <= 0.06);
    mascot_bubble.setAlphaValue(if bubble_visible { bubble_pop } else { 0.0 });
    mascot_bubble.setFrame(NSRect::new(
        NSPoint::new(
            body_x + body_width * 0.58,
            body_y + body_height * 0.86 + (bubble_pop * 1.4),
        ),
        NSSize::new(body_width * 0.54, body_height * 0.30),
    ));

    let sleep_visible = frame.state == NativeMascotState::Sleepy;
    let sleep_phase = (frame.t % 2.5) / 2.5;
    let sleep_rise = smoothstep_range(0.0, 0.66, sleep_phase);
    let sleep_fade = 1.0 - smoothstep_range(0.58, 1.0, sleep_phase);
    let sleep_alpha = if sleep_visible {
        sleep_rise * sleep_fade
    } else {
        0.0
    };
    mascot_sleep_label.setHidden(sleep_alpha <= 0.03);
    mascot_sleep_label.setAlphaValue(sleep_alpha);
    mascot_sleep_label.setFrame(NSRect::new(
        NSPoint::new(
            body_x + body_width * 0.66 + sleep_rise * body_width * 0.16,
            body_y + body_height * 0.78 + sleep_rise * body_width * 0.16,
        ),
        NSSize::new(10.0, 10.0),
    ));

    for face_part in [mascot_left_eye, mascot_right_eye, mascot_mouth] {
        face_part.setHidden(false);
        if let Some(layer) = face_part.layer() {
            layer.setShadowOpacity(0.22);
            layer.setShadowRadius(6.0);
        }
    }

    mascot_shell.displayIfNeeded();
    mascot_body.displayIfNeeded();
    mascot_left_eye.displayIfNeeded();
    mascot_right_eye.displayIfNeeded();
    mascot_mouth.displayIfNeeded();
    mascot_bubble.displayIfNeeded();
    mascot_sleep_label.displayIfNeeded();
}

fn native_mascot_blink_scale(t: f64, state: NativeMascotState) -> f64 {
    if state == NativeMascotState::WakeAngry {
        return 1.0;
    }

    let phase = (t + 0.35).rem_euclid(4.8);
    let mut scale = if phase < 0.09 {
        1.0 - (phase / 0.09) * 0.9
    } else if phase < 0.18 {
        0.1 + ((phase - 0.09) / 0.09) * 0.9
    } else {
        1.0
    };

    if state == NativeMascotState::Sleepy {
        let sleepy_phase = (t + 1.1).rem_euclid(7.4);
        scale *= 0.72;
        if sleepy_phase > 4.7 && sleepy_phase < 5.45 {
            let pct = (sleepy_phase - 4.7) / 0.75;
            scale = if pct < 0.5 {
                0.18
            } else {
                0.18 + (pct - 0.5) * 0.36
            };
        }
        return scale.max(0.16);
    }

    match state {
        NativeMascotState::Approval => (scale * 0.92).max(0.34),
        NativeMascotState::Question => scale.max(0.48),
        NativeMascotState::Bouncing => scale.max(0.72),
        _ => scale,
    }
}

fn native_mascot_eye_metrics(state: NativeMascotState, open_pct: f64) -> (f64, f64, f64) {
    match state {
        NativeMascotState::Bouncing => {
            (lerp(0.24, 0.20, open_pct), lerp(0.24, 0.20, open_pct), 0.18)
        }
        NativeMascotState::Approval => (0.22, 0.22, 0.18),
        NativeMascotState::Question => (0.26, 0.055, 0.20),
        NativeMascotState::Sleepy => (0.22, 0.085, 0.20),
        NativeMascotState::WakeAngry => (0.20, 0.12, 0.18),
        NativeMascotState::MessageBubble => (0.14, 0.16, 0.20),
        NativeMascotState::Idle => (0.24, 0.24, 0.21),
    }
}

fn native_mascot_mouth_metrics(
    state: NativeMascotState,
    body_width: f64,
    body_height: f64,
    open_pct: f64,
) -> (f64, f64, f64) {
    match state {
        NativeMascotState::Approval => (body_width * 0.34, body_height * 0.11, 1.0),
        NativeMascotState::Question => (body_width * 0.18, body_height * 0.10, 1.0),
        NativeMascotState::Sleepy => (body_width * 0.16, body_height * 0.095, 0.92),
        NativeMascotState::WakeAngry => (body_width * 0.34, body_height * 0.105, 1.0),
        NativeMascotState::MessageBubble => (body_width * 0.16, body_height * 0.085, 1.0),
        NativeMascotState::Bouncing => (
            lerp(body_width * 0.21, body_width * 0.28, open_pct),
            lerp(body_height * 0.08, body_height * 0.30, open_pct),
            1.0,
        ),
        NativeMascotState::Idle => (
            lerp(body_width * 0.20, body_width * 0.32, open_pct),
            lerp(body_height * 0.09, body_height * 0.13, open_pct),
            1.0,
        ),
    }
}

fn set_mascot_face_part_frame(
    view: &NSView,
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
) {
    view.setFrame(NSRect::new(
        NSPoint::new(center_x - (width / 2.0), center_y - (height / 2.0)),
        NSSize::new(width.max(1.0), height.max(1.0)),
    ));
}
