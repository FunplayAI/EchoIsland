use crate::{
    native_panel_core::{
        ExpandedSurface, PanelPoint, PanelRect, StatusQueuePayload, completion_preview_text,
        display_snippet, format_source, format_status, session_meta_line, session_prompt_preview,
        session_reply_preview, session_title, short_session_id,
    },
    native_panel_scene::{SceneCard, SceneMascotPose},
};

use super::descriptors::{NativePanelEdgeAction, NativePanelHostWindowState};
use super::visual_primitives::{
    NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
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
    pub(crate) content_frame: PanelRect,
    pub(crate) shell_frame: PanelRect,
    pub(crate) headline_text: String,
    pub(crate) headline_emphasized: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) cards_visible: bool,
    pub(crate) card_count: usize,
    pub(crate) cards: Vec<NativePanelVisualCardInput>,
    pub(crate) glow_visible: bool,
    pub(crate) action_buttons_visible: bool,
    pub(crate) action_buttons: Vec<NativePanelVisualActionButtonInput>,
    pub(crate) completion_count: usize,
    pub(crate) mascot_pose: SceneMascotPose,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelVisualCardInput {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) badge: Option<NativePanelVisualCardBadgeInput>,
    pub(crate) rows: Vec<NativePanelVisualCardRowInput>,
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

    primitives.push(NativePanelVisualPrimitive::RoundRect {
        frame: panel_frame,
        radius: panel_frame.height.min(64.0) / 2.0,
        color: NativePanelVisualColor::rgb(18, 18, 22),
    });

    if input.glow_visible {
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: inset_rect(panel_frame, 4.0),
            radius: panel_frame.height.min(64.0) / 2.0,
            color: NativePanelVisualColor::rgb(42, 156, 92),
        });
    }

    if input.display_mode == NativePanelVisualDisplayMode::Expanded {
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: shell_frame,
            radius: 24.0,
            color: NativePanelVisualColor::rgb(24, 24, 29),
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
    }

    primitives.push(NativePanelVisualPrimitive::Text {
        origin: PanelPoint {
            x: compact_frame.x + 56.0,
            y: compact_frame.y + (compact_frame.height - 16.0) / 2.0,
        },
        max_width: (compact_frame.width - 124.0).max(32.0),
        text: input.headline_text.clone(),
        color: if input.headline_emphasized {
            NativePanelVisualColor::rgb(255, 255, 255)
        } else {
            NativePanelVisualColor::rgb(214, 214, 220)
        },
        size: 14,
    });

    if input.completion_count > 0 {
        let badge = PanelRect {
            x: compact_frame.x + compact_frame.width - 46.0,
            y: compact_frame.y + compact_frame.height - 24.0,
            width: 28.0,
            height: 18.0,
        };
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: badge,
            radius: 9.0,
            color: NativePanelVisualColor::rgb(36, 188, 113),
        });
        primitives.push(NativePanelVisualPrimitive::Text {
            origin: PanelPoint {
                x: badge.x + 8.0,
                y: badge.y + 2.0,
            },
            max_width: badge.width - 8.0,
            text: input.completion_count.to_string(),
            color: NativePanelVisualColor::rgb(255, 255, 255),
            size: 12,
        });
    }

    if input.cards_visible {
        push_card_primitives(&mut primitives, input, content_frame);
    }

    if input.action_buttons_visible {
        for button in &input.action_buttons {
            push_action_button_icon(&mut primitives, button.action, button.frame);
        }
    }

    push_mascot_primitives(
        &mut primitives,
        PanelPoint {
            x: compact_frame.x + 30.0,
            y: compact_frame.y + compact_frame.height / 2.0,
        },
        11.0,
        input.mascot_pose,
    );

    NativePanelVisualPlan {
        hidden: false,
        primitives,
    }
}

pub(crate) fn native_panel_visual_card_input_from_scene_card(
    card: &SceneCard,
) -> NativePanelVisualCardInput {
    match card {
        SceneCard::Settings {
            title,
            version,
            rows,
        } => NativePanelVisualCardInput {
            title: title.clone(),
            subtitle: Some(version.text.clone()),
            body: None,
            badge: None,
            rows: rows
                .iter()
                .map(|row| NativePanelVisualCardRowInput {
                    title: row.title.clone(),
                    value: row.value.text.clone(),
                    active: row.value.emphasized,
                })
                .collect(),
        },
        SceneCard::PendingPermission { pending, count } => NativePanelVisualCardInput {
            title: "Approval Required".to_string(),
            subtitle: Some(format_source(&pending.source)),
            body: display_snippet(pending.tool_description.as_deref(), 78)
                .or_else(|| Some("Waiting for your approval".to_string())),
            badge: (*count > 1).then(|| NativePanelVisualCardBadgeInput {
                text: count.to_string(),
                emphasized: true,
            }),
            rows: Vec::new(),
        },
        SceneCard::PendingQuestion { pending, count } => NativePanelVisualCardInput {
            title: pending
                .header
                .clone()
                .unwrap_or_else(|| "Question".to_string()),
            subtitle: Some(format_source(&pending.source)),
            body: display_snippet(Some(&pending.text), 82)
                .or_else(|| Some("Waiting for your answer".to_string())),
            badge: (*count > 1).then(|| NativePanelVisualCardBadgeInput {
                text: count.to_string(),
                emphasized: true,
            }),
            rows: Vec::new(),
        },
        SceneCard::PromptAssist { session } => NativePanelVisualCardInput {
            title: session_title(session),
            subtitle: Some(format!("{} · Prompt", format_source(&session.source))),
            body: session_prompt_preview(session)
                .or_else(|| display_snippet(session.tool_description.as_deref(), 78)),
            badge: None,
            rows: Vec::new(),
        },
        SceneCard::Session {
            session,
            title,
            status,
            snippet,
        } => NativePanelVisualCardInput {
            title: if title.trim().is_empty() {
                session_title(session)
            } else {
                title.clone()
            },
            subtitle: Some(session_meta_line(session)),
            body: snippet
                .clone()
                .or_else(|| session_reply_preview(session))
                .or_else(|| session_prompt_preview(session)),
            badge: Some(NativePanelVisualCardBadgeInput {
                text: if status.text.trim().is_empty() {
                    format_status(&session.status)
                } else {
                    status.text.clone()
                },
                emphasized: status.emphasized,
            }),
            rows: Vec::new(),
        },
        SceneCard::StatusApproval { item } => match &item.payload {
            StatusQueuePayload::Approval(pending) => NativePanelVisualCardInput {
                title: "Approval Required".to_string(),
                subtitle: Some(format!(
                    "#{} · Approval",
                    short_session_id(&item.session_id)
                )),
                body: display_snippet(pending.tool_description.as_deref(), 78)
                    .or_else(|| Some("Waiting for your approval".to_string())),
                badge: Some(NativePanelVisualCardBadgeInput {
                    text: format_source(&pending.source),
                    emphasized: item.is_live,
                }),
                rows: Vec::new(),
            },
            StatusQueuePayload::Completion(session) => NativePanelVisualCardInput {
                title: session_title(session),
                subtitle: Some(session_meta_line(session)),
                body: Some(completion_preview_text(session)),
                badge: Some(NativePanelVisualCardBadgeInput {
                    text: "Done".to_string(),
                    emphasized: true,
                }),
                rows: Vec::new(),
            },
        },
        SceneCard::StatusCompletion { item } => match &item.payload {
            StatusQueuePayload::Completion(session) => NativePanelVisualCardInput {
                title: session_title(session),
                subtitle: Some(session_meta_line(session)),
                body: Some(completion_preview_text(session)),
                badge: Some(NativePanelVisualCardBadgeInput {
                    text: "Done".to_string(),
                    emphasized: true,
                }),
                rows: Vec::new(),
            },
            StatusQueuePayload::Approval(pending) => NativePanelVisualCardInput {
                title: "Approval Required".to_string(),
                subtitle: Some(format_source(&pending.source)),
                body: display_snippet(pending.tool_description.as_deref(), 78),
                badge: None,
                rows: Vec::new(),
            },
        },
        SceneCard::Empty => NativePanelVisualCardInput {
            title: "No active sessions".to_string(),
            subtitle: None,
            body: Some("EchoIsland is watching for new activity.".to_string()),
            badge: None,
            rows: Vec::new(),
        },
    }
}

fn push_action_button_icon(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    action: NativePanelEdgeAction,
    frame: PanelRect,
) {
    match action {
        NativePanelEdgeAction::Settings => push_settings_icon(primitives, frame),
        NativePanelEdgeAction::Quit => push_quit_icon(primitives, frame),
    }
}

fn push_settings_icon(primitives: &mut Vec<NativePanelVisualPrimitive>, frame: PanelRect) {
    let color = NativePanelVisualColor::rgb(156, 156, 168);
    let center = rect_center(frame);
    let radius = frame.width.min(frame.height) * 0.18;
    primitives.push(NativePanelVisualPrimitive::Ellipse {
        frame: centered_rect(center, radius, radius),
        color,
    });
    for (dx, dy) in [
        (0.0, -1.0),
        (0.0, 1.0),
        (-1.0, 0.0),
        (1.0, 0.0),
        (-0.7, -0.7),
        (0.7, -0.7),
        (-0.7, 0.7),
        (0.7, 0.7),
    ] {
        let inner = frame.width.min(frame.height) * 0.28;
        let outer = frame.width.min(frame.height) * 0.42;
        primitives.push(NativePanelVisualPrimitive::StrokeLine {
            from: PanelPoint {
                x: center.x + dx * inner,
                y: center.y + dy * inner,
            },
            to: PanelPoint {
                x: center.x + dx * outer,
                y: center.y + dy * outer,
            },
            color,
            width: 2,
        });
    }
}

fn push_quit_icon(primitives: &mut Vec<NativePanelVisualPrimitive>, frame: PanelRect) {
    let color = NativePanelVisualColor::rgb(237, 88, 88);
    let center = rect_center(frame);
    let radius = frame.width.min(frame.height) * 0.34;
    primitives.push(NativePanelVisualPrimitive::StrokeLine {
        from: PanelPoint {
            x: center.x,
            y: center.y - radius * 0.95,
        },
        to: PanelPoint {
            x: center.x,
            y: center.y - radius * 0.05,
        },
        color,
        width: 3,
    });
    for (from_angle, to_angle) in [(220.0, 270.0), (270.0, 320.0), (40.0, 90.0), (90.0, 140.0)] {
        primitives.push(NativePanelVisualPrimitive::StrokeLine {
            from: point_on_circle(center, radius, from_angle),
            to: point_on_circle(center, radius, to_angle),
            color,
            width: 3,
        });
    }
}

fn push_mascot_primitives(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    center: PanelPoint,
    radius: f64,
    pose: SceneMascotPose,
) {
    primitives.push(NativePanelVisualPrimitive::MascotDot {
        center,
        radius,
        pose,
    });
    let eye_color = NativePanelVisualColor::rgb(54, 45, 18);
    for x_offset in [-radius * 0.34, radius * 0.34] {
        primitives.push(NativePanelVisualPrimitive::Ellipse {
            frame: centered_rect(
                PanelPoint {
                    x: center.x + x_offset,
                    y: center.y - radius * 0.18,
                },
                radius * 0.09,
                radius * 0.14,
            ),
            color: eye_color,
        });
    }
    if pose == SceneMascotPose::Complete {
        primitives.push(NativePanelVisualPrimitive::StrokeLine {
            from: PanelPoint {
                x: center.x - radius * 0.36,
                y: center.y + radius * 0.26,
            },
            to: PanelPoint {
                x: center.x,
                y: center.y + radius * 0.40,
            },
            color: eye_color,
            width: 2,
        });
        primitives.push(NativePanelVisualPrimitive::StrokeLine {
            from: PanelPoint {
                x: center.x,
                y: center.y + radius * 0.40,
            },
            to: PanelPoint {
                x: center.x + radius * 0.36,
                y: center.y + radius * 0.26,
            },
            color: eye_color,
            width: 2,
        });
    }
}

fn push_card_primitives(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    input: &NativePanelVisualPlanInput,
    content_frame: PanelRect,
) {
    let card_inset_x = 14.0;
    let mut cursor_y = content_frame.y + 14.0;
    let cards = if input.cards.is_empty() {
        Vec::new()
    } else {
        input.cards.iter().collect::<Vec<_>>()
    };

    if cards.is_empty() {
        push_placeholder_card_primitives(primitives, input, content_frame, cursor_y, card_inset_x);
        return;
    }

    for card in cards.into_iter().take(3) {
        let remaining_height = (content_frame.y + content_frame.height - cursor_y - 14.0).max(0.0);
        if remaining_height <= 0.0 {
            break;
        }
        let preferred_height = if card.rows.is_empty() {
            72.0
        } else {
            (56.0 + card.rows.len() as f64 * 30.0).min(remaining_height)
        };
        let card_frame = PanelRect {
            x: content_frame.x + card_inset_x,
            y: cursor_y,
            width: (content_frame.width - card_inset_x * 2.0).max(0.0),
            height: preferred_height.min(remaining_height).max(44.0),
        };
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: card_frame,
            radius: 18.0,
            color: NativePanelVisualColor::rgb(32, 32, 38),
        });
        push_text(
            primitives,
            card_frame.x + 14.0,
            card_frame.y + 12.0,
            (card_frame.width - 28.0).max(20.0),
            card.title.clone(),
            NativePanelVisualColor::rgb(245, 246, 250),
            13,
        );

        if let Some(badge) = &card.badge {
            let badge_width = (badge.text.chars().count() as f64 * 7.0 + 16.0).max(28.0);
            let badge_frame = PanelRect {
                x: card_frame.x + card_frame.width - badge_width - 14.0,
                y: card_frame.y + 10.0,
                width: badge_width,
                height: 18.0,
            };
            primitives.push(NativePanelVisualPrimitive::RoundRect {
                frame: badge_frame,
                radius: 9.0,
                color: if badge.emphasized {
                    NativePanelVisualColor::rgb(39, 159, 96)
                } else {
                    NativePanelVisualColor::rgb(58, 58, 66)
                },
            });
            push_text(
                primitives,
                badge_frame.x + 8.0,
                badge_frame.y + 2.0,
                badge_frame.width - 10.0,
                badge.text.clone(),
                NativePanelVisualColor::rgb(255, 255, 255),
                11,
            );
        }

        if let Some(subtitle) = &card.subtitle {
            push_text(
                primitives,
                card_frame.x + 14.0,
                card_frame.y + 31.0,
                (card_frame.width - 28.0).max(20.0),
                subtitle.clone(),
                NativePanelVisualColor::rgb(156, 160, 172),
                11,
            );
        }

        if let Some(body) = &card.body {
            push_text(
                primitives,
                card_frame.x + 14.0,
                card_frame.y + 49.0,
                (card_frame.width - 28.0).max(20.0),
                body.clone(),
                NativePanelVisualColor::rgb(206, 209, 218),
                11,
            );
        }

        for (index, row) in card.rows.iter().take(5).enumerate() {
            let row_y = card_frame.y + 44.0 + index as f64 * 29.0;
            push_text(
                primitives,
                card_frame.x + 14.0,
                row_y,
                (card_frame.width * 0.52).max(40.0),
                row.title.clone(),
                NativePanelVisualColor::rgb(213, 216, 224),
                11,
            );
            push_text(
                primitives,
                card_frame.x + card_frame.width * 0.58,
                row_y,
                (card_frame.width * 0.34).max(40.0),
                row.value.clone(),
                if row.active {
                    NativePanelVisualColor::rgb(86, 214, 135)
                } else {
                    NativePanelVisualColor::rgb(154, 158, 170)
                },
                11,
            );
            if row.active {
                primitives.push(NativePanelVisualPrimitive::RoundRect {
                    frame: PanelRect {
                        x: card_frame.x + card_frame.width - 36.0,
                        y: row_y + 1.0,
                        width: 22.0,
                        height: 12.0,
                    },
                    radius: 6.0,
                    color: NativePanelVisualColor::rgb(39, 159, 96),
                });
                primitives.push(NativePanelVisualPrimitive::Ellipse {
                    frame: PanelRect {
                        x: card_frame.x + card_frame.width - 25.0,
                        y: row_y + 3.0,
                        width: 8.0,
                        height: 8.0,
                    },
                    color: NativePanelVisualColor::rgb(255, 255, 255),
                });
            }
        }

        cursor_y += card_frame.height + 10.0;
    }
}

fn push_placeholder_card_primitives(
    primitives: &mut Vec<NativePanelVisualPrimitive>,
    input: &NativePanelVisualPlanInput,
    content_frame: PanelRect,
    cursor_y: f64,
    card_inset_x: f64,
) {
    for index in 0..input.card_count.min(3) {
        let y = cursor_y + index as f64 * 34.0;
        primitives.push(NativePanelVisualPrimitive::RoundRect {
            frame: PanelRect {
                x: content_frame.x + card_inset_x,
                y,
                width: (content_frame.width - card_inset_x * 2.0).max(0.0),
                height: 24.0,
            },
            radius: 12.0,
            color: NativePanelVisualColor::rgb(42, 42, 49),
        });
    }
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
    primitives.push(NativePanelVisualPrimitive::Text {
        origin: PanelPoint { x, y },
        max_width,
        text,
        color,
        size,
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

#[cfg(test)]
mod tests {
    use super::{
        NativePanelVisualActionButtonInput, NativePanelVisualCardBadgeInput,
        NativePanelVisualCardInput, NativePanelVisualCardRowInput, NativePanelVisualDisplayMode,
        NativePanelVisualPlanInput, resolve_native_panel_visual_plan,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelRect},
        native_panel_renderer::{
            descriptors::{NativePanelEdgeAction, NativePanelHostWindowState},
            visual_primitives::NativePanelVisualPrimitive,
        },
        native_panel_scene::SceneMascotPose,
    };

    fn visual_input(display_mode: NativePanelVisualDisplayMode) -> NativePanelVisualPlanInput {
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
                x: 40.0,
                y: 12.0,
                width: 240.0,
                height: 36.0,
            },
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            shell_frame: PanelRect {
                x: 20.0,
                y: 0.0,
                width: 280.0,
                height: 150.0,
            },
            headline_text: "Codex ready".to_string(),
            headline_emphasized: false,
            separator_visibility: 0.5,
            cards_visible: true,
            card_count: 2,
            cards: vec![
                NativePanelVisualCardInput {
                    title: "Settings".to_string(),
                    subtitle: Some("EchoIsland v0.2.0".to_string()),
                    body: None,
                    badge: None,
                    rows: vec![NativePanelVisualCardRowInput {
                        title: "Mute Sound".to_string(),
                        value: "Off".to_string(),
                        active: true,
                    }],
                },
                NativePanelVisualCardInput {
                    title: "Done".to_string(),
                    subtitle: Some("#abcdef · now".to_string()),
                    body: Some("Task complete".to_string()),
                    badge: Some(NativePanelVisualCardBadgeInput {
                        text: "Done".to_string(),
                        emphasized: true,
                    }),
                    rows: Vec::new(),
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
            mascot_pose: SceneMascotPose::Complete,
        }
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
        let plan =
            resolve_native_panel_visual_plan(&visual_input(NativePanelVisualDisplayMode::Expanded));

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
            matches!(primitive, NativePanelVisualPrimitive::StrokeLine { .. })
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == "Mute Sound")
        }));
        assert!(plan.primitives.iter().any(|primitive| {
            matches!(primitive, NativePanelVisualPrimitive::Text { text, .. } if text == "Done")
        }));
    }
}
