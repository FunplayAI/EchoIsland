use std::time::Instant;

use super::mascot_motion::{
    native_mascot_color, native_mascot_lerp_motion, native_mascot_target_motion, smoothstep_unit,
};
use super::mascot_render::apply_native_mascot_frame;
use super::mascot_scene::resolve_native_mascot_frame_input;
use super::panel_constants::{
    MASCOT_IDLE_LONG_SECONDS, MASCOT_STATE_TRANSITION_SECONDS, MASCOT_WAKE_ANGRY_SECONDS,
};
use super::panel_refs::native_panel_state;
use super::panel_types::NativePanelHandles;

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
    pub(super) mascot_hidden: bool,
    pub(super) debug_mode_enabled: bool,
    pub(super) completion_glow_opacity: f64,
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
            mascot_hidden: false,
            debug_mode_enabled: false,
            completion_glow_opacity: 0.0,
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
        let input = resolve_native_mascot_frame_input(&state);
        let mut frame = state.mascot_runtime.next_frame(
            input.base_state,
            input.expanded,
            input.completion_count,
            now,
        );
        frame.mascot_hidden = input.mascot_hidden;
        frame.debug_mode_enabled = input.debug_mode_enabled;
        frame.completion_glow_opacity = input.completion_glow_opacity;
        frame
    };

    apply_native_mascot_frame(handles, frame);
}
