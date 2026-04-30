use super::compact_bar_layout::relayout_compact_content;
use super::panel_geometry::{
    apply_panel_frame, native_panel_core_layout, native_panel_geometry_metrics,
    resolve_native_panel_layout,
};
use super::panel_helpers::native_panel_content_visibility;
use super::panel_interaction::native_status_surface_active;
use super::panel_refs::{NativePanelRefs, native_panel_state, resolve_native_panel_refs};
use super::panel_scene_adapter::{
    resolve_and_cache_native_panel_presentation, resolve_current_native_panel_runtime_render_state,
    resolve_current_native_panel_scene,
};
use super::panel_screen_geometry::resolve_screen_frame_for_panel;
use super::panel_shoulder::apply_shoulder_path_scale_x;
use super::panel_style::apply_panel_layer_styles;
use super::panel_types::{NativePanelHandles, NativePanelLayout, NativePanelTransitionFrame};
use crate::macos_shared_expanded_window;
use crate::native_panel_core::{
    PanelRenderProgress, PanelRenderState, PanelRenderStateInput, resolve_panel_render_progress,
    resolve_panel_render_state,
};
use crate::native_panel_renderer::facade::{
    descriptor::{
        NativePanelEdgeAction, NativePanelEdgeActionFrames, NativePanelPointerRegionInput,
    },
    presentation::{NativePanelActionButtonCommand, resolve_native_panel_presentation},
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
    sync_native_panel_pointer_regions(&layout, &refs, runtime_state, render_state);
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
    relayout_compact_content(
        refs,
        layout.pill_frame.size,
        progress.bar >= crate::native_panel_core::PANEL_EDGE_ACTIONS_REVEAL_START_PROGRESS,
    );
    expanded_container.setFrame(layout.expanded_frame);
    cards_container.setFrame(layout.cards_frame);
    body_separator.setFrame(layout.separator_frame);
}

fn sync_native_panel_pointer_regions(
    layout: &NativePanelLayout,
    refs: &NativePanelRefs,
    runtime_state: crate::native_panel_scene::PanelRuntimeRenderState,
    render_state: PanelRenderState,
) {
    let pointer_region_input = Some(edge_action_pointer_region_input(layout, refs));
    let resolved = native_panel_state()
        .and_then(|state| state.lock().ok())
        .and_then(|mut guard| {
            resolve_and_cache_native_panel_presentation(
                &mut guard,
                native_panel_core_layout(layout),
                render_state,
                pointer_region_input,
            )
        })
        .or_else(|| {
            let scene = resolve_current_native_panel_scene()?;
            Some(resolve_native_panel_presentation(
                native_panel_core_layout(layout),
                &scene,
                runtime_state,
                render_state,
                pointer_region_input,
            ))
        });
    let Some(resolved) = resolved else {
        return;
    };
    apply_edge_action_button_commands(
        refs,
        layout,
        &resolved.presentation.action_button_commands(),
    );
}

fn apply_edge_action_button_commands(
    refs: &NativePanelRefs,
    layout: &NativePanelLayout,
    commands: &[NativePanelActionButtonCommand],
) {
    for command in commands {
        let frame = edge_action_command_local_frame(layout, command.frame);
        match command.action {
            NativePanelEdgeAction::Settings => {
                refs.settings_button.setFrame(frame);
            }
            NativePanelEdgeAction::Quit => {
                refs.quit_button.setFrame(frame);
            }
        }
    }
}

fn edge_action_command_local_frame(
    layout: &NativePanelLayout,
    frame: crate::native_panel_core::PanelRect,
) -> objc2_foundation::NSRect {
    objc2_foundation::NSRect::new(
        objc2_foundation::NSPoint::new(
            frame.x - layout.panel_frame.origin.x - layout.pill_frame.origin.x,
            frame.y - layout.panel_frame.origin.y - layout.pill_frame.origin.y,
        ),
        objc2_foundation::NSSize::new(frame.width, frame.height),
    )
}

fn edge_action_pointer_region_input(
    layout: &NativePanelLayout,
    refs: &NativePanelRefs,
) -> NativePanelPointerRegionInput {
    NativePanelPointerRegionInput {
        edge_action_frames: NativePanelEdgeActionFrames {
            settings_action: Some(edge_action_button_pointer_frame(
                layout,
                refs.settings_button.frame(),
            )),
            quit_action: Some(edge_action_button_pointer_frame(
                layout,
                refs.quit_button.frame(),
            )),
        },
    }
}

fn edge_action_button_pointer_frame(
    layout: &NativePanelLayout,
    button_frame: objc2_foundation::NSRect,
) -> crate::native_panel_core::PanelRect {
    crate::native_panel_core::PanelRect {
        x: layout.panel_frame.origin.x + layout.pill_frame.origin.x + button_frame.origin.x,
        y: layout.panel_frame.origin.y + layout.pill_frame.origin.y + button_frame.origin.y,
        width: button_frame.size.width,
        height: button_frame.size.height,
    }
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
        shoulder_progress: progress.shoulder,
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

#[cfg(test)]
mod tests {
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::edge_action_button_pointer_frame;
    use crate::{
        macos_native_test_panel::panel_types::NativePanelLayout, native_panel_core::PanelRect,
    };

    fn layout() -> NativePanelLayout {
        NativePanelLayout {
            panel_frame: NSRect::new(NSPoint::new(500.0, 700.0), NSSize::new(420.0, 180.0)),
            content_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(420.0, 180.0)),
            pill_frame: NSRect::new(NSPoint::new(68.5, 118.0), NSSize::new(283.0, 38.0)),
            left_shoulder_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            right_shoulder_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            expanded_frame: NSRect::new(NSPoint::new(68.5, 0.0), NSSize::new(283.0, 180.0)),
            cards_frame: NSRect::new(NSPoint::new(14.0, 40.0), NSSize::new(255.0, 120.0)),
            separator_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            shared_content_frame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0)),
            shell_visible: true,
            separator_visibility: 0.88,
        }
    }

    #[test]
    fn edge_action_pointer_frame_uses_actual_button_frame() {
        let frame = edge_action_button_pointer_frame(
            &layout(),
            NSRect::new(NSPoint::new(72.0, 6.0), NSSize::new(26.0, 26.0)),
        );

        assert_eq!(
            frame,
            PanelRect {
                x: 640.5,
                y: 824.0,
                width: 26.0,
                height: 26.0,
            }
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
