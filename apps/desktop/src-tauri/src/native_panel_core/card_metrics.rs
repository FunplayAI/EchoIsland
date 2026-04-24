use super::constants::{
    CARD_CHAT_GAP, CARD_CHAT_LINE_HEIGHT, CARD_CHAT_PREFIX_WIDTH, CARD_CONTENT_BOTTOM_INSET,
    CARD_HEADER_HEIGHT, CARD_INSET_X, CARD_PENDING_ACTION_GAP, CARD_PENDING_ACTION_HEIGHT,
    CARD_PENDING_ACTION_Y, CARD_TOOL_GAP,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelCardMetricConstants {
    pub(crate) card_inset_x: f64,
    pub(crate) chat_prefix_width: f64,
    pub(crate) chat_line_height: f64,
    pub(crate) header_height: f64,
    pub(crate) content_bottom_inset: f64,
    pub(crate) chat_gap: f64,
    pub(crate) tool_gap: f64,
    pub(crate) pending_action_y: f64,
    pub(crate) pending_action_height: f64,
    pub(crate) pending_action_gap: f64,
}

pub(crate) fn default_panel_card_metric_constants() -> PanelCardMetricConstants {
    PanelCardMetricConstants {
        card_inset_x: CARD_INSET_X,
        chat_prefix_width: CARD_CHAT_PREFIX_WIDTH,
        chat_line_height: CARD_CHAT_LINE_HEIGHT,
        header_height: CARD_HEADER_HEIGHT,
        content_bottom_inset: CARD_CONTENT_BOTTOM_INSET,
        chat_gap: CARD_CHAT_GAP,
        tool_gap: CARD_TOOL_GAP,
        pending_action_y: CARD_PENDING_ACTION_Y,
        pending_action_height: CARD_PENDING_ACTION_HEIGHT,
        pending_action_gap: CARD_PENDING_ACTION_GAP,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SessionCardContentInput<'a> {
    pub(crate) prompt: Option<&'a str>,
    pub(crate) reply: Option<&'a str>,
    pub(crate) has_tool: bool,
    pub(crate) default_body_width: f64,
    pub(crate) metrics: PanelCardMetricConstants,
}

pub(crate) fn resolve_card_chat_body_width(
    card_width: f64,
    metrics: PanelCardMetricConstants,
) -> f64 {
    (card_width - (metrics.card_inset_x * 2.0) - metrics.chat_prefix_width).max(1.0)
}

pub(crate) fn resolve_estimated_chat_body_height(
    body: &str,
    width: f64,
    max_lines: isize,
    metrics: PanelCardMetricConstants,
) -> f64 {
    resolve_estimated_chat_line_count(body, width, max_lines) as f64 * metrics.chat_line_height
}

pub(crate) fn resolve_estimated_chat_line_count(body: &str, width: f64, max_lines: isize) -> isize {
    let max_lines = max_lines.max(1);
    let line_count = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                1
            } else {
                (resolve_estimated_text_width(trimmed, 10.0) / width.max(1.0)).ceil() as isize
            }
        })
        .sum::<isize>()
        .max(1);
    line_count.min(max_lines)
}

pub(crate) fn resolve_estimated_text_width(text: &str, font_size: f64) -> f64 {
    text.chars()
        .map(|ch| {
            let factor = if ch.is_ascii_whitespace() {
                0.34
            } else if ch.is_ascii_uppercase() {
                0.66
            } else if ch.is_ascii_punctuation() {
                0.42
            } else if ch.is_ascii() {
                0.60
            } else {
                1.0
            };
            factor * font_size
        })
        .sum::<f64>()
        .max(font_size)
}

pub(crate) fn resolve_pending_like_card_height(
    body: &str,
    min_height: f64,
    max_height: f64,
    default_body_width: f64,
    metrics: PanelCardMetricConstants,
) -> f64 {
    let body_height = resolve_estimated_chat_body_height(body, default_body_width, 2, metrics);
    (58.0
        + metrics.pending_action_y
        + metrics.pending_action_height
        + metrics.pending_action_gap
        + body_height)
        .clamp(min_height, max_height)
}

pub(crate) fn resolve_session_card_collapsed_height(target_height: f64, is_compact: bool) -> f64 {
    let limit = if is_compact { 52.0 } else { 64.0 };
    let factor = if is_compact { 0.76 } else { 0.58 };
    target_height
        .mul_add(factor, 0.0)
        .round()
        .clamp(34.0, limit)
}

pub(crate) fn resolve_session_card_content_height(input: SessionCardContentInput<'_>) -> f64 {
    let mut content_height = input.metrics.content_bottom_inset;
    let has_prompt = input.prompt.is_some_and(|value| !value.is_empty());
    let has_reply = input.reply.is_some_and(|value| !value.is_empty());

    if input.has_tool {
        content_height += 22.0;
        if has_reply || has_prompt {
            content_height += input.metrics.tool_gap;
        }
    }
    if let Some(body) = input.reply.filter(|value| !value.is_empty()) {
        content_height +=
            resolve_estimated_chat_body_height(body, input.default_body_width, 2, input.metrics);
        if has_prompt {
            content_height += input.metrics.chat_gap;
        }
    }
    if let Some(body) = input.prompt.filter(|value| !value.is_empty()) {
        content_height +=
            resolve_estimated_chat_body_height(body, input.default_body_width, 1, input.metrics);
    }

    content_height
}

pub(crate) fn resolve_session_card_height(
    is_long_idle: bool,
    has_visible_body: bool,
    content_height: f64,
    metrics: PanelCardMetricConstants,
) -> f64 {
    if is_long_idle || !has_visible_body {
        return 58.0;
    }

    (metrics.header_height + content_height).max(58.0)
}

pub(crate) fn resolve_completion_card_height(
    preview: &str,
    default_body_width: f64,
    metrics: PanelCardMetricConstants,
) -> f64 {
    let body_height = resolve_estimated_chat_body_height(preview, default_body_width, 2, metrics);
    (metrics.header_height + metrics.content_bottom_inset + body_height).max(76.0)
}

pub(crate) fn resolve_stacked_cards_total_height(
    card_heights: &[f64],
    card_gap: f64,
    empty_height: f64,
) -> f64 {
    if card_heights.is_empty() {
        return empty_height;
    }

    let total = card_heights.iter().sum::<f64>();
    let gaps = card_gap * (card_heights.len().saturating_sub(1) as f64);
    total + gaps
}
