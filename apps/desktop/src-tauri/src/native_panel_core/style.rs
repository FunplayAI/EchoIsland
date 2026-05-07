use super::constants::{
    DEFAULT_COMPACT_PILL_WIDTH, DEFAULT_EXPANDED_PILL_WIDTH,
    PANEL_COMPACT_CORNER_MASK_MAX_PROGRESS, PANEL_EDGE_ACTIONS_MIN_SCALE,
    PANEL_HIGHLIGHT_MAX_ALPHA, PANEL_PILL_BORDER_MAX_WIDTH, PANEL_VISIBILITY_EPSILON,
};
use super::{PanelRect, transitions::ease_out_cubic};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ActionButtonVisibilitySpecInput {
    pub(crate) semantic_visible: bool,
    pub(crate) expanded_display_mode: bool,
    pub(crate) transition_visibility_progress: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ActionButtonVisibilitySpec {
    pub(crate) visible: bool,
    pub(crate) reserves_headline_space: bool,
    pub(crate) opacity: f64,
    pub(crate) retract_offset_y: f64,
    pub(crate) scale: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelStyleResolverInput {
    pub(crate) shell_visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
    pub(crate) compact_pill_radius: f64,
    pub(crate) panel_morph_pill_radius: f64,
    pub(crate) expanded_panel_radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelStyleResolved {
    pub(crate) expanded_hidden: bool,
    pub(crate) expanded_alpha: f64,
    pub(crate) separator_hidden: bool,
    pub(crate) separator_alpha: f64,
    pub(crate) cards_hidden: bool,
    pub(crate) highlight_hidden: bool,
    pub(crate) highlight_alpha: f64,
    pub(crate) actions_hidden: bool,
    pub(crate) action_alpha: f64,
    pub(crate) action_scale: f64,
    pub(crate) pill_corner_radius: f64,
    pub(crate) use_compact_corner_mask: bool,
    pub(crate) pill_border_width: f64,
    pub(crate) expanded_corner_radius: f64,
}

pub(crate) fn resolve_panel_style(input: PanelStyleResolverInput) -> PanelStyleResolved {
    let bar_progress = input.bar_progress.clamp(0.0, 1.0);
    let height_progress = input.height_progress.clamp(0.0, 1.0);
    let highlight_alpha = if input.headline_emphasized {
        lerp(0.0, PANEL_HIGHLIGHT_MAX_ALPHA, bar_progress)
    } else {
        0.0
    };
    let action_visibility =
        resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
            semantic_visible: input.edge_actions_visible,
            expanded_display_mode: input.shell_visible,
            transition_visibility_progress: bar_progress,
        });

    PanelStyleResolved {
        expanded_hidden: !input.shell_visible,
        expanded_alpha: if input.shell_visible { 1.0 } else { 0.0 },
        separator_hidden: input.separator_visibility <= PANEL_VISIBILITY_EPSILON,
        separator_alpha: input.separator_visibility,
        cards_hidden: input.shared_visible,
        highlight_hidden: highlight_alpha <= PANEL_VISIBILITY_EPSILON,
        highlight_alpha,
        actions_hidden: !action_visibility.visible,
        action_alpha: action_visibility.opacity,
        action_scale: action_visibility.scale,
        pill_corner_radius: lerp(
            input.compact_pill_radius,
            input.panel_morph_pill_radius,
            bar_progress,
        ),
        use_compact_corner_mask: bar_progress <= PANEL_COMPACT_CORNER_MASK_MAX_PROGRESS,
        pill_border_width: lerp(PANEL_PILL_BORDER_MAX_WIDTH, 0.0, bar_progress),
        expanded_corner_radius: lerp(
            input.compact_pill_radius,
            input.expanded_panel_radius,
            bar_progress.max(height_progress),
        ),
    }
}

pub(crate) fn resolve_action_button_visibility_spec(
    input: ActionButtonVisibilitySpecInput,
) -> ActionButtonVisibilitySpec {
    let eligible = input.semantic_visible && input.expanded_display_mode;
    let progress = input.transition_visibility_progress.clamp(0.0, 1.0);
    let opacity = if eligible {
        ease_out_cubic(progress)
    } else {
        0.0
    };
    let visible = eligible && opacity > PANEL_VISIBILITY_EPSILON;
    ActionButtonVisibilitySpec {
        visible,
        reserves_headline_space: eligible,
        opacity,
        retract_offset_y: lerp(-4.0, 0.0, opacity),
        scale: lerp(PANEL_EDGE_ACTIONS_MIN_SCALE, 1.0, opacity),
    }
}

pub(crate) fn action_button_transition_progress_from_compact_width(compact_width: f64) -> f64 {
    let width_delta = (DEFAULT_EXPANDED_PILL_WIDTH - DEFAULT_COMPACT_PILL_WIDTH).max(1.0);
    ((compact_width - DEFAULT_COMPACT_PILL_WIDTH) / width_delta).clamp(0.0, 1.0)
}

pub(crate) fn action_button_visual_frame_for_phase(
    frame: PanelRect,
    visibility: ActionButtonVisibilitySpec,
) -> PanelRect {
    PanelRect {
        y: frame.y + visibility.retract_offset_y,
        ..frame
    }
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}
