use super::*;
use objc2::rc::Retained;

pub(super) struct MascotViews {
    pub(super) shell: Retained<NSView>,
    pub(super) body: Retained<NSView>,
    pub(super) left_eye: Retained<NSView>,
    pub(super) right_eye: Retained<NSView>,
    pub(super) mouth: Retained<NSView>,
    pub(super) bubble: Retained<NSView>,
    pub(super) sleep_label: Retained<NSTextField>,
    pub(super) completion_badge: Retained<NSView>,
    pub(super) completion_badge_label: Retained<NSTextField>,
}

pub(super) fn create_mascot_views(
    mtm: MainThreadMarker,
    shell_border: &NSColor,
    body_fill: &NSColor,
    stroke: &NSColor,
    face: &NSColor,
) -> MascotViews {
    let shell = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(10.0, 6.0 + MASCOT_VERTICAL_NUDGE_Y),
            NSSize::new(28.0, 28.0),
        ),
    );
    shell.setWantsLayer(true);
    let shell_layer = CALayer::layer();
    shell_layer.setCornerRadius(7.0);
    shell_layer.setMasksToBounds(false);
    shell_layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    shell_layer.setBorderWidth(0.0);
    shell_layer.setBorderColor(Some(&shell_border.CGColor()));
    shell.setLayer(Some(&shell_layer));

    let body = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(2.0, 4.0 + MASCOT_VERTICAL_NUDGE_Y),
            NSSize::new(24.0, 20.0),
        ),
    );
    body.setWantsLayer(true);
    let body_layer = CALayer::layer();
    body_layer.setCornerRadius(6.0);
    body_layer.setMasksToBounds(false);
    body_layer.setBackgroundColor(Some(&body_fill.CGColor()));
    body_layer.setBorderWidth(2.2);
    body_layer.setBorderColor(Some(&stroke.CGColor()));
    body_layer.setShadowColor(Some(&stroke.CGColor()));
    body_layer.setShadowOpacity(0.18);
    body_layer.setShadowRadius(4.0);
    body.setLayer(Some(&body_layer));
    shell.addSubview(&body);

    let left_eye = create_eye_view(mtm, face, 7.0);
    shell.addSubview(&left_eye);

    let right_eye = create_eye_view(mtm, face, 15.3);
    shell.addSubview(&right_eye);

    let mouth = create_mouth_view(mtm, face);
    shell.addSubview(&mouth);

    let bubble = create_bubble_view(mtm, face);
    shell.addSubview(&bubble);

    let sleep_label = create_sleep_label(mtm, face);
    shell.addSubview(&sleep_label);

    let (completion_badge, completion_badge_label) = create_completion_badge(mtm);
    shell.addSubview(&completion_badge);

    MascotViews {
        shell,
        body,
        left_eye,
        right_eye,
        mouth,
        bubble,
        sleep_label,
        completion_badge,
        completion_badge_label,
    }
}

fn create_eye_view(mtm: MainThreadMarker, face: &NSColor, x: f64) -> Retained<NSView> {
    let eye = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(x, 14.1), NSSize::new(5.7, 4.8)),
    );
    eye.setWantsLayer(true);
    let eye_layer = CALayer::layer();
    eye_layer.setCornerRadius(2.4);
    eye_layer.setMasksToBounds(true);
    eye_layer.setBackgroundColor(Some(&face.CGColor()));
    eye_layer.setShadowColor(Some(&face.CGColor()));
    eye_layer.setShadowOpacity(0.22);
    eye_layer.setShadowRadius(6.0);
    eye.setLayer(Some(&eye_layer));
    eye
}

fn create_mouth_view(mtm: MainThreadMarker, face: &NSColor) -> Retained<NSView> {
    let mouth = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(10.5, 9.0), NSSize::new(7.0, 2.2)),
    );
    mouth.setWantsLayer(true);
    let mouth_layer = CALayer::layer();
    mouth_layer.setCornerRadius(1.1);
    mouth_layer.setMasksToBounds(true);
    mouth_layer.setBackgroundColor(Some(&face.CGColor()));
    mouth_layer.setShadowColor(Some(&face.CGColor()));
    mouth_layer.setShadowOpacity(0.20);
    mouth_layer.setShadowRadius(6.0);
    mouth.setLayer(Some(&mouth_layer));
    mouth
}

fn create_bubble_view(mtm: MainThreadMarker, face: &NSColor) -> Retained<NSView> {
    let bubble = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(18.0, 19.5), NSSize::new(14.0, 7.5)),
    );
    bubble.setWantsLayer(true);
    let bubble_layer = CALayer::layer();
    bubble_layer.setCornerRadius(3.7);
    bubble_layer.setMasksToBounds(true);
    bubble_layer.setBackgroundColor(Some(&face.CGColor()));
    bubble_layer.setShadowColor(Some(&face.CGColor()));
    bubble_layer.setShadowOpacity(0.24);
    bubble_layer.setShadowRadius(7.0);
    bubble.setLayer(Some(&bubble_layer));
    bubble.setHidden(true);
    for x in [3.6, 6.8, 10.0] {
        let dot = NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(NSPoint::new(x, 3.0), NSSize::new(1.6, 1.6)),
        );
        dot.setWantsLayer(true);
        let dot_layer = CALayer::layer();
        dot_layer.setCornerRadius(0.8);
        dot_layer.setMasksToBounds(true);
        dot_layer.setBackgroundColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.02, 0.02, 0.02, 0.72).CGColor(),
        ));
        dot.setLayer(Some(&dot_layer));
        bubble.addSubview(&dot);
    }
    bubble
}

fn create_sleep_label(mtm: MainThreadMarker, face: &NSColor) -> Retained<NSTextField> {
    let sleep_label = NSTextField::labelWithString(ns_string!("Z"), mtm);
    sleep_label.setFrame(NSRect::new(
        NSPoint::new(20.0, 18.0),
        NSSize::new(10.0, 10.0),
    ));
    sleep_label.setAlignment(NSTextAlignment::Center);
    sleep_label.setTextColor(Some(&face));
    sleep_label.setFont(Some(&NSFont::boldSystemFontOfSize(8.0)));
    sleep_label.setDrawsBackground(false);
    sleep_label.setBezeled(false);
    sleep_label.setBordered(false);
    sleep_label.setEditable(false);
    sleep_label.setSelectable(false);
    sleep_label.setHidden(true);
    sleep_label
}

fn create_completion_badge(mtm: MainThreadMarker) -> (Retained<NSView>, Retained<NSTextField>) {
    let badge = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(21.0, 17.0), NSSize::new(13.0, 13.0)),
    );
    badge.setWantsLayer(true);
    if let Some(layer) = badge.layer() {
        layer.setCornerRadius(6.5);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.40, 0.87, 0.57, 1.0).CGColor(),
        ));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.94, 1.0, 0.96, 0.92).CGColor(),
        ));
        layer.setShadowColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.40, 0.87, 0.57, 1.0).CGColor(),
        ));
        layer.setShadowOpacity(0.30);
        layer.setShadowRadius(4.0);
    }
    badge.setHidden(true);

    let label = NSTextField::labelWithString(ns_string!("1"), mtm);
    label.setFrame(NSRect::new(NSPoint::new(0.0, 1.0), NSSize::new(13.0, 11.0)));
    label.setAlignment(NSTextAlignment::Center);
    label.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
        0.02, 0.02, 0.02, 0.90,
    )));
    label.setFont(Some(&NSFont::boldSystemFontOfSize(8.0)));
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
    badge.addSubview(&label);

    (badge, label)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum NativeMascotState {
    Idle,
    Bouncing,
    Approval,
    Question,
    MessageBubble,
    Complete,
    Sleepy,
    WakeAngry,
}

#[derive(Clone, Copy)]
pub(super) struct NativeMascotMotion {
    pub(super) offset_x: f64,
    pub(super) offset_y: f64,
    pub(super) scale_x: f64,
    pub(super) scale_y: f64,
    pub(super) shell_alpha: f64,
    pub(super) shadow_opacity: f32,
    pub(super) shadow_radius: f64,
}

#[derive(Clone, Copy)]
pub(super) struct NativeMascotFrame {
    pub(super) state: NativeMascotState,
    pub(super) t: f64,
    pub(super) motion: NativeMascotMotion,
    pub(super) color: [f64; 4],
    pub(super) completion_count: usize,
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
        completion_count: usize,
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
            completion_count,
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
    has_completion_badge: bool,
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
    if has_completion_badge {
        return NativeMascotState::Complete;
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
        NativeMascotState::Complete => {
            let bob = ((t * 2.4).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: bob * 0.8,
                scale_x: 1.0 + bob * 0.010,
                scale_y: 1.0 - bob * 0.006,
                shell_alpha: 1.0,
                shadow_opacity: 0.48,
                shadow_radius: 5.4,
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
        NativeMascotState::MessageBubble | NativeMascotState::Complete => [1.0, 0.48, 0.14, 1.0],
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

pub(super) fn smoothstep_unit(progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - (2.0 * progress))
}

pub(super) fn smoothstep_range(edge0: f64, edge1: f64, value: f64) -> f64 {
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
        let has_status_completion = state.expanded
            && state.surface_mode == NativeExpandedSurface::Status
            && state
                .status_queue
                .iter()
                .any(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)));
        let has_completion_badge = !state.completion_badge_items.is_empty();
        let base_state = infer_native_mascot_base_state(
            state.last_snapshot.as_ref(),
            has_status_completion,
            has_completion_badge,
        );
        let expanded = state.expanded;
        let completion_count = state.completion_badge_items.len();
        state
            .mascot_runtime
            .next_frame(base_state, expanded, completion_count, now)
    };

    apply_native_mascot_frame(handles, frame);
}
