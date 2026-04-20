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
    apply_panel_layer_styles(&refs, &layout, progress, shared_state);
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
    left_shoulder.setFrame(layout.left_shoulder_frame);
    right_shoulder.setFrame(layout.right_shoulder_frame);
    left_shoulder.setAlphaValue(1.0 - progress.shoulder);
    right_shoulder.setAlphaValue(1.0 - progress.shoulder);
    left_shoulder.setHidden(progress.shoulder >= 0.98);
    right_shoulder.setHidden(progress.shoulder >= 0.98);
    relayout_compact_content(refs, layout.pill_frame.size);
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
unsafe fn apply_panel_layer_styles(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    progress: PanelRenderProgress,
    shared_state: SharedExpandedRenderState,
) {
    let expanded_container = refs.expanded_container;
    let body_separator = refs.body_separator;
    let cards_container = refs.cards_container;

    expanded_container.setHidden(!layout.shell_visible);
    expanded_container.setAlphaValue(if layout.shell_visible { 1.0 } else { 0.0 });
    body_separator.setHidden(layout.separator_visibility <= 0.02);
    body_separator.setAlphaValue(layout.separator_visibility);
    cards_container.setHidden(shared_state.visible);

    if let Some(layer) = refs.pill_view.layer() {
        layer.setCornerRadius(lerp(
            COMPACT_PILL_RADIUS,
            PANEL_MORPH_PILL_RADIUS,
            progress.bar,
        ));
        if progress.bar <= 0.01 {
            layer.setMaskedCorners(compact_pill_corner_mask());
        } else {
            layer.setMaskedCorners(all_corner_mask());
        }
        layer.setBorderWidth(lerp(1.0, 0.0, progress.bar));
    }
    if let Some(layer) = expanded_container.layer() {
        layer.setCornerRadius(lerp(
            COMPACT_PILL_RADIUS,
            EXPANDED_PANEL_RADIUS,
            progress.bar.max(progress.height),
        ));
        layer.setBorderWidth(0.0);
        layer.setOpacity(if layout.shell_visible { 1.0 } else { 0.0 });
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
