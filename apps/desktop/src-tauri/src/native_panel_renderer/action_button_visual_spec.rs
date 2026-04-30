use crate::native_panel_core::{
    DEFAULT_COMPACT_PILL_WIDTH, DEFAULT_EXPANDED_PILL_WIDTH, PanelRect, ease_out_cubic, lerp,
    resolve_compact_action_button_layout,
};

use super::{
    descriptors::NativePanelEdgeAction,
    visual_primitives::{NativePanelVisualColor, NativePanelVisualTextWeight},
};

const SETTINGS_ACTION_ICON_TEXT: &str = "\u{E713}";
const QUIT_ACTION_ICON_TEXT: &str = "⏻";
const ACTION_ICON_SIZE: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ActionButtonVisualSpecInput<'a> {
    pub(crate) visible: bool,
    pub(crate) compact_frame: PanelRect,
    pub(crate) buttons: &'a [(NativePanelEdgeAction, PanelRect)],
}

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
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActionButtonVisualSpec {
    pub(crate) action: NativePanelEdgeAction,
    pub(crate) frame: PanelRect,
    pub(crate) text: String,
    pub(crate) size: i32,
    pub(crate) weight: NativePanelVisualTextWeight,
    pub(crate) color: NativePanelVisualColor,
}

pub(crate) fn resolve_action_button_visual_specs(
    input: ActionButtonVisualSpecInput<'_>,
) -> Vec<ActionButtonVisualSpec> {
    if !input.visible {
        return Vec::new();
    }
    input
        .buttons
        .iter()
        .map(|(action, frame)| action_button_visual_spec(*action, *frame, input.compact_frame))
        .collect()
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
    let visible = eligible && opacity > 0.01;
    ActionButtonVisibilitySpec {
        visible,
        reserves_headline_space: eligible,
        opacity,
        retract_offset_y: lerp(-4.0, 0.0, opacity),
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

pub(crate) fn action_button_visual_color_for_phase(
    color: NativePanelVisualColor,
    visibility: ActionButtonVisibilitySpec,
) -> NativePanelVisualColor {
    blend_action_button_color(
        NativePanelVisualColor::rgb(12, 12, 15),
        color,
        visibility.opacity,
    )
}

fn action_button_visual_spec(
    action: NativePanelEdgeAction,
    frame: PanelRect,
    compact_frame: PanelRect,
) -> ActionButtonVisualSpec {
    let frame = action_button_icon_frame(action, frame, compact_frame);
    let (text, weight, color) = match action {
        NativePanelEdgeAction::Settings => (
            SETTINGS_ACTION_ICON_TEXT,
            NativePanelVisualTextWeight::Normal,
            NativePanelVisualColor::rgb(245, 247, 252),
        ),
        NativePanelEdgeAction::Quit => (
            QUIT_ACTION_ICON_TEXT,
            NativePanelVisualTextWeight::Bold,
            NativePanelVisualColor::rgb(255, 82, 82),
        ),
    };
    ActionButtonVisualSpec {
        action,
        frame,
        text: text.to_string(),
        size: ACTION_ICON_SIZE,
        weight,
        color,
    }
}

fn blend_action_button_color(
    from: NativePanelVisualColor,
    to: NativePanelVisualColor,
    progress: f64,
) -> NativePanelVisualColor {
    let progress = progress.clamp(0.0, 1.0);
    NativePanelVisualColor::rgb(
        lerp(from.r as f64, to.r as f64, progress).round() as u8,
        lerp(from.g as f64, to.g as f64, progress).round() as u8,
        lerp(from.b as f64, to.b as f64, progress).round() as u8,
    )
}

fn action_button_icon_frame(
    action: NativePanelEdgeAction,
    _frame: PanelRect,
    compact_frame: PanelRect,
) -> PanelRect {
    let local = resolve_compact_action_button_layout(compact_frame);
    match action {
        NativePanelEdgeAction::Settings => local.settings,
        NativePanelEdgeAction::Quit => local.quit,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        native_panel_core::{PanelRect, resolve_compact_action_button_layout},
        native_panel_renderer::{
            descriptors::NativePanelEdgeAction,
            visual_primitives::{NativePanelVisualColor, NativePanelVisualTextWeight},
        },
    };

    use super::{
        ActionButtonVisibilitySpecInput, ActionButtonVisualSpecInput,
        action_button_transition_progress_from_compact_width,
        resolve_action_button_visibility_spec, resolve_action_button_visual_specs,
    };

    #[test]
    fn action_button_visual_spec_resolves_icons_from_shared_compact_layout() {
        let compact_frame = PanelRect {
            x: 0.0,
            y: 0.0,
            width: 226.0,
            height: 36.0,
        };
        let wide_hit_frame = PanelRect {
            x: 0.0,
            y: 0.0,
            width: 64.0,
            height: 36.0,
        };

        let specs = resolve_action_button_visual_specs(ActionButtonVisualSpecInput {
            visible: true,
            compact_frame,
            buttons: &[
                (NativePanelEdgeAction::Settings, wide_hit_frame),
                (NativePanelEdgeAction::Quit, wide_hit_frame),
            ],
        });

        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].action, NativePanelEdgeAction::Settings);
        assert_eq!(specs[0].text, "\u{E713}");
        assert_eq!(specs[0].size, 16);
        assert_eq!(specs[0].weight, NativePanelVisualTextWeight::Normal);
        assert_eq!(specs[0].color, NativePanelVisualColor::rgb(245, 247, 252));
        assert_eq!(specs[1].action, NativePanelEdgeAction::Quit);
        assert_eq!(specs[1].size, 16);
        assert_eq!(specs[1].weight, NativePanelVisualTextWeight::Bold);
        assert_eq!(specs[1].color, NativePanelVisualColor::rgb(255, 82, 82));
        assert!(specs[0].frame.width <= 30.0);
        assert!(specs[1].frame.x > specs[0].frame.x);
    }

    #[test]
    fn action_button_visual_spec_ignores_narrow_hit_frame_for_icon_position() {
        let compact_frame = PanelRect {
            x: 0.0,
            y: 0.0,
            width: 226.0,
            height: 36.0,
        };
        let drifted_hit_frame = PanelRect {
            x: 132.0,
            y: 3.0,
            width: 26.0,
            height: 26.0,
        };

        let specs = resolve_action_button_visual_specs(ActionButtonVisualSpecInput {
            visible: true,
            compact_frame,
            buttons: &[(NativePanelEdgeAction::Settings, drifted_hit_frame)],
        });

        assert_eq!(specs.len(), 1);
        assert_eq!(
            specs[0].frame,
            resolve_compact_action_button_layout(compact_frame).settings
        );
    }

    #[test]
    fn action_button_visual_spec_returns_no_icons_when_hidden() {
        let specs = resolve_action_button_visual_specs(ActionButtonVisualSpecInput {
            visible: false,
            compact_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 226.0,
                height: 36.0,
            },
            buttons: &[(
                NativePanelEdgeAction::Settings,
                PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 26.0,
                    height: 26.0,
                },
            )],
        });

        assert!(specs.is_empty());
    }

    #[test]
    fn action_button_visibility_spec_only_shows_and_reserves_space_when_expanded() {
        assert_eq!(
            resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
                semantic_visible: true,
                expanded_display_mode: true,
                transition_visibility_progress: 1.0,
            }),
            super::ActionButtonVisibilitySpec {
                visible: true,
                reserves_headline_space: true,
                opacity: 1.0,
                retract_offset_y: 0.0,
            }
        );
        assert_eq!(
            resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
                semantic_visible: true,
                expanded_display_mode: false,
                transition_visibility_progress: 1.0,
            }),
            super::ActionButtonVisibilitySpec {
                visible: false,
                reserves_headline_space: false,
                opacity: 0.0,
                retract_offset_y: -4.0,
            }
        );
        assert_eq!(
            resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
                semantic_visible: false,
                expanded_display_mode: true,
                transition_visibility_progress: 1.0,
            }),
            super::ActionButtonVisibilitySpec {
                visible: false,
                reserves_headline_space: false,
                opacity: 0.0,
                retract_offset_y: -4.0,
            }
        );
    }

    #[test]
    fn action_button_visibility_spec_exposes_shared_transition_phase() {
        let hidden = resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
            semantic_visible: true,
            expanded_display_mode: true,
            transition_visibility_progress: 0.0,
        });
        let mid = resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
            semantic_visible: true,
            expanded_display_mode: true,
            transition_visibility_progress: 0.5,
        });

        assert!(!hidden.visible);
        assert!(hidden.reserves_headline_space);
        assert_eq!(hidden.opacity, 0.0);
        assert_eq!(hidden.retract_offset_y, -4.0);
        assert!(mid.visible);
        assert!(mid.reserves_headline_space);
        assert!(mid.opacity > 0.0 && mid.opacity < 1.0);
        assert!(mid.retract_offset_y > -4.0 && mid.retract_offset_y < 0.0);
    }

    #[test]
    fn action_button_transition_progress_follows_compact_width() {
        assert_eq!(
            action_button_transition_progress_from_compact_width(
                crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH
            ),
            0.0
        );
        assert_eq!(
            action_button_transition_progress_from_compact_width(
                crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH
            ),
            1.0
        );
        let mid = action_button_transition_progress_from_compact_width(
            (crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH
                + crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH)
                / 2.0,
        );
        assert!((mid - 0.5).abs() < 0.0001);
    }
}
