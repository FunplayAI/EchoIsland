use chrono::Utc;
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

use super::card_views::settings_surface_card_height;
use super::display_helpers::{display_snippet, normalize_status};
use super::panel_constants::{
    EXPANDED_CARD_GAP, EXPANDED_MAX_BODY_HEIGHT, PENDING_QUESTION_CARD_MAX_HEIGHT,
    PENDING_QUESTION_CARD_MIN_HEIGHT,
};
use super::panel_helpers::estimated_default_chat_body_width;
use super::panel_scene_adapter::resolve_or_build_native_panel_scene;
use super::panel_types::{NativeStatusQueueItem, NativeStatusQueuePayload};
use crate::native_panel_scene::{PanelScene, SceneCard};

pub(super) fn estimated_expanded_body_height(snapshot: &RuntimeSnapshot) -> f64 {
    estimated_expanded_content_height(snapshot).min(EXPANDED_MAX_BODY_HEIGHT)
}

pub(super) fn estimated_expanded_content_height(snapshot: &RuntimeSnapshot) -> f64 {
    let scene = resolve_or_build_native_panel_scene(snapshot);
    estimated_scene_content_height(&scene)
}

pub(super) fn estimated_scene_content_height(scene: &PanelScene) -> f64 {
    estimated_scene_cards_content_height(&scene.cards)
}

pub(super) fn estimated_scene_cards_content_height(cards: &[SceneCard]) -> f64 {
    let card_heights = cards
        .iter()
        .map(estimated_scene_card_height)
        .collect::<Vec<_>>();
    crate::native_panel_core::resolve_stacked_cards_total_height(
        &card_heights,
        EXPANDED_CARD_GAP,
        crate::native_panel_core::EMPTY_CARD_HEIGHT,
    )
}

pub(super) fn estimated_scene_card_height(card: &SceneCard) -> f64 {
    match crate::native_panel_scene::resolve_scene_card_height_input(card) {
        crate::native_panel_scene::SceneCardHeightInput::Settings => settings_surface_card_height(),
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
            estimated_card_height(session)
        }
        crate::native_panel_scene::SceneCardHeightInput::StatusItem(item) => {
            native_status_queue_card_height(item)
        }
        crate::native_panel_scene::SceneCardHeightInput::Empty => {
            crate::native_panel_core::EMPTY_CARD_HEIGHT
        }
    }
}

pub(super) fn native_status_queue_card_height(item: &NativeStatusQueueItem) -> f64 {
    match &item.payload {
        NativeStatusQueuePayload::Approval(pending) => pending_permission_card_height(pending),
        NativeStatusQueuePayload::Completion(session) => completion_card_height(session),
    }
}

pub(super) fn pending_permission_card_height(pending: &PendingPermissionView) -> f64 {
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());
    pending_like_card_height(
        &body,
        crate::native_panel_core::PENDING_PERMISSION_CARD_MIN_HEIGHT,
        crate::native_panel_core::PENDING_PERMISSION_CARD_MAX_HEIGHT,
    )
}

pub(super) fn pending_question_card_height(pending: &PendingQuestionView) -> f64 {
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

pub(super) fn prompt_assist_card_height(_session: &SessionSnapshotView) -> f64 {
    pending_like_card_height(
        "A command may be waiting for approval in the Codex terminal. Allow or deny it there.",
        crate::native_panel_core::PROMPT_ASSIST_CARD_MIN_HEIGHT,
        crate::native_panel_core::PROMPT_ASSIST_CARD_MAX_HEIGHT,
    )
}

pub(super) fn completion_card_height(session: &SessionSnapshotView) -> f64 {
    let preview = completion_preview_text(session);
    crate::native_panel_core::resolve_completion_card_height(
        &preview,
        estimated_default_chat_body_width(),
        native_card_metrics(),
    )
}

pub(super) fn pending_like_card_height(body: &str, min_height: f64, max_height: f64) -> f64 {
    crate::native_panel_core::resolve_pending_like_card_height(
        body,
        min_height,
        max_height,
        estimated_default_chat_body_width(),
        native_card_metrics(),
    )
}

pub(super) fn session_card_collapsed_height(target_height: f64, is_compact: bool) -> f64 {
    crate::native_panel_core::resolve_session_card_collapsed_height(target_height, is_compact)
}

pub(super) fn estimated_card_height(session: &SessionSnapshotView) -> f64 {
    if is_long_idle_session(session) || !session_has_visible_card_body(session) {
        return 58.0;
    }

    let prompt = session_prompt_preview(session);
    let reply = session_reply_preview(session);
    let content_height = estimated_session_card_content_height(
        prompt.as_deref(),
        reply.as_deref(),
        session_tool_preview(session).is_some(),
    );
    crate::native_panel_core::resolve_session_card_height(
        false,
        true,
        content_height,
        native_card_metrics(),
    )
}

pub(super) fn estimated_session_card_content_height(
    prompt: Option<&str>,
    reply: Option<&str>,
    has_tool: bool,
) -> f64 {
    crate::native_panel_core::resolve_session_card_content_height(
        crate::native_panel_core::SessionCardContentInput {
            prompt,
            reply,
            has_tool,
            default_body_width: estimated_default_chat_body_width(),
            metrics: native_card_metrics(),
        },
    )
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

pub(super) fn native_card_metrics() -> crate::native_panel_core::PanelCardMetricConstants {
    crate::native_panel_core::default_panel_card_metric_constants()
}
