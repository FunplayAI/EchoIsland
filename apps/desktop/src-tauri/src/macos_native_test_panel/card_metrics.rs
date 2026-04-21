use super::*;

pub(super) fn native_status_queue_card_height(item: &NativeStatusQueueItem) -> f64 {
    match &item.payload {
        NativeStatusQueuePayload::Approval(pending) => pending_permission_card_height(pending),
        NativeStatusQueuePayload::Completion(session) => completion_card_height(session),
    }
}

pub(super) fn pending_permission_card_height(pending: &PendingPermissionView) -> f64 {
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());
    pending_like_card_height(&body, 92.0, 120.0)
}

pub(super) fn pending_question_card_height(pending: &PendingQuestionView) -> f64 {
    let body = display_snippet(Some(&pending.text), 82)
        .unwrap_or_else(|| "Waiting for your answer".to_string());
    let min_height = if pending.options.is_empty() {
        PENDING_QUESTION_CARD_MIN_HEIGHT
    } else {
        PENDING_QUESTION_CARD_MIN_HEIGHT + 6.0
    };
    pending_like_card_height(
        &body,
        min_height,
        144.0_f64.max(PENDING_QUESTION_CARD_MAX_HEIGHT),
    )
}

pub(super) fn prompt_assist_card_height(_session: &SessionSnapshotView) -> f64 {
    pending_like_card_height(
        "A command may be waiting for approval in the Codex terminal. Allow or deny it there.",
        92.0,
        108.0,
    )
}

pub(super) fn completion_card_height(session: &SessionSnapshotView) -> f64 {
    let preview = completion_preview_text(session);
    let body_height = estimated_chat_body_height(&preview, estimated_default_chat_body_width(), 2);
    (CARD_HEADER_HEIGHT + CARD_CONTENT_BOTTOM_INSET + body_height).max(76.0)
}

pub(super) fn pending_like_card_height(body: &str, min_height: f64, max_height: f64) -> f64 {
    let body_height = estimated_chat_body_height(body, estimated_default_chat_body_width(), 2);
    (58.0
        + CARD_PENDING_ACTION_Y
        + CARD_PENDING_ACTION_HEIGHT
        + CARD_PENDING_ACTION_GAP
        + body_height)
        .clamp(min_height, max_height)
}

pub(super) fn session_card_collapsed_height(target_height: f64, is_compact: bool) -> f64 {
    let limit = if is_compact { 52.0 } else { 64.0 };
    let factor = if is_compact { 0.76 } else { 0.58 };
    target_height
        .mul_add(factor, 0.0)
        .round()
        .clamp(34.0, limit)
}

pub(super) fn estimated_card_height(session: &SessionSnapshotView) -> f64 {
    if is_long_idle_session(session) || !session_has_visible_card_body(session) {
        return 58.0;
    }

    let content_height = estimated_session_card_content_height(
        session_prompt_preview(session).as_deref(),
        session_reply_preview(session).as_deref(),
        session_tool_preview(session).is_some(),
    );
    (CARD_HEADER_HEIGHT + content_height).max(58.0)
}

pub(super) fn estimated_session_card_content_height(
    prompt: Option<&str>,
    reply: Option<&str>,
    has_tool: bool,
) -> f64 {
    let mut content_height = CARD_CONTENT_BOTTOM_INSET;
    let has_prompt = prompt.is_some_and(|value| !value.is_empty());
    let has_reply = reply.is_some_and(|value| !value.is_empty());

    if has_tool {
        content_height += 22.0;
        if has_reply || has_prompt {
            content_height += CARD_TOOL_GAP;
        }
    }
    if let Some(body) = reply.filter(|value| !value.is_empty()) {
        content_height += estimated_chat_body_height(body, estimated_default_chat_body_width(), 2);
        if has_prompt {
            content_height += CARD_CHAT_GAP;
        }
    }
    if let Some(body) = prompt.filter(|value| !value.is_empty()) {
        content_height += estimated_chat_body_height(body, estimated_default_chat_body_width(), 1);
    }

    content_height
}

pub(super) fn session_prompt_preview(session: &SessionSnapshotView) -> Option<String> {
    display_snippet(session.last_user_prompt.as_deref(), 68)
}

pub(super) fn session_reply_preview(session: &SessionSnapshotView) -> Option<String> {
    display_snippet(
        session
            .last_assistant_message
            .as_deref()
            .or(session.tool_description.as_deref()),
        92,
    )
}

pub(super) fn session_tool_preview(
    session: &SessionSnapshotView,
) -> Option<(String, Option<String>)> {
    let tool_name = session.current_tool.as_deref()?.trim();
    if tool_name.is_empty() {
        return None;
    }

    Some((
        tool_name.to_string(),
        display_snippet(session.tool_description.as_deref(), 48),
    ))
}

pub(super) fn session_has_visible_card_body(session: &SessionSnapshotView) -> bool {
    session_prompt_preview(session).is_some()
        || session_reply_preview(session).is_some()
        || session_tool_preview(session).is_some()
}

pub(super) fn completion_preview_text(session: &SessionSnapshotView) -> String {
    session_reply_preview(session).unwrap_or_else(|| "Task complete".to_string())
}

pub(super) fn is_long_idle_session(session: &SessionSnapshotView) -> bool {
    normalize_status(&session.status) == "idle"
        && (Utc::now() - session.last_activity).num_minutes() > 15
}
