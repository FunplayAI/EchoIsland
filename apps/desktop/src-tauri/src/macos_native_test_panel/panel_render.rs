use super::panel_shoulder::apply_shoulder_path_scale_x;
use super::panel_style::{PanelLayerStyleState, apply_panel_layer_styles};
use super::*;

#[derive(Clone, Copy)]
struct PanelRenderProgress {
    bar: f64,
    height: f64,
    shoulder: f64,
    drop: f64,
}

#[derive(Clone, Copy)]
struct SharedExpandedRenderState {
    enabled: bool,
    visible: bool,
    interactive: bool,
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_panel_geometry(
    handles: NativePanelHandles,
    frame: NativePanelTransitionFrame,
) {
    let refs = resolve_native_panel_refs(handles);
    let panel = refs.panel;
    let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
    let progress = panel_render_progress(frame);
    let content_visibility = native_panel_content_visibility();
    let layout = resolve_native_panel_layout(
        screen_frame,
        native_panel_geometry_metrics(panel.screen().as_deref(), screen_frame),
        frame.canvas_height,
        frame.visible_height,
        progress.bar,
        progress.height,
        progress.drop,
        content_visibility,
    );

    apply_panel_view_frames(&refs, &layout, progress);
    let shared_state = shared_expanded_render_state(&layout, progress, content_visibility);
    apply_panel_layer_styles(
        &refs,
        PanelLayerStyleState {
            shell_visible: layout.shell_visible,
            separator_visibility: layout.separator_visibility,
            shared_visible: shared_state.visible,
            bar_progress: progress.bar,
            height_progress: progress.height,
        },
    );
    sync_shared_expanded_render_frame(&layout, shared_state);
    invalidate_panel_render_views(&refs);
}

fn panel_render_progress(frame: NativePanelTransitionFrame) -> PanelRenderProgress {
    PanelRenderProgress {
        bar: frame.bar_progress.clamp(0.0, 1.0),
        height: frame.height_progress.clamp(0.0, 1.0),
        shoulder: frame.shoulder_progress.clamp(0.0, 1.0),
        drop: frame.drop_progress.clamp(0.0, 1.0),
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_panel_view_frames(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    progress: PanelRenderProgress,
) {
    let panel = refs.panel;
    let content_view = refs.content_view;
    let left_shoulder = refs.left_shoulder;
    let right_shoulder = refs.right_shoulder;
    let pill_view = refs.pill_view;
    let expanded_container = refs.expanded_container;
    let cards_container = refs.cards_container;
    let body_separator = refs.body_separator;

    apply_panel_frame(panel, layout.panel_frame);
    content_view.setFrame(layout.content_frame);
    pill_view.setFrame(layout.pill_frame);
    apply_shoulder_path_scale_x(
        left_shoulder,
        layout.left_shoulder_frame,
        progress.shoulder,
        true,
    );
    apply_shoulder_path_scale_x(
        right_shoulder,
        layout.right_shoulder_frame,
        progress.shoulder,
        false,
    );
    relayout_compact_content(refs, layout.pill_frame.size, progress.bar >= 0.48);
    expanded_container.setFrame(layout.expanded_frame);
    cards_container.setFrame(layout.cards_frame);
    body_separator.setFrame(layout.separator_frame);
}

fn shared_expanded_render_state(
    layout: &NativePanelLayout,
    progress: PanelRenderProgress,
    content_visibility: f64,
) -> SharedExpandedRenderState {
    let transitioning = native_panel_state()
        .and_then(|state| state.lock().ok().map(|guard| guard.transitioning))
        .unwrap_or(false);
    let shared_expanded_enabled = macos_shared_expanded_window::shared_expanded_enabled();
    let status_surface_active = native_status_surface_active();
    let (shared_content_visible, shared_content_interactive) = shared_expanded_content_state(
        shared_expanded_enabled,
        layout.shell_visible,
        progress.height,
        progress.bar,
        layout.cards_frame.size.height,
        status_surface_active,
        content_visibility,
    );

    SharedExpandedRenderState {
        enabled: shared_expanded_enabled,
        visible: shared_content_visible && !transitioning,
        interactive: shared_content_interactive && !transitioning,
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sync_shared_expanded_render_frame(
    layout: &NativePanelLayout,
    shared_state: SharedExpandedRenderState,
) {
    if shared_state.enabled {
        let _ = macos_shared_expanded_window::sync_shared_expanded_frame(
            layout.shared_content_frame,
            shared_state.visible,
            shared_state.interactive,
        );
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn invalidate_panel_render_views(refs: &NativePanelRefs) {
    refs.pill_view.displayIfNeeded();
    refs.expanded_container.displayIfNeeded();
    refs.left_shoulder.setNeedsDisplay(true);
    refs.right_shoulder.setNeedsDisplay(true);
    refs.pill_view.setNeedsDisplay(true);
    refs.expanded_container.setNeedsDisplay(true);
    refs.content_view.setNeedsDisplay(true);
    refs.content_view.layoutSubtreeIfNeeded();
    refs.content_view.displayIfNeededIgnoringOpacity();
    refs.panel.displayIfNeeded();
}
