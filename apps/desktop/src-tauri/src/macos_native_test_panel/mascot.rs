use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum NativeMascotState {
    Idle,
    Bouncing,
    Approval,
    Question,
    MessageBubble,
    Sleepy,
    WakeAngry,
}

#[derive(Clone, Copy)]
pub(super) struct NativeMascotMotion {
    offset_x: f64,
    offset_y: f64,
    scale_x: f64,
    scale_y: f64,
    shell_alpha: f64,
    shadow_opacity: f32,
    shadow_radius: f64,
}

#[derive(Clone, Copy)]
pub(super) struct NativeMascotFrame {
    state: NativeMascotState,
    t: f64,
    motion: NativeMascotMotion,
    color: [f64; 4],
}

pub(super) struct NativeMascotRuntime {
    animation_started_at: Instant,
    last_non_idle_at: Instant,
    last_resolved_state: NativeMascotState,
    wake_started_at: Option<Instant>,
    wake_next_state: NativeMascotState,
    transition_target: NativeMascotState,
    transition_started_at: Instant,
    transition_start_motion: NativeMascotMotion,
    last_motion: NativeMascotMotion,
}

impl NativeMascotMotion {
    fn idle() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shell_alpha: 1.0,
            shadow_opacity: 0.34,
            shadow_radius: 4.0,
        }
    }
}

impl NativeMascotRuntime {
    pub(super) fn new(now: Instant) -> Self {
        let idle_motion = NativeMascotMotion::idle();
        Self {
            animation_started_at: now,
            last_non_idle_at: now,
            last_resolved_state: NativeMascotState::Idle,
            wake_started_at: None,
            wake_next_state: NativeMascotState::Idle,
            transition_target: NativeMascotState::Idle,
            transition_started_at: now,
            transition_start_motion: idle_motion,
            last_motion: idle_motion,
        }
    }

    fn next_frame(
        &mut self,
        base_state: NativeMascotState,
        expanded: bool,
        now: Instant,
    ) -> NativeMascotFrame {
        let t = now
            .saturating_duration_since(self.animation_started_at)
            .as_secs_f64();
        let visual_state = self.resolve_visual_state(base_state, expanded, now);
        let target_motion = native_mascot_target_motion(visual_state, t, self.wake_started_at, now);

        if self.transition_target != visual_state {
            self.transition_target = visual_state;
            self.transition_started_at = now;
            self.transition_start_motion = self.last_motion;
        }

        let transition_progress = now
            .saturating_duration_since(self.transition_started_at)
            .as_secs_f64()
            / MASCOT_STATE_TRANSITION_SECONDS;
        let motion = native_mascot_lerp_motion(
            self.transition_start_motion,
            target_motion,
            smoothstep_unit(transition_progress),
        );
        self.last_motion = motion;

        NativeMascotFrame {
            state: visual_state,
            t,
            motion,
            color: native_mascot_color(visual_state, t, self.wake_started_at, now),
        }
    }

    fn resolve_visual_state(
        &mut self,
        base_state: NativeMascotState,
        expanded: bool,
        now: Instant,
    ) -> NativeMascotState {
        let mut next_state = if base_state != NativeMascotState::Idle {
            self.last_non_idle_at = now;
            base_state
        } else if expanded {
            self.last_non_idle_at = now;
            NativeMascotState::Idle
        } else if now
            .saturating_duration_since(self.last_non_idle_at)
            .as_secs()
            >= MASCOT_IDLE_LONG_SECONDS
        {
            NativeMascotState::Sleepy
        } else {
            NativeMascotState::Idle
        };

        let waking_from_sleep = next_state != NativeMascotState::Sleepy
            && self.wake_started_at.is_none()
            && self.last_resolved_state == NativeMascotState::Sleepy;
        if waking_from_sleep {
            self.wake_started_at = Some(now);
            self.wake_next_state = next_state;
            self.last_resolved_state = NativeMascotState::WakeAngry;
            return NativeMascotState::WakeAngry;
        }

        if let Some(started_at) = self.wake_started_at {
            self.wake_next_state = if next_state == NativeMascotState::Sleepy {
                NativeMascotState::Idle
            } else {
                next_state
            };

            if now.saturating_duration_since(started_at).as_secs_f64() < MASCOT_WAKE_ANGRY_SECONDS {
                self.last_resolved_state = NativeMascotState::WakeAngry;
                return NativeMascotState::WakeAngry;
            }

            self.wake_started_at = None;
            next_state = self.wake_next_state;
        }

        self.last_resolved_state = next_state;
        next_state
    }
}

fn infer_native_mascot_base_state(
    snapshot: Option<&RuntimeSnapshot>,
    has_status_completion: bool,
) -> NativeMascotState {
    let Some(snapshot) = snapshot else {
        return NativeMascotState::Idle;
    };

    if snapshot.pending_permission_count > 0 {
        return NativeMascotState::Approval;
    }
    if snapshot.pending_question_count > 0 {
        return NativeMascotState::Question;
    }
    if has_status_completion {
        return NativeMascotState::MessageBubble;
    }
    if compact_active_count_value(snapshot) > 0 || snapshot.active_session_count > 0 {
        return NativeMascotState::Bouncing;
    }

    NativeMascotState::Idle
}

fn native_mascot_target_motion(
    state: NativeMascotState,
    t: f64,
    wake_started_at: Option<Instant>,
    now: Instant,
) -> NativeMascotMotion {
    match state {
        NativeMascotState::Bouncing => {
            let bounce = (t * 5.8).sin().abs();
            let hang = bounce.powf(0.72);
            let landing = (1.0 - bounce).powf(3.2);
            NativeMascotMotion {
                offset_x: (t * 3.1).sin() * 0.28,
                offset_y: hang * 5.6,
                scale_x: 1.0 + landing * 0.18 + hang * 0.018,
                scale_y: 1.0 - landing * 0.16 + hang * 0.018,
                shell_alpha: 1.0,
                shadow_opacity: 0.46,
                shadow_radius: 5.4,
            }
        }
        NativeMascotState::Approval => {
            let pulse = ((t * 7.2).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: (t * 9.0).sin() * 0.34,
                offset_y: 0.0,
                scale_x: 1.0 + pulse * 0.025,
                scale_y: 1.0 - pulse * 0.018,
                shell_alpha: 1.0,
                shadow_opacity: 0.52,
                shadow_radius: 6.0,
            }
        }
        NativeMascotState::Question => {
            let tilt = (t * 4.4).sin();
            NativeMascotMotion {
                offset_x: tilt * 0.28,
                offset_y: (t * 5.1).sin() * 0.55,
                scale_x: 1.0 + tilt.abs() * 0.012,
                scale_y: 1.0,
                shell_alpha: 1.0,
                shadow_opacity: 0.50,
                shadow_radius: 5.8,
            }
        }
        NativeMascotState::MessageBubble => {
            let bob = ((t * 3.2).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: bob * 1.6,
                scale_x: 1.0 + bob * 0.012,
                scale_y: 1.0 - bob * 0.008,
                shell_alpha: 1.0,
                shadow_opacity: 0.46,
                shadow_radius: 5.2,
            }
        }
        NativeMascotState::Sleepy => {
            let breath = ((t * 0.9).sin() + 1.0) * 0.5;
            let sleepy_phase = (t + 0.9).rem_euclid(7.6);
            let nod = if sleepy_phase > 5.1 && sleepy_phase < 5.95 {
                (((sleepy_phase - 5.1) / 0.85) * std::f64::consts::PI).sin()
            } else {
                0.0
            };
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: nod * -0.7,
                scale_x: 1.0 + breath * 0.012,
                scale_y: 0.96 - breath * 0.012 + nod * 0.01,
                shell_alpha: 0.70,
                shadow_opacity: 0.18,
                shadow_radius: 3.0,
            }
        }
        NativeMascotState::WakeAngry => {
            let elapsed = wake_started_at
                .map(|started_at| now.saturating_duration_since(started_at).as_secs_f64())
                .unwrap_or(0.0);
            let fade = 1.0 - smoothstep_range(0.52, MASCOT_WAKE_ANGRY_SECONDS, elapsed);
            NativeMascotMotion {
                offset_x: (elapsed * 30.0).sin() * 1.85 * fade,
                offset_y: 0.0,
                scale_x: 1.0 + 0.045 * fade,
                scale_y: 1.0 - 0.04 * fade,
                shell_alpha: 1.0,
                shadow_opacity: 0.56,
                shadow_radius: 6.4,
            }
        }
        NativeMascotState::Idle => {
            let breath = ((t * 1.1).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: 1.0 + breath * 0.006,
                scale_y: 1.0 - breath * 0.004,
                shell_alpha: 1.0,
                shadow_opacity: 0.34,
                shadow_radius: 4.0,
            }
        }
    }
}

fn native_mascot_color(
    state: NativeMascotState,
    _t: f64,
    wake_started_at: Option<Instant>,
    now: Instant,
) -> [f64; 4] {
    match state {
        NativeMascotState::Approval | NativeMascotState::Question => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::MessageBubble => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::Bouncing => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::Sleepy => [0.72, 0.30, 0.13, 1.0],
        NativeMascotState::WakeAngry => {
            let elapsed = wake_started_at
                .map(|started_at| now.saturating_duration_since(started_at).as_secs_f64())
                .unwrap_or(0.0);
            let blink = if (elapsed * 12.0).sin() >= 0.0 {
                1.0
            } else {
                0.0
            };
            [
                lerp(1.0, 1.0, blink),
                lerp(0.38, 0.48, blink),
                lerp(0.24, 0.14, blink),
                1.0,
            ]
        }
        NativeMascotState::Idle => [1.0, 0.48, 0.14, 1.0],
    }
}

fn native_mascot_lerp_motion(
    start: NativeMascotMotion,
    end: NativeMascotMotion,
    progress: f64,
) -> NativeMascotMotion {
    NativeMascotMotion {
        offset_x: lerp(start.offset_x, end.offset_x, progress),
        offset_y: lerp(start.offset_y, end.offset_y, progress),
        scale_x: lerp(start.scale_x, end.scale_x, progress),
        scale_y: lerp(start.scale_y, end.scale_y, progress),
        shell_alpha: lerp(start.shell_alpha, end.shell_alpha, progress),
        shadow_opacity: lerp(
            start.shadow_opacity as f64,
            end.shadow_opacity as f64,
            progress,
        ) as f32,
        shadow_radius: lerp(start.shadow_radius, end.shadow_radius, progress),
    }
}

fn smoothstep_unit(progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - (2.0 * progress))
}

fn smoothstep_range(edge0: f64, edge1: f64, value: f64) -> f64 {
    if (edge1 - edge0).abs() <= f64::EPSILON {
        return if value >= edge1 { 1.0 } else { 0.0 };
    }
    smoothstep_unit((value - edge0) / (edge1 - edge0))
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn sync_native_mascot(handles: NativePanelHandles) {
    let now = Instant::now();
    let Some(state_mutex) = native_panel_state() else {
        return;
    };

    let frame = {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let has_status_completion = state
            .status_queue
            .iter()
            .any(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)));
        let base_state =
            infer_native_mascot_base_state(state.last_snapshot.as_ref(), has_status_completion);
        let expanded = state.expanded;
        state.mascot_runtime.next_frame(base_state, expanded, now)
    };

    apply_native_mascot_frame(handles, frame);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_native_mascot_frame(handles: NativePanelHandles, frame: NativeMascotFrame) {
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
