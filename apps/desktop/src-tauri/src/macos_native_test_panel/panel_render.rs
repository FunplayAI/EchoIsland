use super::panel_shoulder::apply_shoulder_path_scale_x;
use super::panel_style::apply_panel_layer_styles;
use super::*;
use crate::native_panel_core::{
    PanelRenderProgress, PanelRenderState, PanelRenderStateInput, resolve_panel_render_progress,
    resolve_panel_render_state,
};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_panel_geometry(
    handles: NativePanelHandles,
    frame: NativePanelTransitionFrame,
) {
    let refs = resolve_native_panel_refs(handles);
    let panel = refs.panel;
    let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
    let progress = resolve_panel_render_progress(frame);
    let runtime_state = resolve_current_native_panel_runtime_render_state();
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
    let render_state = native_panel_render_state(
        &layout,
        progress,
        content_visibility,
        runtime_state.transitioning,
        runtime_state.shell_scene,
    );
    apply_panel_layer_styles(&refs, render_state.layer_style);
    sync_shared_expanded_render_frame(&layout, render_state.shared);
    invalidate_panel_render_views(&refs);
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

fn native_panel_render_state(
    layout: &NativePanelLayout,
    progress: PanelRenderProgress,
    content_visibility: f64,
    transitioning: bool,
    shell_scene: crate::native_panel_scene::PanelShellSceneState,
) -> PanelRenderState {
    let shared_expanded_enabled = macos_shared_expanded_window::shared_expanded_enabled();
    let status_surface_active = native_status_surface_active();
    resolve_panel_render_state(PanelRenderStateInput {
        shared_expanded_enabled,
        shell_visible: layout.shell_visible,
        separator_visibility: layout.separator_visibility,
        bar_progress: progress.bar,
        height_progress: progress.height,
        cards_height: layout.cards_frame.size.height,
        status_surface_active,
        content_visibility,
        transitioning,
        headline_emphasized: shell_scene.headline_emphasized,
        edge_actions_visible: shell_scene.edge_actions_visible,
    })
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sync_shared_expanded_render_frame(
    layout: &NativePanelLayout,
    shared_state: crate::native_panel_core::SharedExpandedRenderState,
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
