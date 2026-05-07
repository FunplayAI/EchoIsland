use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::card_animation::apply_card_stack_transition;
use super::card_stack::render_expanded_cards_with_plan;
use super::card_views::clear_subviews;
use super::panel_constants::{COLLAPSED_PANEL_HEIGHT, EXPANDED_CARDS_SIDE_INSET};
use super::panel_geometry::{expanded_cards_width, expanded_total_height_for_body_height};
use super::panel_refs::{NativePanelRefs, resolve_native_panel_refs};
use super::panel_render::apply_panel_geometry;
use super::panel_screen_geometry::{
    compact_pill_height_for_screen_rect, expanded_panel_width_for_screen_rect,
    resolve_screen_frame_for_panel,
};
use super::panel_types::{NativePanelHandles, NativePanelTransitionFrame};
use crate::native_panel_core::ExpandedSurface;
use crate::native_panel_renderer::facade::{
    descriptor::NativePanelTimelineDescriptor, presentation::NativePanelSnapshotRenderPlan,
};

#[derive(Clone, Copy)]
pub(super) struct NativeTransitionContext {
    pub(super) refs: NativePanelRefs,
    pub(super) screen_frame: NSRect,
    pub(super) expanded_width: f64,
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn resolve_native_transition_context(
    handles: NativePanelHandles,
) -> NativeTransitionContext {
    let refs = resolve_native_panel_refs(handles);
    let screen_frame = resolve_screen_frame_for_panel(refs.panel).unwrap_or(refs.panel.frame());
    let expanded_width =
        expanded_panel_width_for_screen_rect(refs.panel.screen().as_deref(), screen_frame);
    NativeTransitionContext {
        refs,
        screen_frame,
        expanded_width,
    }
}

pub(super) fn resolved_expanded_target_height_for_plan(
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    shared_body_height: Option<f64>,
) -> f64 {
    resolved_snapshot_expanded_height_for_plan(context, plan, shared_body_height)
}

pub(super) fn resolved_snapshot_expanded_height_for_plan(
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    shared_body_height: Option<f64>,
) -> f64 {
    expanded_total_height_for_body_height(
        plan.expanded_body_height(),
        compact_pill_height_for_screen_rect(
            context.refs.panel.screen().as_deref(),
            context.screen_frame,
        ),
        resolve_snapshot_shared_body_height(plan.surface(), shared_body_height),
    )
}

pub(super) fn resolved_snapshot_panel_height_for_plan(
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    expanded: bool,
    shared_body_height: Option<f64>,
) -> f64 {
    if !expanded {
        return COLLAPSED_PANEL_HEIGHT;
    }

    resolved_snapshot_expanded_height_for_plan(context, plan, shared_body_height)
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_transition_cards_with_plan(
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
) -> usize {
    render_expanded_cards_with_plan(context.refs.cards_container, plan, context.expanded_width);
    context.refs.cards_container.subviews().len()
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn reset_collapsed_cards(context: NativeTransitionContext) {
    clear_subviews(context.refs.cards_container);
    context.refs.cards_container.setFrame(NSRect::new(
        NSPoint::new(EXPANDED_CARDS_SIDE_INSET, 0.0),
        NSSize::new(expanded_cards_width(context.expanded_width), 0.0),
    ));
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn prepare_open_transition(
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    initial_cards_progress: f64,
) -> usize {
    let card_count = render_transition_cards_with_plan(context, plan);
    apply_card_stack_transition(context.refs.cards_container, initial_cards_progress, true);
    context.refs.cards_container.setHidden(false);
    context.refs.cards_container.setAlphaValue(1.0);
    context.refs.expanded_container.setHidden(true);
    context.refs.expanded_container.setAlphaValue(0.0);
    card_count
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn finalize_open_transition(
    handles: NativePanelHandles,
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    target_height: f64,
) {
    render_transition_cards_with_plan(context, plan);
    context.refs.cards_container.setHidden(false);
    context.refs.cards_container.setAlphaValue(1.0);
    apply_card_stack_transition(context.refs.cards_container, 1.0, true);
    apply_panel_geometry(handles, NativePanelTransitionFrame::expanded(target_height));
    context.refs.panel.displayIfNeeded();
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn prepare_close_transition(
    context: NativeTransitionContext,
    skip_close_card_exit: bool,
    initial_cards_progress: f64,
) -> usize {
    if skip_close_card_exit {
        clear_subviews(context.refs.cards_container);
        context.refs.cards_container.setHidden(true);
        context.refs.cards_container.setAlphaValue(0.0);
    }

    let card_count = if skip_close_card_exit {
        0
    } else {
        context.refs.cards_container.subviews().len()
    };

    context.refs.expanded_container.setHidden(false);
    if !skip_close_card_exit {
        context.refs.cards_container.setHidden(false);
        context.refs.cards_container.setAlphaValue(1.0);
        apply_card_stack_transition(context.refs.cards_container, initial_cards_progress, false);
    }

    card_count
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn finalize_close_transition(
    handles: NativePanelHandles,
    context: NativeTransitionContext,
) {
    reset_collapsed_cards(context);
    context.refs.expanded_container.setHidden(true);
    context.refs.expanded_container.setAlphaValue(1.0);
    apply_panel_geometry(
        handles,
        NativePanelTransitionFrame::collapsed(COLLAPSED_PANEL_HEIGHT),
    );
    context.refs.panel.displayIfNeeded();
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn prepare_surface_switch_transition(
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    initial_cards_progress: f64,
) -> usize {
    let card_count = render_transition_cards_with_plan(context, plan);
    apply_card_stack_transition(context.refs.cards_container, initial_cards_progress, true);
    context.refs.cards_container.setHidden(false);
    context.refs.cards_container.setAlphaValue(1.0);
    context.refs.expanded_container.setHidden(false);
    context.refs.expanded_container.setAlphaValue(1.0);
    card_count
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn finalize_surface_switch_transition(
    handles: NativePanelHandles,
    context: NativeTransitionContext,
    plan: &NativePanelSnapshotRenderPlan,
    target_height: f64,
) {
    render_transition_cards_with_plan(context, plan);
    context.refs.cards_container.setHidden(false);
    context.refs.cards_container.setAlphaValue(1.0);
    apply_card_stack_transition(context.refs.cards_container, 1.0, true);
    apply_panel_geometry(handles, NativePanelTransitionFrame::expanded(target_height));
    context.refs.panel.displayIfNeeded();
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_transition_timeline_frame(
    handles: NativePanelHandles,
    frame: NativePanelTransitionFrame,
    cards_entering: bool,
) {
    let context = resolve_native_transition_context(handles);
    apply_panel_geometry(handles, frame);
    apply_card_stack_transition(
        context.refs.cards_container,
        frame.cards_progress,
        cards_entering,
    );
    context.refs.panel.displayIfNeeded();
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_transition_timeline_descriptor(
    handles: NativePanelHandles,
    descriptor: NativePanelTimelineDescriptor,
) {
    let animation = descriptor.animation;
    let frame = NativePanelTransitionFrame {
        canvas_height: animation.canvas_height,
        visible_height: animation.visible_height,
        bar_progress: animation.width_progress,
        height_progress: animation.height_progress,
        shoulder_progress: animation.shoulder_progress,
        drop_progress: animation.drop_progress,
        cards_progress: animation.cards_progress,
    };
    apply_transition_timeline_frame(handles, frame, descriptor.cards_entering);
}

fn resolve_snapshot_shared_body_height_for_surface(
    shared_expanded_enabled: bool,
    surface: ExpandedSurface,
    shared_body_height: Option<f64>,
) -> Option<f64> {
    if shared_expanded_enabled && surface == ExpandedSurface::Default {
        shared_body_height
    } else {
        None
    }
}

pub(super) fn resolve_snapshot_shared_body_height(
    surface: ExpandedSurface,
    shared_body_height: Option<f64>,
) -> Option<f64> {
    resolve_snapshot_shared_body_height_for_surface(false, surface, shared_body_height)
}

#[cfg(test)]
mod tests {
    use super::resolve_snapshot_shared_body_height_for_surface;
    use crate::native_panel_core::ExpandedSurface;

    #[test]
    fn shared_body_height_is_kept_only_for_default_shared_surface() {
        assert_eq!(
            resolve_snapshot_shared_body_height_for_surface(
                true,
                ExpandedSurface::Default,
                Some(180.0)
            ),
            Some(180.0)
        );
        assert_eq!(
            resolve_snapshot_shared_body_height_for_surface(
                true,
                ExpandedSurface::Status,
                Some(180.0)
            ),
            None
        );
        assert_eq!(
            resolve_snapshot_shared_body_height_for_surface(
                true,
                ExpandedSurface::Settings,
                Some(180.0)
            ),
            None
        );
        assert_eq!(
            resolve_snapshot_shared_body_height_for_surface(
                false,
                ExpandedSurface::Default,
                Some(180.0)
            ),
            None
        );
    }
}
