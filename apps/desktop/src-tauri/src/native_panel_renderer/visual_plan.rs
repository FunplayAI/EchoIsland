use crate::{
    native_panel_core::{
        ACTIVE_COUNT_SCROLL_TRAVEL, ACTIVE_COUNT_TEXT_OFFSET_X, ACTIVE_COUNT_TEXT_WIDTH,
        ActiveCountMarqueeInput, CARD_RADIUS, COMPACT_PILL_RADIUS, CompactBarContentLayout,
        CompactBarContentLayoutInput, EXPANDED_CARD_GAP, EXPANDED_CARD_OVERHANG, ExpandedSurface,
        PanelPoint, PanelRect, compact_title, default_panel_card_metric_constants, ease_out_cubic,
        lerp, resolve_active_count_marquee_frame, resolve_compact_action_button_layout,
        resolve_compact_bar_content_layout, resolve_estimated_chat_body_height,
        resolve_estimated_text_width, resolve_next_stacked_card_frame,
    },
    native_panel_scene::{SceneCard, SceneMascotPose},
};

use super::action_button_visual_spec::{
    ActionButtonVisibilitySpec, ActionButtonVisibilitySpecInput, ActionButtonVisualSpec,
    ActionButtonVisualSpecInput, action_button_transition_progress_from_compact_width,
    action_button_visual_color_for_phase, action_button_visual_frame_for_phase,
    resolve_action_button_visibility_spec, resolve_action_button_visual_specs,
};
use super::card_visual_spec::{
    CardVisualBadgeRole, CardVisualBadgeSpec, CardVisualBodyRole, CardVisualBodySpec,
    CardVisualColorSpec, CardVisualRowSpec, CardVisualSpec, CardVisualStyle,
    card_visual_action_hint_layout, card_visual_badge_layout, card_visual_body_layout,
    card_visual_body_line_paint_spec, card_visual_content_layout,
    card_visual_content_transition_frame, card_visual_header_text_paint_spec,
    card_visual_settings_row_layout, card_visual_shell_border_color, card_visual_shell_fill_color,
    card_visual_shell_reveal_frame, card_visual_spec_from_scene_card_with_height,
    card_visual_stack_reveal_frame, card_visual_tool_pill_layout,
};
use super::descriptors::{NativePanelEdgeAction, NativePanelHostWindowState};
use super::mascot_visual_spec::{
    MascotCompletionBadgeVisualSpec, MascotMessageBubbleVisualSpec, MascotRoundRectVisualSpec,
    MascotTextVisualSpec, MascotVisualSpec, MascotVisualSpecInput, resolve_mascot_visual_spec,
};
use super::visual_primitives::{
    NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
    NativePanelVisualShoulderSide, NativePanelVisualTextAlignment, NativePanelVisualTextWeight,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelVisualDisplayMode {
    Hidden,
    Compact,
    Expanded,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelVisualPlanInput {
    pub(crate) window_state: NativePanelHostWindowState,
    pub(crate) display_mode: NativePanelVisualDisplayMode,
    pub(crate) surface: ExpandedSurface,
    pub(crate) panel_frame: PanelRect,
    pub(crate) compact_bar_frame: PanelRect,
    pub(crate) left_shoulder_frame: PanelRect,
    pub(crate) right_shoulder_frame: PanelRect,
    pub(crate) shoulder_progress: f64,
    pub(crate) content_frame: PanelRect,
    pub(crate) card_stack_frame: PanelRect,
    pub(crate) card_stack_content_height: f64,
    pub(crate) shell_frame: PanelRect,
    pub(crate) headline_text: String,
    pub(crate) headline_emphasized: bool,
    pub(crate) active_count: String,
    pub(crate) active_count_elapsed_ms: u128,
    pub(crate) total_count: String,
    pub(crate) separator_visibility: f64,
    pub(crate) cards_visible: bool,
    pub(crate) card_count: usize,
    pub(crate) cards: Vec<NativePanelVisualCardInput>,
    pub(crate) glow_visible: bool,
    pub(crate) action_buttons_visible: bool,
    pub(crate) action_buttons: Vec<NativePanelVisualActionButtonInput>,
    pub(crate) completion_count: usize,
    pub(crate) mascot_elapsed_ms: u128,
    pub(crate) mascot_pose: SceneMascotPose,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelVisualCardInput {
    pub(crate) style: NativePanelVisualCardStyle,
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) badge: Option<NativePanelVisualCardBadgeInput>,
    pub(crate) source_badge: Option<NativePanelVisualCardBadgeInput>,
    pub(crate) body_prefix: Option<String>,
    pub(crate) body_lines: Vec<NativePanelVisualCardBodyLineInput>,
    pub(crate) action_hint: Option<String>,
    pub(crate) rows: Vec<NativePanelVisualCardRowInput>,
    pub(crate) height: f64,
    pub(crate) collapsed_height: f64,
    pub(crate) compact: bool,
    pub(crate) removing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelVisualCardStyle {
    Default,
    Pending,
    PendingApproval,
    PendingQuestion,
    PromptAssist,
    Completion,
    Settings,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelVisualCardBodyRole {
    Assistant,
    User,
    Tool,
    Plain,
    ActionHint,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelVisualCardBodyLineInput {
    pub(crate) role: NativePanelVisualCardBodyRole,
    pub(crate) prefix: Option<String>,
    pub(crate) text: String,
    pub(crate) max_lines: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelVisualCardBadgeInput {
    pub(crate) text: String,
    pub(crate) emphasized: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelVisualCardRowInput {
    pub(crate) title: String,
    pub(crate) value: String,
    pub(crate) active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelVisualActionButtonInput {
    pub(crate) action: NativePanelEdgeAction,
    pub(crate) frame: PanelRect,
}

pub(crate) fn resolve_native_panel_visual_plan(
    input: &NativePanelVisualPlanInput,
) -> NativePanelVisualPlan {
    if input.display_mode == NativePanelVisualDisplayMode::Hidden || !input.window_state.visible {
        return NativePanelVisualPlan {
            hidden: true,
            primitives: Vec::new(),
        };
    }

    let mut primitives = Vec::new();
    let panel_frame = visual_panel_frame(input);
    let compact_frame = non_zero_rect(input.compact_bar_frame).unwrap_or(panel_frame);
    let shell_frame = non_zero_rect(input.shell_frame).unwrap_or(panel_frame);
    let content_frame = non_zero_rect(input.content_frame).unwrap_or(panel_frame);
    let action_button_visibility =
        resolve_action_button_visibility_spec(ActionButtonVisibilitySpecInput {
            semantic_visible: input.action_buttons_visible,
            expanded_display_mode: input.display_mode == NativePanelVisualDisplayMode::Expanded,
            transition_visibility_progress: action_button_transition_progress_from_compact_width(
                compact_frame.width,
            ),
        });

    if input.display_mode == NativePanelVisualDisplayMode::Compact {
        push_compact_island_background(&mut primitives, input, compact_frame);
    }

    if input.display_mode == NativePanelVisualDisplayMode::Expanded {
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: shell_frame,
            radius: crate::native_panel_core::EXPANDED_PANEL_RADIUS,
            color: NativePanelVisualColor::rgb(12, 12, 15),
        });

        if input.separator_visibility > 0.01 {
            primitives.push(NativePanelVisualPrimitive::Rect {
                frame: PanelRect {
                    x: shell_frame.x + 20.0,
                    y: compact_frame.y + compact_frame.height + 8.0,
                    width: (shell_frame.width - 40.0).max(0.0),
                    height: 1.0,
                },
                color: NativePanelVisualColor::rgb(62, 62, 70),
            });
        }

        push_expanded_card_shells(&mut primitives, input, shell_frame);
    }

    let compact_content = compact_content_layout(
        compact_frame,
        action_button_visibility.reserves_headline_space,
    );
    let headline_text = fit_text_to_width(
        &input.headline_text,
        compact_content.headline_width,
        13.0,
        1,
    );
    let headline_width =
        resolve_estimated_text_width(&headline_text, 13.0).min(compact_content.headline_width);
    let headline_center_x = compact_frame.x + compact_content.headline_center_x;
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: PanelPoint {
            x: headline_center_x - headline_width / 2.0,
            y: compact_frame.y + compact_headline_y(compact_frame.height),
        },
        max_width: headline_width,
        text: headline_text,
        color: if input.headline_emphasized {
            NativePanelVisualColor::rgb(255, 255, 255)
        } else {
            NativePanelVisualColor::rgb(230, 235, 245)
        },
        size: 13,
        weight: NativePanelVisualTextWeight::Semibold,
        alignment: NativePanelVisualTextAlignment::Center,
    });

    if !input.active_count.is_empty() || !input.total_count.is_empty() {
        let active_count_marquee = resolve_active_count_marquee_frame(ActiveCountMarqueeInput {
            text: &input.active_count,
            elapsed_ms: input.active_count_elapsed_ms,
        });
        let active_count_y = compact_frame.y + compact_digit_y(compact_frame.height);
        let active_count_x =
            compact_frame.x + compact_content.active_x + ACTIVE_COUNT_TEXT_OFFSET_X;
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: PanelPoint {
                x: active_count_x,
                y: active_count_y - active_count_marquee.scroll_offset,
            },
            max_width: ACTIVE_COUNT_TEXT_WIDTH,
            text: active_count_marquee.current.clone(),
            color: if input.active_count.parse::<usize>().unwrap_or_default() > 0 {
                NativePanelVisualColor::rgb(102, 222, 145)
            } else {
                NativePanelVisualColor::rgb(156, 166, 184)
            },
            size: 15,
            weight: NativePanelVisualTextWeight::Semibold,
            alignment: NativePanelVisualTextAlignment::Right,
        });
        if active_count_marquee.show_next {
            primitives.push(NativePanelVisualPrimitive::Text {
                origin: PanelPoint {
                    x: active_count_x,
                    y: active_count_y + ACTIVE_COUNT_SCROLL_TRAVEL
                        - active_count_marquee.scroll_offset,
                },
                max_width: ACTIVE_COUNT_TEXT_WIDTH,
                text: active_count_marquee.next.clone(),
                color: NativePanelVisualColor::rgb(102, 222, 145),
                size: 15,
                weight: NativePanelVisualTextWeight::Semibold,
                alignment: NativePanelVisualTextAlignment::Right,
            });
        }
        if !input.total_count.is_empty() {
            primitives.push(NativePanelVisualPrimitive::Text {
                origin: PanelPoint {
                    x: compact_frame.x + compact_content.slash_x,
                    y: compact_frame.y + compact_digit_y(compact_frame.height),
                },
                max_width: 10.0,
                text: "/".to_string(),
                color: NativePanelVisualColor::rgb(245, 247, 252),
                size: 15,
                weight: NativePanelVisualTextWeight::Semibold,
                alignment: NativePanelVisualTextAlignment::Center,
            });
            primitives.push(NativePanelVisualPrimitive::Text {
                origin: PanelPoint {
                    x: compact_frame.x + compact_content.total_x,
                    y: compact_frame.y + compact_digit_y(compact_frame.height),
                },
                max_width: 24.0,
                text: input.total_count.clone(),
                color: NativePanelVisualColor::rgb(245, 247, 252),
                size: 15,
                weight: NativePanelVisualTextWeight::Semibold,
                alignment: NativePanelVisualTextAlignment::Left,
            });
        }
    }

    let _ = (content_frame, input.cards_visible);

    if action_button_visibility.visible {
        let button_frames = input
            .action_buttons
            .iter()
            .map(|button| (button.action, button.frame))
            .collect::<Vec<_>>();
        for spec in resolve_action_button_visual_specs(ActionButtonVisualSpecInput {
            visible: true,
            compact_frame,
            buttons: &button_frames,
        }) {
            push_action_button_icon(&mut primitives, &spec, action_button_visibility);
        }
    }

    let mascot_completion_count = if input.display_mode == NativePanelVisualDisplayMode::Compact
        && input.mascot_pose == SceneMascotPose::Complete
    {
        input.completion_count
    } else {
        0
    };
    let mascot_spec = resolve_mascot_visual_spec(MascotVisualSpecInput {
        center: PanelPoint {
            x: compact_frame.x + compact_content.mascot_center_x,
            y: compact_frame.y + compact_frame.height / 2.0,
        },
        radius: 11.0,
        pose: input.mascot_pose,
        completion_count: mascot_completion_count,
        elapsed_ms: input.mascot_elapsed_ms,
    });
    push_mascot_primitives(&mut primitives, &mascot_spec);

    NativePanelVisualPlan {
        hidden: false,
        primitives,
    }
}

pub(crate) fn native_panel_visual_card_input_from_scene_card(
    card: &SceneCard,
) -> NativePanelVisualCardInput {
    native_panel_visual_card_input_from_scene_card_with_height(card, 72.0)
}

pub(crate) fn native_panel_visual_card_input_from_scene_card_with_height(
    card: &SceneCard,
    height: f64,
) -> NativePanelVisualCardInput {
    visual_card_input_from_spec(card_visual_spec_from_scene_card_with_height(card, height))
}

fn visual_card_input_from_spec(spec: CardVisualSpec) -> NativePanelVisualCardInput {
    NativePanelVisualCardInput {
        style: visual_card_style_from_spec(spec.style),
        title: spec.title,
        subtitle: spec.subtitle,
        body: None,
        badge: visual_card_badge_from_spec(&spec.badges, CardVisualBadgeRole::Status),
        source_badge: visual_card_badge_from_spec(&spec.badges, CardVisualBadgeRole::Source),
        body_prefix: None,
        body_lines: spec
            .body
            .iter()
            .map(visual_card_body_line_from_spec)
            .collect(),
        action_hint: spec.action_hint,
        rows: spec.rows.iter().map(visual_card_row_from_spec).collect(),
        height: spec.height,
        collapsed_height: spec.animation.collapsed_height,
        compact: spec.compact,
        removing: spec.removing,
    }
}

fn visual_card_body_line_from_spec(
    line: &CardVisualBodySpec,
) -> NativePanelVisualCardBodyLineInput {
    NativePanelVisualCardBodyLineInput {
        role: visual_card_body_role_from_spec(line.role),
        prefix: line.prefix.clone(),
        text: line.text.clone(),
        max_lines: line.max_lines,
    }
}

fn visual_card_body_role_from_spec(role: CardVisualBodyRole) -> NativePanelVisualCardBodyRole {
    match role {
        CardVisualBodyRole::Assistant => NativePanelVisualCardBodyRole::Assistant,
        CardVisualBodyRole::User => NativePanelVisualCardBodyRole::User,
        CardVisualBodyRole::Tool => NativePanelVisualCardBodyRole::Tool,
        CardVisualBodyRole::Plain => NativePanelVisualCardBodyRole::Plain,
        CardVisualBodyRole::ActionHint => NativePanelVisualCardBodyRole::ActionHint,
    }
}

fn card_visual_body_role_from_visual_role(
    role: NativePanelVisualCardBodyRole,
) -> CardVisualBodyRole {
    match role {
        NativePanelVisualCardBodyRole::Assistant => CardVisualBodyRole::Assistant,
        NativePanelVisualCardBodyRole::User => CardVisualBodyRole::User,
        NativePanelVisualCardBodyRole::Tool => CardVisualBodyRole::Tool,
        NativePanelVisualCardBodyRole::Plain => CardVisualBodyRole::Plain,
        NativePanelVisualCardBodyRole::ActionHint => CardVisualBodyRole::ActionHint,
    }
}

fn visual_card_style_from_spec(style: CardVisualStyle) -> NativePanelVisualCardStyle {
    match style {
        CardVisualStyle::Default => NativePanelVisualCardStyle::Default,
        CardVisualStyle::Pending => NativePanelVisualCardStyle::Pending,
        CardVisualStyle::PendingApproval => NativePanelVisualCardStyle::PendingApproval,
        CardVisualStyle::PendingQuestion => NativePanelVisualCardStyle::PendingQuestion,
        CardVisualStyle::PromptAssist => NativePanelVisualCardStyle::PromptAssist,
        CardVisualStyle::Completion => NativePanelVisualCardStyle::Completion,
        CardVisualStyle::Settings => NativePanelVisualCardStyle::Settings,
        CardVisualStyle::Empty => NativePanelVisualCardStyle::Empty,
    }
}

fn visual_card_badge_from_spec(
    badges: &[CardVisualBadgeSpec],
    role: CardVisualBadgeRole,
) -> Option<NativePanelVisualCardBadgeInput> {
    badges
        .iter()
        .find(|badge| badge.role == role)
        .map(|badge| NativePanelVisualCardBadgeInput {
            text: badge.text.clone(),
            emphasized: badge.emphasized,
        })
}

fn visual_card_row_from_spec(row: &CardVisualRowSpec) -> NativePanelVisualCardRowInput {
    NativePanelVisualCardRowInput {
        title: row.title.clone(),
        value: row.value.clone(),
        active: row.active,
    }
}

fn push_action_button_icon(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    spec: &ActionButtonVisualSpec,
    visibility: ActionButtonVisibilitySpec,
) {
    let frame = action_button_visual_frame_for_phase(spec.frame, visibility);
    let text_height = visual_text_box_height(&spec.text, spec.size);
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: PanelPoint {
            x: frame.x,
            y: frame.y + ((frame.height - text_height) / 2.0).round() - 1.5,
        },
        max_width: frame.width,
        text: spec.text.clone(),
        color: action_button_visual_color_for_phase(spec.color, visibility),
        size: spec.size,
        weight: spec.weight,
        alignment: NativePanelVisualTextAlignment::Center,
    });
}

fn push_mascot_primitives(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    spec: &MascotVisualSpec,
) {
    primitives.push(NativePanelVisualPrimitive::MascotDot {
        center: spec.body.center,
        radius: spec.body.radius,
        scale_x: spec.body.scale_x,
        scale_y: spec.body.scale_y,
        pose: spec.pose,
    });
    if let Some(message_bubble) = &spec.message_bubble {
        push_mascot_message_bubble(primitives, message_bubble);
    }
    if let Some(sleep_label) = &spec.sleep_label {
        push_mascot_text(primitives, sleep_label);
    }
    for eye in &spec.eyes {
        primitives.push(NativePanelVisualPrimitive::Ellipse {
            frame: eye.frame,
            color: eye.color,
        });
    }
    push_mascot_round_rect(primitives, spec.mouth);
    if let Some(badge) = &spec.completion_badge {
        push_mascot_completion_badge(primitives, badge);
    }
}

fn push_mascot_message_bubble(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    bubble: &MascotMessageBubbleVisualSpec,
) {
    push_mascot_round_rect(primitives, bubble.bubble);
    for dot in &bubble.dots {
        primitives.push(NativePanelVisualPrimitive::Ellipse {
            frame: dot.frame,
            color: dot.color,
        });
    }
}

fn push_mascot_completion_badge(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    badge: &MascotCompletionBadgeVisualSpec,
) {
    push_mascot_round_rect(primitives, badge.outline);
    push_mascot_round_rect(primitives, badge.fill);
    push_mascot_text(primitives, &badge.label);
}

fn push_mascot_round_rect(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    spec: MascotRoundRectVisualSpec,
) {
    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: spec.frame,
        radius: spec.radius,
        color: spec.color,
    });
}

fn push_mascot_text(primitives: &mut Vec<NativePanelVisualPrimitive>, spec: &MascotTextVisualSpec) {
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: spec.origin,
        max_width: spec.max_width,
        text: spec.text.clone(),
        color: spec.color,
        size: spec.size,
        weight: spec.weight,
        alignment: NativePanelVisualTextAlignment::Center,
    });
}

fn push_text(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    x: f64,
    y: f64,
    max_width: f64,
    text: String,
    color: NativePanelVisualColor,
    size: i32,
) {
    push_text_with_style(
        primitives,
        x,
        y,
        max_width,
        text,
        color,
        size,
        NativePanelVisualTextWeight::Normal,
        NativePanelVisualTextAlignment::Left,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_text_with_style(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    x: f64,
    y: f64,
    max_width: f64,
    text: String,
    color: NativePanelVisualColor,
    size: i32,
    weight: NativePanelVisualTextWeight,
    alignment: NativePanelVisualTextAlignment,
) {
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: PanelPoint { x, y },
        max_width,
        text,
        color,
        size,
        weight,
        alignment,
    });
}

fn rect_center(rect: PanelRect) -> PanelPoint {
    PanelPoint {
        x: rect.x + rect.width / 2.0,
        y: rect.y + rect.height / 2.0,
    }
}

fn centered_rect(center: PanelPoint, radius_x: f64, radius_y: f64) -> PanelRect {
    PanelRect {
        x: center.x - radius_x,
        y: center.y - radius_y,
        width: radius_x * 2.0,
        height: radius_y * 2.0,
    }
}

fn point_on_circle(center: PanelPoint, radius: f64, degrees: f64) -> PanelPoint {
    let radians = degrees.to_radians();
    PanelPoint {
        x: center.x + radius * radians.cos(),
        y: center.y + radius * radians.sin(),
    }
}

fn visual_panel_frame(input: &NativePanelVisualPlanInput) -> PanelRect {
    non_zero_rect(input.content_frame)
        .or_else(|| {
            input.window_state.frame.map(|frame| PanelRect {
                x: 0.0,
                y: 0.0,
                width: frame.width,
                height: frame.height,
            })
        })
        .unwrap_or(input.panel_frame)
}

fn compact_content_layout(
    compact_frame: PanelRect,
    actions_active: bool,
) -> CompactBarContentLayout {
    let mut layout = resolve_compact_bar_content_layout(CompactBarContentLayoutInput {
        bar_width: compact_frame.width,
        bar_height: compact_frame.height,
    });
    if actions_active {
        // Compute the headline safe area against the STEADY expanded bar width.
        // Using compact_frame.width here makes the headline_width grow as the
        // panel widens during the open animation (and shrink as it narrows
        // during close), which causes fit_text_to_width to ellipsize/truncate
        // the headline character by character — a "typewriter" effect we don't
        // want. Action buttons only become active in expanded mode, so the
        // settings / quit positions for the safe-area calc are stable.
        let action_layout = resolve_compact_action_button_layout(PanelRect {
            x: 0.0,
            y: 0.0,
            width: crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
            height: compact_frame.height,
        });
        let side_gap = 4.0;
        let safe_left = action_layout.settings.x + action_layout.settings.width + side_gap;
        let safe_right = action_layout.quit.x - side_gap;
        let center_x = crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH / 2.0;
        let centered_safe_width = ((center_x - safe_left).min(safe_right - center_x) * 2.0)
            .clamp(0.0, layout.headline_width);
        if centered_safe_width > 0.0 {
            layout.headline_width = centered_safe_width;
            layout.headline_x = (compact_frame.width / 2.0) - centered_safe_width / 2.0;
            layout.headline_center_x = compact_frame.width / 2.0;
        }
    }
    layout
}

fn compact_headline_y(bar_height: f64) -> f64 {
    ((bar_height - 24.0) / 2.0).round() - 1.5
}

fn compact_digit_y(bar_height: f64) -> f64 {
    ((bar_height - crate::native_panel_core::ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 1.5
}

fn push_compact_island_background(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    input: &NativePanelVisualPlanInput,
    compact_frame: PanelRect,
) {
    push_compact_shoulder_primitive(
        primitives,
        input.left_shoulder_frame,
        NativePanelVisualShoulderSide::Left,
        input.shoulder_progress,
    );
    push_compact_shoulder_primitive(
        primitives,
        input.right_shoulder_frame,
        NativePanelVisualShoulderSide::Right,
        input.shoulder_progress,
    );
    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: compact_frame,
        radius: COMPACT_PILL_RADIUS,
        color: NativePanelVisualColor::rgb(12, 12, 15),
    });
}

fn push_expanded_card_shells(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    input: &NativePanelVisualPlanInput,
    shell_frame: PanelRect,
) {
    if !input.cards_visible || input.cards.is_empty() || input.separator_visibility <= 0.01 {
        return;
    }

    let mut cursor_y = input.card_stack_content_height;
    let stack_overflow_y =
        (input.card_stack_content_height - input.card_stack_frame.height).max(0.0);
    let clip_bounds = PanelRect {
        x: shell_frame.x + input.card_stack_frame.x,
        y: shell_frame.y + input.card_stack_frame.y,
        width: input.card_stack_frame.width,
        height: input.card_stack_frame.height,
    };
    for (index, card) in input.cards.iter().enumerate() {
        let single_empty_card =
            input.cards.len() == 1 && card.style == NativePanelVisualCardStyle::Empty;
        let card_height = if single_empty_card {
            card.height
                .min(cursor_y.max(input.card_stack_frame.height))
                .max(card.collapsed_height)
        } else {
            card.height
        };
        let Some(frame) = resolve_next_stacked_card_frame(
            &mut cursor_y,
            index > 0,
            card_height,
            input.card_stack_frame.width,
            EXPANDED_CARD_GAP,
            EXPANDED_CARD_OVERHANG,
        ) else {
            break;
        };
        let stable_card_frame = PanelRect {
            x: shell_frame.x + input.card_stack_frame.x + frame.x,
            y: shell_frame.y + input.card_stack_frame.y + frame.y - stack_overflow_y,
            width: frame.width,
            height: frame.height,
        };
        let stable_visible_height =
            clip_rect_vertically(stable_card_frame, clip_bounds).map(|frame| frame.height);
        if stable_visible_height.is_none_or(|height| height <= 6.0) && !single_empty_card {
            continue;
        }
        let phase =
            card_visual_stack_reveal_frame(input.separator_visibility, input.cards.len(), index)
                .card_phase;
        if phase <= 0.001 {
            continue;
        }
        let expanded_frame = PanelRect {
            x: shell_frame.x + input.card_stack_frame.x + frame.x,
            y: shell_frame.y + input.card_stack_frame.y + frame.y - stack_overflow_y,
            width: frame.width,
            height: frame.height,
        };
        let unclipped_card_frame =
            card_visual_shell_reveal_frame(expanded_frame, card.collapsed_height, phase);
        let Some(card_frame) = clip_rect_vertically(unclipped_card_frame, clip_bounds) else {
            continue;
        };
        if card_frame.height <= 6.0 {
            continue;
        }

        let content_visible =
            single_empty_card || card_frame.height >= card.collapsed_height.min(48.0);
        push_card_shell(primitives, card, card_frame);
        if content_visible {
            let content_layout_frame = if single_empty_card {
                card_frame
            } else {
                unclipped_card_frame
            };
            push_expanded_card_content(primitives, card, content_layout_frame, card_frame, phase);
        }
    }
}

fn push_card_shell(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    card: &NativePanelVisualCardInput,
    frame: PanelRect,
) {
    let radius = CARD_RADIUS.min(frame.height / 2.0);
    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame,
        radius,
        color: card_shell_border_color(card.style),
    });

    let inner = inset_rect(frame, 1.0);
    if inner.width <= 0.0 || inner.height <= 0.0 {
        return;
    }
    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: inner,
        radius: (radius - 1.0).max(0.0).min(inner.height / 2.0),
        color: card_shell_fill_color(card.style),
    });
}

fn card_shell_border_color(style: NativePanelVisualCardStyle) -> NativePanelVisualColor {
    visual_color_from_card_spec(card_visual_shell_border_color(
        card_visual_style_from_visual_style(style),
    ))
}

fn card_shell_fill_color(style: NativePanelVisualCardStyle) -> NativePanelVisualColor {
    visual_color_from_card_spec(card_visual_shell_fill_color(
        card_visual_style_from_visual_style(style),
    ))
}

fn card_visual_style_from_visual_style(style: NativePanelVisualCardStyle) -> CardVisualStyle {
    match style {
        NativePanelVisualCardStyle::Default => CardVisualStyle::Default,
        NativePanelVisualCardStyle::Pending => CardVisualStyle::Pending,
        NativePanelVisualCardStyle::PendingApproval => CardVisualStyle::PendingApproval,
        NativePanelVisualCardStyle::PendingQuestion => CardVisualStyle::PendingQuestion,
        NativePanelVisualCardStyle::PromptAssist => CardVisualStyle::PromptAssist,
        NativePanelVisualCardStyle::Completion => CardVisualStyle::Completion,
        NativePanelVisualCardStyle::Settings => CardVisualStyle::Settings,
        NativePanelVisualCardStyle::Empty => CardVisualStyle::Empty,
    }
}

fn visual_color_from_card_spec(color: CardVisualColorSpec) -> NativePanelVisualColor {
    NativePanelVisualColor::rgb(color.r, color.g, color.b)
}

fn push_expanded_card_content(
    output: &mut Vec<NativePanelVisualPrimitive>,
    card: &NativePanelVisualCardInput,
    frame: PanelRect,
    visible_frame: PanelRect,
    phase: f64,
) {
    let content_reveal = card_visual_content_transition_frame(phase, card.removing);
    if content_reveal.visibility_progress <= 0.001 || frame.height < card.collapsed_height.min(48.0)
    {
        return;
    }

    let fade_base = card_shell_fill_color(card.style);
    let content_layout = card_visual_content_layout(frame);
    if content_layout.content_width <= 8.0 {
        return;
    }

    let mut content_primitives = Vec::new();
    let primitives = &mut content_primitives;
    let header_text_spec =
        card_visual_header_text_paint_spec(card_visual_style_from_visual_style(card.style));
    if card.style == NativePanelVisualCardStyle::Empty {
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: PanelPoint {
                x: frame.x,
                y: content_layout.empty_title_y,
            },
            max_width: frame.width,
            text: card.title.clone(),
            color: visual_color_from_card_spec(header_text_spec.title.color),
            size: header_text_spec.title.size,
            weight: NativePanelVisualTextWeight::Semibold,
            alignment: NativePanelVisualTextAlignment::Center,
        });
        apply_card_content_reveal_to_primitives(
            &mut content_primitives,
            content_reveal.translate_y,
            content_reveal.visibility_progress,
            fade_base,
        );
        extend_visible_content_primitives(output, content_primitives, visible_frame);
        return;
    }

    let mut badge_right = frame.x + frame.width - 12.0;
    let status_left = if let Some(badge) = &card.badge {
        push_expanded_card_badge(
            primitives,
            badge,
            badge_right,
            content_layout.title_y,
            card.style,
            CardVisualBadgeRole::Status,
        )
    } else {
        badge_right
    };
    badge_right = status_left;
    let source_left = if let Some(badge) = &card.source_badge {
        push_expanded_card_badge(
            primitives,
            badge,
            badge_right - 6.0,
            content_layout.title_y,
            card.style,
            CardVisualBadgeRole::Source,
        )
    } else {
        status_left
    };
    let title_width = (source_left - content_layout.content_x - 8.0).max(92.0);
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: PanelPoint {
            x: content_layout.content_x,
            y: content_layout.title_y,
        },
        max_width: title_width,
        text: compact_title(&card.title, header_text_spec.title_max_chars),
        color: visual_color_from_card_spec(header_text_spec.title.color),
        size: header_text_spec.title.size,
        weight: NativePanelVisualTextWeight::Semibold,
        alignment: NativePanelVisualTextAlignment::Left,
    });

    if let Some(subtitle) = &card.subtitle {
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: PanelPoint {
                x: content_layout.content_x,
                y: content_layout.subtitle_y,
            },
            max_width: content_layout.content_width,
            text: fit_text_to_width(
                subtitle,
                content_layout.content_width,
                header_text_spec.subtitle.size as f64,
                1,
            ),
            color: visual_color_from_card_spec(header_text_spec.subtitle.color),
            size: header_text_spec.subtitle.size,
            weight: NativePanelVisualTextWeight::Normal,
            alignment: NativePanelVisualTextAlignment::Left,
        });
    }

    if !card.rows.is_empty() {
        push_expanded_settings_rows(
            primitives,
            card,
            frame,
            content_layout.content_x,
            content_layout.content_width,
        );
        apply_card_content_reveal_to_primitives(
            &mut content_primitives,
            content_reveal.translate_y,
            content_reveal.visibility_progress,
            fade_base,
        );
        extend_visible_content_primitives(output, content_primitives, visible_frame);
        return;
    }

    if card.body.is_some() || !card.body_lines.is_empty() {
        push_expanded_card_body_line(primitives, card, frame, card.body.as_deref().unwrap_or(""));
    }

    if let Some(action_hint) = &card.action_hint {
        push_pending_action_hint_pill(primitives, frame, action_hint);
    }

    apply_card_content_reveal_to_primitives(
        &mut content_primitives,
        content_reveal.translate_y,
        content_reveal.visibility_progress,
        fade_base,
    );
    extend_visible_content_primitives(output, content_primitives, visible_frame);
}

fn apply_card_content_reveal_to_primitives(
    primitives: &mut [NativePanelVisualPrimitive],
    translate_y: f64,
    progress: f64,
    fade_base: NativePanelVisualColor,
) {
    for primitive in primitives {
        translate_primitive_y(primitive, translate_y);
        fade_primitive_color(primitive, fade_base, progress);
    }
}

fn translate_primitive_y(primitive: &mut NativePanelVisualPrimitive, translate_y: f64) {
    match primitive {
        NativePanelVisualPrimitive::RoundRect { frame, .. }
        | NativePanelVisualPrimitive::Rect { frame, .. }
        | NativePanelVisualPrimitive::Ellipse { frame, .. }
        | NativePanelVisualPrimitive::CompactShoulder { frame, .. } => {
            frame.y += translate_y;
        }
        NativePanelVisualPrimitive::StrokeLine { from, to, .. } => {
            from.y += translate_y;
            to.y += translate_y;
        }
        NativePanelVisualPrimitive::Text { origin, .. } => {
            origin.y += translate_y;
        }
        NativePanelVisualPrimitive::MascotDot { center, .. } => {
            center.y += translate_y;
        }
    }
}

fn fade_primitive_color(
    primitive: &mut NativePanelVisualPrimitive,
    fade_base: NativePanelVisualColor,
    progress: f64,
) {
    match primitive {
        NativePanelVisualPrimitive::RoundRect { color, .. }
        | NativePanelVisualPrimitive::Rect { color, .. }
        | NativePanelVisualPrimitive::Ellipse { color, .. }
        | NativePanelVisualPrimitive::StrokeLine { color, .. }
        | NativePanelVisualPrimitive::Text { color, .. } => {
            *color = blend_visual_color(fade_base, *color, progress);
        }
        NativePanelVisualPrimitive::CompactShoulder { fill, border, .. } => {
            *fill = blend_visual_color(fade_base, *fill, progress);
            *border = blend_visual_color(fade_base, *border, progress);
        }
        NativePanelVisualPrimitive::MascotDot { .. } => {}
    }
}

fn blend_visual_color(
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

fn extend_visible_content_primitives(
    output: &mut Vec<NativePanelVisualPrimitive>,
    primitives: Vec<NativePanelVisualPrimitive>,
    visible_frame: PanelRect,
) {
    output.extend(
        primitives
            .into_iter()
            .filter(|primitive| primitive_fits_vertical_bounds(primitive, visible_frame)),
    );
}

fn push_expanded_card_body_line(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    card: &NativePanelVisualCardInput,
    frame: PanelRect,
    body: &str,
) {
    let metrics = default_panel_card_metric_constants();
    let body_layout = card_visual_body_layout(frame, card.action_hint.is_some());
    let mut cursor_y = body_layout.initial_y;
    let body_lines = expanded_card_body_lines(card, body);
    for (index, line) in body_lines.iter().enumerate() {
        if line.role == NativePanelVisualCardBodyRole::Tool {
            push_expanded_tool_pill_line(primitives, frame, cursor_y, &line.text);
            cursor_y += 22.0;
            if index + 1 < body_lines.len() {
                cursor_y += metrics.tool_gap;
            }
            continue;
        }

        let body_text =
            fit_text_to_lines(&line.text, body_layout.body_width, 10.0, line.max_lines).join("\n");
        let body_height = resolve_estimated_chat_body_height(
            &body_text,
            body_layout.body_width,
            line.max_lines as isize,
            metrics,
        );
        if let Some(prefix) = &line.prefix {
            let line_spec = card_visual_body_line_paint_spec(
                card_visual_style_from_visual_style(card.style),
                card_visual_body_role_from_visual_role(line.role),
                Some(prefix),
            );
            primitives.push(NativePanelVisualPrimitive::Text {
                origin: PanelPoint {
                    x: body_layout.prefix_x,
                    y: cursor_y + body_height - 12.0,
                },
                max_width: 10.0,
                text: prefix.clone(),
                color: visual_color_from_card_spec(line_spec.prefix_color),
                size: line_spec.prefix_size,
                weight: NativePanelVisualTextWeight::Bold,
                alignment: NativePanelVisualTextAlignment::Center,
            });
        }
        let line_spec = card_visual_body_line_paint_spec(
            card_visual_style_from_visual_style(card.style),
            card_visual_body_role_from_visual_role(line.role),
            line.prefix.as_deref(),
        );
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: PanelPoint {
                x: body_layout.text_x,
                y: cursor_y,
            },
            max_width: body_layout.body_width,
            text: body_text,
            color: visual_color_from_card_spec(line_spec.text_color),
            size: line_spec.text_size,
            weight: NativePanelVisualTextWeight::Normal,
            alignment: NativePanelVisualTextAlignment::Left,
        });
        cursor_y += body_height;
        if index + 1 < body_lines.len() {
            cursor_y += metrics.chat_gap;
        }
    }
}

fn push_pending_action_hint_pill(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    frame: PanelRect,
    action_hint: &str,
) {
    let Some(layout) = card_visual_action_hint_layout(frame, action_hint) else {
        return;
    };
    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: layout.pill_frame,
        radius: layout.paint.radius,
        color: visual_color_from_card_spec(layout.paint.background_color),
    });
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: layout.text_origin,
        max_width: layout.text_max_width,
        text: fit_text_to_width(
            &layout.paint.text,
            layout.text_max_width,
            layout.paint.text_size as f64,
            1,
        ),
        color: visual_color_from_card_spec(layout.paint.foreground_color),
        size: layout.paint.text_size,
        weight: NativePanelVisualTextWeight::Semibold,
        alignment: NativePanelVisualTextAlignment::Left,
    });
}

fn push_expanded_tool_pill_line(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    frame: PanelRect,
    y: f64,
    text: &str,
) {
    let Some(layout) = card_visual_tool_pill_layout(frame, y, text) else {
        return;
    };

    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: layout.pill_frame,
        radius: layout.paint.radius,
        color: visual_color_from_card_spec(layout.paint.background_color),
    });

    primitives.push(NativePanelVisualPrimitive::Text {
        origin: layout.tool_name_origin,
        max_width: layout.tool_name_max_width,
        text: layout.paint.tool_name.clone(),
        color: visual_color_from_card_spec(layout.paint.tool_name_color),
        size: layout.paint.text_size,
        weight: NativePanelVisualTextWeight::Bold,
        alignment: NativePanelVisualTextAlignment::Left,
    });

    if let Some(description) = layout.description {
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: description.origin,
            max_width: description.max_width,
            text: fit_text_to_width(
                &description.text,
                description.max_width,
                layout.paint.text_size as f64,
                1,
            ),
            color: visual_color_from_card_spec(layout.paint.description_color),
            size: layout.paint.text_size,
            weight: NativePanelVisualTextWeight::Normal,
            alignment: NativePanelVisualTextAlignment::Left,
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExpandedCardBodyLine {
    role: NativePanelVisualCardBodyRole,
    prefix: Option<String>,
    text: String,
    max_lines: usize,
}

fn expanded_card_body_lines(
    card: &NativePanelVisualCardInput,
    body: &str,
) -> Vec<ExpandedCardBodyLine> {
    if !card.body_lines.is_empty() {
        return card
            .body_lines
            .iter()
            .filter_map(|line| {
                let text = line.text.trim();
                (!text.is_empty()).then(|| ExpandedCardBodyLine {
                    role: line.role,
                    prefix: line.prefix.clone(),
                    text: text.to_string(),
                    max_lines: line.max_lines.max(1),
                })
            })
            .collect();
    }

    let raw_lines = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let prefixes = card
        .body_prefix
        .as_deref()
        .unwrap_or_default()
        .chars()
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>();
    if raw_lines.len() > 1 && prefixes.len() >= raw_lines.len() {
        return raw_lines
            .into_iter()
            .zip(prefixes)
            .map(|(text, prefix)| ExpandedCardBodyLine {
                role: expanded_card_body_role_for_prefix(Some(prefix.as_str())),
                max_lines: expanded_card_body_max_lines_for_prefix(
                    card.style,
                    Some(prefix.as_str()),
                ),
                prefix: Some(prefix),
                text: text.to_string(),
            })
            .collect();
    }

    let prefix = card.body_prefix.clone();
    vec![ExpandedCardBodyLine {
        role: expanded_card_body_role_for_prefix(prefix.as_deref()),
        max_lines: expanded_card_body_max_lines_for_prefix(card.style, prefix.as_deref()),
        prefix,
        text: body.to_string(),
    }]
}

fn expanded_card_body_role_for_prefix(prefix: Option<&str>) -> NativePanelVisualCardBodyRole {
    match prefix {
        Some("$") => NativePanelVisualCardBodyRole::Assistant,
        Some(">") => NativePanelVisualCardBodyRole::User,
        Some("!") => NativePanelVisualCardBodyRole::Tool,
        _ => NativePanelVisualCardBodyRole::Plain,
    }
}

fn expanded_card_body_max_lines_for_prefix(
    style: NativePanelVisualCardStyle,
    prefix: Option<&str>,
) -> usize {
    match (style, prefix) {
        (NativePanelVisualCardStyle::Default, Some(">")) => 1,
        (NativePanelVisualCardStyle::Default, Some("$"))
        | (NativePanelVisualCardStyle::Completion, _)
        | (NativePanelVisualCardStyle::Pending, _)
        | (NativePanelVisualCardStyle::PendingApproval, _)
        | (NativePanelVisualCardStyle::PendingQuestion, _)
        | (NativePanelVisualCardStyle::PromptAssist, _) => 2,
        _ => 1,
    }
}

fn push_expanded_card_badge(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    badge: &NativePanelVisualCardBadgeInput,
    right: f64,
    title_y: f64,
    style: NativePanelVisualCardStyle,
    role: CardVisualBadgeRole,
) -> f64 {
    let badge_spec = CardVisualBadgeSpec {
        role,
        text: badge.text.clone(),
        emphasized: badge.emphasized,
    };
    let layout = card_visual_badge_layout(
        card_visual_style_from_visual_style(style),
        &badge_spec,
        right,
        title_y,
    );
    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: layout.badge_frame,
        radius: layout.paint.radius,
        color: visual_color_from_card_spec(layout.paint.background_color),
    });
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: layout.text_origin,
        max_width: layout.text_max_width,
        text: badge.text.clone(),
        color: visual_color_from_card_spec(layout.paint.foreground_color),
        size: layout.paint.text_size,
        weight: NativePanelVisualTextWeight::Semibold,
        alignment: NativePanelVisualTextAlignment::Center,
    });
    layout.badge_frame.x
}

fn fit_text_to_width(text: &str, width: f64, font_size: f64, max_lines: usize) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return String::new();
    }
    let max_width = width.max(font_size) * max_lines.max(1) as f64;
    if resolve_estimated_text_width(&normalized, font_size) <= max_width {
        return normalized;
    }

    let mut clipped = String::new();
    for ch in normalized.chars() {
        let candidate = format!("{clipped}{ch}...");
        if resolve_estimated_text_width(&candidate, font_size) > max_width {
            break;
        }
        clipped.push(ch);
    }
    if clipped.is_empty() {
        "...".to_string()
    } else {
        format!("{}...", clipped.trim_end())
    }
}

fn fit_text_to_lines(text: &str, width: f64, font_size: f64, max_lines: usize) -> Vec<String> {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Vec::new();
    }

    let max_lines = max_lines.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in normalized.chars() {
        let candidate = format!("{current}{ch}");
        if !current.is_empty() && resolve_estimated_text_width(&candidate, font_size) > width {
            lines.push(current.trim_end().to_string());
            current.clear();
            if lines.len() == max_lines {
                break;
            }
        }
        current.push(ch);
    }
    if lines.len() < max_lines && !current.is_empty() {
        lines.push(current.trim_end().to_string());
    }

    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
    if !text_fits_in_lines(&normalized, &lines) {
        if let Some(last) = lines.last_mut() {
            *last = ellipsize_text_to_width(last, width, font_size);
        }
    }
    lines
}

fn text_fits_in_lines(original: &str, lines: &[String]) -> bool {
    lines.join("").chars().count() >= original.chars().count()
}

fn ellipsize_text_to_width(text: &str, width: f64, font_size: f64) -> String {
    let ellipsis = "...";
    if resolve_estimated_text_width(text, font_size) <= width
        && !text.ends_with(ellipsis)
        && resolve_estimated_text_width(&format!("{text}{ellipsis}"), font_size) <= width
    {
        return text.to_string();
    }

    let mut clipped = String::new();
    for ch in text.chars() {
        let candidate = format!("{clipped}{ch}{ellipsis}");
        if resolve_estimated_text_width(&candidate, font_size) > width {
            break;
        }
        clipped.push(ch);
    }
    if clipped.is_empty() {
        ellipsis.to_string()
    } else {
        format!("{}{}", clipped.trim_end(), ellipsis)
    }
}

fn push_expanded_settings_rows(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    card: &NativePanelVisualCardInput,
    frame: PanelRect,
    _content_x: f64,
    _content_width: f64,
) {
    for (index, row) in card.rows.iter().enumerate() {
        let row_spec = CardVisualRowSpec {
            title: row.title.clone(),
            value: row.value.clone(),
            active: row.active,
        };
        let Some(layout) = card_visual_settings_row_layout(frame, index, &row_spec) else {
            break;
        };
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: layout.row_frame,
            radius: layout.paint.border_radius,
            color: visual_color_from_card_spec(layout.paint.border_color),
        });
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: layout.row_inner_frame,
            radius: layout.paint.fill_radius,
            color: visual_color_from_card_spec(layout.paint.fill_color),
        });

        primitives.push(NativePanelVisualPrimitive::Text {
            origin: layout.title_origin,
            max_width: layout.title_max_width,
            text: row.title.clone(),
            color: visual_color_from_card_spec(layout.paint.title_color),
            size: layout.paint.title_size,
            weight: NativePanelVisualTextWeight::Semibold,
            alignment: NativePanelVisualTextAlignment::Left,
        });
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: layout.value_badge_frame,
            radius: layout.paint.value_badge.radius,
            color: visual_color_from_card_spec(layout.paint.value_badge.background_color),
        });
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: layout.value_origin,
            max_width: layout.value_max_width,
            text: fit_text_to_width(
                &row.value,
                layout.value_max_width,
                layout.paint.value_badge.text_size as f64,
                1,
            ),
            color: visual_color_from_card_spec(layout.paint.value_badge.foreground_color),
            size: layout.paint.value_badge.text_size,
            weight: NativePanelVisualTextWeight::Semibold,
            alignment: NativePanelVisualTextAlignment::Center,
        });
    }
}

fn push_compact_shoulder_primitive(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    frame: PanelRect,
    side: NativePanelVisualShoulderSide,
    progress: f64,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }
    let progress = progress.clamp(0.0, 1.0);
    if progress >= 0.999 {
        return;
    }
    primitives.push(NativePanelVisualPrimitive::CompactShoulder {
        frame,
        side,
        progress,
        fill: NativePanelVisualColor::rgb(12, 12, 15),
        border: NativePanelVisualColor::rgb(44, 44, 50),
    });
}

fn non_zero_rect(rect: PanelRect) -> Option<PanelRect> {
    (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

fn inset_rect(rect: PanelRect, inset: f64) -> PanelRect {
    PanelRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2.0).max(0.0),
        height: (rect.height - inset * 2.0).max(0.0),
    }
}

fn clip_rect_vertically(rect: PanelRect, bounds: PanelRect) -> Option<PanelRect> {
    let bottom = rect.y.max(bounds.y);
    let top = (rect.y + rect.height).min(bounds.y + bounds.height);
    (top > bottom).then_some(PanelRect {
        x: rect.x,
        y: bottom,
        width: rect.width,
        height: top - bottom,
    })
}

fn primitive_intersects_vertical_bounds(
    primitive: &NativePanelVisualPrimitive,
    bounds: PanelRect,
) -> bool {
    let Some((bottom, top)) = primitive_vertical_bounds(primitive) else {
        return true;
    };
    top > bounds.y && bottom < bounds.y + bounds.height
}

fn primitive_fits_vertical_bounds(
    primitive: &NativePanelVisualPrimitive,
    bounds: PanelRect,
) -> bool {
    let Some((bottom, top)) = primitive_vertical_bounds(primitive) else {
        return true;
    };
    bottom >= bounds.y && top <= bounds.y + bounds.height
}

fn primitive_vertical_bounds(primitive: &NativePanelVisualPrimitive) -> Option<(f64, f64)> {
    match primitive {
        NativePanelVisualPrimitive::RoundRect { frame, .. }
        | NativePanelVisualPrimitive::Rect { frame, .. }
        | NativePanelVisualPrimitive::Ellipse { frame, .. }
        | NativePanelVisualPrimitive::CompactShoulder { frame, .. } => {
            Some((frame.y, frame.y + frame.height))
        }
        NativePanelVisualPrimitive::StrokeLine { from, to, .. } => {
            Some((from.y.min(to.y), from.y.max(to.y)))
        }
        NativePanelVisualPrimitive::Text {
            origin, text, size, ..
        } => {
            let height = visual_text_box_height(text, *size);
            Some((origin.y, origin.y + height))
        }
        NativePanelVisualPrimitive::MascotDot { center, radius, .. } => {
            Some((center.y - radius, center.y + radius))
        }
    }
}

fn visual_text_box_height(text: &str, size: i32) -> f64 {
    let line_count = text.lines().count().max(1) as f64;
    let line_height = if size >= 13 { 24.0 } else { size as f64 + 8.0 };
    line_count * line_height
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelVisualActionButtonInput, NativePanelVisualCardBadgeInput,
        NativePanelVisualCardBodyLineInput, NativePanelVisualCardBodyRole,
        NativePanelVisualCardInput, NativePanelVisualCardRowInput, NativePanelVisualCardStyle,
        NativePanelVisualDisplayMode, NativePanelVisualPlan, NativePanelVisualPlanInput,
        compact_digit_y, native_panel_visual_card_input_from_scene_card_with_height,
        resolve_native_panel_visual_plan, visual_text_box_height,
    };
    use crate::{
        native_panel_core::{
            ACTIVE_COUNT_SCROLL_HOLD_MS, ACTIVE_COUNT_SCROLL_MOVE_MS, ACTIVE_COUNT_TEXT_OFFSET_X,
            ExpandedSurface, PanelPoint, PanelRect,
        },
        native_panel_renderer::{
            descriptors::{NativePanelEdgeAction, NativePanelHostWindowState},
            visual_primitives::{
                NativePanelVisualPrimitive, NativePanelVisualTextAlignment,
                NativePanelVisualTextWeight,
            },
        },
        native_panel_scene::{SceneBadge, SceneCard, SceneMascotPose},
    };
    use chrono::Utc;
    use echoisland_runtime::{PendingPermissionView, PendingQuestionView, SessionSnapshotView};

    const SETTINGS_ACTION_ICON_TEXT: &str = "\u{E713}";
    const QUIT_ACTION_ICON_TEXT: &str = "⏻";
    const SETTINGS_ACTION_ICON_SIZE: i32 = 16;
    const QUIT_ACTION_ICON_SIZE: i32 = 16;

    fn visual_input(display_mode: NativePanelVisualDisplayMode) -> NativePanelVisualPlanInput {
        let compact_bar_width = if display_mode == NativePanelVisualDisplayMode::Expanded {
            crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH
        } else {
            240.0
        };
        NativePanelVisualPlanInput {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 20.0,
                    width: 320.0,
                    height: 160.0,
                }),
                visible: display_mode != NativePanelVisualDisplayMode::Hidden,
                preferred_display_index: 0,
            },
            display_mode,
            surface: ExpandedSurface::Default,
            panel_frame: PanelRect {
                x: 100.0,
                y: 20.0,
                width: 320.0,
                height: 160.0,
            },
            compact_bar_frame: PanelRect {
                x: (320.0 - compact_bar_width) / 2.0,
                y: 12.0,
                width: compact_bar_width,
                height: 36.0,
            },
            left_shoulder_frame: PanelRect {
                x: 34.0,
                y: 42.0,
                width: 6.0,
                height: 6.0,
            },
            right_shoulder_frame: PanelRect {
                x: 280.0,
                y: 42.0,
                width: 6.0,
                height: 6.0,
            },
            shoulder_progress: 0.0,
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            card_stack_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            card_stack_content_height: 180.0,
            shell_frame: PanelRect {
                x: 20.0,
                y: 0.0,
                width: 280.0,
                height: 150.0,
            },
            headline_text: "Codex ready".to_string(),
            headline_emphasized: false,
            active_count: "1".to_string(),
            active_count_elapsed_ms: 0,
            total_count: "3".to_string(),
            separator_visibility: 0.88,
            cards_visible: true,
            card_count: 2,
            cards: vec![
                NativePanelVisualCardInput {
                    style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Settings,
                    title: "Settings".to_string(),
                    subtitle: Some("EchoIsland v0.2.0".to_string()),
                    body: None,
                    badge: None,
                    source_badge: None,
                    body_prefix: None,
                    body_lines: Vec::new(),
                    action_hint: None,
                    rows: vec![NativePanelVisualCardRowInput {
                        title: "Mute Sound".to_string(),
                        value: "Off".to_string(),
                        active: true,
                    }],
                    height: 92.0,
                    collapsed_height: 64.0,
                    compact: false,
                    removing: false,
                },
                NativePanelVisualCardInput {
                    style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Completion,
                    title: "Done".to_string(),
                    subtitle: Some("#abcdef · now".to_string()),
                    body: Some("Task complete".to_string()),
                    badge: Some(NativePanelVisualCardBadgeInput {
                        text: "Done".to_string(),
                        emphasized: true,
                    }),
                    source_badge: Some(NativePanelVisualCardBadgeInput {
                        text: "Codex".to_string(),
                        emphasized: false,
                    }),
                    body_prefix: Some("$".to_string()),
                    body_lines: Vec::new(),
                    action_hint: None,
                    rows: Vec::new(),
                    height: 76.0,
                    collapsed_height: 52.0,
                    compact: false,
                    removing: false,
                },
            ],
            glow_visible: true,
            action_buttons_visible: true,
            action_buttons: vec![
                NativePanelVisualActionButtonInput {
                    action: NativePanelEdgeAction::Settings,
                    frame: PanelRect {
                        x: 250.0,
                        y: 20.0,
                        width: 18.0,
                        height: 18.0,
                    },
                },
                NativePanelVisualActionButtonInput {
                    action: NativePanelEdgeAction::Quit,
                    frame: PanelRect {
                        x: 280.0,
                        y: 20.0,
                        width: 18.0,
                        height: 18.0,
                    },
                },
            ],
            completion_count: 2,
            mascot_elapsed_ms: 0,
            mascot_pose: SceneMascotPose::Complete,
        }
    }

    fn mascot_green_bubble_frame(plan: &NativePanelVisualPlan) -> Option<PanelRect> {
        plan.primitives.iter().find_map(|primitive| {
            match primitive {
            NativePanelVisualPrimitive::RoundRect { frame, color, .. }
                if *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                        102, 222, 145,
                    ) =>
            {
                Some(*frame)
            }
            _ => None,
        }
        })
    }

    fn mascot_sleep_label_origin(plan: &NativePanelVisualPlan) -> Option<PanelPoint> {
        plan.primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text { origin, text, .. } if text == "Z" => {
                    Some(*origin)
                }
                _ => None,
            })
    }

    fn session_with_chat_lines() -> SessionSnapshotView {
        SessionSnapshotView {
            session_id: "session-123456".to_string(),
            source: "codex".to_string(),
            project_name: Some("EchoIsland".to_string()),
            cwd: None,
            model: None,
            terminal_app: None,
            terminal_bundle: None,
            host_app: None,
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
            status: "running".to_string(),
            current_tool: Some("Bash".to_string()),
            tool_description: Some("cargo test".to_string()),
            last_user_prompt: Some("Fix Windows card".to_string()),
            last_assistant_message: Some("Adjusting layout".to_string()),
            tool_history_count: 0,
            tool_history: Vec::new(),
            last_activity: Utc::now(),
        }
    }

    fn pending_permission() -> PendingPermissionView {
        PendingPermissionView {
            request_id: "approval-1".to_string(),
            session_id: "session-approval".to_string(),
            source: "claude".to_string(),
            tool_name: Some("Bash".to_string()),
            tool_description: Some("Run command".to_string()),
            requested_at: Utc::now(),
        }
    }

    fn pending_question() -> PendingQuestionView {
        PendingQuestionView {
            request_id: "question-1".to_string(),
            session_id: "session-question".to_string(),
            source: "claude".to_string(),
            header: Some("Pick one".to_string()),
            text: "Choose the deployment target".to_string(),
            options: vec!["Local".to_string(), "Production".to_string()],
            requested_at: Utc::now(),
        }
    }

    #[test]
    fn visual_card_input_preserves_structured_body_roles_from_shared_spec() {
        let session = session_with_chat_lines();
        let card = native_panel_visual_card_input_from_scene_card_with_height(
            &SceneCard::Session {
                session,
                title: "EchoIsland".to_string(),
                status: SceneBadge {
                    text: "Running".to_string(),
                    emphasized: true,
                },
                snippet: Some("Adjusting layout".to_string()),
            },
            112.0,
        );

        assert_eq!(
            card.body_lines
                .iter()
                .map(|line| (
                    line.role,
                    line.prefix.as_deref(),
                    line.text.as_str(),
                    line.max_lines
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    NativePanelVisualCardBodyRole::Tool,
                    Some("!"),
                    "Bash cargo test",
                    1,
                ),
                (
                    NativePanelVisualCardBodyRole::Assistant,
                    Some("$"),
                    "Adjusting layout",
                    2,
                ),
                (
                    NativePanelVisualCardBodyRole::User,
                    Some(">"),
                    "Fix Windows card",
                    1,
                ),
            ]
        );
        assert!(card.body.is_none());
        assert!(card.body_prefix.is_none());
    }

    #[test]
    fn visual_card_input_preserves_pending_tones_from_shared_spec() {
        let approval = native_panel_visual_card_input_from_scene_card_with_height(
            &SceneCard::PendingPermission {
                pending: pending_permission(),
                count: 1,
            },
            72.0,
        );
        let question = native_panel_visual_card_input_from_scene_card_with_height(
            &SceneCard::PendingQuestion {
                pending: pending_question(),
                count: 1,
            },
            72.0,
        );
        let prompt = native_panel_visual_card_input_from_scene_card_with_height(
            &SceneCard::PromptAssist {
                session: session_with_chat_lines(),
            },
            72.0,
        );

        assert_eq!(approval.style, NativePanelVisualCardStyle::PendingApproval);
        assert_eq!(question.style, NativePanelVisualCardStyle::PendingQuestion);
        assert_eq!(prompt.style, NativePanelVisualCardStyle::PromptAssist);
    }

    #[test]
    fn expanded_visual_plan_draws_question_pending_tone_distinct_from_approval() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 160.0;
        input.card_stack_content_height = 160.0;
        input.cards = vec![
            native_panel_visual_card_input_from_scene_card_with_height(
                &SceneCard::PendingPermission {
                    pending: pending_permission(),
                    count: 1,
                },
                72.0,
            ),
            native_panel_visual_card_input_from_scene_card_with_height(
                &SceneCard::PendingQuestion {
                    pending: pending_question(),
                    count: 2,
                },
                72.0,
            ),
        ];

        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { color, .. }
                if *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(87, 61, 39)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { color, .. }
                if *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(74, 62, 103)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "?"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(201, 176, 255)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "2"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(201, 176, 255)
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_pending_action_hint_as_bottom_pill() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 96.0;
        input.card_stack_content_height = 96.0;
        input.cards = vec![native_panel_visual_card_input_from_scene_card_with_height(
            &SceneCard::PendingPermission {
                pending: pending_permission(),
                count: 1,
            },
            72.0,
        )];

        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, radius, color }
                if (frame.height - crate::native_panel_core::CARD_PENDING_ACTION_HEIGHT).abs() < 0.001
                    && (*radius - crate::native_panel_core::CARD_PENDING_ACTION_HEIGHT / 2.0).abs() < 0.001
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(49, 49, 53)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, size, .. }
                if text == "Allow / Deny in terminal"
                    && *size == 10
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(230, 235, 245)
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "Allow / Deny in terminal"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(104, 213, 145)
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_tool_body_role_as_mac_style_pill() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 120.0;
        input.card_stack_content_height = 120.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Default,
            title: "EchoIsland".to_string(),
            subtitle: Some("#c1d5-7 · now".to_string()),
            body: Some("Bash cargo test".to_string()),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Running".to_string(),
                emphasized: true,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: false,
            }),
            body_prefix: Some("!".to_string()),
            body_lines: vec![NativePanelVisualCardBodyLineInput {
                role: NativePanelVisualCardBodyRole::Tool,
                prefix: Some("!".to_string()),
                text: "Bash cargo test".to_string(),
                max_lines: 1,
            }],
            action_hint: None,
            rows: Vec::new(),
            height: 120.0,
            collapsed_height: 64.0,
            compact: false,
            removing: false,
        }];

        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, radius, color }
                if (frame.height - 22.0).abs() < 0.001
                    && (*radius - 5.0).abs() < 0.001
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(47, 47, 52)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, size, .. }
                if text == "Bash"
                    && *size == 9
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(125, 242, 163)
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Bash cargo test"
        )));
    }

    #[test]
    fn expanded_visual_plan_does_not_draw_card_text_outside_clipped_shell() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 96.0;
        input.card_stack_content_height = 140.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Default,
            title: "EchoIsland".to_string(),
            subtitle: Some("#c1d5-7 路 gpt-5.4 路 7m".to_string()),
            body: None,
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Idle".to_string(),
                emphasized: false,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: true,
            }),
            body_prefix: None,
            body_lines: vec![
                NativePanelVisualCardBodyLineInput {
                    role: NativePanelVisualCardBodyRole::User,
                    prefix: Some(">".to_string()),
                    text: "第二次移出鼠标还是有问题，之前没有抽代码没有这个问题".to_string(),
                    max_lines: 1,
                },
                NativePanelVisualCardBodyLineInput {
                    role: NativePanelVisualCardBodyRole::Assistant,
                    prefix: Some("$".to_string()),
                    text: "已检查并修了一处抽共享代码后引入的布局抖动点".to_string(),
                    max_lines: 2,
                },
            ],
            action_hint: None,
            rows: Vec::new(),
            height: 140.0,
            collapsed_height: 64.0,
            compact: false,
            removing: false,
        }];

        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "EchoIsland"
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. }
                if text.contains("第二次移出鼠标") || text.contains("已检查并修了")
        )));
    }

    #[test]
    fn expanded_visual_plan_reveals_card_content_before_shell_is_mostly_open() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.22;
        input.card_stack_frame.height = input.card_stack_content_height;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Completion,
            title: "Done".to_string(),
            subtitle: Some("#abcdef · now".to_string()),
            body: Some("Task complete".to_string()),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Done".to_string(),
                emphasized: true,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: false,
            }),
            body_prefix: Some("$".to_string()),
            body_lines: Vec::new(),
            action_hint: None,
            rows: Vec::new(),
            height: 76.0,
            collapsed_height: 52.0,
            compact: false,
            removing: false,
        }];

        let plan = resolve_native_panel_visual_plan(&input);

        let title = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text {
                    text,
                    origin,
                    color,
                    ..
                } if text == "Done" => Some((*origin, *color)),
                _ => None,
            })
            .expect("title should start revealing before the shell is mostly open");

        let stable_title_y = input.shell_frame.y
            + input.card_stack_frame.y
            + (input.card_stack_content_height - input.cards[0].height)
            + input.cards[0].height
            - 24.0;
        assert!(title.0.y < stable_title_y);
        assert_ne!(
            title.1,
            crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                245, 247, 252,
            )
        );
    }

    #[test]
    fn expanded_visual_plan_fades_removing_status_card_content_before_shell_exit() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.22;
        input.card_stack_frame.height = input.card_stack_content_height;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Completion,
            title: "Done".to_string(),
            subtitle: Some("#abcdef · now".to_string()),
            body: Some("Task complete".to_string()),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Done".to_string(),
                emphasized: true,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: false,
            }),
            body_prefix: Some("$".to_string()),
            body_lines: Vec::new(),
            action_hint: None,
            rows: Vec::new(),
            height: 76.0,
            collapsed_height: 52.0,
            compact: false,
            removing: true,
        }];

        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == "Done")
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::RoundRect { frame, .. } if frame.height > 40.0)
        }));
    }

    fn headline_text_frame(
        plan: &crate::native_panel_renderer::visual_primitives::NativePanelVisualPlan,
    ) -> (f64, f64, f64) {
        plan.primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text {
                    origin,
                    max_width,
                    text,
                    size,
                    ..
                } if text == "Codex ready" && *size == 13 => {
                    Some((origin.x, *max_width, origin.x + *max_width / 2.0))
                }
                _ => None,
            })
            .expect("headline text")
    }

    fn centered_text_visual_bounds(
        origin_x: f64,
        max_width: f64,
        text: &str,
        size: i32,
    ) -> (f64, f64) {
        let estimated_width =
            crate::native_panel_core::resolve_estimated_text_width(text, size as f64)
                .min(max_width);
        let left = origin_x + (max_width - estimated_width) / 2.0;
        (left, left + estimated_width)
    }

    fn text_primitive<'a>(
        plan: &'a crate::native_panel_renderer::visual_primitives::NativePanelVisualPlan,
        expected: &str,
    ) -> &'a NativePanelVisualPrimitive {
        plan.primitives
            .iter()
            .find(|primitive| {
                matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == expected)
            })
            .unwrap_or_else(|| panic!("missing text primitive {expected}"))
    }

    fn use_wide_action_button_hit_regions(input: &mut NativePanelVisualPlanInput) {
        let compact = input.compact_bar_frame;
        input.action_buttons = vec![
            NativePanelVisualActionButtonInput {
                action: NativePanelEdgeAction::Settings,
                frame: PanelRect {
                    x: compact.x,
                    y: compact.y,
                    width: 58.0,
                    height: compact.height,
                },
            },
            NativePanelVisualActionButtonInput {
                action: NativePanelEdgeAction::Quit,
                frame: PanelRect {
                    x: compact.x + compact.width - 58.0,
                    y: compact.y,
                    width: 58.0,
                    height: compact.height,
                },
            },
        ];
    }

    #[test]
    fn visual_plan_is_empty_when_hidden() {
        let plan =
            resolve_native_panel_visual_plan(&visual_input(NativePanelVisualDisplayMode::Hidden));

        assert!(plan.hidden);
        assert!(plan.primitives.is_empty());
    }

    #[test]
    fn visual_plan_contains_panel_content_and_icons() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.card_stack_frame.height = input.card_stack_content_height;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.hidden);
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == "Codex ready")
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(
                primitive,
                NativePanelVisualPrimitive::MascotDot {
                    pose: SceneMascotPose::Complete,
                    ..
                }
            )
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == SETTINGS_ACTION_ICON_TEXT)
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == QUIT_ACTION_ICON_TEXT)
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == "Done")
        }));
    }

    #[test]
    fn expanded_visual_plan_draws_action_icons_from_mac_button_layout_not_wide_hit_regions() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        let compact = input.compact_bar_frame;
        input.action_buttons = vec![
            NativePanelVisualActionButtonInput {
                action: NativePanelEdgeAction::Settings,
                frame: PanelRect {
                    x: compact.x,
                    y: compact.y,
                    width: 58.0,
                    height: compact.height,
                },
            },
            NativePanelVisualActionButtonInput {
                action: NativePanelEdgeAction::Quit,
                frame: PanelRect {
                    x: compact.x + compact.width - 58.0,
                    y: compact.y,
                    width: 58.0,
                    height: compact.height,
                },
            },
        ];

        let plan = resolve_native_panel_visual_plan(&input);
        let mascot_center = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::MascotDot { center, .. } => Some(*center),
                _ => None,
            })
            .expect("mascot primitive");
        let settings_center = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text {
                    origin,
                    max_width,
                    text,
                    ..
                } if text == SETTINGS_ACTION_ICON_TEXT => {
                    Some(crate::native_panel_core::PanelPoint {
                        x: origin.x + max_width / 2.0,
                        y: origin.y,
                    })
                }
                _ => None,
            })
            .expect("settings icon center");
        let quit_right = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text {
                    origin,
                    max_width,
                    text,
                    ..
                } if text == QUIT_ACTION_ICON_TEXT => Some(origin.x + max_width),
                _ => None,
            })
            .expect("quit icon right edge");
        let active_count_visible_left = match text_primitive(&plan, "1") {
            NativePanelVisualPrimitive::Text {
                origin,
                max_width,
                text,
                size,
                ..
            } => {
                origin.x + max_width
                    - crate::native_panel_core::resolve_estimated_text_width(text, *size as f64)
                        .min(*max_width)
            }
            _ => unreachable!(),
        };

        assert!(settings_center.x > mascot_center.x + 34.0);
        assert!(quit_right <= active_count_visible_left - 4.0);
    }

    #[test]
    fn expanded_visual_plan_draws_platform_matching_action_icon_glyphs() {
        let input = visual_input(NativePanelVisualDisplayMode::Expanded);
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text {
                text,
                size,
                weight,
                alignment,
                color,
                ..
            } if text == SETTINGS_ACTION_ICON_TEXT
                && *size == SETTINGS_ACTION_ICON_SIZE
                && *weight
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Normal
                && *alignment
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
                && *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                        245, 247, 252,
                    )
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text {
                text,
                size,
                weight,
                alignment,
                color,
                ..
            } if text == QUIT_ACTION_ICON_TEXT
                && *size == QUIT_ACTION_ICON_SIZE
                && *weight
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Bold
                && *alignment
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
                && *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                        255, 82, 82,
                    )
        )));
    }

    #[test]
    fn expanded_visual_plan_uses_windows_settings_icon_glyph() {
        let input = visual_input(NativePanelVisualDisplayMode::Expanded);
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text {
                text,
                size,
                weight,
                alignment,
                ..
            } if text == "\u{E713}"
                && *size == 16
                && *weight
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Normal
                && *alignment
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
        )));
    }

    #[test]
    fn expanded_visual_plan_applies_shared_action_button_transition_phase() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.compact_bar_frame.width = crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH;
        input.separator_visibility = 0.0;
        let hidden_plan = resolve_native_panel_visual_plan(&input);

        assert!(!hidden_plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == SETTINGS_ACTION_ICON_TEXT)
        }));

        input.compact_bar_frame.width = (crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH
            + crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH)
            / 2.0;
        let mid_plan = resolve_native_panel_visual_plan(&input);
        let mid_settings = text_primitive(&mid_plan, SETTINGS_ACTION_ICON_TEXT);
        let mut full_input = visual_input(NativePanelVisualDisplayMode::Expanded);
        full_input.compact_bar_frame.width = crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH;
        full_input.separator_visibility = 0.0;
        let full_plan = resolve_native_panel_visual_plan(&full_input);
        let full_settings_y = match text_primitive(&full_plan, SETTINGS_ACTION_ICON_TEXT) {
            NativePanelVisualPrimitive::Text { origin, .. } => origin.y,
            _ => unreachable!(),
        };

        match mid_settings {
            NativePanelVisualPrimitive::Text { origin, color, .. } => {
                assert!(origin.y < full_settings_y);
                assert!(
                    *color
                        != crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                            245, 247, 252,
                        )
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn expanded_visual_plan_keeps_long_headline_inside_action_button_safe_area() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.compact_bar_frame.width = crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH;
        input.headline_text = "Permission waiting".to_string();

        let plan = resolve_native_panel_visual_plan(&input);
        let settings_right = match text_primitive(&plan, SETTINGS_ACTION_ICON_TEXT) {
            NativePanelVisualPrimitive::Text {
                origin, max_width, ..
            } => origin.x + max_width,
            _ => unreachable!(),
        };
        let quit_left = match text_primitive(&plan, QUIT_ACTION_ICON_TEXT) {
            NativePanelVisualPrimitive::Text { origin, .. } => origin.x,
            _ => unreachable!(),
        };
        let headline_bounds = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text {
                    origin,
                    max_width,
                    text,
                    size,
                    weight,
                    alignment,
                    ..
                } if text.contains("Permission")
                    && *size == 13
                    && *weight == NativePanelVisualTextWeight::Semibold
                    && *alignment == NativePanelVisualTextAlignment::Center =>
                {
                    Some(centered_text_visual_bounds(
                        origin.x, *max_width, text, *size,
                    ))
                }
                _ => None,
            })
            .expect("headline text bounds");

        assert!(settings_right + 4.0 <= headline_bounds.0);
        assert!(headline_bounds.1 + 4.0 <= quit_left);
    }

    #[test]
    fn expanded_visual_plan_places_headline_after_settings_icon_when_actions_are_visible() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        use_wide_action_button_hit_regions(&mut input);
        input.compact_bar_frame.width = 283.0;
        let plan = resolve_native_panel_visual_plan(&input);
        let settings_right = match text_primitive(&plan, SETTINGS_ACTION_ICON_TEXT) {
            NativePanelVisualPrimitive::Text {
                origin, max_width, ..
            } => origin.x + max_width,
            _ => unreachable!(),
        };
        let headline_visual_left = match text_primitive(&plan, "Codex ready") {
            NativePanelVisualPrimitive::Text {
                origin,
                max_width,
                text,
                size,
                ..
            } => centered_text_visual_bounds(origin.x, *max_width, text, *size).0,
            _ => unreachable!(),
        };

        assert!(settings_right + 4.0 <= headline_visual_left);
    }

    #[test]
    fn expanded_visual_plan_keeps_headline_center_stable_when_actions_are_visible() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.action_buttons_visible = false;
        let base_plan = resolve_native_panel_visual_plan(&input);
        let (_, _, base_headline_center_x) = headline_text_frame(&base_plan);

        use_wide_action_button_hit_regions(&mut input);
        input.action_buttons_visible = true;
        let plan = resolve_native_panel_visual_plan(&input);
        let (_, _, headline_center_x) = headline_text_frame(&plan);

        assert!((headline_center_x - base_headline_center_x).abs() <= 0.001);
    }

    #[test]
    fn expanded_visual_plan_vertically_aligns_action_icons_with_headline_text_box() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        use_wide_action_button_hit_regions(&mut input);
        let plan = resolve_native_panel_visual_plan(&input);
        let headline_center_y = match text_primitive(&plan, "Codex ready") {
            NativePanelVisualPrimitive::Text {
                origin, text, size, ..
            } => origin.y + visual_text_box_height(text, *size) / 2.0,
            _ => unreachable!(),
        };

        for icon in [SETTINGS_ACTION_ICON_TEXT, QUIT_ACTION_ICON_TEXT] {
            let icon_center_y = match text_primitive(&plan, icon) {
                NativePanelVisualPrimitive::Text {
                    origin, text, size, ..
                } => origin.y + visual_text_box_height(text, *size) / 2.0,
                _ => unreachable!(),
            };
            assert!((icon_center_y - headline_center_y).abs() <= 0.001);
        }
    }

    #[test]
    fn compact_visual_plan_draws_pill_and_shoulders_without_canvas_block() {
        let input = visual_input(NativePanelVisualDisplayMode::Compact);
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.hidden);
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                radius,
                ..
            } if *frame == input.compact_bar_frame
                && (*radius - crate::native_panel_core::COMPACT_PILL_RADIUS).abs() < 0.001
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::CompactShoulder {
                frame,
                side: crate::native_panel_renderer::visual_primitives::NativePanelVisualShoulderSide::Left,
                progress,
                ..
            } if *frame == input.left_shoulder_frame && *progress == 0.0
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::CompactShoulder {
                frame,
                side: crate::native_panel_renderer::visual_primitives::NativePanelVisualShoulderSide::Right,
                progress,
                ..
            } if *frame == input.right_shoulder_frame && *progress == 0.0
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, .. } if *frame == input.content_frame
        )));
    }

    #[test]
    fn compact_visual_plan_does_not_draw_completion_glow_as_large_panel_block() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.glow_visible = true;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, .. }
                if frame.width > input.compact_bar_frame.width
                    || frame.height > input.compact_bar_frame.height
        )));
    }

    #[test]
    fn compact_visual_plan_places_mascot_headline_and_count_text() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.completion_count = 0;
        input.mascot_pose = SceneMascotPose::Idle;
        input.compact_bar_frame.width = 253.0;
        input.compact_bar_frame.height = 37.0;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::MascotDot { center, radius, .. }
                if (center.x - (input.compact_bar_frame.x + 22.0)).abs() < 0.001
                    && (center.y - (input.compact_bar_frame.y + input.compact_bar_frame.height / 2.0)).abs() < 0.001
                    && (*radius - 11.0).abs() < 0.001
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text {
                origin,
                max_width,
                text,
                size,
                weight,
                alignment,
                ..
            } if text == "Codex ready"
                && ((origin.x + max_width / 2.0)
                    - (input.compact_bar_frame.x + input.compact_bar_frame.width / 2.0)).abs() < 0.001
                && (*max_width - crate::native_panel_core::resolve_estimated_text_width("Codex ready", 13.0)).abs() < 0.001
                && *size == 13
                && *weight == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Semibold
                && *alignment == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, weight, alignment, .. }
                if text == "1"
                    && (origin.x - (input.compact_bar_frame.x + 197.0)).abs() < 0.001
                    && *size == 15
                    && *weight == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Semibold
                    && *alignment == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Right
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, weight, alignment, .. }
                if text == "/"
                    && (origin.x - (input.compact_bar_frame.x + 217.0)).abs() < 0.001
                    && *size == 15
                    && *weight == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Semibold
                    && *alignment == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, weight, alignment, .. }
                if text == "3"
                    && (origin.x - (input.compact_bar_frame.x + 229.0)).abs() < 0.001
                    && *size == 15
                    && *weight == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Semibold
                    && *alignment == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Left
        )));
    }

    #[test]
    fn compact_visual_plan_uses_shared_active_count_marquee_frame() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.compact_bar_frame.width = 253.0;
        input.compact_bar_frame.height = 37.0;
        input.active_count = "23".to_string();
        input.active_count_elapsed_ms =
            ACTIVE_COUNT_SCROLL_HOLD_MS + (ACTIVE_COUNT_SCROLL_MOVE_MS / 2);
        let plan = resolve_native_panel_visual_plan(&input);
        let current = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text { origin, text, .. } if text == "2" => {
                    Some(*origin)
                }
                _ => None,
            })
            .expect("current active count digit");
        let next = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Text { origin, text, .. } if text == "3" => {
                    Some(*origin)
                }
                _ => None,
            })
            .expect("next active count digit");

        assert!(
            current.y < input.compact_bar_frame.y + compact_digit_y(input.compact_bar_frame.height)
        );
        assert!(next.y > current.y);
    }

    #[test]
    fn compact_visual_plan_positions_active_count_marquee_in_slot() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.compact_bar_frame.width = 253.0;
        input.compact_bar_frame.height = 37.0;
        input.active_count = "22".to_string();
        input.total_count = "3".to_string();
        input.active_count_elapsed_ms = 0;
        let plan = resolve_native_panel_visual_plan(&input);
        let compact_content = crate::native_panel_core::resolve_compact_bar_content_layout(
            crate::native_panel_core::CompactBarContentLayoutInput {
                bar_width: input.compact_bar_frame.width,
                bar_height: input.compact_bar_frame.height,
            },
        );

        let NativePanelVisualPrimitive::Text {
            origin,
            max_width,
            alignment,
            ..
        } = text_primitive(&plan, "2")
        else {
            panic!("active count text should be text primitive");
        };

        assert!(
            (origin.x
                - (input.compact_bar_frame.x
                    + compact_content.active_x
                    + ACTIVE_COUNT_TEXT_OFFSET_X))
                .abs()
                < 0.001
        );
        assert!((*max_width - crate::native_panel_core::ACTIVE_COUNT_TEXT_WIDTH).abs() < 0.001);
        assert_eq!(
            *alignment,
            crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Right
        );
    }

    #[test]
    fn compact_visual_plan_clips_headline_to_single_line() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.headline_text =
            "apps/desktop/src-tauri/src/windows_native_panel/host_runtime.rs\nsrc/native_panel_renderer"
                .to_string();
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, size, .. }
                if *size == 13
                    && !text.contains('\n')
                    && text.starts_with("apps/desktop")
                    && text.ends_with("...")
        )));
    }

    #[test]
    fn compact_visual_plan_keeps_headline_stable_when_expanded_actions_are_visible() {
        let mut compact = visual_input(NativePanelVisualDisplayMode::Compact);
        compact.completion_count = 0;
        compact.compact_bar_frame.width = 253.0;
        compact.action_buttons_visible = false;

        let mut expanded = compact.clone();
        expanded.display_mode = NativePanelVisualDisplayMode::Expanded;
        expanded.compact_bar_frame.x -= 15.0;
        expanded.compact_bar_frame.width = 283.0;
        expanded.action_buttons_visible = true;

        let compact_plan = resolve_native_panel_visual_plan(&compact);
        let expanded_plan = resolve_native_panel_visual_plan(&expanded);

        assert_eq!(
            headline_text_frame(&compact_plan),
            headline_text_frame(&expanded_plan)
        );
    }

    #[test]
    fn compact_visual_plan_places_mascot_face_in_mac_coordinate_order() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.completion_count = 0;
        input.mascot_pose = SceneMascotPose::Complete;
        let plan = resolve_native_panel_visual_plan(&input);
        let mascot_center = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::MascotDot { center, .. } => Some(*center),
                _ => None,
            })
            .expect("mascot primitive");

        let face_color =
            crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                255, 255, 255,
            );
        let eye_centers = plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Ellipse { frame, color } if *color == face_color => {
                    Some(frame.y + frame.height / 2.0)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(eye_centers.len(), 2);
        assert!(eye_centers.iter().all(|eye_y| *eye_y > mascot_center.y));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, color, .. }
                if *color == face_color
                    && frame.y + frame.height / 2.0 < mascot_center.y
        )));
    }

    #[test]
    fn compact_visual_plan_uses_shared_mascot_visual_frame_for_blink() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.completion_count = 0;
        input.mascot_pose = SceneMascotPose::Idle;
        let mut open_input = input.clone();
        open_input.mascot_elapsed_ms = 0;
        input.mascot_elapsed_ms = 4535;
        let open_plan = resolve_native_panel_visual_plan(&open_input);
        let plan = resolve_native_panel_visual_plan(&input);
        let face_color =
            crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                255, 255, 255,
            );
        let open_eye_heights = open_plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Ellipse { frame, color } if *color == face_color => {
                    Some(frame.height)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let eye_heights = plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::Ellipse { frame, color } if *color == face_color => {
                    Some(frame.height)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(eye_heights.len(), 2);
        assert_eq!(open_eye_heights.len(), 2);
        assert!(
            eye_heights
                .iter()
                .zip(open_eye_heights.iter())
                .all(|(closed, open)| *closed < *open)
        );
    }

    #[test]
    fn compact_visual_plan_uses_mac_style_non_uniform_mascot_motion() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.completion_count = 0;
        input.mascot_pose = SceneMascotPose::Running;
        input.mascot_elapsed_ms = 100;
        let plan = resolve_native_panel_visual_plan(&input);

        let NativePanelVisualPrimitive::MascotDot {
            center,
            radius,
            scale_x,
            scale_y,
            ..
        } = plan
            .primitives
            .iter()
            .find(|primitive| matches!(primitive, NativePanelVisualPrimitive::MascotDot { .. }))
            .expect("mascot primitive")
        else {
            panic!("mascot primitive should be MascotDot");
        };

        let compact_content = crate::native_panel_core::resolve_compact_bar_content_layout(
            crate::native_panel_core::CompactBarContentLayoutInput {
                bar_width: input.compact_bar_frame.width,
                bar_height: input.compact_bar_frame.height,
            },
        );
        assert!(center.x > input.compact_bar_frame.x + compact_content.mascot_center_x);
        assert!((*scale_x - *scale_y).abs() > 0.02);
        assert!((*radius - 11.0).abs() < 0.001);
    }

    #[test]
    fn compact_visual_plan_draws_mac_sleepy_and_wake_angry_mascot_details() {
        let mut sleepy_input = visual_input(NativePanelVisualDisplayMode::Compact);
        sleepy_input.completion_count = 0;
        sleepy_input.mascot_pose = SceneMascotPose::Sleepy;
        sleepy_input.mascot_elapsed_ms = 4550;
        let sleepy_plan = resolve_native_panel_visual_plan(&sleepy_input);

        assert!(sleepy_plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Z"
        )));

        let mut wake_input = visual_input(NativePanelVisualDisplayMode::Compact);
        wake_input.completion_count = 0;
        wake_input.mascot_pose = SceneMascotPose::WakeAngry;
        wake_input.mascot_elapsed_ms = 0;
        let wake_plan = resolve_native_panel_visual_plan(&wake_input);

        assert!(wake_plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::MascotDot { scale_x, scale_y, .. }
                if *scale_x > 1.04 && *scale_y < 0.97
        )));
    }

    #[test]
    fn compact_visual_plan_places_completion_badge_on_mascot_and_keeps_active_count() {
        let input = visual_input(NativePanelVisualDisplayMode::Compact);
        let plan = resolve_native_panel_visual_plan(&input);
        let mascot_center = plan
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                NativePanelVisualPrimitive::MascotDot { center, .. } => Some(*center),
                _ => None,
            })
            .expect("mascot primitive");

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, .. }
                if text == "1"
                    && (origin.x - (input.compact_bar_frame.x + 184.0)).abs() < 0.001
                    && *size == 15
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, .. }
                if text == "/"
                    && (origin.x - (input.compact_bar_frame.x + 204.0)).abs() < 0.001
                    && *size == 15
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, .. }
                if text == "3"
                    && (origin.x - (input.compact_bar_frame.x + 216.0)).abs() < 0.001
                    && *size == 15
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                radius,
                color,
            } if (frame.x - (mascot_center.x + 5.0)).abs() < 0.001
                && (frame.y - (mascot_center.y + 1.0)).abs() < 0.001
                && (frame.width - 15.0).abs() < 0.001
                && (frame.height - 15.0).abs() < 0.001
                && (*radius - 7.5).abs() < 0.001
                && *color == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(240, 255, 246)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                radius,
                color,
            } if (frame.x - (mascot_center.x + 6.0)).abs() < 0.001
                && (frame.y - (mascot_center.y + 2.0)).abs() < 0.001
                && (frame.width - 13.0).abs() < 0.001
                && (frame.height - 13.0).abs() < 0.001
                && (*radius - 6.5).abs() < 0.001
                && *color == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(102, 222, 145)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { origin, text, size, .. }
                if text == "2"
                    && (origin.x - (mascot_center.x + 10.0)).abs() < 0.001
                    && (origin.y - (mascot_center.y + 0.5)).abs() < 0.001
                    && *size == 8
        )));
    }

    #[test]
    fn compact_visual_plan_draws_message_bubble_without_completion_count_text() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Compact);
        input.completion_count = 2;
        input.mascot_pose = SceneMascotPose::MessageBubble;
        input.mascot_elapsed_ms = 500;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                color,
                ..
            } if *color == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(102, 222, 145)
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, size, .. }
                if text == "2" && *size == 8
        )));
    }

    #[test]
    fn compact_visual_plan_animates_message_bubble_like_mac_pop() {
        let mut early = visual_input(NativePanelVisualDisplayMode::Compact);
        early.completion_count = 0;
        early.mascot_pose = SceneMascotPose::MessageBubble;
        early.mascot_elapsed_ms = 120;
        let early_plan = resolve_native_panel_visual_plan(&early);

        let mut open = early.clone();
        open.mascot_elapsed_ms = 500;
        let open_plan = resolve_native_panel_visual_plan(&open);

        let early_bubble = mascot_green_bubble_frame(&early_plan).expect("early bubble");
        let open_bubble = mascot_green_bubble_frame(&open_plan).expect("open bubble");
        assert!(open_bubble.y > early_bubble.y);
    }

    #[test]
    fn compact_visual_plan_animates_sleep_label_like_mac_rise() {
        let mut early = visual_input(NativePanelVisualDisplayMode::Compact);
        early.mascot_pose = SceneMascotPose::Sleepy;
        early.mascot_elapsed_ms = 200;
        let early_plan = resolve_native_panel_visual_plan(&early);

        let mut risen = early.clone();
        risen.mascot_elapsed_ms = 1500;
        let risen_plan = resolve_native_panel_visual_plan(&risen);

        let early_origin = mascot_sleep_label_origin(&early_plan).expect("early sleep label");
        let risen_origin = mascot_sleep_label_origin(&risen_plan).expect("risen sleep label");
        assert!(risen_origin.x > early_origin.x);
        assert!(risen_origin.y > early_origin.y);
    }

    #[test]
    fn expanded_visual_plan_hides_completion_badge_even_when_completion_count_is_cached() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.completion_count = 2;
        input.mascot_pose = SceneMascotPose::Complete;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                color,
                ..
            } if *color == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(102, 222, 145)
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, size, .. }
                if text == "2" && *size == 8
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_message_bubble_for_status_message_state() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.completion_count = 2;
        input.mascot_pose = SceneMascotPose::MessageBubble;
        input.mascot_elapsed_ms = 500;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                color,
                ..
            } if *color == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(102, 222, 145)
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, size, .. }
                if text == "2" && *size == 8
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_card_content_from_shared_inputs() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 180.0;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, weight, .. }
                if text == "Settings"
                    && *weight == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Semibold
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "EchoIsland v0.2.0"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Mute Sound"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Off"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Done"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Task complete"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Codex"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, radius, .. }
                if frame.width > 20.0
                    && (frame.height - 22.0).abs() < 0.001
                    && (*radius - 11.0).abs() < 0.001
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_settings_rows_as_surfaces_with_value_badges() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 180.0;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, radius, color }
                if (frame.height - crate::native_panel_core::SETTINGS_ROW_HEIGHT).abs() < 0.001
                    && (*radius - 8.0).abs() < 0.001
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(50, 84, 61)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, radius, color }
                if (frame.height - 18.0).abs() < 0.001
                    && (*radius - 9.0).abs() < 0.001
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(46, 68, 54)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, size, alignment, .. }
                if text == "Off"
                    && *size == 10
                    && *alignment
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(104, 222, 145)
        )));
    }

    #[test]
    fn expanded_visual_plan_matches_mac_session_card_density_and_clips_long_body() {
        let long_body =
            "abcdefghijklmnopqrstuvwxyz abcdefghijklmnopqrstuvwxyz abcdefghijklmnopqrstuvwxyz";
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 180.0;
        input.cards[1].title = "Finished".to_string();
        input.cards[1].body = Some(long_body.to_string());
        let plan = resolve_native_panel_visual_plan(&input);

        let NativePanelVisualPrimitive::Text { size, weight, .. } =
            text_primitive(&plan, "Finished")
        else {
            panic!("expected title text");
        };
        assert_eq!(*size, 12);
        assert_eq!(
            *weight,
            crate::native_panel_renderer::visual_primitives::NativePanelVisualTextWeight::Semibold
        );

        let NativePanelVisualPrimitive::Text { size, .. } = plan
            .primitives
            .iter()
            .find(|primitive| {
                matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text.starts_with("#abcdef"))
            })
            .expect("expected meta text")
        else {
            panic!("expected meta text");
        };
        assert_eq!(*size, 9);

        let NativePanelVisualPrimitive::Text { size, .. } = text_primitive(&plan, "Codex") else {
            panic!("expected source badge text");
        };
        assert_eq!(*size, 10);
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, radius, .. }
                if (frame.height - 22.0).abs() < 0.001
                    && (*radius - 11.0).abs() < 0.001
        )));

        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == long_body
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, size, .. }
                if text.starts_with("abcdefghijklmnopqrstuvwxyz")
                    && text.contains('\n')
                    && text.lines().count() == 2
                    && *size == 10
        )));
    }

    #[test]
    fn expanded_visual_plan_uses_mac_chat_line_tones_for_default_and_completion_cards() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 180.0;
        input.card_stack_content_height = 76.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Default,
            title: "EchoIsland".to_string(),
            subtitle: Some("#c1d5-7 · now".to_string()),
            body: Some("Assistant reply".to_string()),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Idle".to_string(),
                emphasized: false,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: false,
            }),
            body_prefix: Some("$".to_string()),
            body_lines: Vec::new(),
                    action_hint: None,
            rows: Vec::new(),
            height: 76.0,
            collapsed_height: 64.0,
            compact: false,
            removing: false,
        }];
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "$"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(217, 120, 87)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "Assistant reply"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(177, 183, 194)
        )));

        input.cards[0].style =
            crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Completion;
        let plan = resolve_native_panel_visual_plan(&input);
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "$"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(104, 222, 145)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, color, .. }
                if *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(37, 37, 41)
                    && frame.height > 60.0
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, color, .. }
                if *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(46, 79, 61)
                    && frame.height > 60.0
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect { frame, color, .. }
                if *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(30, 40, 38)
                    && frame.height > 60.0
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_session_reply_and_prompt_lines() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 112.0;
        input.card_stack_content_height = 112.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Default,
            title: "EchoIsland".to_string(),
            subtitle: Some("#c1d5-7 路 7m".to_string()),
            body: Some("Assistant reply".to_string() + "\n" + "User prompt"),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Idle".to_string(),
                emphasized: false,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: false,
            }),
            body_prefix: Some("$>".to_string()),
            body_lines: Vec::new(),
                    action_hint: None,
            rows: Vec::new(),
            height: 112.0,
            collapsed_height: 64.0,
            compact: false,
            removing: false,
        }];
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == "$"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(217, 120, 87)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, color, .. }
                if text == ">"
                    && *color
                        == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(104, 222, 145)
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Assistant reply"
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "User prompt"
        )));
    }

    #[test]
    fn expanded_visual_plan_keeps_partially_clipped_card_shell_without_content() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 42.0;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                radius,
                ..
            } if (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. }
                if text == "Settings"
                    || text == "Mute Sound"
                    || text == "Done"
                    || text == "Task complete"
        )));
    }

    #[test]
    fn expanded_visual_plan_anchors_overflowing_card_stack_to_visible_bottom() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 180.0;
        input.card_stack_content_height = 260.0;
        input.cards[0].height = 180.0;
        input.cards[0].collapsed_height = 120.0;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                radius,
                ..
            } if (frame.y - (input.shell_frame.y + input.card_stack_frame.y)).abs() < 0.001
                && (frame.height - input.cards[0].height).abs() < 0.001
                && (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Settings"
        )));
    }

    #[test]
    fn expanded_visual_plan_does_not_relayout_content_from_bottom_clipped_card() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 60.0;
        input.card_stack_content_height = 100.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Completion,
            title: "Done".to_string(),
            subtitle: Some("#abcdef now".to_string()),
            body: Some("Task complete".to_string()),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: "Done".to_string(),
                emphasized: true,
            }),
            source_badge: Some(NativePanelVisualCardBadgeInput {
                text: "Codex".to_string(),
                emphasized: false,
            }),
            body_prefix: Some("$".to_string()),
            body_lines: Vec::new(),
                    action_hint: None,
            rows: Vec::new(),
            height: 100.0,
            collapsed_height: 52.0,
            compact: false,
            removing: false,
        }];
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Done"
        )));
        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "Task complete"
        )));
    }

    #[test]
    fn expanded_visual_plan_centers_empty_card_prompt() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_content_height = 84.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Empty,
            title: "No active sessions".to_string(),
            subtitle: None,
            body: Some("EchoIsland is watching for new activity.".to_string()),
            badge: None,
            source_badge: None,
            body_prefix: None,
            body_lines: Vec::new(),
                    action_hint: None,
            rows: Vec::new(),
            height: 84.0,
            collapsed_height: 34.0,
            compact: true,
            removing: false,
        }];
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text {
                text,
                origin,
                max_width,
                size,
                alignment,
                ..
            } if text == "No active sessions"
                && (origin.x - (input.shell_frame.x + input.card_stack_frame.x)).abs() < 0.001
                && (*max_width - input.card_stack_frame.width).abs() < 0.001
                && *size == 12
                && *alignment
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualTextAlignment::Center
        )));
    }

    #[test]
    fn expanded_visual_plan_keeps_single_empty_card_when_viewport_is_shorter_than_empty_height() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 70.0;
        input.card_stack_content_height = 70.0;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Empty,
            title: "No active sessions".to_string(),
            subtitle: None,
            body: Some("EchoIsland is watching for new activity.".to_string()),
            badge: None,
            source_badge: None,
            body_prefix: None,
            body_lines: Vec::new(),
                    action_hint: None,
            rows: Vec::new(),
            height: crate::native_panel_core::EMPTY_CARD_HEIGHT,
            collapsed_height: 34.0,
            compact: true,
            removing: false,
        }];
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                radius,
                ..
            } if (frame.height - input.card_stack_frame.height).abs() < 0.001
                && (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "No active sessions"
        )));
    }

    #[test]
    fn expanded_visual_plan_keeps_single_empty_card_after_stable_clipped_reveal() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = 70.0;
        input.card_stack_content_height = crate::native_panel_core::EMPTY_CARD_HEIGHT;
        input.cards = vec![NativePanelVisualCardInput {
            style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Empty,
            title: "No active sessions".to_string(),
            subtitle: None,
            body: Some("EchoIsland is watching for new activity.".to_string()),
            badge: None,
            source_badge: None,
            body_prefix: None,
            body_lines: Vec::new(),
                    action_hint: None,
            rows: Vec::new(),
            height: crate::native_panel_core::EMPTY_CARD_HEIGHT,
            collapsed_height: 34.0,
            compact: true,
            removing: false,
        }];
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::Text { text, .. } if text == "No active sessions"
        )));
    }

    #[test]
    fn expanded_visual_plan_does_not_fill_transparent_canvas_background() {
        let input = visual_input(NativePanelVisualDisplayMode::Expanded);
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                color,
                ..
            } if *frame == input.content_frame
                && *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                        18, 18, 22,
                    )
        )));
    }

    #[test]
    fn expanded_visual_plan_keeps_shell_color_stable_with_compact_island() {
        let input = visual_input(NativePanelVisualDisplayMode::Expanded);
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                radius,
                color,
            } if *frame == input.shell_frame
                && (*radius - crate::native_panel_core::EXPANDED_PANEL_RADIUS).abs() < 0.001
                && *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                        12, 12, 15,
                    )
        )));
    }

    #[test]
    fn expanded_visual_plan_does_not_fill_transparent_canvas_with_completion_glow() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.glow_visible = true;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                frame,
                color,
                ..
            } if (frame.width - (input.content_frame.width - 8.0)).abs() < 0.001
                && (frame.height - (input.content_frame.height - 8.0)).abs() < 0.001
                && *color
                    == crate::native_panel_renderer::visual_primitives::NativePanelVisualColor::rgb(
                        42, 156, 92,
                    )
        )));
    }

    #[test]
    fn expanded_visual_plan_draws_card_shells_from_shared_stack_layout() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.88;
        input.card_stack_frame.height = input.card_stack_content_height;
        let plan = resolve_native_panel_visual_plan(&input);

        let card_shells = plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::RoundRect { frame, radius, .. }
                    if frame.width > 80.0
                        && (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001 =>
                {
                    Some(*frame)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            card_shells,
            vec![
                PanelRect {
                    x: input.shell_frame.x + input.card_stack_frame.x,
                    y: input.shell_frame.y + input.card_stack_frame.y + 88.0,
                    width: input.card_stack_frame.width,
                    height: 92.0,
                },
                PanelRect {
                    x: input.shell_frame.x + input.card_stack_frame.x,
                    y: input.shell_frame.y + input.card_stack_frame.y,
                    width: input.card_stack_frame.width,
                    height: 76.0,
                },
            ]
        );
    }

    #[test]
    fn expanded_visual_plan_reveals_card_shells_with_staggered_collapsed_height() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.44;
        input.card_stack_frame.height = input.card_stack_content_height;
        let plan = resolve_native_panel_visual_plan(&input);

        let card_shells = plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::RoundRect { frame, radius, .. }
                    if frame.width > 80.0
                        && (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001 =>
                {
                    Some(*frame)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(card_shells.len(), 2);
        assert!(card_shells[0].height > input.cards[0].collapsed_height);
        assert!(card_shells[0].height < input.cards[0].height);
        assert!(card_shells[1].height > input.cards[1].collapsed_height);
        assert!(card_shells[1].height < input.cards[1].height);
    }

    #[test]
    fn expanded_visual_plan_reveals_card_shells_from_centered_narrow_width() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.44;
        input.card_stack_frame.height = input.card_stack_content_height;
        let plan = resolve_native_panel_visual_plan(&input);

        let card_shells = plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::RoundRect { frame, radius, .. }
                    if frame.width > 80.0
                        && (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001 =>
                {
                    Some(*frame)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let full_x = input.shell_frame.x + input.card_stack_frame.x;
        assert_eq!(card_shells.len(), 2);
        assert!(card_shells[0].x > full_x);
        assert!(card_shells[0].width < input.card_stack_frame.width);
        assert!(card_shells[1].x > full_x);
        assert!(card_shells[1].width < input.card_stack_frame.width);
    }

    #[test]
    fn expanded_visual_plan_hides_card_shells_before_card_reveal_progress() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.0;
        let plan = resolve_native_panel_visual_plan(&input);

        assert!(!plan.primitives.iter().any(|primitive| matches!(
            primitive,
            NativePanelVisualPrimitive::RoundRect {
                radius,
                ..
            } if (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001
        )));
    }

    #[test]
    fn expanded_visual_plan_hides_fully_clipped_card_shells_during_reveal() {
        let mut input = visual_input(NativePanelVisualDisplayMode::Expanded);
        input.separator_visibility = 0.44;
        input.card_stack_frame.height = 4.0;
        let plan = resolve_native_panel_visual_plan(&input);

        let card_shells = plan
            .primitives
            .iter()
            .filter_map(|primitive| match primitive {
                NativePanelVisualPrimitive::RoundRect { frame, radius, .. }
                    if (*radius - crate::native_panel_core::CARD_RADIUS).abs() < 0.001 =>
                {
                    Some(*frame)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(card_shells.is_empty());
    }
}
