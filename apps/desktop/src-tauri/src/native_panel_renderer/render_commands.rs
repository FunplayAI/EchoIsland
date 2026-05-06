use crate::{
    native_panel_core::{ExpandedSurface, PanelLayout, PanelRect, PanelRenderState},
    native_panel_scene::{
        PanelRuntimeRenderState, PanelScene, SceneCard, SceneGlow, SceneMascotPose, SceneText,
    },
};

use super::descriptors::{
    NativePanelEdgeAction, NativePanelPointerRegion, NativePanelPointerRegionInput,
    NativePanelPointerRegionKind, resolve_native_panel_interaction_plan,
};
use super::presentation_model::estimated_scene_content_height;

#[derive(Clone, Debug)]
pub(crate) struct NativePanelRenderCommandBundle {
    pub(crate) scene: PanelScene,
    pub(crate) runtime: PanelRuntimeRenderState,
    pub(crate) layout: PanelLayout,
    pub(crate) render_state: PanelRenderState,
    pub(crate) shell: NativePanelShellCommand,
    pub(crate) compact_bar: NativePanelCompactBarCommand,
    pub(crate) card_stack: NativePanelCardStackCommand,
    pub(crate) mascot: NativePanelMascotCommand,
    pub(crate) glow: Option<NativePanelGlowCommand>,
    pub(crate) action_buttons: Vec<NativePanelActionButtonCommand>,
    pub(crate) pointer_regions: Vec<NativePanelPointerRegion>,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelShellCommand {
    pub(crate) surface: ExpandedSurface,
    pub(crate) frame: PanelRect,
    pub(crate) visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelCompactBarCommand {
    pub(crate) frame: PanelRect,
    pub(crate) left_shoulder_frame: PanelRect,
    pub(crate) right_shoulder_frame: PanelRect,
    pub(crate) shoulder_progress: f64,
    pub(crate) headline: SceneText,
    pub(crate) active_count: String,
    pub(crate) total_count: String,
    pub(crate) completion_count: usize,
    pub(crate) headline_emphasized: bool,
    pub(crate) actions_visible: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelCardStackCommand {
    pub(crate) frame: PanelRect,
    pub(crate) surface: ExpandedSurface,
    pub(crate) cards: Vec<SceneCard>,
    pub(crate) content_height: f64,
    pub(crate) body_height: f64,
    pub(crate) visible: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelMascotCommand {
    pub(crate) pose: SceneMascotPose,
    pub(crate) debug_mode_enabled: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelGlowCommand {
    pub(crate) glow: SceneGlow,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelActionButtonCommand {
    pub(crate) action: NativePanelEdgeAction,
    pub(crate) frame: PanelRect,
    pub(crate) visible: bool,
}

pub(crate) fn resolve_native_panel_render_command_bundle(
    layout: PanelLayout,
    scene: &PanelScene,
    runtime: PanelRuntimeRenderState,
    render_state: PanelRenderState,
    pointer_region_input: Option<NativePanelPointerRegionInput>,
) -> NativePanelRenderCommandBundle {
    let interaction_plan =
        resolve_native_panel_interaction_plan(layout, scene, pointer_region_input);
    let pointer_regions = interaction_plan.pointer_regions;

    NativePanelRenderCommandBundle {
        scene: scene.clone(),
        runtime,
        layout,
        render_state,
        shell: NativePanelShellCommand {
            surface: scene.surface,
            frame: layout.expanded_frame,
            visible: layout.shell_visible,
            separator_visibility: layout.separator_visibility,
            shared_visible: render_state.shared.visible,
        },
        compact_bar: native_panel_compact_bar_command(
            scene,
            layout.pill_frame,
            layout.left_shoulder_frame,
            layout.right_shoulder_frame,
            render_state.layer_style.shoulder_progress,
        ),
        card_stack: native_panel_card_stack_command(
            scene,
            layout.cards_frame,
            layout.shell_visible && !scene.cards.is_empty(),
        ),
        mascot: native_panel_mascot_command(scene),
        glow: native_panel_glow_command(scene),
        action_buttons: resolve_action_button_commands(&pointer_regions, scene),
        pointer_regions,
    }
}

pub(crate) fn native_panel_compact_bar_command(
    scene: &PanelScene,
    frame: PanelRect,
    left_shoulder_frame: PanelRect,
    right_shoulder_frame: PanelRect,
    shoulder_progress: f64,
) -> NativePanelCompactBarCommand {
    NativePanelCompactBarCommand {
        frame,
        left_shoulder_frame,
        right_shoulder_frame,
        shoulder_progress,
        headline: scene.compact_bar.headline.clone(),
        active_count: scene.compact_bar.active_count.clone(),
        total_count: scene.compact_bar.total_count.clone(),
        completion_count: scene.compact_bar.completion_count,
        headline_emphasized: scene.compact_bar.headline.emphasized,
        actions_visible: scene.compact_bar.actions_visible,
    }
}

pub(crate) fn native_panel_card_stack_command(
    scene: &PanelScene,
    frame: PanelRect,
    visible: bool,
) -> NativePanelCardStackCommand {
    let content_height = estimated_scene_content_height(scene);
    NativePanelCardStackCommand {
        frame,
        surface: scene.surface,
        cards: scene.cards.clone(),
        content_height,
        body_height: content_height.min(crate::native_panel_core::EXPANDED_MAX_BODY_HEIGHT),
        visible,
    }
}

pub(crate) fn native_panel_mascot_command(scene: &PanelScene) -> NativePanelMascotCommand {
    NativePanelMascotCommand {
        pose: scene.mascot_pose,
        debug_mode_enabled: scene.debug_mode_enabled,
    }
}

pub(crate) fn native_panel_glow_command(scene: &PanelScene) -> Option<NativePanelGlowCommand> {
    scene
        .glow
        .clone()
        .map(|glow| NativePanelGlowCommand { glow })
}

fn resolve_action_button_commands(
    pointer_regions: &[NativePanelPointerRegion],
    scene: &PanelScene,
) -> Vec<NativePanelActionButtonCommand> {
    pointer_regions
        .iter()
        .filter_map(|region| match region.kind {
            NativePanelPointerRegionKind::EdgeAction(action) => {
                Some(NativePanelActionButtonCommand {
                    action,
                    frame: region.frame,
                    visible: scene.compact_bar.actions_visible,
                })
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use echoisland_runtime::RuntimeSnapshot;

    use super::*;
    use crate::{
        native_panel_core::{
            ExpandedSurface, PanelGeometryMetrics, PanelLayoutInput, PanelRect,
            PanelRenderLayerStyleState, PanelRenderState, PanelState, SharedExpandedRenderState,
            resolve_panel_layout,
        },
        native_panel_renderer::descriptors::{
            NativePanelEdgeAction, NativePanelEdgeActionFrames, NativePanelPointerRegionInput,
            NativePanelPointerRegionKind,
        },
        native_panel_scene::{PanelSceneBuildInput, build_panel_scene},
    };

    fn snapshot() -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: "Idle".to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: Vec::new(),
            pending_questions: Vec::new(),
            sessions: Vec::new(),
        }
    }

    #[test]
    fn render_command_bundle_carries_shared_scene_layout_state_and_pointer_regions() {
        let mut state = PanelState {
            expanded: true,
            surface_mode: ExpandedSurface::Default,
            ..PanelState::default()
        };
        state.transitioning = false;
        let scene = build_panel_scene(&state, &snapshot(), &PanelSceneBuildInput::default());
        let layout = resolve_panel_layout(PanelLayoutInput {
            screen_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            },
            metrics: PanelGeometryMetrics {
                compact_height: crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT,
                compact_width: crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH,
                expanded_width: crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
                panel_width: crate::native_panel_core::DEFAULT_PANEL_CANVAS_WIDTH,
            },
            canvas_height: 180.0,
            visible_height: 180.0,
            bar_progress: 1.0,
            height_progress: 1.0,
            drop_progress: 1.0,
            content_visibility: 1.0,
            collapsed_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            drop_distance: crate::native_panel_core::PANEL_DROP_DISTANCE,
            content_top_gap: crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP,
            content_bottom_inset: crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET,
            cards_side_inset: crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
            shoulder_size: crate::native_panel_core::COMPACT_SHOULDER_SIZE,
            separator_side_inset: crate::native_panel_core::EXPANDED_SEPARATOR_SIDE_INSET,
        });
        let render_state = PanelRenderState {
            shared: SharedExpandedRenderState {
                enabled: false,
                visible: false,
                interactive: false,
            },
            layer_style: PanelRenderLayerStyleState {
                shell_visible: true,
                separator_visibility: 1.0,
                shared_visible: false,
                bar_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 0.0,
                headline_emphasized: false,
                edge_actions_visible: true,
            },
        };
        let input = NativePanelPointerRegionInput {
            edge_action_frames: NativePanelEdgeActionFrames {
                settings_action: Some(PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 24.0,
                    height: 24.0,
                }),
                quit_action: None,
            },
        };

        let bundle = resolve_native_panel_render_command_bundle(
            layout,
            &scene,
            PanelRuntimeRenderState::default(),
            render_state,
            Some(input),
        );

        assert_eq!(bundle.layout, layout);
        assert_eq!(bundle.render_state, render_state);
        assert_eq!(bundle.shell.surface, scene.surface);
        assert_eq!(bundle.shell.frame, layout.expanded_frame);
        assert_eq!(bundle.compact_bar.frame, layout.pill_frame);
        assert_eq!(
            bundle.compact_bar.headline.text,
            scene.compact_bar.headline.text
        );
        assert_eq!(bundle.card_stack.frame, layout.cards_frame);
        assert_eq!(bundle.card_stack.cards.len(), scene.cards.len());
        assert_eq!(bundle.mascot.pose, scene.mascot_pose);
        assert_eq!(bundle.action_buttons.len(), 2);
        assert!(bundle.pointer_regions.iter().any(|region| matches!(
            region.kind,
            NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings)
        ) && region.frame
            == input.edge_action_frames.settings_action.unwrap()));
        assert!(bundle.action_buttons.iter().any(|button| {
            button.action == NativePanelEdgeAction::Settings
                && button.frame == input.edge_action_frames.settings_action.unwrap()
                && button.visible
        }));
    }
}
