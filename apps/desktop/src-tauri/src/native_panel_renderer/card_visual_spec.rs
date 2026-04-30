use crate::{
    native_panel_core::{
        PanelPoint, PanelRect, StatusQueuePayload, card_transition_total_ms,
        completion_preview_text, default_panel_card_metric_constants, display_snippet,
        ease_out_cubic, format_source, format_status, is_long_idle_session, lerp,
        resolve_card_chat_body_width, resolve_estimated_text_width, session_has_visible_card_body,
        session_meta_line, session_prompt_preview, session_reply_preview, session_title,
        session_tool_preview, settings_surface_row_frame, short_session_id,
    },
    native_panel_scene::{SceneCard, SettingsRowScene},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardVisualStyle {
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
pub(crate) enum CardVisualBodyRole {
    Assistant,
    User,
    Tool,
    Plain,
    ActionHint,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualBodySpec {
    pub(crate) role: CardVisualBodyRole,
    pub(crate) prefix: Option<String>,
    pub(crate) text: String,
    pub(crate) max_lines: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualBodyLinePaintSpec {
    pub(crate) prefix_color: CardVisualColorSpec,
    pub(crate) text_color: CardVisualColorSpec,
    pub(crate) prefix_size: i32,
    pub(crate) text_size: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualToolPillPaintSpec {
    pub(crate) tool_name: String,
    pub(crate) description: Option<String>,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) radius: f64,
    pub(crate) text_inset_x: f64,
    pub(crate) text_offset_y: f64,
    pub(crate) text_size: i32,
    pub(crate) tool_name_width: f64,
    pub(crate) tool_description_gap: f64,
    pub(crate) background_color: CardVisualColorSpec,
    pub(crate) tool_name_color: CardVisualColorSpec,
    pub(crate) description_color: CardVisualColorSpec,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualActionHintPaintSpec {
    pub(crate) text: String,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) radius: f64,
    pub(crate) text_inset_x: f64,
    pub(crate) text_offset_y: f64,
    pub(crate) text_size: i32,
    pub(crate) background_color: CardVisualColorSpec,
    pub(crate) foreground_color: CardVisualColorSpec,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualTextPaintSpec {
    pub(crate) color: CardVisualColorSpec,
    pub(crate) size: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualHeaderTextPaintSpec {
    pub(crate) title: CardVisualTextPaintSpec,
    pub(crate) subtitle: CardVisualTextPaintSpec,
    pub(crate) title_max_chars: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualContentRevealFrameSpec {
    pub(crate) visibility_progress: f64,
    pub(crate) translate_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualStackRevealFrameSpec {
    pub(crate) progress: f64,
    pub(crate) elapsed_ms: f64,
    pub(crate) card_phase: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualContentLayoutSpec {
    pub(crate) content_x: f64,
    pub(crate) content_width: f64,
    pub(crate) title_y: f64,
    pub(crate) subtitle_y: f64,
    pub(crate) empty_title_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualBodyLayoutSpec {
    pub(crate) prefix_x: f64,
    pub(crate) text_x: f64,
    pub(crate) body_width: f64,
    pub(crate) initial_y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualActionHintLayoutSpec {
    pub(crate) paint: CardVisualActionHintPaintSpec,
    pub(crate) pill_frame: PanelRect,
    pub(crate) text_origin: PanelPoint,
    pub(crate) text_max_width: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualToolPillLayoutSpec {
    pub(crate) paint: CardVisualToolPillPaintSpec,
    pub(crate) pill_frame: PanelRect,
    pub(crate) tool_name_origin: PanelPoint,
    pub(crate) tool_name_max_width: f64,
    pub(crate) description: Option<CardVisualTextLayoutSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualTextLayoutSpec {
    pub(crate) text: String,
    pub(crate) origin: PanelPoint,
    pub(crate) max_width: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualBadgeLayoutSpec {
    pub(crate) paint: CardVisualBadgePaintSpec,
    pub(crate) badge_frame: PanelRect,
    pub(crate) text_origin: PanelPoint,
    pub(crate) text_max_width: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualSettingsRowLayoutSpec {
    pub(crate) paint: CardVisualSettingsRowPaintSpec,
    pub(crate) row_frame: PanelRect,
    pub(crate) row_inner_frame: PanelRect,
    pub(crate) title_origin: PanelPoint,
    pub(crate) title_max_width: f64,
    pub(crate) value_badge_frame: PanelRect,
    pub(crate) value_origin: PanelPoint,
    pub(crate) value_max_width: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardVisualBadgeRole {
    Status,
    Source,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualBadgeSpec {
    pub(crate) role: CardVisualBadgeRole,
    pub(crate) text: String,
    pub(crate) emphasized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualBadgePaintSpec {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) radius: f64,
    pub(crate) text_inset_x: f64,
    pub(crate) text_offset_y: f64,
    pub(crate) text_size: i32,
    pub(crate) background_color: CardVisualColorSpec,
    pub(crate) foreground_color: CardVisualColorSpec,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualRowSpec {
    pub(crate) title: String,
    pub(crate) value: String,
    pub(crate) active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualSettingsRowPaintSpec {
    pub(crate) border_radius: f64,
    pub(crate) fill_radius: f64,
    pub(crate) border_color: CardVisualColorSpec,
    pub(crate) fill_color: CardVisualColorSpec,
    pub(crate) title_color: CardVisualColorSpec,
    pub(crate) title_size: i32,
    pub(crate) value_badge: CardVisualSettingsValueBadgePaintSpec,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualSettingsValueBadgePaintSpec {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) radius: f64,
    pub(crate) text_inset_x: f64,
    pub(crate) text_offset_y: f64,
    pub(crate) text_size: i32,
    pub(crate) background_color: CardVisualColorSpec,
    pub(crate) foreground_color: CardVisualColorSpec,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualShellSpec {
    pub(crate) emphasized: bool,
    pub(crate) border_color: CardVisualColorSpec,
    pub(crate) fill_color: CardVisualColorSpec,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualColorSpec {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardVisualAnimationSpec {
    pub(crate) collapsed_height: f64,
    pub(crate) reveal_scale_x_from: f64,
    pub(crate) reveal_scale_y_from: f64,
    pub(crate) reveal_translate_y_from: f64,
    pub(crate) content_reveal_delay_progress: f64,
    pub(crate) content_early_exit_progress: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CardVisualSpec {
    pub(crate) style: CardVisualStyle,
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) badges: Vec<CardVisualBadgeSpec>,
    pub(crate) body: Vec<CardVisualBodySpec>,
    pub(crate) action_hint: Option<String>,
    pub(crate) rows: Vec<CardVisualRowSpec>,
    pub(crate) height: f64,
    pub(crate) compact: bool,
    pub(crate) removing: bool,
    pub(crate) shell: CardVisualShellSpec,
    pub(crate) animation: CardVisualAnimationSpec,
}

pub(crate) fn card_visual_spec_from_scene_card_with_height(
    card: &SceneCard,
    height: f64,
) -> CardVisualSpec {
    match card {
        SceneCard::Settings {
            title,
            version,
            rows,
        } => card_visual_spec(
            CardVisualStyle::Settings,
            title.clone(),
            Some(version.text.clone()),
            Vec::new(),
            Vec::new(),
            None,
            settings_rows(rows),
            height,
            64.0,
            false,
            false,
        ),
        SceneCard::PendingPermission { pending, count } => card_visual_spec(
            CardVisualStyle::PendingApproval,
            "Approval Required".to_string(),
            Some(format!(
                "#{} · Approval",
                short_session_id(&pending.session_id)
            )),
            badge_pair(
                (*count > 1).then_some((count.to_string(), true)),
                Some((format_source(&pending.source), false)),
            ),
            plain_body(
                CardVisualBodyRole::Plain,
                Some("!"),
                display_snippet(pending.tool_description.as_deref(), 78)
                    .unwrap_or_else(|| "Waiting for your approval".to_string()),
                2,
            ),
            Some("Allow / Deny in terminal".to_string()),
            Vec::new(),
            height,
            46.0,
            false,
            false,
        ),
        SceneCard::PendingQuestion { pending, count } => card_visual_spec(
            CardVisualStyle::PendingQuestion,
            pending
                .header
                .clone()
                .unwrap_or_else(|| "Question".to_string()),
            Some(format!(
                "#{} · Question",
                short_session_id(&pending.session_id)
            )),
            badge_pair(
                (*count > 1).then_some((count.to_string(), true)),
                Some((format_source(&pending.source), false)),
            ),
            plain_body(
                CardVisualBodyRole::Plain,
                Some("?"),
                display_snippet(Some(&pending.text), 82)
                    .unwrap_or_else(|| "Waiting for your answer".to_string()),
                2,
            ),
            Some("Answer in terminal".to_string()),
            Vec::new(),
            height,
            46.0,
            false,
            false,
        ),
        SceneCard::PromptAssist { session } => card_visual_spec(
            CardVisualStyle::PromptAssist,
            session_title(session),
            Some(format!("{} · Prompt", format_source(&session.source))),
            badge_pair(
                Some(("Check".to_string(), true)),
                Some(("Codex".to_string(), false)),
            ),
            optional_body(
                CardVisualBodyRole::Plain,
                Some("!"),
                session_prompt_preview(session)
                    .or_else(|| display_snippet(session.tool_description.as_deref(), 78)),
                2,
            ),
            Some("Open terminal to check".to_string()),
            Vec::new(),
            height,
            52.0,
            false,
            false,
        ),
        SceneCard::Session {
            session,
            title,
            status,
            snippet,
        } => {
            let has_chat = session_reply_preview(session).is_some()
                || session_prompt_preview(session).is_some();
            card_visual_spec(
                CardVisualStyle::Default,
                if title.trim().is_empty() {
                    session_title(session)
                } else {
                    title.clone()
                },
                Some(session_meta_line(session)),
                badge_pair(
                    Some((
                        if status.text.trim().is_empty() {
                            format_status(&session.status)
                        } else {
                            status.text.clone()
                        },
                        status.emphasized,
                    )),
                    Some((format_source(&session.source), false)),
                ),
                session_body_lines(session),
                None,
                Vec::new(),
                height,
                if has_chat { 64.0 } else { 52.0 },
                snippet.is_none()
                    && session_reply_preview(session).is_none()
                    && session_prompt_preview(session).is_none(),
                false,
            )
        }
        SceneCard::StatusApproval { item } => match &item.payload {
            StatusQueuePayload::Approval(pending) => card_visual_spec(
                CardVisualStyle::PendingApproval,
                "Approval Required".to_string(),
                Some(format!(
                    "#{} · Approval",
                    short_session_id(&item.session_id)
                )),
                badge_pair(
                    Some(("Approval".to_string(), true)),
                    Some((format_source(&pending.source), false)),
                ),
                plain_body(
                    CardVisualBodyRole::Plain,
                    Some("!"),
                    display_snippet(pending.tool_description.as_deref(), 78)
                        .unwrap_or_else(|| "Waiting for your approval".to_string()),
                    2,
                ),
                Some("Allow / Deny in terminal".to_string()),
                Vec::new(),
                height,
                46.0,
                false,
                item.is_removing,
            ),
            StatusQueuePayload::Completion(session) => {
                completion_card_spec(session, height, item.is_removing)
            }
            StatusQueuePayload::Question(pending) => pending_question_card_spec(
                pending,
                height,
                item.is_removing,
                Some("Question".to_string()),
            ),
        },
        SceneCard::StatusQuestion { item } => match &item.payload {
            StatusQueuePayload::Question(pending) => pending_question_card_spec(
                pending,
                height,
                item.is_removing,
                Some("Question".to_string()),
            ),
            StatusQueuePayload::Approval(pending) => card_visual_spec(
                CardVisualStyle::PendingApproval,
                "Approval Required".to_string(),
                Some(format_source(&pending.source)),
                badge_pair(None, Some((format_source(&pending.source), false))),
                optional_body(
                    CardVisualBodyRole::Plain,
                    Some("!"),
                    display_snippet(pending.tool_description.as_deref(), 78),
                    2,
                ),
                Some("Allow / Deny in terminal".to_string()),
                Vec::new(),
                height,
                46.0,
                false,
                item.is_removing,
            ),
            StatusQueuePayload::Completion(session) => {
                completion_card_spec(session, height, item.is_removing)
            }
        },
        SceneCard::StatusCompletion { item } => match &item.payload {
            StatusQueuePayload::Completion(session) => {
                completion_card_spec(session, height, item.is_removing)
            }
            StatusQueuePayload::Approval(pending) => card_visual_spec(
                CardVisualStyle::PendingApproval,
                "Approval Required".to_string(),
                Some(format_source(&pending.source)),
                badge_pair(None, Some((format_source(&pending.source), false))),
                optional_body(
                    CardVisualBodyRole::Plain,
                    Some("!"),
                    display_snippet(pending.tool_description.as_deref(), 78),
                    2,
                ),
                Some("Allow / Deny in terminal".to_string()),
                Vec::new(),
                height,
                46.0,
                false,
                item.is_removing,
            ),
            StatusQueuePayload::Question(pending) => pending_question_card_spec(
                pending,
                height,
                item.is_removing,
                Some("Question".to_string()),
            ),
        },
        SceneCard::Empty => card_visual_spec(
            CardVisualStyle::Empty,
            "No active sessions".to_string(),
            None,
            Vec::new(),
            vec![CardVisualBodySpec {
                role: CardVisualBodyRole::Plain,
                prefix: None,
                text: "EchoIsland is watching for new activity.".to_string(),
                max_lines: 1,
            }],
            None,
            Vec::new(),
            height,
            34.0,
            true,
            false,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn card_visual_spec(
    style: CardVisualStyle,
    title: String,
    subtitle: Option<String>,
    badges: Vec<CardVisualBadgeSpec>,
    body: Vec<CardVisualBodySpec>,
    action_hint: Option<String>,
    rows: Vec<CardVisualRowSpec>,
    height: f64,
    collapsed_height: f64,
    compact: bool,
    removing: bool,
) -> CardVisualSpec {
    CardVisualSpec {
        style,
        title,
        subtitle,
        badges,
        body,
        action_hint,
        rows,
        height,
        compact,
        removing,
        shell: CardVisualShellSpec {
            emphasized: style == CardVisualStyle::Completion,
            border_color: card_visual_shell_border_color(style),
            fill_color: card_visual_shell_fill_color(style),
        },
        animation: CardVisualAnimationSpec {
            collapsed_height,
            reveal_scale_x_from: 0.96,
            reveal_scale_y_from: 0.82,
            reveal_translate_y_from: crate::native_panel_core::PANEL_CARD_REVEAL_Y,
            content_reveal_delay_progress:
                crate::native_panel_core::PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS,
            content_early_exit_progress:
                crate::native_panel_core::PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS,
        },
    }
}

fn completion_card_spec(
    session: &echoisland_runtime::SessionSnapshotView,
    height: f64,
    removing: bool,
) -> CardVisualSpec {
    card_visual_spec(
        CardVisualStyle::Completion,
        session_title(session),
        Some(session_meta_line(session)),
        badge_pair(
            Some(("Done".to_string(), true)),
            Some((format_source(&session.source), false)),
        ),
        plain_body(
            CardVisualBodyRole::Assistant,
            Some("$"),
            completion_preview_text(session),
            2,
        ),
        None,
        Vec::new(),
        height,
        52.0,
        false,
        removing,
    )
}

fn pending_question_card_spec(
    pending: &echoisland_runtime::PendingQuestionView,
    height: f64,
    removing: bool,
    badge_text: Option<String>,
) -> CardVisualSpec {
    card_visual_spec(
        CardVisualStyle::PendingQuestion,
        pending
            .header
            .clone()
            .unwrap_or_else(|| "Question".to_string()),
        Some(format!(
            "#{} · Question",
            short_session_id(&pending.session_id)
        )),
        badge_pair(
            badge_text.map(|text| (text, true)),
            Some((format_source(&pending.source), false)),
        ),
        plain_body(
            CardVisualBodyRole::Plain,
            Some("?"),
            display_snippet(Some(&pending.text), 82)
                .unwrap_or_else(|| "Waiting for your answer".to_string()),
            2,
        ),
        Some("Answer in terminal".to_string()),
        Vec::new(),
        height,
        46.0,
        false,
        removing,
    )
}

pub(crate) fn card_visual_shell_border_color(style: CardVisualStyle) -> CardVisualColorSpec {
    match style {
        CardVisualStyle::Completion => CardVisualColorSpec::rgb(46, 79, 61),
        CardVisualStyle::Pending
        | CardVisualStyle::PendingApproval
        | CardVisualStyle::PromptAssist => CardVisualColorSpec::rgb(87, 61, 39),
        CardVisualStyle::PendingQuestion => CardVisualColorSpec::rgb(74, 62, 103),
        CardVisualStyle::Settings => CardVisualColorSpec::rgb(42, 42, 47),
        CardVisualStyle::Default | CardVisualStyle::Empty => CardVisualColorSpec::rgb(42, 42, 47),
    }
}

pub(crate) fn card_visual_shell_fill_color(style: CardVisualStyle) -> CardVisualColorSpec {
    match style {
        CardVisualStyle::Completion => CardVisualColorSpec::rgb(37, 37, 41),
        CardVisualStyle::Pending | CardVisualStyle::PendingApproval => {
            CardVisualColorSpec::rgb(54, 41, 34)
        }
        CardVisualStyle::PendingQuestion => CardVisualColorSpec::rgb(45, 42, 57),
        CardVisualStyle::PromptAssist => CardVisualColorSpec::rgb(48, 41, 35),
        CardVisualStyle::Settings => CardVisualColorSpec::rgb(37, 37, 41),
        CardVisualStyle::Default | CardVisualStyle::Empty => CardVisualColorSpec::rgb(37, 37, 41),
    }
}

pub(crate) fn card_visual_badge_paint_spec(
    style: CardVisualStyle,
    badge: &CardVisualBadgeSpec,
) -> CardVisualBadgePaintSpec {
    CardVisualBadgePaintSpec {
        width: card_visual_badge_width(&badge.text),
        height: 22.0,
        radius: 11.0,
        text_inset_x: 7.0,
        text_offset_y: 4.0,
        text_size: 10,
        background_color: card_visual_badge_background_color(style, badge),
        foreground_color: card_visual_badge_foreground_color(style, badge),
    }
}

pub(crate) fn card_visual_settings_row_paint_spec(
    row: &CardVisualRowSpec,
) -> CardVisualSettingsRowPaintSpec {
    CardVisualSettingsRowPaintSpec {
        border_radius: 8.0,
        fill_radius: 7.0,
        border_color: card_visual_settings_row_border_color(row.active),
        fill_color: card_visual_settings_row_fill_color(row.active),
        title_color: CardVisualColorSpec::rgb(245, 247, 252),
        title_size: 11,
        value_badge: CardVisualSettingsValueBadgePaintSpec {
            width: card_visual_settings_value_badge_width(&row.value),
            height: 18.0,
            radius: 9.0,
            text_inset_x: 9.0,
            text_offset_y: 4.0,
            text_size: 10,
            background_color: card_visual_settings_value_badge_background(row.active),
            foreground_color: card_visual_settings_value_badge_foreground(row.active),
        },
    }
}

pub(crate) fn card_visual_body_line_paint_spec(
    style: CardVisualStyle,
    role: CardVisualBodyRole,
    prefix: Option<&str>,
) -> CardVisualBodyLinePaintSpec {
    CardVisualBodyLinePaintSpec {
        prefix_color: card_visual_prefix_color(style, prefix.unwrap_or_default()),
        text_color: card_visual_body_line_text_color(style, role, prefix),
        prefix_size: 10,
        text_size: 10,
    }
}

pub(crate) fn card_visual_tool_pill_paint_spec(text: &str) -> Option<CardVisualToolPillPaintSpec> {
    let (tool_name, description) = split_tool_body_text(text);
    if tool_name.is_empty() {
        return None;
    }
    let description_width = description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| resolve_estimated_text_width(value, 9.0) + 6.0)
        .unwrap_or(0.0);
    let width =
        (resolve_estimated_text_width(&tool_name, 9.0) + description_width + 14.0).max(36.0);
    Some(CardVisualToolPillPaintSpec {
        tool_name: tool_name.clone(),
        description,
        width,
        height: 22.0,
        radius: 5.0,
        text_inset_x: 7.0,
        text_offset_y: 5.0,
        text_size: 9,
        tool_name_width: resolve_estimated_text_width(&tool_name, 9.0),
        tool_description_gap: 6.0,
        background_color: CardVisualColorSpec::rgb(47, 47, 52),
        tool_name_color: card_visual_tool_tone_color(&tool_name),
        description_color: CardVisualColorSpec::rgb(214, 218, 225),
    })
}

pub(crate) fn card_visual_action_hint_paint_spec(
    text: &str,
) -> Option<CardVisualActionHintPaintSpec> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return None;
    }
    let metrics = default_panel_card_metric_constants();
    Some(CardVisualActionHintPaintSpec {
        width: (resolve_estimated_text_width(&text, 10.0) + 18.0).max(32.0),
        height: metrics.pending_action_height,
        radius: metrics.pending_action_height / 2.0,
        text,
        text_inset_x: 9.0,
        text_offset_y: 4.0,
        text_size: 10,
        background_color: CardVisualColorSpec::rgb(49, 49, 53),
        foreground_color: CardVisualColorSpec::rgb(230, 235, 245),
    })
}

pub(crate) fn card_visual_header_text_paint_spec(
    style: CardVisualStyle,
) -> CardVisualHeaderTextPaintSpec {
    CardVisualHeaderTextPaintSpec {
        title: CardVisualTextPaintSpec {
            color: if style == CardVisualStyle::Empty {
                CardVisualColorSpec::rgb(171, 179, 194)
            } else {
                CardVisualColorSpec::rgb(245, 247, 252)
            },
            size: 12,
        },
        subtitle: CardVisualTextPaintSpec {
            color: CardVisualColorSpec::rgb(171, 179, 194),
            size: 9,
        },
        title_max_chars: 30,
    }
}

pub(crate) fn card_visual_content_reveal_frame(phase: f64) -> CardVisualContentRevealFrameSpec {
    let phase = phase.clamp(0.0, 1.0);
    let delay = crate::native_panel_core::PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS;
    CardVisualContentRevealFrameSpec {
        visibility_progress: ease_out_cubic(((phase - delay) / (1.0 - delay)).clamp(0.0, 1.0)),
        translate_y: lerp(-5.0, 0.0, phase),
    }
}

pub(crate) fn card_visual_content_transition_frame(
    phase: f64,
    exiting: bool,
) -> CardVisualContentRevealFrameSpec {
    if !exiting {
        return card_visual_content_reveal_frame(phase);
    }

    let phase = phase.clamp(0.0, 1.0);
    let exit = crate::native_panel_core::PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS;
    CardVisualContentRevealFrameSpec {
        visibility_progress: ease_out_cubic(((phase - exit) / (1.0 - exit)).clamp(0.0, 1.0)),
        translate_y: lerp(-5.0, 0.0, phase),
    }
}

pub(crate) fn card_visual_shell_reveal_frame(
    expanded_frame: PanelRect,
    collapsed_height: f64,
    phase: f64,
) -> PanelRect {
    let progress = ease_out_cubic(phase.clamp(0.0, 1.0));
    let width = lerp(expanded_frame.width * 0.96, expanded_frame.width, progress);
    let height = lerp(collapsed_height, expanded_frame.height, progress);
    PanelRect {
        x: expanded_frame.x + (expanded_frame.width - width) / 2.0,
        y: expanded_frame.y + (expanded_frame.height - height),
        width,
        height,
    }
}

pub(crate) fn card_visual_stack_reveal_frame(
    separator_visibility: f64,
    card_count: usize,
    card_index: usize,
) -> CardVisualStackRevealFrameSpec {
    let progress = (separator_visibility / 0.88).clamp(0.0, 1.0);
    let total_ms = card_transition_total_ms(
        card_count,
        crate::native_panel_core::PANEL_CARD_REVEAL_MS,
        crate::native_panel_core::PANEL_CARD_REVEAL_STAGGER_MS,
    ) as f64;
    let elapsed_ms = progress * total_ms;
    let delay_ms =
        card_index as f64 * crate::native_panel_core::PANEL_CARD_REVEAL_STAGGER_MS as f64;
    CardVisualStackRevealFrameSpec {
        progress,
        elapsed_ms,
        card_phase: ((elapsed_ms - delay_ms)
            / crate::native_panel_core::PANEL_CARD_REVEAL_MS as f64)
            .clamp(0.0, 1.0),
    }
}

pub(crate) fn card_visual_content_layout(frame: PanelRect) -> CardVisualContentLayoutSpec {
    let metrics = default_panel_card_metric_constants();
    CardVisualContentLayoutSpec {
        content_x: frame.x + metrics.card_inset_x,
        content_width: (frame.width - metrics.card_inset_x * 2.0).max(0.0),
        title_y: frame.y + frame.height - 24.0,
        subtitle_y: frame.y + frame.height - 40.0,
        empty_title_y: frame.y + (frame.height - 20.0) / 2.0,
    }
}

pub(crate) fn card_visual_body_layout(
    frame: PanelRect,
    action_hint_present: bool,
) -> CardVisualBodyLayoutSpec {
    let metrics = default_panel_card_metric_constants();
    let body_bottom = if action_hint_present {
        metrics.pending_action_y + metrics.pending_action_height + metrics.pending_action_gap
    } else {
        metrics.content_bottom_inset
    };
    CardVisualBodyLayoutSpec {
        prefix_x: frame.x + metrics.card_inset_x,
        text_x: frame.x + metrics.card_inset_x + metrics.chat_prefix_width,
        body_width: resolve_card_chat_body_width(frame.width, metrics),
        initial_y: frame.y + body_bottom,
    }
}

pub(crate) fn card_visual_action_hint_layout(
    frame: PanelRect,
    action_hint: &str,
) -> Option<CardVisualActionHintLayoutSpec> {
    let paint = card_visual_action_hint_paint_spec(action_hint)?;
    let metrics = default_panel_card_metric_constants();
    let max_width = (frame.width - metrics.card_inset_x * 2.0).max(0.0);
    if max_width <= 0.0 {
        return None;
    }
    let pill_frame = PanelRect {
        x: frame.x + metrics.card_inset_x,
        y: frame.y + metrics.pending_action_y,
        width: paint.width.min(max_width),
        height: paint.height,
    };
    Some(CardVisualActionHintLayoutSpec {
        text_origin: PanelPoint {
            x: pill_frame.x + paint.text_inset_x,
            y: pill_frame.y + paint.text_offset_y,
        },
        text_max_width: (pill_frame.width - paint.text_inset_x * 2.0).max(0.0),
        pill_frame,
        paint,
    })
}

pub(crate) fn card_visual_tool_pill_layout(
    frame: PanelRect,
    y: f64,
    text: &str,
) -> Option<CardVisualToolPillLayoutSpec> {
    let paint = card_visual_tool_pill_paint_spec(text)?;
    let metrics = default_panel_card_metric_constants();
    let max_width = (frame.width - metrics.card_inset_x * 2.0).max(0.0);
    if max_width <= 0.0 {
        return None;
    }
    let pill_frame = PanelRect {
        x: frame.x + metrics.card_inset_x,
        y,
        width: paint.width.min(max_width),
        height: paint.height,
    };
    let tool_name_max_width = paint
        .tool_name_width
        .min((pill_frame.width - paint.text_inset_x * 2.0).max(0.0));
    let tool_name_origin = PanelPoint {
        x: pill_frame.x + paint.text_inset_x,
        y: pill_frame.y + paint.text_offset_y,
    };
    let description = paint
        .description
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .and_then(|description| {
            let desc_x = pill_frame.x
                + paint.text_inset_x
                + tool_name_max_width
                + paint.tool_description_gap;
            let desc_width =
                (pill_frame.x + pill_frame.width - desc_x - paint.text_inset_x).max(0.0);
            (desc_width > 0.0).then(|| CardVisualTextLayoutSpec {
                text: description.clone(),
                origin: PanelPoint {
                    x: desc_x,
                    y: pill_frame.y + paint.text_offset_y,
                },
                max_width: desc_width,
            })
        });
    Some(CardVisualToolPillLayoutSpec {
        paint,
        pill_frame,
        tool_name_origin,
        tool_name_max_width,
        description,
    })
}

pub(crate) fn card_visual_badge_layout(
    style: CardVisualStyle,
    badge: &CardVisualBadgeSpec,
    right: f64,
    title_y: f64,
) -> CardVisualBadgeLayoutSpec {
    let paint = card_visual_badge_paint_spec(style, badge);
    let badge_frame = PanelRect {
        x: right - paint.width,
        y: title_y - 3.0,
        width: paint.width,
        height: paint.height,
    };
    CardVisualBadgeLayoutSpec {
        text_origin: PanelPoint {
            x: badge_frame.x + paint.text_inset_x,
            y: badge_frame.y + paint.text_offset_y,
        },
        text_max_width: (badge_frame.width - paint.text_inset_x * 2.0).max(0.0),
        paint,
        badge_frame,
    }
}

pub(crate) fn card_visual_settings_row_layout(
    card_frame: PanelRect,
    index: usize,
    row: &CardVisualRowSpec,
) -> Option<CardVisualSettingsRowLayoutSpec> {
    let paint = card_visual_settings_row_paint_spec(row);
    let row_frame = settings_surface_row_frame(card_frame, index);
    if row_frame.y < card_frame.y + 10.0 || row_frame.width <= 0.0 || row_frame.height <= 0.0 {
        return None;
    }
    let row_inner_frame = inset_rect(row_frame, 1.0);
    let badge_width = paint
        .value_badge
        .width
        .min((row_inner_frame.width - 24.0).max(0.0));
    let value_badge_frame = PanelRect {
        x: row_inner_frame.x + row_inner_frame.width - badge_width - 9.0,
        y: row_inner_frame.y + ((row_inner_frame.height - paint.value_badge.height) / 2.0).round(),
        width: badge_width,
        height: paint.value_badge.height,
    };
    Some(CardVisualSettingsRowLayoutSpec {
        title_origin: PanelPoint {
            x: row_inner_frame.x + 11.0,
            y: row_inner_frame.y + ((row_inner_frame.height - 16.0) / 2.0).round(),
        },
        title_max_width: (value_badge_frame.x - row_inner_frame.x - 22.0).max(0.0),
        value_origin: PanelPoint {
            x: value_badge_frame.x + paint.value_badge.text_inset_x,
            y: value_badge_frame.y + paint.value_badge.text_offset_y,
        },
        value_max_width: (value_badge_frame.width - paint.value_badge.text_inset_x * 2.0).max(0.0),
        paint,
        row_frame,
        row_inner_frame,
        value_badge_frame,
    })
}

fn inset_rect(rect: PanelRect, inset: f64) -> PanelRect {
    PanelRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2.0).max(0.0),
        height: (rect.height - inset * 2.0).max(0.0),
    }
}

fn card_visual_prefix_color(style: CardVisualStyle, prefix: &str) -> CardVisualColorSpec {
    match (style, prefix) {
        (CardVisualStyle::Default, "$") => CardVisualColorSpec::rgb(217, 120, 87),
        (CardVisualStyle::Default, ">") | (CardVisualStyle::Completion, _) => {
            CardVisualColorSpec::rgb(104, 222, 145)
        }
        (CardVisualStyle::PendingQuestion, _) | (CardVisualStyle::Pending, "?") => {
            CardVisualColorSpec::rgb(201, 176, 255)
        }
        _ => card_visual_accent_color(style),
    }
}

fn card_visual_body_line_text_color(
    style: CardVisualStyle,
    role: CardVisualBodyRole,
    prefix: Option<&str>,
) -> CardVisualColorSpec {
    match (style, role) {
        (CardVisualStyle::Default, CardVisualBodyRole::User) => {
            CardVisualColorSpec::rgb(218, 222, 229)
        }
        _ => card_visual_body_text_color(style, prefix),
    }
}

fn card_visual_body_text_color(
    style: CardVisualStyle,
    prefix: Option<&str>,
) -> CardVisualColorSpec {
    match (style, prefix) {
        (CardVisualStyle::Default, Some(">")) => CardVisualColorSpec::rgb(218, 222, 229),
        _ => CardVisualColorSpec::rgb(177, 183, 194),
    }
}

fn card_visual_accent_color(style: CardVisualStyle) -> CardVisualColorSpec {
    match style {
        CardVisualStyle::Pending
        | CardVisualStyle::PendingApproval
        | CardVisualStyle::PromptAssist => CardVisualColorSpec::rgb(255, 184, 77),
        CardVisualStyle::PendingQuestion => CardVisualColorSpec::rgb(201, 176, 255),
        CardVisualStyle::Completion => CardVisualColorSpec::rgb(104, 213, 145),
        CardVisualStyle::Settings => CardVisualColorSpec::rgb(142, 166, 255),
        CardVisualStyle::Default | CardVisualStyle::Empty => {
            CardVisualColorSpec::rgb(142, 150, 166)
        }
    }
}

fn split_tool_body_text(text: &str) -> (String, Option<String>) {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized
        .split_once(' ')
        .map(|(name, description)| {
            (
                name.to_string(),
                (!description.trim().is_empty()).then(|| description.trim().to_string()),
            )
        })
        .unwrap_or((normalized, None))
}

fn card_visual_tool_tone_color(tool: &str) -> CardVisualColorSpec {
    match tool.to_ascii_lowercase().as_str() {
        "bash" => CardVisualColorSpec::rgb(125, 242, 163),
        "edit" | "write" => CardVisualColorSpec::rgb(135, 171, 255),
        "read" => CardVisualColorSpec::rgb(240, 209, 125),
        "grep" | "glob" => CardVisualColorSpec::rgb(194, 161, 255),
        "agent" => CardVisualColorSpec::rgb(255, 156, 102),
        _ => CardVisualColorSpec::rgb(245, 247, 252),
    }
}

fn card_visual_settings_row_border_color(active: bool) -> CardVisualColorSpec {
    if active {
        CardVisualColorSpec::rgb(50, 84, 61)
    } else {
        CardVisualColorSpec::rgb(50, 50, 56)
    }
}

fn card_visual_settings_row_fill_color(active: bool) -> CardVisualColorSpec {
    if active {
        CardVisualColorSpec::rgb(42, 50, 44)
    } else {
        CardVisualColorSpec::rgb(43, 43, 48)
    }
}

fn card_visual_settings_value_badge_background(active: bool) -> CardVisualColorSpec {
    if active {
        CardVisualColorSpec::rgb(46, 68, 54)
    } else {
        CardVisualColorSpec::rgb(54, 54, 58)
    }
}

fn card_visual_settings_value_badge_foreground(active: bool) -> CardVisualColorSpec {
    if active {
        CardVisualColorSpec::rgb(104, 222, 145)
    } else {
        CardVisualColorSpec::rgb(230, 235, 245)
    }
}

fn card_visual_settings_value_badge_width(value: &str) -> f64 {
    (resolve_estimated_text_width(value, 10.0) + 18.0).max(44.0)
}

fn card_visual_badge_width(text: &str) -> f64 {
    (text.chars().count() as f64 * 10.0 * 0.58 + 16.0).max(24.0)
}

fn card_visual_badge_background_color(
    style: CardVisualStyle,
    badge: &CardVisualBadgeSpec,
) -> CardVisualColorSpec {
    if badge.emphasized {
        return match (badge.role, style) {
            (
                CardVisualBadgeRole::Status,
                CardVisualStyle::Pending
                | CardVisualStyle::PendingApproval
                | CardVisualStyle::PromptAssist,
            ) => CardVisualColorSpec::rgb(70, 53, 36),
            (CardVisualBadgeRole::Status, CardVisualStyle::PendingQuestion) => {
                CardVisualColorSpec::rgb(61, 52, 83)
            }
            _ => CardVisualColorSpec::rgb(58, 84, 65),
        };
    }

    match badge.role {
        CardVisualBadgeRole::Source => CardVisualColorSpec::rgb(47, 52, 67),
        CardVisualBadgeRole::Status => CardVisualColorSpec::rgb(54, 54, 58),
    }
}

fn card_visual_badge_foreground_color(
    style: CardVisualStyle,
    badge: &CardVisualBadgeSpec,
) -> CardVisualColorSpec {
    if badge.emphasized {
        return match (badge.role, style) {
            (
                CardVisualBadgeRole::Status,
                CardVisualStyle::Pending
                | CardVisualStyle::PendingApproval
                | CardVisualStyle::PromptAssist,
            ) => CardVisualColorSpec::rgb(255, 184, 77),
            (CardVisualBadgeRole::Status, CardVisualStyle::PendingQuestion) => {
                CardVisualColorSpec::rgb(201, 176, 255)
            }
            _ => CardVisualColorSpec::rgb(102, 222, 145),
        };
    }

    match badge.role {
        CardVisualBadgeRole::Source => CardVisualColorSpec::rgb(120, 166, 255),
        CardVisualBadgeRole::Status => CardVisualColorSpec::rgb(230, 235, 245),
    }
}

impl CardVisualColorSpec {
    pub(crate) const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

fn settings_rows(rows: &[SettingsRowScene]) -> Vec<CardVisualRowSpec> {
    rows.iter()
        .map(|row| CardVisualRowSpec {
            title: row.title.clone(),
            value: row.value.text.clone(),
            active: row.value.emphasized,
        })
        .collect()
}

fn badge_pair(
    status: Option<(String, bool)>,
    source: Option<(String, bool)>,
) -> Vec<CardVisualBadgeSpec> {
    let mut badges = Vec::new();
    if let Some((text, emphasized)) = status {
        badges.push(CardVisualBadgeSpec {
            role: CardVisualBadgeRole::Status,
            text,
            emphasized,
        });
    }
    if let Some((text, emphasized)) = source {
        badges.push(CardVisualBadgeSpec {
            role: CardVisualBadgeRole::Source,
            text,
            emphasized,
        });
    }
    badges
}

fn plain_body(
    role: CardVisualBodyRole,
    prefix: Option<&str>,
    text: String,
    max_lines: usize,
) -> Vec<CardVisualBodySpec> {
    optional_body(role, prefix, Some(text), max_lines)
}

fn optional_body(
    role: CardVisualBodyRole,
    prefix: Option<&str>,
    text: Option<String>,
    max_lines: usize,
) -> Vec<CardVisualBodySpec> {
    text.filter(|value| !value.trim().is_empty())
        .map(|text| {
            vec![CardVisualBodySpec {
                role,
                prefix: prefix.map(str::to_string),
                text,
                max_lines,
            }]
        })
        .unwrap_or_default()
}

fn session_body_lines(
    session: &echoisland_runtime::SessionSnapshotView,
) -> Vec<CardVisualBodySpec> {
    if is_long_idle_session(session) || !session_has_visible_card_body(session) {
        return Vec::new();
    }

    let mut lines = Vec::new();
    if let Some((tool_name, tool_description)) = session_tool_preview(session) {
        let tool_text = tool_description
            .map(|description| format!("{tool_name} {description}"))
            .unwrap_or(tool_name);
        lines.push(CardVisualBodySpec {
            role: CardVisualBodyRole::Tool,
            prefix: Some("!".to_string()),
            text: display_snippet(Some(&tool_text), 78).unwrap_or(tool_text),
            max_lines: 1,
        });
    }
    if let Some(reply) = session_reply_preview(session) {
        lines.push(CardVisualBodySpec {
            role: CardVisualBodyRole::Assistant,
            prefix: Some("$".to_string()),
            text: reply,
            max_lines: 2,
        });
    }
    if let Some(prompt) = session_prompt_preview(session) {
        lines.push(CardVisualBodySpec {
            role: CardVisualBodyRole::User,
            prefix: Some(">".to_string()),
            text: prompt,
            max_lines: 1,
        });
    }
    lines
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use echoisland_runtime::SessionSnapshotView;

    use super::*;
    use crate::native_panel_scene::{SceneBadge, SceneCard};

    fn session() -> SessionSnapshotView {
        SessionSnapshotView {
            session_id: "session-123456".to_string(),
            source: "claude".to_string(),
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
            last_user_prompt: Some("修复测试".to_string()),
            last_assistant_message: Some("正在运行测试".to_string()),
            tool_history_count: 0,
            tool_history: Vec::new(),
            last_activity: Utc::now(),
        }
    }

    #[test]
    fn session_card_spec_preserves_mac_header_badges_and_chat_lines() {
        let session = session();
        let spec = card_visual_spec_from_scene_card_with_height(
            &SceneCard::Session {
                session: session.clone(),
                title: "EchoIsland".to_string(),
                status: SceneBadge {
                    text: "Running".to_string(),
                    emphasized: true,
                },
                snippet: session.last_assistant_message.clone(),
            },
            96.0,
        );

        assert_eq!(spec.title, "EchoIsland");
        assert!(spec.subtitle.as_deref().unwrap_or_default().contains("#"));
        assert_eq!(
            spec.badges,
            vec![
                CardVisualBadgeSpec {
                    role: CardVisualBadgeRole::Status,
                    text: "Running".to_string(),
                    emphasized: true,
                },
                CardVisualBadgeSpec {
                    role: CardVisualBadgeRole::Source,
                    text: "Claude".to_string(),
                    emphasized: false,
                },
            ]
        );
        assert_eq!(
            spec.body
                .iter()
                .map(|line| (line.role, line.prefix.as_deref(), line.max_lines))
                .collect::<Vec<_>>(),
            vec![
                (CardVisualBodyRole::Tool, Some("!"), 1),
                (CardVisualBodyRole::Assistant, Some("$"), 2),
                (CardVisualBodyRole::User, Some(">"), 1),
            ]
        );
        assert_eq!(spec.animation.collapsed_height, 64.0);
    }

    #[test]
    fn card_spec_exposes_mac_card_animation_constants() {
        let spec = card_visual_spec_from_scene_card_with_height(&SceneCard::Empty, 52.0);

        assert_eq!(spec.animation.reveal_scale_x_from, 0.96);
        assert_eq!(spec.animation.reveal_scale_y_from, 0.82);
        assert_eq!(
            spec.animation.reveal_translate_y_from,
            crate::native_panel_core::PANEL_CARD_REVEAL_Y
        );
        assert_eq!(
            spec.animation.content_reveal_delay_progress,
            crate::native_panel_core::PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS
        );
        assert_eq!(
            spec.animation.content_early_exit_progress,
            crate::native_panel_core::PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS
        );
    }

    #[test]
    fn card_spec_exposes_shared_shell_palette() {
        let spec = card_visual_spec_from_scene_card_with_height(&SceneCard::Empty, 52.0);

        assert_eq!(
            spec.shell.border_color,
            card_visual_shell_border_color(CardVisualStyle::Empty)
        );
        assert_eq!(
            spec.shell.fill_color,
            card_visual_shell_fill_color(CardVisualStyle::Empty)
        );
        assert_eq!(
            card_visual_shell_border_color(CardVisualStyle::Completion),
            CardVisualColorSpec::rgb(46, 79, 61)
        );
        assert_eq!(
            card_visual_shell_fill_color(CardVisualStyle::Completion),
            CardVisualColorSpec::rgb(37, 37, 41)
        );
    }

    #[test]
    fn card_spec_exposes_shared_header_badge_palette_and_metrics() {
        let completion_status = card_visual_badge_paint_spec(
            CardVisualStyle::Completion,
            &CardVisualBadgeSpec {
                role: CardVisualBadgeRole::Status,
                text: "Done".to_string(),
                emphasized: true,
            },
        );
        let source = card_visual_badge_paint_spec(
            CardVisualStyle::Completion,
            &CardVisualBadgeSpec {
                role: CardVisualBadgeRole::Source,
                text: "Claude".to_string(),
                emphasized: false,
            },
        );
        let pending = card_visual_badge_paint_spec(
            CardVisualStyle::PendingApproval,
            &CardVisualBadgeSpec {
                role: CardVisualBadgeRole::Status,
                text: "Approval".to_string(),
                emphasized: true,
            },
        );
        let question = card_visual_badge_paint_spec(
            CardVisualStyle::PendingQuestion,
            &CardVisualBadgeSpec {
                role: CardVisualBadgeRole::Status,
                text: "Question".to_string(),
                emphasized: true,
            },
        );

        assert_eq!(
            completion_status.background_color,
            CardVisualColorSpec::rgb(58, 84, 65)
        );
        assert_eq!(
            completion_status.foreground_color,
            CardVisualColorSpec::rgb(102, 222, 145)
        );
        assert_eq!(
            source.background_color,
            CardVisualColorSpec::rgb(47, 52, 67)
        );
        assert_eq!(
            source.foreground_color,
            CardVisualColorSpec::rgb(120, 166, 255)
        );
        assert_eq!(
            pending.background_color,
            CardVisualColorSpec::rgb(70, 53, 36)
        );
        assert_eq!(
            pending.foreground_color,
            CardVisualColorSpec::rgb(255, 184, 77)
        );
        assert_eq!(
            question.background_color,
            CardVisualColorSpec::rgb(61, 52, 83)
        );
        assert_eq!(
            question.foreground_color,
            CardVisualColorSpec::rgb(201, 176, 255)
        );
        assert_eq!(completion_status.height, 22.0);
        assert_eq!(completion_status.radius, 11.0);
        assert!(completion_status.width >= 24.0);
    }

    #[test]
    fn card_spec_exposes_shared_settings_row_surface_and_value_badge_metrics() {
        let active = card_visual_settings_row_paint_spec(&CardVisualRowSpec {
            title: "Mascot".to_string(),
            value: "On".to_string(),
            active: true,
        });
        let inactive = card_visual_settings_row_paint_spec(&CardVisualRowSpec {
            title: "Sound".to_string(),
            value: "Off".to_string(),
            active: false,
        });

        assert_eq!(active.border_radius, 8.0);
        assert_eq!(active.fill_radius, 7.0);
        assert_eq!(active.border_color, CardVisualColorSpec::rgb(50, 84, 61));
        assert_eq!(active.fill_color, CardVisualColorSpec::rgb(42, 50, 44));
        assert_eq!(
            active.value_badge.background_color,
            CardVisualColorSpec::rgb(46, 68, 54)
        );
        assert_eq!(
            active.value_badge.foreground_color,
            CardVisualColorSpec::rgb(104, 222, 145)
        );
        assert_eq!(inactive.border_color, CardVisualColorSpec::rgb(50, 50, 56));
        assert_eq!(inactive.fill_color, CardVisualColorSpec::rgb(43, 43, 48));
        assert_eq!(
            inactive.value_badge.background_color,
            CardVisualColorSpec::rgb(54, 54, 58)
        );
        assert_eq!(
            inactive.value_badge.foreground_color,
            CardVisualColorSpec::rgb(230, 235, 245)
        );
        assert_eq!(active.title_size, 11);
        assert_eq!(active.value_badge.text_size, 10);
        assert_eq!(active.value_badge.width, 44.0);
    }

    #[test]
    fn card_spec_exposes_shared_body_line_palette_and_metrics() {
        let assistant = card_visual_body_line_paint_spec(
            CardVisualStyle::Default,
            CardVisualBodyRole::Assistant,
            Some("$"),
        );
        let user = card_visual_body_line_paint_spec(
            CardVisualStyle::Default,
            CardVisualBodyRole::User,
            Some(">"),
        );
        let completion = card_visual_body_line_paint_spec(
            CardVisualStyle::Completion,
            CardVisualBodyRole::Assistant,
            Some("$"),
        );
        let pending_question = card_visual_body_line_paint_spec(
            CardVisualStyle::PendingQuestion,
            CardVisualBodyRole::Plain,
            Some("?"),
        );

        assert_eq!(
            assistant.prefix_color,
            CardVisualColorSpec::rgb(217, 120, 87)
        );
        assert_eq!(user.prefix_color, CardVisualColorSpec::rgb(104, 222, 145));
        assert_eq!(user.text_color, CardVisualColorSpec::rgb(218, 222, 229));
        assert_eq!(
            completion.prefix_color,
            CardVisualColorSpec::rgb(104, 222, 145)
        );
        assert_eq!(
            pending_question.prefix_color,
            CardVisualColorSpec::rgb(201, 176, 255)
        );
        assert_eq!(assistant.prefix_size, 10);
        assert_eq!(assistant.text_size, 10);
    }

    #[test]
    fn card_spec_exposes_shared_tool_pill_palette_and_metrics() {
        let bash = card_visual_tool_pill_paint_spec("Bash cargo test").expect("bash tool");
        let edit = card_visual_tool_pill_paint_spec("Edit src/main.rs").expect("edit tool");
        let unknown = card_visual_tool_pill_paint_spec("Custom task").expect("custom tool");

        assert_eq!(bash.tool_name, "Bash");
        assert_eq!(bash.description.as_deref(), Some("cargo test"));
        assert_eq!(
            bash.tool_name_color,
            CardVisualColorSpec::rgb(125, 242, 163)
        );
        assert_eq!(
            edit.tool_name_color,
            CardVisualColorSpec::rgb(135, 171, 255)
        );
        assert_eq!(
            unknown.tool_name_color,
            CardVisualColorSpec::rgb(245, 247, 252)
        );
        assert_eq!(bash.background_color, CardVisualColorSpec::rgb(47, 47, 52));
        assert_eq!(
            bash.description_color,
            CardVisualColorSpec::rgb(214, 218, 225)
        );
        assert_eq!(bash.height, 22.0);
        assert_eq!(bash.radius, 5.0);
        assert!(bash.width > edit.tool_name_width);
    }

    #[test]
    fn card_spec_exposes_shared_action_hint_pill_palette_and_metrics() {
        let spec = card_visual_action_hint_paint_spec("Allow / Deny in terminal")
            .expect("action hint spec");

        assert_eq!(spec.height, 18.0);
        assert_eq!(spec.radius, 9.0);
        assert_eq!(spec.text_inset_x, 9.0);
        assert_eq!(spec.text_offset_y, 4.0);
        assert_eq!(spec.text_size, 10);
        assert_eq!(spec.background_color, CardVisualColorSpec::rgb(49, 49, 53));
        assert_eq!(
            spec.foreground_color,
            CardVisualColorSpec::rgb(230, 235, 245)
        );
        assert!(spec.width > 32.0);
        assert!(card_visual_action_hint_paint_spec("   ").is_none());
    }

    #[test]
    fn card_spec_exposes_shared_header_text_palette_and_metrics() {
        let regular = card_visual_header_text_paint_spec(CardVisualStyle::Default);
        let empty = card_visual_header_text_paint_spec(CardVisualStyle::Empty);

        assert_eq!(regular.title.color, CardVisualColorSpec::rgb(245, 247, 252));
        assert_eq!(regular.title.size, 12);
        assert_eq!(regular.title_max_chars, 30);
        assert_eq!(
            regular.subtitle.color,
            CardVisualColorSpec::rgb(171, 179, 194)
        );
        assert_eq!(regular.subtitle.size, 9);
        assert_eq!(empty.title.color, CardVisualColorSpec::rgb(171, 179, 194));
        assert_eq!(empty.title.size, 12);
    }

    #[test]
    fn card_spec_exposes_shared_content_reveal_animation_frame() {
        let hidden = card_visual_content_reveal_frame(0.0);
        let open = card_visual_content_reveal_frame(1.0);

        assert_eq!(hidden.visibility_progress, 0.0);
        assert_eq!(hidden.translate_y, -5.0);
        assert_eq!(open.visibility_progress, 1.0);
        assert_eq!(open.translate_y, 0.0);
        assert!(card_visual_content_reveal_frame(0.5).visibility_progress > 0.0);
    }

    #[test]
    fn card_spec_exposes_shared_content_exit_animation_frame() {
        let opening_mid = card_visual_content_transition_frame(0.5, false);
        let closing_mid = card_visual_content_transition_frame(0.5, true);
        let closing_hidden = card_visual_content_transition_frame(
            crate::native_panel_core::PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS - 0.01,
            true,
        );

        assert!(closing_mid.visibility_progress < opening_mid.visibility_progress);
        assert_eq!(closing_hidden.visibility_progress, 0.0);
        assert_eq!(
            card_visual_content_transition_frame(1.0, true).visibility_progress,
            1.0
        );
    }

    #[test]
    fn card_spec_exposes_shared_stack_reveal_animation_frame() {
        let first = card_visual_stack_reveal_frame(0.88, 2, 0);
        let second = card_visual_stack_reveal_frame(0.88, 2, 1);
        let hidden = card_visual_stack_reveal_frame(0.0, 2, 0);

        assert_eq!(first.progress, 1.0);
        assert_eq!(first.card_phase, 1.0);
        assert_eq!(second.card_phase, 1.0);
        assert_eq!(hidden.progress, 0.0);
        assert_eq!(hidden.card_phase, 0.0);
        assert!(card_visual_stack_reveal_frame(0.44, 2, 0).card_phase > 0.0);
    }

    #[test]
    fn card_spec_exposes_shared_shell_reveal_frame() {
        let expanded = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        };

        let collapsed = card_visual_shell_reveal_frame(expanded, 52.0, 0.0);
        let open = card_visual_shell_reveal_frame(expanded, 52.0, 1.0);

        assert_eq!(collapsed.width, 192.0);
        assert_eq!(collapsed.height, 52.0);
        assert_eq!(collapsed.x, 14.0);
        assert_eq!(collapsed.y, 68.0);
        assert_eq!(open, expanded);
    }

    #[test]
    fn card_spec_exposes_shared_content_layout_metrics() {
        let frame = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        };
        let layout = card_visual_content_layout(frame);

        assert_eq!(layout.content_x, 20.0);
        assert_eq!(layout.content_width, 180.0);
        assert_eq!(layout.title_y, 96.0);
        assert_eq!(layout.subtitle_y, 80.0);
        assert_eq!(layout.empty_title_y, 60.0);
    }

    #[test]
    fn card_spec_exposes_shared_body_layout_metrics() {
        let frame = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        };
        let default = card_visual_body_layout(frame, false);
        let with_hint = card_visual_body_layout(frame, true);

        assert_eq!(default.prefix_x, 20.0);
        assert_eq!(default.text_x, 35.0);
        assert_eq!(default.body_width, 165.0);
        assert_eq!(default.initial_y, 26.0);
        assert_eq!(with_hint.initial_y, 53.0);
    }

    #[test]
    fn card_spec_exposes_shared_action_hint_layout() {
        let frame = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        };

        let layout = card_visual_action_hint_layout(frame, "Go").expect("action hint layout");

        assert_eq!(
            layout.pill_frame,
            PanelRect {
                x: 20.0,
                y: 29.0,
                width: 32.0,
                height: 18.0,
            }
        );
        assert_eq!(layout.text_origin.x, 29.0);
        assert_eq!(layout.text_origin.y, 33.0);
        assert_eq!(layout.text_max_width, 14.0);
        assert!(card_visual_action_hint_layout(frame, "   ").is_none());
    }

    #[test]
    fn card_spec_exposes_shared_tool_pill_layout() {
        let frame = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        };

        let layout =
            card_visual_tool_pill_layout(frame, 42.0, "Bash cargo test").expect("tool pill layout");

        assert_eq!(layout.pill_frame.x, 20.0);
        assert_eq!(layout.pill_frame.y, 42.0);
        assert_eq!(layout.pill_frame.height, 22.0);
        assert_eq!(layout.tool_name_origin.x, 27.0);
        assert_eq!(layout.tool_name_origin.y, 47.0);
        assert!(layout.tool_name_max_width > 0.0);
        let description = layout.description.expect("description layout");
        assert!(description.origin.x > layout.tool_name_origin.x);
        assert_eq!(description.origin.y, 47.0);
        assert!(description.max_width > 0.0);
    }

    #[test]
    fn card_spec_exposes_shared_header_badge_layout() {
        let badge = CardVisualBadgeSpec {
            role: CardVisualBadgeRole::Status,
            text: "Done".to_string(),
            emphasized: true,
        };

        let layout = card_visual_badge_layout(CardVisualStyle::Completion, &badge, 210.0, 96.0);

        assert_eq!(layout.badge_frame.y, 93.0);
        assert_eq!(layout.badge_frame.height, 22.0);
        assert_eq!(layout.badge_frame.x + layout.badge_frame.width, 210.0);
        assert_eq!(layout.text_origin.x, layout.badge_frame.x + 7.0);
        assert_eq!(layout.text_origin.y, 97.0);
        assert_eq!(
            layout.paint.foreground_color,
            CardVisualColorSpec::rgb(102, 222, 145)
        );
    }

    #[test]
    fn card_spec_exposes_shared_settings_row_layout() {
        let frame = PanelRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
        };
        let row = CardVisualRowSpec {
            title: "Mascot".to_string(),
            value: "On".to_string(),
            active: true,
        };

        let layout = card_visual_settings_row_layout(frame, 0, &row).expect("settings row layout");

        assert_eq!(
            layout.row_frame,
            PanelRect {
                x: 24.0,
                y: 44.0,
                width: 172.0,
                height: 30.0,
            }
        );
        assert_eq!(
            layout.row_inner_frame,
            PanelRect {
                x: 25.0,
                y: 45.0,
                width: 170.0,
                height: 28.0,
            }
        );
        assert_eq!(layout.value_badge_frame.x, 142.0);
        assert_eq!(layout.value_badge_frame.y, 50.0);
        assert_eq!(layout.title_origin.x, 36.0);
        assert_eq!(layout.title_origin.y, 51.0);
        assert_eq!(layout.title_max_width, 95.0);
        assert_eq!(layout.value_origin.x, 151.0);
        assert_eq!(layout.value_origin.y, 54.0);
    }
}
