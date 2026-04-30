use crate::native_panel_core::{
    PanelAnimationDescriptor, PanelAnimationKind, resolve_panel_cards_visibility_progress,
};

use super::{
    card_visual_spec::{CardVisualStackRevealFrameSpec, card_visual_stack_reveal_frame},
    descriptors::NativePanelTimelineDescriptor,
    transition_controller::NativePanelTransitionRequest,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelAnimationPlan {
    pub(crate) timeline: NativePanelTimelineDescriptor,
    pub(crate) card_stack: NativePanelCardStackAnimationPlan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelCardStackAnimationPlan {
    pub(crate) card_count: usize,
    pub(crate) entering: bool,
    pub(crate) transition_progress: f64,
    pub(crate) visibility_progress: f64,
    pub(crate) separator_visibility: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelStatusClosePreservationInput {
    pub(crate) last_transition_request: Option<NativePanelTransitionRequest>,
    pub(crate) skip_next_close_card_exit: bool,
    pub(crate) transitioning: bool,
    pub(crate) last_animation: Option<PanelAnimationDescriptor>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativePanelStatusClosePreservationPlan {
    pub(crate) active_close: bool,
    pub(crate) pending_close: bool,
    pub(crate) should_store_pending_stack: bool,
    pub(crate) should_preserve_frame_after_refresh: bool,
    pub(crate) should_prepare_close_animation_stack: bool,
}

impl NativePanelCardStackAnimationPlan {
    pub(crate) fn reveal_frame(self, card_index: usize) -> CardVisualStackRevealFrameSpec {
        card_visual_stack_reveal_frame(self.separator_visibility, self.card_count, card_index)
    }
}

pub(crate) fn resolve_native_panel_status_close_preservation_plan(
    input: NativePanelStatusClosePreservationInput,
) -> NativePanelStatusClosePreservationPlan {
    let pending_close = input.last_transition_request == Some(NativePanelTransitionRequest::Close)
        && input.skip_next_close_card_exit;
    let active_close = input.transitioning
        && input
            .last_animation
            .is_some_and(|descriptor| descriptor.kind == PanelAnimationKind::Close);

    NativePanelStatusClosePreservationPlan {
        active_close,
        pending_close,
        should_store_pending_stack: pending_close,
        should_preserve_frame_after_refresh: active_close || pending_close,
        should_prepare_close_animation_stack: pending_close,
    }
}

pub(crate) fn resolve_native_panel_animation_plan(
    timeline: NativePanelTimelineDescriptor,
    card_count: usize,
) -> NativePanelAnimationPlan {
    let transition_progress = timeline.animation.cards_progress.clamp(0.0, 1.0);
    let visibility_progress = resolve_panel_cards_visibility_progress(timeline.animation);
    NativePanelAnimationPlan {
        timeline,
        card_stack: NativePanelCardStackAnimationPlan {
            card_count,
            entering: timeline.cards_entering,
            transition_progress,
            visibility_progress,
            separator_visibility: visibility_progress * 0.88,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        native_panel_core::{PanelAnimationDescriptor, PanelAnimationKind},
        native_panel_renderer::facade::descriptor::native_panel_timeline_descriptor_for_animation,
    };

    use super::{
        NativePanelStatusClosePreservationInput, resolve_native_panel_animation_plan,
        resolve_native_panel_status_close_preservation_plan,
    };

    #[test]
    fn animation_plan_keeps_card_reveal_direction_and_progress_shared() {
        let timeline = native_panel_timeline_descriptor_for_animation(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: 180.0,
            visible_height: 120.0,
            width_progress: 0.5,
            height_progress: 0.5,
            shoulder_progress: 1.0,
            drop_progress: 0.0,
            cards_progress: 0.25,
        });

        let plan = resolve_native_panel_animation_plan(timeline, 2);

        assert!(!plan.card_stack.entering);
        assert_eq!(plan.card_stack.transition_progress, 0.25);
        assert_eq!(plan.card_stack.visibility_progress, 0.75);
        assert_eq!(plan.card_stack.separator_visibility, 0.66);
        assert!(plan.card_stack.reveal_frame(0).card_phase > 0.0);
    }

    #[test]
    fn animation_plan_staggers_stack_reveal_frames() {
        let timeline = native_panel_timeline_descriptor_for_animation(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 120.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 0.5,
        });

        let plan = resolve_native_panel_animation_plan(timeline, 3);
        let first = plan.card_stack.reveal_frame(0);
        let second = plan.card_stack.reveal_frame(1);

        assert!(plan.card_stack.entering);
        assert!(first.card_phase > second.card_phase);
        assert_eq!(first.progress, 0.5);
    }

    #[test]
    fn status_close_preservation_plan_keeps_pending_and_active_close_semantics_shared() {
        let pending = resolve_native_panel_status_close_preservation_plan(
            NativePanelStatusClosePreservationInput {
                last_transition_request: Some(
                    crate::native_panel_renderer::facade::transition::NativePanelTransitionRequest::Close,
                ),
                skip_next_close_card_exit: true,
                transitioning: false,
                last_animation: None,
            },
        );

        assert!(pending.pending_close);
        assert!(pending.should_store_pending_stack);
        assert!(pending.should_preserve_frame_after_refresh);
        assert!(pending.should_prepare_close_animation_stack);

        let active = resolve_native_panel_status_close_preservation_plan(
            NativePanelStatusClosePreservationInput {
                last_transition_request: None,
                skip_next_close_card_exit: false,
                transitioning: true,
                last_animation: Some(PanelAnimationDescriptor {
                    kind: PanelAnimationKind::Close,
                    canvas_height: 120.0,
                    visible_height: 80.0,
                    width_progress: 0.0,
                    height_progress: 0.0,
                    shoulder_progress: 0.0,
                    drop_progress: 0.0,
                    cards_progress: 0.5,
                }),
            },
        );

        assert!(active.active_close);
        assert!(!active.pending_close);
        assert!(!active.should_store_pending_stack);
        assert!(active.should_preserve_frame_after_refresh);
        assert!(!active.should_prepare_close_animation_stack);
    }
}
