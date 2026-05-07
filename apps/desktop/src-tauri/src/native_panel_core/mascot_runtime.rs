use super::{
    MASCOT_IDLE_LONG_SECONDS, MASCOT_WAKE_ANGRY_SECONDS, MascotVisualFrame, MascotVisualFrameInput,
    MascotVisualFrameTransitionInput, PanelMascotBaseState, resolve_mascot_visual_frame,
    resolve_mascot_visual_frame_transition,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MascotRuntimeFrameInput {
    pub(crate) base_state: PanelMascotBaseState,
    pub(crate) expanded: bool,
    pub(crate) elapsed_ms: u128,
    pub(crate) transition_duration_ms: u128,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MascotRuntimeFrame {
    pub(crate) state: PanelMascotBaseState,
    pub(crate) motion: MascotVisualFrame,
    pub(crate) color: [f64; 4],
    pub(crate) elapsed_ms: u128,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MascotRuntimeState {
    last_non_idle_elapsed_ms: u128,
    last_resolved_state: PanelMascotBaseState,
    wake_started_elapsed_ms: Option<u128>,
    wake_next_state: PanelMascotBaseState,
    transition_target: PanelMascotBaseState,
    transition_started_elapsed_ms: u128,
    transition_start_motion: MascotVisualFrame,
    last_motion: MascotVisualFrame,
}

impl MascotRuntimeState {
    pub(crate) fn new(elapsed_ms: u128) -> Self {
        let idle_motion = mascot_idle_visual_frame();
        Self {
            last_non_idle_elapsed_ms: elapsed_ms,
            last_resolved_state: PanelMascotBaseState::Idle,
            wake_started_elapsed_ms: None,
            wake_next_state: PanelMascotBaseState::Idle,
            transition_target: PanelMascotBaseState::Idle,
            transition_started_elapsed_ms: elapsed_ms,
            transition_start_motion: idle_motion,
            last_motion: idle_motion,
        }
    }

    pub(crate) fn reset(&mut self, elapsed_ms: u128) {
        *self = Self::new(elapsed_ms);
    }

    pub(crate) fn next_frame(&mut self, input: MascotRuntimeFrameInput) -> MascotRuntimeFrame {
        let visual_state =
            self.resolve_visual_state(input.base_state, input.expanded, input.elapsed_ms);
        let visual_elapsed_ms = self.motion_elapsed_ms_for_state(visual_state, input.elapsed_ms);
        let target_motion = resolve_mascot_visual_frame(MascotVisualFrameInput {
            state: visual_state,
            elapsed_ms: visual_elapsed_ms,
        });

        if self.transition_target != visual_state {
            self.transition_target = visual_state;
            self.transition_started_elapsed_ms = input.elapsed_ms;
            self.transition_start_motion = self.last_motion;
        }

        let motion = resolve_mascot_visual_frame_transition(MascotVisualFrameTransitionInput {
            start: self.transition_start_motion,
            target: target_motion,
            elapsed_ms: input
                .elapsed_ms
                .saturating_sub(self.transition_started_elapsed_ms),
            duration_ms: input.transition_duration_ms,
        });
        self.last_motion = motion;

        MascotRuntimeFrame {
            state: visual_state,
            motion,
            color: mascot_runtime_color(visual_state, self.wake_elapsed_ms(input.elapsed_ms)),
            elapsed_ms: visual_elapsed_ms,
        }
    }

    fn resolve_visual_state(
        &mut self,
        base_state: PanelMascotBaseState,
        expanded: bool,
        elapsed_ms: u128,
    ) -> PanelMascotBaseState {
        let mut next_state = if base_state != PanelMascotBaseState::Idle {
            self.last_non_idle_elapsed_ms = elapsed_ms;
            base_state
        } else if expanded {
            self.last_non_idle_elapsed_ms = elapsed_ms;
            PanelMascotBaseState::Idle
        } else if elapsed_ms.saturating_sub(self.last_non_idle_elapsed_ms)
            >= MASCOT_IDLE_LONG_SECONDS as u128 * 1000
        {
            PanelMascotBaseState::Sleepy
        } else {
            PanelMascotBaseState::Idle
        };

        let waking_from_sleep = next_state != PanelMascotBaseState::Sleepy
            && self.wake_started_elapsed_ms.is_none()
            && self.last_resolved_state == PanelMascotBaseState::Sleepy;
        if waking_from_sleep {
            self.wake_started_elapsed_ms = Some(elapsed_ms);
            self.wake_next_state = next_state;
            self.last_resolved_state = PanelMascotBaseState::WakeAngry;
            return PanelMascotBaseState::WakeAngry;
        }

        if let Some(started_at) = self.wake_started_elapsed_ms {
            self.wake_next_state = if next_state == PanelMascotBaseState::Sleepy {
                PanelMascotBaseState::Idle
            } else {
                next_state
            };

            if elapsed_ms.saturating_sub(started_at) < mascot_wake_duration_ms() {
                self.last_resolved_state = PanelMascotBaseState::WakeAngry;
                return PanelMascotBaseState::WakeAngry;
            }

            self.wake_started_elapsed_ms = None;
            next_state = self.wake_next_state;
        }

        self.last_resolved_state = next_state;
        next_state
    }

    fn motion_elapsed_ms_for_state(&self, state: PanelMascotBaseState, elapsed_ms: u128) -> u128 {
        if state == PanelMascotBaseState::WakeAngry {
            self.wake_elapsed_ms(elapsed_ms).unwrap_or(0)
        } else {
            elapsed_ms
        }
    }

    fn wake_elapsed_ms(&self, elapsed_ms: u128) -> Option<u128> {
        self.wake_started_elapsed_ms
            .map(|started_at| elapsed_ms.saturating_sub(started_at))
    }
}

impl Default for MascotRuntimeState {
    fn default() -> Self {
        Self::new(0)
    }
}

pub(crate) fn mascot_idle_visual_frame() -> MascotVisualFrame {
    MascotVisualFrame {
        offset_x: 0.0,
        offset_y: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        shell_alpha: 1.0,
        shadow_opacity: 0.34,
        shadow_radius: 4.0,
        eye_open: 1.0,
    }
}

pub(crate) fn mascot_runtime_color(
    state: PanelMascotBaseState,
    wake_elapsed_ms: Option<u128>,
) -> [f64; 4] {
    match state {
        PanelMascotBaseState::Approval
        | PanelMascotBaseState::Question
        | PanelMascotBaseState::MessageBubble
        | PanelMascotBaseState::Complete
        | PanelMascotBaseState::Running
        | PanelMascotBaseState::Idle => [1.0, 0.48, 0.14, 1.0],
        PanelMascotBaseState::Sleepy => [0.72, 0.30, 0.13, 1.0],
        PanelMascotBaseState::WakeAngry => {
            let elapsed = wake_elapsed_ms.unwrap_or(0) as f64 / 1000.0;
            let blink = if (elapsed * 12.0).sin() >= 0.0 {
                1.0
            } else {
                0.0
            };
            [1.0, lerp(0.38, 0.48, blink), lerp(0.24, 0.14, blink), 1.0]
        }
    }
}

fn mascot_wake_duration_ms() -> u128 {
    (MASCOT_WAKE_ANGRY_SECONDS * 1000.0).round() as u128
}

fn lerp(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

#[cfg(test)]
mod tests {
    use super::{MascotRuntimeFrameInput, MascotRuntimeState, mascot_runtime_color};
    use crate::native_panel_core::{MASCOT_IDLE_LONG_SECONDS, PanelMascotBaseState};

    #[test]
    fn mascot_runtime_enters_sleep_after_idle_threshold() {
        let mut runtime = MascotRuntimeState::new(0);
        let frame = runtime.next_frame(MascotRuntimeFrameInput {
            base_state: PanelMascotBaseState::Idle,
            expanded: false,
            elapsed_ms: MASCOT_IDLE_LONG_SECONDS as u128 * 1000,
            transition_duration_ms: 240,
        });

        assert_eq!(frame.state, PanelMascotBaseState::Sleepy);
    }

    #[test]
    fn mascot_runtime_plays_wake_angry_when_leaving_sleep() {
        let mut runtime = MascotRuntimeState::new(0);
        runtime.next_frame(MascotRuntimeFrameInput {
            base_state: PanelMascotBaseState::Idle,
            expanded: false,
            elapsed_ms: MASCOT_IDLE_LONG_SECONDS as u128 * 1000,
            transition_duration_ms: 240,
        });
        let wake = runtime.next_frame(MascotRuntimeFrameInput {
            base_state: PanelMascotBaseState::Running,
            expanded: false,
            elapsed_ms: MASCOT_IDLE_LONG_SECONDS as u128 * 1000 + 1,
            transition_duration_ms: 240,
        });

        assert_eq!(wake.state, PanelMascotBaseState::WakeAngry);
        assert!(wake.motion.scale_x >= 1.0);
    }

    #[test]
    fn mascot_runtime_keeps_transition_motion_between_states() {
        let mut runtime = MascotRuntimeState::new(0);
        let idle = runtime.next_frame(MascotRuntimeFrameInput {
            base_state: PanelMascotBaseState::Idle,
            expanded: true,
            elapsed_ms: 0,
            transition_duration_ms: 240,
        });
        runtime.next_frame(MascotRuntimeFrameInput {
            base_state: PanelMascotBaseState::Running,
            expanded: false,
            elapsed_ms: 120,
            transition_duration_ms: 240,
        });
        let running = runtime.next_frame(MascotRuntimeFrameInput {
            base_state: PanelMascotBaseState::Running,
            expanded: false,
            elapsed_ms: 240,
            transition_duration_ms: 240,
        });

        assert_eq!(idle.state, PanelMascotBaseState::Idle);
        assert_eq!(running.state, PanelMascotBaseState::Running);
        assert!(running.motion.scale_x >= idle.motion.scale_x);
        assert!(running.motion.shadow_opacity > idle.motion.shadow_opacity);
    }

    #[test]
    fn mascot_runtime_color_matches_mac_wake_blink_palette() {
        assert_eq!(
            mascot_runtime_color(PanelMascotBaseState::Sleepy, None),
            [0.72, 0.30, 0.13, 1.0]
        );
        assert_eq!(
            mascot_runtime_color(PanelMascotBaseState::WakeAngry, Some(0)),
            [1.0, 0.48, 0.14, 1.0]
        );
    }
}
