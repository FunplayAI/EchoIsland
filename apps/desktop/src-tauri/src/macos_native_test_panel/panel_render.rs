use super::compact_bar_layout::relayout_compact_content;
use super::macos_visual_plan::{
    apply_macos_visual_plan_compact_primitives, resolve_macos_native_panel_visual_plan,
};
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
    PanelRenderProgress, PanelRenderState, PanelRenderStateInput,
    resolve_compact_action_button_layout, resolve_panel_render_progress,
    resolve_panel_render_state,
};
use crate::native_panel_renderer::facade::{
    descriptor::{
        NativePanelEdgeAction, NativePanelEdgeActionFrames, NativePanelPointerRegionInput,
    },
    presentation::{
        ActionButtonVisibilitySpecInput, NativePanelActionButtonCommand,
        action_button_transition_progress_from_compact_width, action_button_visual_frame_for_phase,
        resolve_action_button_visibility_spec, resolve_native_panel_presentation,
    },
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
    let pointer_region_input = Some(edge_action_pointer_region_input(
        layout,
        render_state.layer_style,
    ));
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
    let visual_plan = resolve_macos_native_panel_visual_plan(layout, &resolved.presentation);
    apply_edge_action_button_commands(
        refs,
        layout,
        &resolved.presentation.action_button_commands(),
    );
    apply_macos_visual_plan_compact_primitives(refs, layout, &resolved.presentation, &visual_plan);
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
    layer_style: crate::native_panel_core::PanelRenderLayerStyleState,
) -> NativePanelPointerRegionInput {
    let frames = edge_action_visual_frames(layout, layer_style);
    NativePanelPointerRegionInput {
        edge_action_frames: frames,
    }
}

fn edge_action_visual_frames(
    layout: &NativePanelLayout,
    layer_style: crate::native_panel_core::PanelRenderLayerStyleState,
) -> NativePanelEdgeActionFrames {
    let compact_frame = crate::native_panel_core::PanelRect {
        x: layout.panel_frame.origin.x + layout.pill_frame.origin.x,
        y: layout.panel_frame.origin.y + layout.pill_frame.origin.y,
        width: layout.pill_frame.size.width,
        height: layout.pill_frame.size.height,
    };
    let action_layout = resolve_compact_action_button_layout(compact_frame);
    let visibility = resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
        semantic_visible: layer_style.edge_actions_visible,
        expanded_display_mode: layer_style.shell_visible,
        transition_visibility_progress: action_button_transition_progress_from_compact_width(
            compact_frame.width,
        ),
    });

    NativePanelEdgeActionFrames {
        settings_action: Some(action_button_visual_frame_for_phase(
            action_layout.settings,
            visibility,
        )),
        quit_action: Some(action_button_visual_frame_for_phase(
            action_layout.quit,
            visibility,
        )),
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

    use super::{edge_action_command_local_frame, edge_action_visual_frames};
    use crate::{
        macos_native_test_panel::panel_types::NativePanelLayout,
        native_panel_core::{
            PanelRect, PanelRenderLayerStyleState, resolve_compact_action_button_layout,
        },
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
    fn edge_action_pointer_frames_use_shared_visual_frames() {
        let layout = layout();
        let layer_style = PanelRenderLayerStyleState {
            shell_visible: true,
            separator_visibility: 1.0,
            shared_visible: false,
            bar_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 0.0,
            headline_emphasized: false,
            edge_actions_visible: true,
        };
        let frames = edge_action_visual_frames(&layout, layer_style);
        let compact_frame = PanelRect {
            x: layout.panel_frame.origin.x + layout.pill_frame.origin.x,
            y: layout.panel_frame.origin.y + layout.pill_frame.origin.y,
            width: layout.pill_frame.size.width,
            height: layout.pill_frame.size.height,
        };
        let expected = resolve_compact_action_button_layout(compact_frame);

        assert_eq!(frames.settings_action, Some(expected.settings));
        assert_eq!(frames.quit_action, Some(expected.quit));
    }

    #[test]
    fn edge_action_command_local_frame_preserves_shared_visual_frame_position() {
        let layout = layout();
        let compact_frame = PanelRect {
            x: layout.panel_frame.origin.x + layout.pill_frame.origin.x,
            y: layout.panel_frame.origin.y + layout.pill_frame.origin.y,
            width: layout.pill_frame.size.width,
            height: layout.pill_frame.size.height,
        };
        let action_layout = resolve_compact_action_button_layout(compact_frame);

        for expected in [action_layout.settings, action_layout.quit] {
            let local = edge_action_command_local_frame(&layout, expected);

            assert_eq!(
                local.origin.x,
                expected.x - layout.panel_frame.origin.x - layout.pill_frame.origin.x
            );
            assert_eq!(
                local.origin.y,
                expected.y - layout.panel_frame.origin.y - layout.pill_frame.origin.y
            );
            assert_eq!(local.size.width, expected.width);
            assert_eq!(local.size.height, expected.height);
        }
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
