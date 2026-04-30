use std::time::Instant;

use crate::native_panel_core::{PANEL_ANIMATION_FRAME_MS, PanelAnimationTimeline};

use super::{
    animation_plan::{NativePanelAnimationPlan, resolve_native_panel_animation_plan},
    descriptors::{NativePanelTimelineDescriptor, native_panel_timeline_descriptor_for_animation},
    transition_controller::{
        NativePanelTransitionRequest, resolve_native_panel_animation_timeline,
    },
};

const MAX_GENERAL_ANIMATION_CATCH_UP_MS: u64 = PANEL_ANIMATION_FRAME_MS * 13;
const MAX_VISUAL_STAGE_CATCH_UP_MS: u64 = PANEL_ANIMATION_FRAME_MS * 4;
const MAX_FRAME_CARD_PROGRESS_JUMP: f64 = 0.30;
const MAX_FRAME_WIDTH_PROGRESS_JUMP: f64 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelAnimationTarget {
    pub(crate) request: NativePanelTransitionRequest,
    pub(crate) start_height: f64,
    pub(crate) target_height: f64,
    pub(crate) card_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelAnimationFrame {
    pub(crate) descriptor: NativePanelTimelineDescriptor,
    pub(crate) plan: NativePanelAnimationPlan,
    pub(crate) elapsed_ms: u64,
    pub(crate) total_ms: u64,
    pub(crate) continue_animating: bool,
}

#[derive(Clone, Copy, Debug)]
struct NativePanelActiveAnimation {
    timeline: PanelAnimationTimeline,
    card_count: usize,
    started_at: Instant,
    last_sampled_at: Option<Instant>,
    last_elapsed_ms: u64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NativePanelAnimationFrameScheduler {
    active: Option<NativePanelActiveAnimation>,
}

impl NativePanelAnimationFrameScheduler {
    pub(crate) fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub(crate) fn start(
        &mut self,
        target: NativePanelAnimationTarget,
        now: Instant,
    ) -> NativePanelAnimationFrame {
        self.active = Some(NativePanelActiveAnimation {
            timeline: resolve_native_panel_animation_timeline(
                target.request,
                target.start_height,
                target.target_height,
                target.card_count,
            ),
            card_count: target.card_count,
            started_at: now,
            last_sampled_at: None,
            last_elapsed_ms: 0,
        });
        self.sample(now).expect("active animation just started")
    }

    pub(crate) fn sample(&mut self, now: Instant) -> Option<NativePanelAnimationFrame> {
        let active = self.active.as_mut()?;
        let total_ms = active.timeline.total_ms();
        let wall_elapsed_ms = now
            .saturating_duration_since(active.started_at)
            .as_millis()
            .min(total_ms as u128) as u64;
        let frame_due = active
            .last_sampled_at
            .map(|last_sampled_at| {
                now.saturating_duration_since(last_sampled_at).as_millis()
                    >= PANEL_ANIMATION_FRAME_MS as u128
            })
            .unwrap_or(true);
        if !frame_due && active.last_elapsed_ms < total_ms {
            return None;
        }
        let elapsed_ms = match active.last_sampled_at {
            None => 0,
            Some(_) => {
                let catch_up_elapsed_ms = active
                    .last_elapsed_ms
                    .saturating_add(MAX_GENERAL_ANIMATION_CATCH_UP_MS)
                    .min(wall_elapsed_ms);
                let catch_up_elapsed_ms = if animation_stage_jump_is_too_large(
                    active.timeline,
                    active.last_elapsed_ms,
                    catch_up_elapsed_ms,
                ) {
                    active
                        .last_elapsed_ms
                        .saturating_add(MAX_VISUAL_STAGE_CATCH_UP_MS)
                        .min(wall_elapsed_ms)
                } else {
                    catch_up_elapsed_ms
                };
                let terminal_guard_ms = total_ms.saturating_sub(PANEL_ANIMATION_FRAME_MS);
                if catch_up_elapsed_ms >= total_ms && active.last_elapsed_ms < terminal_guard_ms {
                    terminal_guard_ms
                } else {
                    catch_up_elapsed_ms
                }
            }
        };
        let continue_animating = elapsed_ms < total_ms;
        let descriptor =
            native_panel_timeline_descriptor_for_animation(active.timeline.sample(elapsed_ms));
        let plan = resolve_native_panel_animation_plan(descriptor, active.card_count);

        if !continue_animating {
            self.active = None;
        } else {
            active.last_sampled_at = Some(now);
            active.last_elapsed_ms = elapsed_ms;
        }

        Some(NativePanelAnimationFrame {
            descriptor,
            plan,
            elapsed_ms,
            total_ms,
            continue_animating,
        })
    }

    pub(crate) fn next_frame_delay_ms(&self) -> Option<u64> {
        self.active.is_some().then_some(PANEL_ANIMATION_FRAME_MS)
    }
}

fn animation_stage_jump_is_too_large(
    timeline: PanelAnimationTimeline,
    last_elapsed_ms: u64,
    next_elapsed_ms: u64,
) -> bool {
    let previous = timeline.sample(last_elapsed_ms);
    let next = timeline.sample(next_elapsed_ms);
    let cards_jump = (next.cards_progress - previous.cards_progress).abs();
    let width_jump = (next.width_progress - previous.width_progress).abs();

    cards_jump > MAX_FRAME_CARD_PROGRESS_JUMP || width_jump > MAX_FRAME_WIDTH_PROGRESS_JUMP
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::native_panel_renderer::facade::transition::NativePanelTransitionRequest;

    use super::{NativePanelAnimationFrameScheduler, NativePanelAnimationTarget};

    #[test]
    fn animation_scheduler_samples_until_timeline_completes() {
        let now = Instant::now();
        let mut scheduler = NativePanelAnimationFrameScheduler::default();

        let first = scheduler.start(
            NativePanelAnimationTarget {
                request: NativePanelTransitionRequest::Open,
                start_height: 80.0,
                target_height: 164.0,
                card_count: 1,
            },
            now,
        );

        assert_eq!(first.elapsed_ms, 0);
        assert_eq!(first.plan.timeline, first.descriptor);
        assert_eq!(first.plan.card_stack.card_count, 1);
        assert!(first.continue_animating);
        assert!(scheduler.is_active());

        let delayed_frame = scheduler
            .sample(now + Duration::from_millis(200))
            .expect("delayed frame");

        assert_eq!(delayed_frame.elapsed_ms, 200);
        assert!(delayed_frame.continue_animating);
        assert!(scheduler.is_active());

        let mut final_frame = delayed_frame;
        for step in 1..=first
            .total_ms
            .div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS)
        {
            if !scheduler.is_active() {
                break;
            }
            final_frame = scheduler
                .sample(
                    now + Duration::from_millis(
                        200 + step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                    ),
                )
                .expect("frame tick");
        }

        assert_eq!(final_frame.elapsed_ms, final_frame.total_ms);
        assert!(!final_frame.continue_animating);
        assert!(!scheduler.is_active());
    }

    #[test]
    fn animation_scheduler_throttles_samples_between_frame_ticks() {
        let now = Instant::now();
        let mut scheduler = NativePanelAnimationFrameScheduler::default();

        scheduler.start(
            NativePanelAnimationTarget {
                request: NativePanelTransitionRequest::Open,
                start_height: 80.0,
                target_height: 164.0,
                card_count: 1,
            },
            now,
        );

        assert!(
            scheduler
                .sample(
                    now + Duration::from_millis(
                        crate::native_panel_core::PANEL_ANIMATION_FRAME_MS - 1
                    )
                )
                .is_none()
        );

        let next = scheduler
            .sample(now + Duration::from_millis(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS))
            .expect("next frame at frame tick");

        assert_eq!(
            next.elapsed_ms,
            crate::native_panel_core::PANEL_ANIMATION_FRAME_MS
        );
    }

    #[test]
    fn animation_scheduler_keeps_one_non_terminal_frame_after_delayed_wake() {
        let now = Instant::now();
        let mut scheduler = NativePanelAnimationFrameScheduler::default();

        let first = scheduler.start(
            NativePanelAnimationTarget {
                request: NativePanelTransitionRequest::Close,
                start_height: 180.0,
                target_height: 80.0,
                card_count: 2,
            },
            now,
        );

        let delayed_frame = scheduler
            .sample(now + Duration::from_millis(first.total_ms + 250))
            .expect("delayed near-terminal frame");

        assert_eq!(
            delayed_frame.elapsed_ms,
            crate::native_panel_core::PANEL_ANIMATION_FRAME_MS * 4
        );
        assert!(delayed_frame.descriptor.animation.cards_progress < 0.5);
        assert!(delayed_frame.continue_animating);
        assert!(scheduler.is_active());

        let mut final_frame = delayed_frame;
        for step in 1..=first
            .total_ms
            .div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS)
        {
            if !scheduler.is_active() {
                break;
            }
            final_frame = scheduler
                .sample(
                    now + Duration::from_millis(
                        first.total_ms
                            + 250
                            + step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                    ),
                )
                .expect("catch-up frame");
        }

        assert_eq!(final_frame.elapsed_ms, first.total_ms);
        assert!(!final_frame.continue_animating);
        assert!(!scheduler.is_active());
    }
}
