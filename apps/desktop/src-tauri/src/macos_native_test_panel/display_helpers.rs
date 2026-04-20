use super::*;

pub(super) fn displayed_sessions(
    snapshot: &RuntimeSnapshot,
    prompt_assist_sessions: &[SessionSnapshotView],
) -> Vec<SessionSnapshotView> {
    let blocked_session_ids = blocked_session_ids(snapshot, prompt_assist_sessions);
    let mut sessions = snapshot
        .sessions
        .iter()
        .filter(|session| !should_hide_legacy_opencode_session(session))
        .filter(|session| !blocked_session_ids.contains(&session.session_id))
        .cloned()
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| {
        let priority_diff = status_priority(&left.status).cmp(&status_priority(&right.status));
        if priority_diff == std::cmp::Ordering::Equal {
            right.last_activity.cmp(&left.last_activity)
        } else {
            priority_diff
        }
    });
    sessions.truncate(MAX_VISIBLE_SESSIONS);
    sessions
}

pub(super) fn displayed_pending_permissions(
    snapshot: &RuntimeSnapshot,
) -> Vec<PendingPermissionView> {
    let mut permissions = if snapshot.pending_permissions.is_empty() {
        snapshot.pending_permission.clone().into_iter().collect()
    } else {
        snapshot.pending_permissions.clone()
    };
    permissions.sort_by(|left, right| left.requested_at.cmp(&right.requested_at));
    permissions
}

pub(super) fn displayed_default_pending_permissions(
    snapshot: &RuntimeSnapshot,
) -> Vec<PendingPermissionView> {
    let permission = snapshot
        .pending_permission
        .clone()
        .or_else(|| displayed_pending_permissions(snapshot).into_iter().next());
    permission.into_iter().collect()
}

pub(super) fn displayed_pending_questions(snapshot: &RuntimeSnapshot) -> Vec<PendingQuestionView> {
    let mut questions = if snapshot.pending_questions.is_empty() {
        snapshot.pending_question.clone().into_iter().collect()
    } else {
        snapshot.pending_questions.clone()
    };
    questions.sort_by(|left, right| left.requested_at.cmp(&right.requested_at));
    questions
}

pub(super) fn displayed_default_pending_questions(
    snapshot: &RuntimeSnapshot,
) -> Vec<PendingQuestionView> {
    let question = snapshot
        .pending_question
        .clone()
        .or_else(|| displayed_pending_questions(snapshot).into_iter().next());
    question.into_iter().collect()
}

pub(super) fn blocked_session_ids(
    snapshot: &RuntimeSnapshot,
    prompt_assist_sessions: &[SessionSnapshotView],
) -> HashSet<String> {
    displayed_pending_permissions(snapshot)
        .into_iter()
        .map(|pending| pending.session_id)
        .chain(
            displayed_pending_questions(snapshot)
                .into_iter()
                .map(|pending| pending.session_id),
        )
        .chain(
            prompt_assist_sessions
                .iter()
                .map(|session| session.session_id.clone()),
        )
        .filter(|session_id| !session_id.trim().is_empty())
        .collect()
}

pub(super) fn displayed_prompt_assist_sessions(
    snapshot: &RuntimeSnapshot,
) -> Vec<SessionSnapshotView> {
    let live_pending_session_ids = live_pending_session_ids(snapshot);
    let now = Utc::now();
    let mut sessions = snapshot
        .sessions
        .iter()
        .filter(|session| !live_pending_session_ids.contains(&session.session_id))
        .filter(|session| is_prompt_assist_session(session, now))
        .cloned()
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| right.last_activity.cmp(&left.last_activity));
    sessions.truncate(1);
    sessions
}

pub(super) fn live_pending_session_ids(snapshot: &RuntimeSnapshot) -> HashSet<String> {
    displayed_pending_permissions(snapshot)
        .into_iter()
        .map(|pending| pending.session_id)
        .chain(
            displayed_pending_questions(snapshot)
                .into_iter()
                .map(|pending| pending.session_id),
        )
        .filter(|session_id| !session_id.trim().is_empty())
        .collect()
}

pub(super) fn is_prompt_assist_session(
    session: &SessionSnapshotView,
    now: chrono::DateTime<Utc>,
) -> bool {
    if session.source.to_ascii_lowercase() != "codex" {
        return false;
    }

    let status = normalize_status(&session.status);
    if status != "processing" && status != "running" {
        return false;
    }

    let age_seconds = (now - session.last_activity).num_seconds();
    let stale_seconds = if status == "running" {
        PROMPT_ASSIST_RUNNING_SECONDS
    } else {
        PROMPT_ASSIST_PROCESSING_SECONDS
    };
    age_seconds >= stale_seconds && age_seconds <= PROMPT_ASSIST_RECENT_SECONDS
}

pub(super) fn should_hide_legacy_opencode_session(session: &SessionSnapshotView) -> bool {
    let source = session.source.to_ascii_lowercase();
    source == "opencode"
        && session.session_id.starts_with("open-")
        && session.cwd.is_none()
        && session.project_name.is_none()
        && session.model.is_none()
        && session.current_tool.is_none()
        && session.tool_description.is_none()
        && session.last_user_prompt.is_none()
        && session.last_assistant_message.is_none()
}

pub(super) fn status_priority(status: &str) -> u8 {
    match normalize_status(status).as_str() {
        "waitingapproval" | "waitingquestion" => 0,
        "running" => 1,
        "processing" => 2,
        _ => 3,
    }
}

pub(super) fn normalize_status(status: &str) -> String {
    status.to_ascii_lowercase()
}

pub(super) fn format_source(source: &str) -> String {
    match source.to_ascii_lowercase().as_str() {
        "claude" => "Claude".to_string(),
        "codex" => "Codex".to_string(),
        "cursor" => "Cursor".to_string(),
        "gemini" => "Gemini".to_string(),
        "copilot" => "Copilot".to_string(),
        "qoder" => "Qoder".to_string(),
        "codebuddy" => "CodeBuddy".to_string(),
        "opencode" => "OpenCode".to_string(),
        "openclaw" => "OpenClaw".to_string(),
        other => {
            let mut chars = other.chars();
            if let Some(first) = chars.next() {
                format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.collect::<String>()
                )
            } else {
                "Unknown".to_string()
            }
        }
    }
}

pub(super) fn format_status(status: &str) -> String {
    match normalize_status(status).as_str() {
        "running" => "Running".to_string(),
        "processing" => "Thinking".to_string(),
        "waitingapproval" => "Approval".to_string(),
        "waitingquestion" => "Question".to_string(),
        "idle" => "Idle".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn session_title(session: &SessionSnapshotView) -> String {
    let project_name = display_project_name(session);
    if project_name != "Session" {
        return project_name;
    }
    format!(
        "{} {}",
        format_source(&session.source),
        short_session_id(&session.session_id)
    )
}

pub(super) fn display_project_name(session: &SessionSnapshotView) -> String {
    let raw = session
        .project_name
        .as_deref()
        .or(session.cwd.as_deref())
        .unwrap_or("Session");
    raw.split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
        .next_back()
        .map(|segment| segment.replace(':', ""))
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "Session".to_string())
}

pub(super) fn compact_title(value: &str, max_length: usize) -> String {
    let text = value.trim();
    if text.chars().count() <= max_length {
        return text.to_string();
    }
    let head_length = (((max_length - 1) as f64) * 0.58).ceil() as usize;
    let tail_length = max_length.saturating_sub(1 + head_length);
    let head = text.chars().take(head_length).collect::<String>();
    let tail = text
        .chars()
        .rev()
        .take(tail_length)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{head}…{tail}")
}

pub(super) fn short_session_id(session_id: &str) -> String {
    session_id
        .split_once('-')
        .map(|(_, tail)| tail.chars().take(6).collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "------".to_string())
}

pub(super) fn time_ago(last_activity: chrono::DateTime<chrono::Utc>) -> String {
    let diff = Utc::now() - last_activity;
    if diff.num_minutes() < 1 {
        "now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h", diff.num_hours())
    } else {
        format!("{}d", diff.num_days())
    }
}

pub(super) fn session_meta_line(session: &SessionSnapshotView) -> String {
    let mut parts = vec![format!("#{}", short_session_id(&session.session_id))];
    if let Some(model) = session
        .model
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(model.to_string());
    }
    parts.push(time_ago(session.last_activity));
    parts.join(" · ")
}

pub(super) fn display_snippet(value: Option<&str>, max_chars: usize) -> Option<String> {
    let value = value?.replace(['\r', '\n'], " ");
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return None;
    }
    let text = compact.replace(['`', '*', '_', '~', '|'], "");
    if text.chars().count() <= max_chars {
        Some(text)
    } else {
        Some(format!(
            "{}…",
            text.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        ))
    }
}
