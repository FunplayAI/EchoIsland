use serde::Serialize;

use echoisland_runtime::{PendingPermissionView, PendingQuestionView, SessionSnapshotView};

use crate::native_panel_core::{
    StatusQueueItem, StatusQueuePayload, compact_title, completion_preview_text, display_snippet,
    format_source, session_title, short_session_id, time_ago,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusCardSceneKind {
    Approval,
    Question,
    Completion,
    PromptAssist,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatusCardScene {
    pub(crate) kind: StatusCardSceneKind,
    pub(crate) session_id: String,
    pub(crate) request_id: Option<String>,
    pub(crate) title: String,
    pub(crate) meta: String,
    pub(crate) status_text: String,
    pub(crate) source_text: String,
    pub(crate) body: String,
    pub(crate) action_hint: Option<String>,
    pub(crate) answer_options: Vec<String>,
    pub(crate) is_live: bool,
    pub(crate) is_removing: bool,
}

pub(crate) fn build_pending_permission_status_card_scene(
    pending: &PendingPermissionView,
) -> StatusCardScene {
    let title = pending
        .tool_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Tool permission");
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());

    StatusCardScene {
        kind: StatusCardSceneKind::Approval,
        session_id: pending.session_id.clone(),
        request_id: Some(pending.request_id.clone()),
        title: compact_title(title, 34),
        meta: format!("#{} · Approval", short_session_id(&pending.session_id)),
        status_text: "Approval".to_string(),
        source_text: format_source(&pending.source),
        body,
        action_hint: Some("Allow / Deny in terminal".to_string()),
        answer_options: Vec::new(),
        is_live: true,
        is_removing: false,
    }
}

pub(crate) fn build_pending_question_status_card_scene(
    pending: &PendingQuestionView,
) -> StatusCardScene {
    let title = pending
        .header
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Need your input");
    let body = display_snippet(Some(&pending.text), 82)
        .unwrap_or_else(|| "Waiting for your answer".to_string());

    StatusCardScene {
        kind: StatusCardSceneKind::Question,
        session_id: pending.session_id.clone(),
        request_id: Some(pending.request_id.clone()),
        title: compact_title(title, 34),
        meta: format!("#{} · Question", short_session_id(&pending.session_id)),
        status_text: "Question".to_string(),
        source_text: format_source(&pending.source),
        body,
        action_hint: Some(pending_question_action_hint(pending)),
        answer_options: pending.options.clone(),
        is_live: true,
        is_removing: false,
    }
}

pub(crate) fn build_completion_status_card_scene(session: &SessionSnapshotView) -> StatusCardScene {
    StatusCardScene {
        kind: StatusCardSceneKind::Completion,
        session_id: session.session_id.clone(),
        request_id: None,
        title: compact_title(&session_title(session), 30),
        meta: format!(
            "#{} · {}",
            short_session_id(&session.session_id),
            time_ago(session.last_activity)
        ),
        status_text: "Complete".to_string(),
        source_text: format_source(&session.source),
        body: completion_preview_text(session),
        action_hint: None,
        answer_options: Vec::new(),
        is_live: true,
        is_removing: false,
    }
}

pub(crate) fn build_prompt_assist_status_card_scene(
    session: &SessionSnapshotView,
) -> StatusCardScene {
    StatusCardScene {
        kind: StatusCardSceneKind::PromptAssist,
        session_id: session.session_id.clone(),
        request_id: None,
        title: "Codex needs attention".to_string(),
        meta: format!(
            "#{} · {} · {}",
            short_session_id(&session.session_id),
            compact_title(&session_title(session), 24),
            time_ago(session.last_activity)
        ),
        status_text: "Check".to_string(),
        source_text: "Codex".to_string(),
        body: "Approval may be required in the Codex terminal.".to_string(),
        action_hint: Some("Open terminal to check".to_string()),
        answer_options: Vec::new(),
        is_live: true,
        is_removing: false,
    }
}

pub(crate) fn build_status_queue_status_card_scene(item: &StatusQueueItem) -> StatusCardScene {
    let mut scene = match &item.payload {
        StatusQueuePayload::Approval(pending) => {
            build_pending_permission_status_card_scene(pending)
        }
        StatusQueuePayload::Question(pending) => build_pending_question_status_card_scene(pending),
        StatusQueuePayload::Completion(session) => build_completion_status_card_scene(session),
    };
    scene.is_live = item.is_live;
    scene.is_removing = item.is_removing;
    scene
}

fn pending_question_action_hint(pending: &PendingQuestionView) -> String {
    if pending.options.is_empty() {
        return "Answer in terminal".to_string();
    }

    let options = pending
        .options
        .iter()
        .take(3)
        .map(|option| compact_title(option, 12))
        .collect::<Vec<_>>()
        .join(" / ");
    if pending.options.len() > 3 {
        format!("{options} / …")
    } else {
        options
    }
}
