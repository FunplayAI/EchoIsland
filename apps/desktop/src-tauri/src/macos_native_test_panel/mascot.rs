use std::time::Instant;

use super::mascot_render::apply_native_mascot_frame;
use super::mascot_scene::resolve_native_mascot_frame_input;
use super::panel_constants::MASCOT_STATE_TRANSITION_SECONDS;
use super::panel_refs::native_panel_state;
use super::panel_types::NativePanelHandles;
use crate::native_panel_core::{
    MascotRuntimeFrameInput, MascotRuntimeState, MascotVisualFrame, PanelMascotBaseState,
};

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
    pub(super) eye_open: f64,
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
    runtime: MascotRuntimeState,
}

impl NativeMascotRuntime {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            animation_started_at: now,
            runtime: MascotRuntimeState::new(0),
        }
    }

    fn next_frame(
        &mut self,
        base_state: NativeMascotState,
        expanded: bool,
        completion_count: usize,
        now: Instant,
    ) -> NativeMascotFrame {
        let elapsed_ms = now
            .saturating_duration_since(self.animation_started_at)
            .as_millis();
        let shared_frame = self.runtime.next_frame(MascotRuntimeFrameInput {
            base_state: panel_mascot_state_from_native(base_state),
            expanded,
            elapsed_ms,
            transition_duration_ms: (MASCOT_STATE_TRANSITION_SECONDS * 1000.0).round() as u128,
        });

        NativeMascotFrame {
            state: native_mascot_state_from_core(shared_frame.state),
            t: elapsed_ms as f64 / 1000.0,
            motion: native_mascot_motion_from_visual_frame(shared_frame.motion),
            color: shared_frame.color,
            completion_count,
            mascot_hidden: false,
            debug_mode_enabled: false,
            completion_glow_opacity: 0.0,
        }
    }
}

fn panel_mascot_state_from_native(state: NativeMascotState) -> PanelMascotBaseState {
    match state {
        NativeMascotState::Idle => PanelMascotBaseState::Idle,
        NativeMascotState::Bouncing => PanelMascotBaseState::Running,
        NativeMascotState::Approval => PanelMascotBaseState::Approval,
        NativeMascotState::Question => PanelMascotBaseState::Question,
        NativeMascotState::MessageBubble => PanelMascotBaseState::MessageBubble,
        NativeMascotState::Complete => PanelMascotBaseState::Complete,
        NativeMascotState::Sleepy => PanelMascotBaseState::Sleepy,
        NativeMascotState::WakeAngry => PanelMascotBaseState::WakeAngry,
    }
}

fn native_mascot_state_from_core(state: PanelMascotBaseState) -> NativeMascotState {
    match state {
        PanelMascotBaseState::Idle => NativeMascotState::Idle,
        PanelMascotBaseState::Running => NativeMascotState::Bouncing,
        PanelMascotBaseState::Approval => NativeMascotState::Approval,
        PanelMascotBaseState::Question => NativeMascotState::Question,
        PanelMascotBaseState::MessageBubble => NativeMascotState::MessageBubble,
        PanelMascotBaseState::Complete => NativeMascotState::Complete,
        PanelMascotBaseState::Sleepy => NativeMascotState::Sleepy,
        PanelMascotBaseState::WakeAngry => NativeMascotState::WakeAngry,
    }
}

fn native_mascot_motion_from_visual_frame(frame: MascotVisualFrame) -> NativeMascotMotion {
    NativeMascotMotion {
        offset_x: frame.offset_x,
        offset_y: frame.offset_y,
        scale_x: frame.scale_x,
        scale_y: frame.scale_y,
        shell_alpha: frame.shell_alpha,
        shadow_opacity: frame.shadow_opacity as f32,
        shadow_radius: frame.shadow_radius,
        eye_open: frame.eye_open,
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
