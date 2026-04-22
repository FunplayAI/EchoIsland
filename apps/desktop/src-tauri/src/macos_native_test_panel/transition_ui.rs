use echoisland_runtime::RuntimeSnapshot;
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::card_animation::apply_card_stack_transition;
use super::card_stack::render_expanded_cards;
use super::card_views::clear_subviews;
use super::panel_constants::{
    COLLAPSED_PANEL_HEIGHT, EXPANDED_CARDS_SIDE_INSET, PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
};
use super::panel_geometry::{expanded_cards_width, expanded_total_height};
use super::panel_interaction::{native_settings_surface_active, native_status_surface_active};
use super::panel_refs::{NativePanelRefs, native_panel_state, resolve_native_panel_refs};
use super::panel_render::apply_panel_geometry;
use super::panel_screen_geometry::{
    compact_pill_height_for_screen_rect, expanded_panel_width_for_screen_rect,
    resolve_screen_frame_for_panel,
};
use super::panel_types::{NativePanelHandles, NativePanelTransitionFrame};
use crate::macos_shared_expanded_window;

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

pub(super) fn resolved_expanded_target_height(
    context: NativeTransitionContext,
    snapshot: &RuntimeSnapshot,
) -> f64 {
    let shared_body_height = if macos_shared_expanded_window::shared_expanded_enabled()
        && !native_status_surface_active()
        && !native_settings_surface_active()
    {
        native_panel_state()
            .and_then(|state| state.lock().ok().and_then(|guard| guard.shared_body_height))
    } else {
        None
    };
    expanded_total_height(
        snapshot,
        compact_pill_height_for_screen_rect(
            context.refs.panel.screen().as_deref(),
            context.screen_frame,
        ),
        shared_body_height,
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_transition_cards(
    context: NativeTransitionContext,
    snapshot: &RuntimeSnapshot,
) {
    render_expanded_cards(
        context.refs.cards_container,
        snapshot,
        context.expanded_width,
    );
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
    snapshot: &RuntimeSnapshot,
) -> usize {
    render_transition_cards(context, snapshot);
    let card_count = context.refs.cards_container.subviews().len();
    apply_card_stack_transition(context.refs.cards_container, 0.0, true);
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
    snapshot: &RuntimeSnapshot,
    target_height: f64,
) {
    render_transition_cards(context, snapshot);
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
        apply_card_stack_transition(context.refs.cards_container, 0.0, false);
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
    snapshot: &RuntimeSnapshot,
) -> usize {
    render_transition_cards(context, snapshot);
    let card_count = context.refs.cards_container.subviews().len();
    apply_card_stack_transition(
        context.refs.cards_container,
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        true,
    );
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
    snapshot: &RuntimeSnapshot,
    target_height: f64,
) {
    render_transition_cards(context, snapshot);
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
