use echoisland_runtime::{PendingPermissionView, PendingQuestionView, SessionSnapshotView};

use crate::{
    native_panel_core::{
        DEFAULT_PANEL_CANVAS_WIDTH, EMPTY_CARD_HEIGHT, EXPANDED_CARD_GAP, EXPANDED_MAX_BODY_HEIGHT,
        PENDING_QUESTION_CARD_MAX_HEIGHT, PENDING_QUESTION_CARD_MIN_HEIGHT, PanelLayout, PanelRect,
        PanelRenderState, SessionCardContentInput, StatusQueueItem, StatusQueuePayload,
        completion_preview_text, default_panel_card_metric_constants, display_snippet,
        is_long_idle_session, resolve_card_chat_body_width, resolve_completion_card_height,
        resolve_pending_like_card_height, resolve_session_card_content_height,
        resolve_session_card_height, resolve_settings_surface_card_height,
        resolve_stacked_cards_total_height, session_has_visible_card_body, session_prompt_preview,
        session_reply_preview, session_tool_preview,
    },
    native_panel_scene::{
        PanelRuntimeRenderState, PanelScene, SceneCard, SceneText, resolve_scene_card_height_input,
    },
};

use super::descriptors::{
    NativePanelEdgeAction, NativePanelHostWindowState, NativePanelPointerRegionInput,
};
use super::render_commands::{
    NativePanelActionButtonCommand, NativePanelCardStackCommand, NativePanelCompactBarCommand,
    NativePanelGlowCommand, NativePanelMascotCommand, NativePanelRenderCommandBundle,
    NativePanelShellCommand, native_panel_card_stack_command, native_panel_glow_command,
    native_panel_mascot_command, resolve_native_panel_render_command_bundle,
};
use super::visual_plan::{
    NativePanelVisualActionButtonInput, NativePanelVisualDisplayMode, NativePanelVisualPlanInput,
    native_panel_visual_card_input_from_scene_card_with_height,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelPresentationMetrics {
    pub(crate) expanded_content_height: f64,
    pub(crate) expanded_body_height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelCompactBarPresentationInput {
    pub(crate) headline: SceneText,
    pub(crate) active_count: String,
    pub(crate) total_count: String,
    pub(crate) completion_count: usize,
    pub(crate) headline_emphasized: bool,
    pub(crate) actions_visible: bool,
    pub(crate) shoulder_progress: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelPresentationModelInput {
    pub(crate) shell: NativePanelShellPresentationInput,
    pub(crate) compact_bar: NativePanelCompactBarPresentationInput,
    pub(crate) card_stack: NativePanelCardStackPresentationInput,
    pub(crate) mascot: NativePanelMascotPresentationInput,
    pub(crate) glow: Option<NativePanelGlowPresentationInput>,
    pub(crate) action_buttons: NativePanelActionButtonsPresentationInput,
    pub(crate) metrics: NativePanelPresentationMetrics,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelShellPresentationInput {
    pub(crate) surface: crate::native_panel_core::ExpandedSurface,
    pub(crate) visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelShellPresentation {
    pub(crate) surface: crate::native_panel_core::ExpandedSurface,
    pub(crate) frame: PanelRect,
    pub(crate) visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
}

impl NativePanelShellPresentation {
    pub(crate) fn command(&self, frame: PanelRect) -> NativePanelShellCommand {
        NativePanelShellCommand {
            surface: self.surface,
            frame,
            visible: self.visible,
            separator_visibility: self.separator_visibility,
            shared_visible: self.shared_visible,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelCompactBarPresentation {
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

impl NativePanelCompactBarPresentation {
    pub(crate) fn command(&self, frame: PanelRect) -> NativePanelCompactBarCommand {
        NativePanelCompactBarCommand {
            frame,
            left_shoulder_frame: self.left_shoulder_frame,
            right_shoulder_frame: self.right_shoulder_frame,
            shoulder_progress: self.shoulder_progress,
            headline: self.headline.clone(),
            active_count: self.active_count.clone(),
            total_count: self.total_count.clone(),
            completion_count: self.completion_count,
            headline_emphasized: self.headline_emphasized,
            actions_visible: self.actions_visible,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelActionButtonPresentation {
    pub(crate) action: NativePanelEdgeAction,
    pub(crate) frame: PanelRect,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelActionButtonPresentationInput {
    pub(crate) action: NativePanelEdgeAction,
    pub(crate) frame: PanelRect,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelCardStackPresentationInput {
    pub(crate) surface: crate::native_panel_core::ExpandedSurface,
    pub(crate) cards: Vec<SceneCard>,
    pub(crate) content_height: f64,
    pub(crate) body_height: f64,
    pub(crate) visible: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelCardStackPresentation {
    pub(crate) frame: PanelRect,
    pub(crate) surface: crate::native_panel_core::ExpandedSurface,
    pub(crate) cards: Vec<SceneCard>,
    pub(crate) content_height: f64,
    pub(crate) body_height: f64,
    pub(crate) visible: bool,
}

impl NativePanelCardStackPresentation {
    pub(crate) fn command(&self, frame: PanelRect, visible: bool) -> NativePanelCardStackCommand {
        NativePanelCardStackCommand {
            frame,
            surface: self.surface,
            cards: self.cards.clone(),
            content_height: self.content_height,
            body_height: self.body_height,
            visible,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelMascotPresentationInput {
    pub(crate) pose: crate::native_panel_scene::SceneMascotPose,
    pub(crate) debug_mode_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelMascotPresentation {
    pub(crate) pose: crate::native_panel_scene::SceneMascotPose,
    pub(crate) debug_mode_enabled: bool,
}

impl NativePanelMascotPresentation {
    pub(crate) fn command(&self) -> NativePanelMascotCommand {
        NativePanelMascotCommand {
            pose: self.pose,
            debug_mode_enabled: self.debug_mode_enabled,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelGlowPresentationInput {
    pub(crate) glow: crate::native_panel_scene::SceneGlow,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelGlowPresentation {
    pub(crate) glow: crate::native_panel_scene::SceneGlow,
}

impl NativePanelGlowPresentation {
    pub(crate) fn command(&self) -> NativePanelGlowCommand {
        NativePanelGlowCommand {
            glow: self.glow.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NativePanelActionButtonsPresentation {
    pub(crate) visible: bool,
    pub(crate) buttons: Vec<NativePanelActionButtonPresentation>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NativePanelActionButtonsPresentationInput {
    pub(crate) visible: bool,
    pub(crate) buttons: Vec<NativePanelActionButtonPresentationInput>,
}

impl NativePanelActionButtonsPresentation {
    pub(crate) fn commands(&self) -> Vec<NativePanelActionButtonCommand> {
        self.buttons
            .iter()
            .map(|button| NativePanelActionButtonCommand {
                action: button.action,
                frame: button.frame,
                visible: self.visible,
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelPresentationModel {
    pub(crate) panel_frame: PanelRect,
    pub(crate) content_frame: PanelRect,
    pub(crate) shell: NativePanelShellPresentation,
    pub(crate) compact_bar: NativePanelCompactBarPresentation,
    pub(crate) card_stack: NativePanelCardStackPresentation,
    pub(crate) mascot: NativePanelMascotPresentation,
    pub(crate) glow: Option<NativePanelGlowPresentation>,
    pub(crate) action_buttons: NativePanelActionButtonsPresentation,
    pub(crate) metrics: NativePanelPresentationMetrics,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelResolvedPresentation {
    pub(crate) bundle: NativePanelRenderCommandBundle,
    pub(crate) presentation: NativePanelPresentationModel,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelSnapshotRenderPlan {
    #[cfg(test)]
    pub(crate) scene: PanelScene,
    presentation: NativePanelPresentationModel,
}

impl NativePanelSnapshotRenderPlan {
    pub(crate) fn compact_bar_command(&self, frame: PanelRect) -> NativePanelCompactBarCommand {
        self.presentation.compact_bar.command(frame)
    }

    pub(crate) fn card_stack_command(
        &self,
        frame: PanelRect,
        visible: bool,
    ) -> NativePanelCardStackCommand {
        self.presentation.card_stack_command(frame, visible)
    }

    pub(crate) fn expanded_body_height(&self) -> f64 {
        self.presentation.metrics.expanded_body_height
    }

    pub(crate) fn surface(&self) -> crate::native_panel_core::ExpandedSurface {
        self.presentation.shell.surface
    }

    #[cfg(test)]
    pub(crate) fn expanded_content_height(&self) -> f64 {
        self.presentation.metrics.expanded_content_height
    }
}

impl NativePanelPresentationModel {
    pub(crate) fn shell_command(&self, frame: PanelRect) -> NativePanelShellCommand {
        self.shell.command(frame)
    }

    pub(crate) fn compact_bar_command(&self, frame: PanelRect) -> NativePanelCompactBarCommand {
        self.compact_bar.command(frame)
    }

    pub(crate) fn card_stack_command(
        &self,
        frame: PanelRect,
        visible: bool,
    ) -> NativePanelCardStackCommand {
        self.card_stack.command(frame, visible)
    }

    pub(crate) fn card_stack_visible(&self) -> bool {
        self.card_stack.visible
    }

    pub(crate) fn action_button_commands(&self) -> Vec<NativePanelActionButtonCommand> {
        self.action_buttons.commands()
    }
}

pub(crate) fn resolve_native_panel_presentation_model(
    bundle: &NativePanelRenderCommandBundle,
) -> NativePanelPresentationModel {
    native_panel_presentation_model_from_input(
        native_panel_presentation_model_input_from_bundle(bundle),
        bundle.layout.panel_frame,
        bundle.layout.content_frame,
        bundle.shell.frame,
        bundle.compact_bar.frame,
        bundle.compact_bar.left_shoulder_frame,
        bundle.compact_bar.right_shoulder_frame,
        bundle.card_stack.frame,
    )
}

pub(crate) fn native_panel_visual_display_mode_from_presentation(
    window_state: NativePanelHostWindowState,
    presentation: Option<&NativePanelPresentationModel>,
) -> NativePanelVisualDisplayMode {
    if !window_state.visible {
        NativePanelVisualDisplayMode::Hidden
    } else if presentation.is_some_and(|presentation| presentation.shell.visible) {
        NativePanelVisualDisplayMode::Expanded
    } else {
        NativePanelVisualDisplayMode::Compact
    }
}

pub(crate) fn native_panel_visual_plan_input_from_presentation(
    window_state: NativePanelHostWindowState,
    display_mode: NativePanelVisualDisplayMode,
    presentation: Option<&NativePanelPresentationModel>,
) -> NativePanelVisualPlanInput {
    let zero = PanelRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    NativePanelVisualPlanInput {
        window_state,
        display_mode,
        surface: presentation
            .map(|presentation| presentation.shell.surface)
            .unwrap_or(crate::native_panel_core::ExpandedSurface::Default),
        panel_frame: presentation
            .map(|presentation| presentation.panel_frame)
            .unwrap_or_else(|| window_state.frame.unwrap_or(zero)),
        compact_bar_frame: presentation
            .map(|presentation| presentation.compact_bar.frame)
            .unwrap_or(zero),
        left_shoulder_frame: presentation
            .map(|presentation| presentation.compact_bar.left_shoulder_frame)
            .unwrap_or(zero),
        right_shoulder_frame: presentation
            .map(|presentation| presentation.compact_bar.right_shoulder_frame)
            .unwrap_or(zero),
        shoulder_progress: presentation
            .map(|presentation| presentation.compact_bar.shoulder_progress)
            .unwrap_or(0.0),
        content_frame: presentation
            .map(|presentation| presentation.content_frame)
            .unwrap_or(zero),
        card_stack_frame: presentation
            .map(|presentation| presentation.card_stack.frame)
            .unwrap_or(zero),
        card_stack_content_height: presentation
            .map(|presentation| presentation.card_stack.content_height)
            .unwrap_or(0.0),
        shell_frame: presentation
            .map(|presentation| presentation.shell.frame)
            .unwrap_or(zero),
        headline_text: presentation
            .map(|presentation| presentation.compact_bar.headline.text.clone())
            .unwrap_or_default(),
        headline_emphasized: presentation
            .map(|presentation| presentation.compact_bar.headline_emphasized)
            .unwrap_or(false),
        active_count: presentation
            .map(|presentation| presentation.compact_bar.active_count.clone())
            .unwrap_or_default(),
        active_count_elapsed_ms: 0,
        total_count: presentation
            .map(|presentation| presentation.compact_bar.total_count.clone())
            .unwrap_or_default(),
        separator_visibility: presentation
            .map(|presentation| presentation.shell.separator_visibility)
            .unwrap_or(0.0),
        cards_visible: presentation
            .map(|presentation| presentation.card_stack_visible())
            .unwrap_or(false),
        card_count: presentation
            .map(|presentation| presentation.card_stack.cards.len())
            .unwrap_or(0),
        cards: presentation
            .map(|presentation| {
                presentation
                    .card_stack
                    .cards
                    .iter()
                    .map(|card| {
                        native_panel_visual_card_input_from_scene_card_with_height(
                            card,
                            estimated_scene_card_height(card),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        glow_visible: presentation
            .map(|presentation| presentation.glow.is_some())
            .unwrap_or(false),
        glow_opacity: presentation
            .and_then(|presentation| presentation.glow.as_ref())
            .map(|glow| glow.glow.opacity)
            .unwrap_or(0.0),
        action_buttons_visible: presentation
            .map(|presentation| presentation.action_buttons.visible)
            .unwrap_or(false),
        action_buttons: presentation
            .map(|presentation| {
                presentation
                    .action_buttons
                    .buttons
                    .iter()
                    .map(|button| NativePanelVisualActionButtonInput {
                        action: button.action,
                        frame: local_visual_frame_from_panel_frame(
                            presentation.panel_frame,
                            button.frame,
                        ),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        completion_count: presentation
            .map(|presentation| presentation.compact_bar.completion_count)
            .unwrap_or(0),
        mascot_elapsed_ms: 0,
        mascot_motion_frame: None,
        mascot_pose: presentation
            .map(|presentation| presentation.mascot.pose)
            .unwrap_or(crate::native_panel_scene::SceneMascotPose::Idle),
        mascot_debug_mode_enabled: presentation
            .map(|presentation| presentation.mascot.debug_mode_enabled)
            .unwrap_or(false),
    }
}

fn local_visual_frame_from_panel_frame(panel_frame: PanelRect, frame: PanelRect) -> PanelRect {
    let frame_is_absolute = frame.x >= panel_frame.x
        && frame.x + frame.width <= panel_frame.x + panel_frame.width
        && frame.y >= panel_frame.y
        && frame.y + frame.height <= panel_frame.y + panel_frame.height;

    if frame_is_absolute {
        PanelRect {
            x: frame.x - panel_frame.x,
            y: frame.y - panel_frame.y,
            width: frame.width,
            height: frame.height,
        }
    } else {
        frame
    }
}

pub(crate) fn resolve_native_panel_presentation(
    layout: PanelLayout,
    scene: &PanelScene,
    runtime: PanelRuntimeRenderState,
    render_state: PanelRenderState,
    pointer_region_input: Option<NativePanelPointerRegionInput>,
) -> NativePanelResolvedPresentation {
    let bundle = resolve_native_panel_render_command_bundle(
        layout,
        scene,
        runtime,
        render_state,
        pointer_region_input,
    );
    let presentation = resolve_native_panel_presentation_model(&bundle);

    NativePanelResolvedPresentation {
        bundle,
        presentation,
    }
}

pub(crate) fn resolve_native_panel_presentation_model_for_scene(
    scene: &PanelScene,
    bundle: Option<&NativePanelRenderCommandBundle>,
) -> NativePanelPresentationModel {
    bundle
        .map(resolve_native_panel_presentation_model)
        .unwrap_or_else(|| build_native_panel_presentation_model(scene))
}

pub(crate) fn resolve_native_panel_snapshot_render_plan_for_scene(
    scene: PanelScene,
    bundle: Option<NativePanelRenderCommandBundle>,
) -> NativePanelSnapshotRenderPlan {
    let presentation = resolve_native_panel_presentation_model_for_scene(&scene, bundle.as_ref());

    NativePanelSnapshotRenderPlan {
        #[cfg(test)]
        scene,
        presentation,
    }
}

pub(crate) fn build_native_panel_presentation_model(
    scene: &PanelScene,
) -> NativePanelPresentationModel {
    let zero = PanelRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    native_panel_presentation_model_from_input(
        native_panel_presentation_model_input_from_scene(scene),
        zero,
        zero,
        zero,
        zero,
        zero,
        zero,
        zero,
    )
}

pub(crate) fn native_panel_presentation_model_input_from_bundle(
    bundle: &NativePanelRenderCommandBundle,
) -> NativePanelPresentationModelInput {
    NativePanelPresentationModelInput {
        shell: shell_presentation_input_from_command(&bundle.shell),
        compact_bar: compact_bar_presentation_input_from_command(&bundle.compact_bar),
        card_stack: card_stack_presentation_input_from_command(&bundle.card_stack),
        mascot: mascot_presentation_input_from_command(&bundle.mascot),
        glow: bundle
            .glow
            .as_ref()
            .map(glow_presentation_input_from_command),
        action_buttons: action_buttons_presentation_input_from_commands(&bundle.action_buttons),
        metrics: resolve_native_panel_presentation_metrics(&bundle.scene),
    }
}

pub(crate) fn native_panel_presentation_model_input_from_scene(
    scene: &PanelScene,
) -> NativePanelPresentationModelInput {
    NativePanelPresentationModelInput {
        shell: shell_presentation_input_from_scene(scene),
        compact_bar: compact_bar_presentation_input_from_scene(scene),
        card_stack: card_stack_presentation_input_from_scene(scene),
        mascot: NativePanelMascotPresentationInput {
            pose: scene.mascot_pose,
            debug_mode_enabled: scene.debug_mode_enabled,
        },
        glow: scene
            .glow
            .clone()
            .map(|glow| NativePanelGlowPresentationInput { glow }),
        action_buttons: NativePanelActionButtonsPresentationInput::default(),
        metrics: resolve_native_panel_presentation_metrics(scene),
    }
}

pub(crate) fn native_panel_presentation_model_from_input(
    input: NativePanelPresentationModelInput,
    panel_frame: PanelRect,
    content_frame: PanelRect,
    shell_frame: PanelRect,
    compact_bar_frame: PanelRect,
    left_shoulder_frame: PanelRect,
    right_shoulder_frame: PanelRect,
    card_stack_frame: PanelRect,
) -> NativePanelPresentationModel {
    NativePanelPresentationModel {
        panel_frame,
        content_frame,
        shell: native_panel_shell_presentation_from_input(input.shell, shell_frame),
        compact_bar: native_panel_compact_bar_presentation_from_input(
            input.compact_bar,
            compact_bar_frame,
            left_shoulder_frame,
            right_shoulder_frame,
        ),
        card_stack: native_panel_card_stack_presentation_from_input(
            input.card_stack,
            card_stack_frame,
        ),
        mascot: native_panel_mascot_presentation_from_input(input.mascot),
        glow: input.glow.map(native_panel_glow_presentation_from_input),
        action_buttons: native_panel_action_buttons_presentation_from_input(input.action_buttons),
        metrics: input.metrics,
    }
}

pub(crate) fn shell_presentation_input_from_scene(
    scene: &PanelScene,
) -> NativePanelShellPresentationInput {
    NativePanelShellPresentationInput {
        surface: scene.surface,
        visible: false,
        separator_visibility: 0.0,
        shared_visible: false,
    }
}

pub(crate) fn shell_presentation_input_from_command(
    command: &NativePanelShellCommand,
) -> NativePanelShellPresentationInput {
    NativePanelShellPresentationInput {
        surface: command.surface,
        visible: command.visible,
        separator_visibility: command.separator_visibility,
        shared_visible: command.shared_visible,
    }
}

pub(crate) fn native_panel_shell_presentation_from_input(
    input: NativePanelShellPresentationInput,
    frame: PanelRect,
) -> NativePanelShellPresentation {
    NativePanelShellPresentation {
        surface: input.surface,
        frame,
        visible: input.visible,
        separator_visibility: input.separator_visibility,
        shared_visible: input.shared_visible,
    }
}

pub(crate) fn native_panel_compact_bar_presentation(
    scene: &PanelScene,
) -> NativePanelCompactBarPresentation {
    let zero = PanelRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    native_panel_compact_bar_presentation_from_input(
        compact_bar_presentation_input_from_scene(scene),
        zero,
        zero,
        zero,
    )
}

pub(crate) fn compact_bar_presentation_input_from_scene(
    scene: &PanelScene,
) -> NativePanelCompactBarPresentationInput {
    NativePanelCompactBarPresentationInput {
        headline: scene.compact_bar.headline.clone(),
        active_count: scene.compact_bar.active_count.clone(),
        total_count: scene.compact_bar.total_count.clone(),
        completion_count: scene.compact_bar.completion_count,
        headline_emphasized: scene.compact_bar.headline.emphasized,
        actions_visible: scene.compact_bar.actions_visible,
        shoulder_progress: 0.0,
    }
}

pub(crate) fn card_stack_presentation_input_from_scene(
    scene: &PanelScene,
) -> NativePanelCardStackPresentationInput {
    let content_height = estimated_scene_content_height(scene);
    NativePanelCardStackPresentationInput {
        surface: scene.surface,
        cards: scene.cards.clone(),
        content_height,
        body_height: content_height.min(EXPANDED_MAX_BODY_HEIGHT),
        visible: !scene.cards.is_empty(),
    }
}

pub(crate) fn card_stack_presentation_input_from_command(
    command: &NativePanelCardStackCommand,
) -> NativePanelCardStackPresentationInput {
    NativePanelCardStackPresentationInput {
        surface: command.surface,
        cards: command.cards.clone(),
        content_height: command.content_height,
        body_height: command.body_height,
        visible: command.visible,
    }
}

pub(crate) fn native_panel_card_stack_presentation_from_input(
    input: NativePanelCardStackPresentationInput,
    frame: PanelRect,
) -> NativePanelCardStackPresentation {
    NativePanelCardStackPresentation {
        frame,
        surface: input.surface,
        cards: input.cards,
        content_height: input.content_height,
        body_height: input.body_height,
        visible: input.visible,
    }
}

pub(crate) fn native_panel_card_stack_presentation(
    scene: &PanelScene,
) -> NativePanelCardStackPresentation {
    native_panel_card_stack_presentation_from_input(
        card_stack_presentation_input_from_scene(scene),
        PanelRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    )
}

pub(crate) fn mascot_presentation_input_from_command(
    command: &NativePanelMascotCommand,
) -> NativePanelMascotPresentationInput {
    NativePanelMascotPresentationInput {
        pose: command.pose,
        debug_mode_enabled: command.debug_mode_enabled,
    }
}

pub(crate) fn native_panel_mascot_presentation_from_input(
    input: NativePanelMascotPresentationInput,
) -> NativePanelMascotPresentation {
    NativePanelMascotPresentation {
        pose: input.pose,
        debug_mode_enabled: input.debug_mode_enabled,
    }
}

pub(crate) fn native_panel_mascot_presentation(
    scene: &PanelScene,
) -> NativePanelMascotPresentation {
    native_panel_mascot_presentation_from_input(NativePanelMascotPresentationInput {
        pose: native_panel_mascot_command(scene).pose,
        debug_mode_enabled: scene.debug_mode_enabled,
    })
}

pub(crate) fn glow_presentation_input_from_command(
    command: &NativePanelGlowCommand,
) -> NativePanelGlowPresentationInput {
    NativePanelGlowPresentationInput {
        glow: command.glow.clone(),
    }
}

pub(crate) fn native_panel_glow_presentation_from_input(
    input: NativePanelGlowPresentationInput,
) -> NativePanelGlowPresentation {
    NativePanelGlowPresentation { glow: input.glow }
}

pub(crate) fn native_panel_glow_presentation(
    scene: &PanelScene,
) -> Option<NativePanelGlowPresentation> {
    native_panel_glow_command(scene)
        .as_ref()
        .map(glow_presentation_input_from_command)
        .map(native_panel_glow_presentation_from_input)
}

pub(crate) fn compact_bar_presentation_input_from_command(
    command: &NativePanelCompactBarCommand,
) -> NativePanelCompactBarPresentationInput {
    NativePanelCompactBarPresentationInput {
        headline: command.headline.clone(),
        active_count: command.active_count.clone(),
        total_count: command.total_count.clone(),
        completion_count: command.completion_count,
        headline_emphasized: command.headline_emphasized,
        actions_visible: command.actions_visible,
        shoulder_progress: command.shoulder_progress,
    }
}

pub(crate) fn native_panel_compact_bar_presentation_from_input(
    input: NativePanelCompactBarPresentationInput,
    frame: PanelRect,
    left_shoulder_frame: PanelRect,
    right_shoulder_frame: PanelRect,
) -> NativePanelCompactBarPresentation {
    NativePanelCompactBarPresentation {
        frame,
        left_shoulder_frame,
        right_shoulder_frame,
        shoulder_progress: input.shoulder_progress,
        headline: input.headline,
        active_count: input.active_count,
        total_count: input.total_count,
        completion_count: input.completion_count,
        headline_emphasized: input.headline_emphasized,
        actions_visible: input.actions_visible,
    }
}

pub(crate) fn action_buttons_presentation_input_from_commands(
    commands: &[NativePanelActionButtonCommand],
) -> NativePanelActionButtonsPresentationInput {
    NativePanelActionButtonsPresentationInput {
        visible: commands.iter().any(|command| command.visible),
        buttons: commands
            .iter()
            .map(|command| NativePanelActionButtonPresentationInput {
                action: command.action,
                frame: command.frame,
            })
            .collect(),
    }
}

pub(crate) fn native_panel_action_buttons_presentation_from_input(
    input: NativePanelActionButtonsPresentationInput,
) -> NativePanelActionButtonsPresentation {
    NativePanelActionButtonsPresentation {
        visible: input.visible,
        buttons: input
            .buttons
            .into_iter()
            .map(|button| NativePanelActionButtonPresentation {
                action: button.action,
                frame: button.frame,
            })
            .collect(),
    }
}

pub(crate) fn resolve_native_panel_presentation_metrics(
    scene: &PanelScene,
) -> NativePanelPresentationMetrics {
    let expanded_content_height = estimated_scene_content_height(scene);

    NativePanelPresentationMetrics {
        expanded_content_height,
        expanded_body_height: expanded_content_height.min(EXPANDED_MAX_BODY_HEIGHT),
    }
}

pub(crate) fn estimated_scene_content_height(scene: &PanelScene) -> f64 {
    estimated_scene_cards_content_height(&scene.cards)
}

pub(crate) fn estimated_scene_cards_content_height(cards: &[SceneCard]) -> f64 {
    let card_heights = cards
        .iter()
        .map(estimated_scene_card_height)
        .collect::<Vec<_>>();
    resolve_stacked_cards_total_height(&card_heights, EXPANDED_CARD_GAP, EMPTY_CARD_HEIGHT)
}

pub(crate) fn estimated_scene_card_height(card: &SceneCard) -> f64 {
    match resolve_scene_card_height_input(card) {
        crate::native_panel_scene::SceneCardHeightInput::Settings { row_count } => {
            resolve_settings_surface_card_height(row_count)
        }
        crate::native_panel_scene::SceneCardHeightInput::PendingPermission(pending) => {
            pending_permission_card_height(pending)
        }
        crate::native_panel_scene::SceneCardHeightInput::PendingQuestion(pending) => {
            pending_question_card_height(pending)
        }
        crate::native_panel_scene::SceneCardHeightInput::PromptAssist(session) => {
            prompt_assist_card_height(session)
        }
        crate::native_panel_scene::SceneCardHeightInput::Session(session) => {
            session_card_height(session)
        }
        crate::native_panel_scene::SceneCardHeightInput::StatusItem(item) => {
            status_queue_card_height(item)
        }
        crate::native_panel_scene::SceneCardHeightInput::Empty => EMPTY_CARD_HEIGHT,
    }
}

fn status_queue_card_height(item: &StatusQueueItem) -> f64 {
    match &item.payload {
        StatusQueuePayload::Approval(pending) => pending_permission_card_height(pending),
        StatusQueuePayload::Question(pending) => pending_question_card_height(pending),
        StatusQueuePayload::Completion(session) => completion_card_height(session),
    }
}

fn pending_permission_card_height(pending: &PendingPermissionView) -> f64 {
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());
    pending_like_card_height(
        &body,
        crate::native_panel_core::PENDING_PERMISSION_CARD_MIN_HEIGHT,
        crate::native_panel_core::PENDING_PERMISSION_CARD_MAX_HEIGHT,
    )
}

fn pending_question_card_height(pending: &PendingQuestionView) -> f64 {
    let body = display_snippet(Some(&pending.text), 82)
        .unwrap_or_else(|| "Waiting for your answer".to_string());
    let min_height = if pending.options.is_empty() {
        PENDING_QUESTION_CARD_MIN_HEIGHT
    } else {
        PENDING_QUESTION_CARD_MIN_HEIGHT
            + crate::native_panel_core::PENDING_QUESTION_CARD_OPTIONS_EXTRA_HEIGHT
    };
    pending_like_card_height(
        &body,
        min_height,
        crate::native_panel_core::PENDING_QUESTION_CARD_FALLBACK_MAX_HEIGHT
            .max(PENDING_QUESTION_CARD_MAX_HEIGHT),
    )
}

fn prompt_assist_card_height(_session: &SessionSnapshotView) -> f64 {
    pending_like_card_height(
        "A command may be waiting for approval in the Codex terminal. Allow or deny it there.",
        crate::native_panel_core::PROMPT_ASSIST_CARD_MIN_HEIGHT,
        crate::native_panel_core::PROMPT_ASSIST_CARD_MAX_HEIGHT,
    )
}

fn completion_card_height(session: &SessionSnapshotView) -> f64 {
    resolve_completion_card_height(
        &completion_preview_text(session),
        default_chat_body_width(),
        default_panel_card_metric_constants(),
    )
}

fn pending_like_card_height(body: &str, min_height: f64, max_height: f64) -> f64 {
    resolve_pending_like_card_height(
        body,
        min_height,
        max_height,
        default_chat_body_width(),
        default_panel_card_metric_constants(),
    )
}

fn session_card_height(session: &SessionSnapshotView) -> f64 {
    if is_long_idle_session(session) || !session_has_visible_card_body(session) {
        return 58.0;
    }

    let prompt = session_prompt_preview(session);
    let reply = session_reply_preview(session);
    let content_height = resolve_session_card_content_height(SessionCardContentInput {
        prompt: prompt.as_deref(),
        reply: reply.as_deref(),
        has_tool: session_tool_preview(session).is_some(),
        default_body_width: default_chat_body_width(),
        metrics: default_panel_card_metric_constants(),
    });
    resolve_session_card_height(
        false,
        true,
        content_height,
        default_panel_card_metric_constants(),
    )
}

fn default_chat_body_width() -> f64 {
    resolve_card_chat_body_width(
        crate::native_panel_core::resolve_expanded_cards_width(
            DEFAULT_PANEL_CANVAS_WIDTH,
            crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
        ),
        default_panel_card_metric_constants(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_native_panel_presentation_model, native_panel_visual_display_mode_from_presentation,
        native_panel_visual_plan_input_from_presentation, resolve_native_panel_presentation,
        resolve_native_panel_presentation_metrics,
        resolve_native_panel_snapshot_render_plan_for_scene,
    };
    use crate::{
        native_panel_core::{
            PanelGeometryMetrics, PanelLayoutInput, PanelRect, resolve_panel_layout,
            resolve_panel_render_state,
        },
        native_panel_renderer::{
            descriptors::{
                NativePanelEdgeAction, NativePanelHostWindowState, NativePanelPointerRegionInput,
            },
            render_commands::resolve_native_panel_render_command_bundle,
            visual_plan::NativePanelVisualDisplayMode,
        },
        native_panel_scene::{PanelRuntimeRenderState, PanelSceneBuildInput, build_panel_scene},
    };

    fn snapshot(status: &str) -> echoisland_runtime::RuntimeSnapshot {
        echoisland_runtime::RuntimeSnapshot {
            status: status.to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        }
    }

    #[test]
    fn fallback_presentation_model_uses_scene_content() {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &snapshot("idle"),
            &PanelSceneBuildInput::default(),
        );

        let model = build_native_panel_presentation_model(&scene);

        assert_eq!(
            model.compact_bar.headline.text,
            scene.compact_bar.headline.text
        );
        assert_eq!(model.card_stack.cards.len(), scene.cards.len());
    }

    #[test]
    fn presentation_metrics_have_capped_body_height() {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState {
                expanded: true,
                ..Default::default()
            },
            &snapshot("running"),
            &PanelSceneBuildInput::default(),
        );

        let metrics = resolve_native_panel_presentation_metrics(&scene);

        assert!(metrics.expanded_content_height >= metrics.expanded_body_height);
    }

    #[test]
    fn resolved_presentation_keeps_render_bundle_and_presentation_in_sync() {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState {
                expanded: true,
                ..Default::default()
            },
            &snapshot("running"),
            &PanelSceneBuildInput::default(),
        );
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
        let runtime = PanelRuntimeRenderState::default();
        let render_state =
            resolve_panel_render_state(crate::native_panel_core::PanelRenderStateInput {
                shared_expanded_enabled: false,
                shell_visible: layout.shell_visible,
                separator_visibility: layout.separator_visibility,
                bar_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 0.0,
                cards_height: layout.cards_frame.height,
                status_surface_active: false,
                content_visibility: 1.0,
                transitioning: false,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            });

        let resolved = resolve_native_panel_presentation(
            layout,
            &scene,
            runtime,
            render_state,
            Some(NativePanelPointerRegionInput::default()),
        );

        assert_eq!(
            resolved.presentation.compact_bar.frame,
            resolved.bundle.compact_bar.frame
        );
        assert_eq!(
            resolved.presentation.compact_bar.headline.text,
            resolved.bundle.compact_bar.headline.text
        );
        assert_eq!(
            resolved.presentation.card_stack.cards.len(),
            resolved.bundle.card_stack.cards.len()
        );
    }

    #[test]
    fn visual_plan_action_button_frames_are_local_to_panel_canvas() {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState {
                expanded: true,
                ..Default::default()
            },
            &snapshot("running"),
            &PanelSceneBuildInput::default(),
        );
        let layout = resolve_panel_layout(PanelLayoutInput {
            screen_frame: PanelRect {
                x: 320.0,
                y: 120.0,
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
        let render_state =
            resolve_panel_render_state(crate::native_panel_core::PanelRenderStateInput {
                shared_expanded_enabled: false,
                shell_visible: layout.shell_visible,
                separator_visibility: layout.separator_visibility,
                bar_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 0.0,
                cards_height: layout.cards_frame.height,
                status_surface_active: false,
                content_visibility: 1.0,
                transitioning: false,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            });
        let resolved = resolve_native_panel_presentation(
            layout,
            &scene,
            PanelRuntimeRenderState::default(),
            render_state,
            Some(NativePanelPointerRegionInput::default()),
        );
        let window_state = NativePanelHostWindowState {
            frame: Some(resolved.presentation.panel_frame),
            visible: true,
            preferred_display_index: 0,
        };
        let visual_input = native_panel_visual_plan_input_from_presentation(
            window_state,
            NativePanelVisualDisplayMode::Expanded,
            Some(&resolved.presentation),
        );
        let settings_frame = visual_input
            .action_buttons
            .iter()
            .find(|button| button.action == NativePanelEdgeAction::Settings)
            .expect("settings button")
            .frame;
        let quit_frame = visual_input
            .action_buttons
            .iter()
            .find(|button| button.action == NativePanelEdgeAction::Quit)
            .expect("quit button")
            .frame;

        for frame in [settings_frame, quit_frame] {
            assert!(frame.x >= 0.0);
            assert!(frame.x + frame.width <= visual_input.content_frame.width);
            assert!(frame.y >= 0.0);
            assert!(frame.y + frame.height <= visual_input.content_frame.height);
        }
    }

    #[test]
    fn snapshot_render_plan_reuses_bundle_presentation_inputs() {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState {
                expanded: true,
                ..Default::default()
            },
            &snapshot("running"),
            &PanelSceneBuildInput::default(),
        );
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
        let render_state =
            resolve_panel_render_state(crate::native_panel_core::PanelRenderStateInput {
                shared_expanded_enabled: false,
                shell_visible: layout.shell_visible,
                separator_visibility: layout.separator_visibility,
                bar_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 0.0,
                cards_height: layout.cards_frame.height,
                status_surface_active: false,
                content_visibility: 1.0,
                transitioning: false,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            });
        let bundle = resolve_native_panel_render_command_bundle(
            layout,
            &scene,
            PanelRuntimeRenderState::default(),
            render_state,
            Some(NativePanelPointerRegionInput::default()),
        );

        let plan = resolve_native_panel_snapshot_render_plan_for_scene(
            scene.clone(),
            Some(bundle.clone()),
        );

        assert_eq!(
            plan.compact_bar_command(bundle.layout.pill_frame)
                .headline
                .text,
            bundle.compact_bar.headline.text
        );
        assert_eq!(
            plan.card_stack_command(bundle.layout.cards_frame, true)
                .cards
                .len(),
            bundle.card_stack.cards.len()
        );
    }

    #[test]
    fn visual_plan_input_reuses_shared_presentation_model() {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState {
                expanded: true,
                ..Default::default()
            },
            &snapshot("running"),
            &PanelSceneBuildInput::default(),
        );
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
        let render_state =
            resolve_panel_render_state(crate::native_panel_core::PanelRenderStateInput {
                shared_expanded_enabled: false,
                shell_visible: layout.shell_visible,
                separator_visibility: layout.separator_visibility,
                bar_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 0.0,
                cards_height: layout.cards_frame.height,
                status_surface_active: false,
                content_visibility: 1.0,
                transitioning: false,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            });
        let resolved = resolve_native_panel_presentation(
            layout,
            &scene,
            PanelRuntimeRenderState::default(),
            render_state,
            Some(NativePanelPointerRegionInput::default()),
        );
        let window_state = NativePanelHostWindowState {
            frame: Some(resolved.presentation.panel_frame),
            visible: true,
            preferred_display_index: 0,
        };

        let display_mode = native_panel_visual_display_mode_from_presentation(
            window_state,
            Some(&resolved.presentation),
        );
        let visual_input = native_panel_visual_plan_input_from_presentation(
            window_state,
            display_mode,
            Some(&resolved.presentation),
        );

        assert_eq!(display_mode, NativePanelVisualDisplayMode::Expanded);
        assert_eq!(
            visual_input.headline_text,
            resolved.presentation.compact_bar.headline.text
        );
        assert_eq!(
            visual_input.card_count,
            resolved.presentation.card_stack.cards.len()
        );
        assert_eq!(
            visual_input.action_buttons_visible,
            resolved.presentation.action_buttons.visible
        );
    }
}
