use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use chrono::Utc;
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

use super::{
    CompletionBadgeItem, ExpandedSurface, PanelMascotBaseState, PanelSnapshotSyncResult,
    PanelState, PendingPermissionCardState, PendingQuestionCardState, StatusQueueItem,
    StatusQueuePayload, StatusQueueSyncResult, StatusSurfaceTransition,
    constants::PANEL_CARD_EXIT_MS,
};

pub(crate) const STATUS_COMPLETION_VISIBLE_SECONDS: u64 = 10;
pub(crate) const STATUS_APPROVAL_VISIBLE_SECONDS: u64 = 30;
pub(crate) const STATUS_QUEUE_EXIT_EXTRA_MS: u64 = 80;
pub(crate) const PENDING_CARD_MIN_VISIBLE_MS: u64 = 2200;
pub(crate) const PENDING_CARD_RELEASE_GRACE_MS: u64 = 1600;
pub(crate) const MAX_VISIBLE_SESSIONS: usize = 5;
pub(crate) const PROMPT_ASSIST_RUNNING_SECONDS: i64 = 12;
pub(crate) const PROMPT_ASSIST_PROCESSING_SECONDS: i64 = 18;
pub(crate) const PROMPT_ASSIST_RECENT_SECONDS: i64 = 20 * 60;

pub(crate) fn sync_panel_snapshot_state(
    state: &mut PanelState,
    raw_snapshot: &RuntimeSnapshot,
    now: chrono::DateTime<Utc>,
) -> PanelSnapshotSyncResult {
    let displayed_snapshot = sync_pending_card_visibility(state, raw_snapshot);
    let completed_session_ids = state
        .last_raw_snapshot
        .as_ref()
        .map_or_else(Vec::new, |previous| {
            detect_completed_sessions(previous, raw_snapshot, now)
        });
    sync_completion_badge(state, raw_snapshot, &completed_session_ids);
    let status_queue_sync = sync_status_queue(state, raw_snapshot);
    let status_surface_transition = sync_status_surface_policy(state, status_queue_sync);
    let play_sound = status_queue_sync.added_approvals
        + status_queue_sync.added_questions
        + status_queue_sync.added_completions
        > 0;
    let reminder =
        super::resolve_panel_sync_reminder_state(state, Some(&displayed_snapshot), play_sound);
    state.last_raw_snapshot = Some(raw_snapshot.clone());

    PanelSnapshotSyncResult {
        displayed_snapshot,
        reminder,
        panel_transition: status_surface_transition.panel_transition,
        surface_transition: status_surface_transition.surface_transition,
    }
}

pub(crate) fn panel_state_needs_status_queue_refresh(state: &PanelState) -> bool {
    !state.status_queue.is_empty()
        || state.pending_permission_card.is_some()
        || state.pending_question_card.is_some()
}

pub(crate) fn sync_pending_card_visibility(
    state: &mut PanelState,
    snapshot: &RuntimeSnapshot,
) -> RuntimeSnapshot {
    let now = Instant::now();
    let next_permission = resolve_pending_permission_card(
        displayed_pending_permissions(snapshot).into_iter().next(),
        state.pending_permission_card.as_ref(),
        now,
    );
    let next_question = resolve_pending_question_card(
        displayed_pending_questions(snapshot).into_iter().next(),
        state.pending_question_card.as_ref(),
        now,
    );

    state.pending_permission_card = next_permission.clone();
    state.pending_question_card = next_question.clone();

    apply_pending_cards_to_snapshot(snapshot, next_permission, next_question)
}

pub(crate) fn resolve_pending_permission_card(
    current_payload: Option<PendingPermissionView>,
    previous: Option<&PendingPermissionCardState>,
    now: Instant,
) -> Option<PendingPermissionCardState> {
    if let Some(payload) = current_payload {
        let started_at = previous
            .filter(|card| card.request_id == payload.request_id)
            .map(|card| card.started_at)
            .unwrap_or(now);
        return Some(PendingPermissionCardState {
            request_id: payload.request_id.clone(),
            payload,
            started_at,
            last_seen_at: now,
            visible_until: previous
                .map(|card| card.visible_until)
                .unwrap_or(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS))
                .max(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS)),
        });
    }

    let previous = previous?;
    let keep_visible_until = previous
        .visible_until
        .max(previous.last_seen_at + Duration::from_millis(PENDING_CARD_RELEASE_GRACE_MS));
    if now > keep_visible_until {
        return None;
    }

    Some(PendingPermissionCardState {
        request_id: previous.request_id.clone(),
        payload: previous.payload.clone(),
        started_at: previous.started_at,
        last_seen_at: previous.last_seen_at,
        visible_until: keep_visible_until,
    })
}

pub(crate) fn resolve_pending_question_card(
    current_payload: Option<PendingQuestionView>,
    previous: Option<&PendingQuestionCardState>,
    now: Instant,
) -> Option<PendingQuestionCardState> {
    if let Some(payload) = current_payload {
        let started_at = previous
            .filter(|card| card.request_id == payload.request_id)
            .map(|card| card.started_at)
            .unwrap_or(now);
        return Some(PendingQuestionCardState {
            request_id: payload.request_id.clone(),
            payload,
            started_at,
            last_seen_at: now,
            visible_until: previous
                .map(|card| card.visible_until)
                .unwrap_or(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS))
                .max(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS)),
        });
    }

    let previous = previous?;
    let keep_visible_until = previous
        .visible_until
        .max(previous.last_seen_at + Duration::from_millis(PENDING_CARD_RELEASE_GRACE_MS));
    if now > keep_visible_until {
        return None;
    }

    Some(PendingQuestionCardState {
        request_id: previous.request_id.clone(),
        payload: previous.payload.clone(),
        started_at: previous.started_at,
        last_seen_at: previous.last_seen_at,
        visible_until: keep_visible_until,
    })
}

pub(crate) fn apply_pending_cards_to_snapshot(
    snapshot: &RuntimeSnapshot,
    pending_permission_card: Option<PendingPermissionCardState>,
    pending_question_card: Option<PendingQuestionCardState>,
) -> RuntimeSnapshot {
    let mut next_snapshot = snapshot.clone();

    if let Some(card) = pending_permission_card {
        let mut permissions = vec![card.payload];
        let held_request_id = permissions[0].request_id.clone();
        permissions.extend(
            displayed_pending_permissions(snapshot)
                .into_iter()
                .filter(|item| item.request_id != held_request_id),
        );
        next_snapshot.pending_permission_count = permissions.len();
        next_snapshot.pending_permission = permissions.first().cloned();
        next_snapshot.pending_permissions = permissions;
    }

    if let Some(card) = pending_question_card {
        let mut questions = vec![card.payload];
        let held_request_id = questions[0].request_id.clone();
        questions.extend(
            displayed_pending_questions(snapshot)
                .into_iter()
                .filter(|item| item.request_id != held_request_id),
        );
        next_snapshot.pending_question_count = questions.len();
        next_snapshot.pending_question = questions.first().cloned();
        next_snapshot.pending_questions = questions;
    }

    next_snapshot
}

pub(crate) fn sync_status_queue(
    state: &mut PanelState,
    snapshot: &RuntimeSnapshot,
) -> StatusQueueSyncResult {
    let now = Instant::now();
    let utc_now = Utc::now();
    let previous_snapshot = state.last_raw_snapshot.as_ref();
    let completed_session_ids = previous_snapshot.map_or_else(Vec::new, |previous| {
        detect_completed_sessions(previous, snapshot, utc_now)
    });
    let previous_live_permission_ids = previous_snapshot
        .map(displayed_pending_permissions)
        .unwrap_or_default()
        .into_iter()
        .map(|pending| pending.request_id)
        .collect::<HashSet<_>>();
    let previous_live_question_ids = previous_snapshot
        .map(displayed_pending_questions)
        .unwrap_or_default()
        .into_iter()
        .map(|pending| pending.request_id)
        .collect::<HashSet<_>>();
    let mut existing_items = state
        .status_queue
        .drain(..)
        .filter(|item| {
            if item.is_removing {
                item.remove_after
                    .is_some_and(|remove_after| remove_after > now)
            } else {
                true
            }
        })
        .map(|item| (item.key.clone(), item))
        .collect::<HashMap<_, _>>();
    let mut next_items = Vec::new();
    let mut added_approvals = 0;
    let mut added_questions = 0;
    let mut added_completions = 0;

    for pending in displayed_pending_permissions(snapshot) {
        let key = format!("approval:{}", pending.request_id);
        let existing = existing_items.remove(&key);
        let is_new_live_permission = !previous_live_permission_ids.contains(&pending.request_id);
        if let Some(existing_item) = existing.as_ref() {
            if existing_item.is_removing
                && existing_item
                    .remove_after
                    .is_some_and(|remove_after| remove_after > now)
            {
                next_items.push(StatusQueueItem {
                    key,
                    session_id: pending.session_id.clone(),
                    sort_time: pending.requested_at,
                    expires_at: existing_item.expires_at,
                    is_live: false,
                    is_removing: true,
                    remove_after: existing_item.remove_after,
                    payload: StatusQueuePayload::Approval(pending),
                });
                continue;
            }
        }
        if existing.is_none() && !is_new_live_permission {
            continue;
        }
        if existing.is_none() && is_new_live_permission {
            added_approvals += 1;
        }
        next_items.push(StatusQueueItem {
            key,
            session_id: pending.session_id.clone(),
            sort_time: pending.requested_at,
            expires_at: existing
                .as_ref()
                .map(|item| item.expires_at)
                .unwrap_or_else(|| now + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS)),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending),
        });
    }

    for pending in displayed_pending_questions(snapshot) {
        let key = format!("question:{}", pending.request_id);
        let existing = existing_items.remove(&key);
        let is_new_live_question = !previous_live_question_ids.contains(&pending.request_id);
        if let Some(existing_item) = existing.as_ref() {
            if existing_item.is_removing
                && existing_item
                    .remove_after
                    .is_some_and(|remove_after| remove_after > now)
            {
                next_items.push(StatusQueueItem {
                    key,
                    session_id: pending.session_id.clone(),
                    sort_time: pending.requested_at,
                    expires_at: existing_item.expires_at,
                    is_live: false,
                    is_removing: true,
                    remove_after: existing_item.remove_after,
                    payload: StatusQueuePayload::Question(pending),
                });
                continue;
            }
        }
        if existing.is_none() && !is_new_live_question {
            continue;
        }
        if existing.is_none() && is_new_live_question {
            added_questions += 1;
        }
        next_items.push(StatusQueueItem {
            key,
            session_id: pending.session_id.clone(),
            sort_time: pending.requested_at,
            expires_at: existing
                .as_ref()
                .map(|item| item.expires_at)
                .unwrap_or_else(|| now + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS)),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Question(pending),
        });
    }

    for session_id in completed_session_ids {
        let Some(session) = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == session_id)
            .cloned()
        else {
            continue;
        };
        let key = format!("completion:{}", session.session_id);
        let existing = existing_items.remove(&key);
        if existing.is_none() {
            added_completions += 1;
        }
        next_items.push(StatusQueueItem {
            key,
            session_id: session.session_id.clone(),
            sort_time: session.last_activity,
            expires_at: existing
                .as_ref()
                .map(|item| item.expires_at)
                .unwrap_or_else(|| now + Duration::from_secs(STATUS_COMPLETION_VISIBLE_SECONDS)),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Completion(session),
        });
    }

    for mut item in existing_items.into_values() {
        if item.is_removing {
            if item
                .remove_after
                .is_some_and(|remove_after| remove_after > now)
            {
                next_items.push(item);
            }
            continue;
        }

        if now >= item.expires_at {
            item.is_live = false;
            item.is_removing = true;
            item.remove_after = Some(now + status_queue_exit_duration());
            next_items.push(item);
            continue;
        }

        match &mut item.payload {
            StatusQueuePayload::Approval(_) | StatusQueuePayload::Question(_) => {
                item.is_live = false;
                item.is_removing = true;
                item.remove_after = Some(now + status_queue_exit_duration());
                next_items.push(item);
            }
            StatusQueuePayload::Completion(session) => {
                if let Some(latest) = snapshot
                    .sessions
                    .iter()
                    .find(|candidate| candidate.session_id == item.session_id)
                {
                    *session = latest.clone();
                    item.sort_time = latest.last_activity;
                }
                item.is_live = false;
                item.is_removing = false;
                item.remove_after = None;
                next_items.push(item);
            }
        }
    }

    next_items.sort_by(compare_status_queue_items);
    next_items.retain(|item| {
        if item.is_removing {
            return item
                .remove_after
                .is_some_and(|remove_after| remove_after > now);
        }
        item.expires_at > now
    });
    state.status_queue = next_items;

    StatusQueueSyncResult {
        added_approvals,
        added_questions,
        added_completions,
    }
}

pub(crate) fn sync_completion_badge(
    state: &mut PanelState,
    snapshot: &RuntimeSnapshot,
    completed_session_ids: &[String],
) {
    if state.expanded && !state.status_auto_expanded {
        state.completion_badge_items.clear();
        return;
    }

    let sessions_by_id = snapshot
        .sessions
        .iter()
        .map(|session| (&session.session_id, session))
        .collect::<HashMap<_, _>>();

    state.completion_badge_items.retain(|item| {
        let Some(session) = sessions_by_id.get(&item.session_id) else {
            return false;
        };
        !session_has_new_dialogue_after_completion(session, item)
    });

    for session_id in completed_session_ids {
        let Some(session) = sessions_by_id.get(session_id) else {
            continue;
        };
        if let Some(item) = state
            .completion_badge_items
            .iter_mut()
            .find(|item| item.session_id == *session_id)
        {
            item.completed_at = session.last_activity;
            item.last_user_prompt = session.last_user_prompt.clone();
            item.last_assistant_message = session.last_assistant_message.clone();
            continue;
        }

        state.completion_badge_items.push(CompletionBadgeItem {
            session_id: session.session_id.clone(),
            completed_at: session.last_activity,
            last_user_prompt: session.last_user_prompt.clone(),
            last_assistant_message: session.last_assistant_message.clone(),
        });
    }
}

pub(crate) fn sync_status_surface_policy(
    state: &mut PanelState,
    status_queue_sync: StatusQueueSyncResult,
) -> StatusSurfaceTransition {
    let was_status_surface =
        state.surface_mode == ExpandedSurface::Status && !state.status_queue.is_empty();
    let added_status_items = status_queue_sync.added_approvals
        + status_queue_sync.added_questions
        + status_queue_sync.added_completions;
    let mut panel_transition = None;

    if added_status_items > 0 && !state.expanded && !state.transitioning {
        state.expanded = true;
        state.status_auto_expanded = true;
        state.surface_mode = ExpandedSurface::Status;
        panel_transition = Some(true);
    } else if added_status_items > 0 && !state.expanded && state.transitioning {
        state.status_auto_expanded = true;
        state.surface_mode = ExpandedSurface::Status;
    } else if added_status_items > 0 && state.expanded && !state.transitioning {
        state.status_auto_expanded = true;
        state.surface_mode = ExpandedSurface::Status;
    } else if state.status_auto_expanded
        && state.status_queue.is_empty()
        && state.expanded
        && !state.transitioning
        && state.pointer_inside_since.is_none()
    {
        state.expanded = false;
        state.status_auto_expanded = false;
        state.surface_mode = ExpandedSurface::Default;
        state.skip_next_close_card_exit = true;
        panel_transition = Some(false);
    } else if state.status_queue.is_empty() && state.surface_mode == ExpandedSurface::Status {
        state.surface_mode = ExpandedSurface::Default;
        state.status_auto_expanded = false;
    }

    let is_status_surface =
        state.surface_mode == ExpandedSurface::Status && !state.status_queue.is_empty();
    StatusSurfaceTransition {
        panel_transition,
        surface_transition: was_status_surface != is_status_surface
            && panel_transition.is_none()
            && state.expanded
            && !state.transitioning,
    }
}

pub(crate) fn take_pending_status_reopen_after_transition(state: &mut PanelState) -> bool {
    if state.transitioning
        || state.expanded
        || !state.status_auto_expanded
        || state.surface_mode != ExpandedSurface::Status
        || state.status_queue.is_empty()
    {
        return false;
    }

    state.expanded = true;
    true
}

fn session_has_new_dialogue_after_completion(
    session: &SessionSnapshotView,
    completion: &CompletionBadgeItem,
) -> bool {
    session.last_activity > completion.completed_at
        && (normalize_status(&session.status) != "idle"
            || session.last_user_prompt != completion.last_user_prompt
            || session.last_assistant_message != completion.last_assistant_message)
}

pub(crate) fn detect_completed_sessions(
    previous: &RuntimeSnapshot,
    snapshot: &RuntimeSnapshot,
    now: chrono::DateTime<Utc>,
) -> Vec<String> {
    let previous_by_id = previous
        .sessions
        .iter()
        .map(|session| (&session.session_id, session))
        .collect::<HashMap<_, _>>();
    snapshot
        .sessions
        .iter()
        .filter_map(|session| {
            let previous = previous_by_id.get(&session.session_id)?;
            let previous_status = normalize_status(&previous.status);
            let current_status = normalize_status(&session.status);
            let became_idle_from_active = current_status == "idle"
                && (previous_status == "processing" || previous_status == "running");
            let idle_message_updated = current_status == "idle"
                && previous_status == "idle"
                && (now - session.last_activity).num_seconds() <= 20
                && session
                    .last_assistant_message
                    .as_deref()
                    .is_some_and(|message| !message.trim().is_empty())
                && session.last_assistant_message != previous.last_assistant_message;

            if became_idle_from_active || idle_message_updated {
                Some(session.session_id.clone())
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn compare_status_queue_items(
    left: &StatusQueueItem,
    right: &StatusQueueItem,
) -> std::cmp::Ordering {
    let left_priority = status_queue_priority(left);
    let right_priority = status_queue_priority(right);
    right_priority
        .cmp(&left_priority)
        .then_with(|| match (&left.payload, &right.payload) {
            (StatusQueuePayload::Approval(_), StatusQueuePayload::Approval(_))
            | (StatusQueuePayload::Question(_), StatusQueuePayload::Question(_))
            | (StatusQueuePayload::Approval(_), StatusQueuePayload::Question(_))
            | (StatusQueuePayload::Question(_), StatusQueuePayload::Approval(_)) => {
                left.sort_time.cmp(&right.sort_time)
            }
            _ => right.sort_time.cmp(&left.sort_time),
        })
        .then_with(|| left.session_id.cmp(&right.session_id))
}

pub(crate) fn status_queue_priority(item: &StatusQueueItem) -> u8 {
    match &item.payload {
        StatusQueuePayload::Approval(_) | StatusQueuePayload::Question(_) => 2,
        StatusQueuePayload::Completion(_) => 1,
    }
}

pub(crate) fn displayed_pending_permissions(
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

pub(crate) fn displayed_pending_questions(snapshot: &RuntimeSnapshot) -> Vec<PendingQuestionView> {
    let mut questions = if snapshot.pending_questions.is_empty() {
        snapshot.pending_question.clone().into_iter().collect()
    } else {
        snapshot.pending_questions.clone()
    };
    questions.sort_by(|left, right| left.requested_at.cmp(&right.requested_at));
    questions
}

pub(crate) fn displayed_default_pending_permissions(
    snapshot: &RuntimeSnapshot,
) -> Vec<PendingPermissionView> {
    let permission = snapshot
        .pending_permission
        .clone()
        .or_else(|| displayed_pending_permissions(snapshot).into_iter().next());
    permission.into_iter().collect()
}

pub(crate) fn displayed_default_pending_questions(
    snapshot: &RuntimeSnapshot,
) -> Vec<PendingQuestionView> {
    let question = snapshot
        .pending_question
        .clone()
        .or_else(|| displayed_pending_questions(snapshot).into_iter().next());
    question.into_iter().collect()
}

pub(crate) fn displayed_sessions(
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

pub(crate) fn blocked_session_ids(
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

pub(crate) fn displayed_prompt_assist_sessions(
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

pub(crate) fn live_pending_session_ids(snapshot: &RuntimeSnapshot) -> HashSet<String> {
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

pub(crate) fn is_prompt_assist_session(
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

pub(crate) fn should_hide_legacy_opencode_session(session: &SessionSnapshotView) -> bool {
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

pub(crate) fn compact_active_session_count(snapshot: &RuntimeSnapshot) -> usize {
    snapshot
        .sessions
        .iter()
        .filter(|session| !should_hide_legacy_opencode_session(session))
        .filter(|session| normalize_status(&session.status) != "idle")
        .count()
}

pub(crate) fn resolve_mascot_base_state(
    snapshot: Option<&RuntimeSnapshot>,
    has_status_completion: bool,
    has_completion_badge: bool,
) -> PanelMascotBaseState {
    let Some(snapshot) = snapshot else {
        return PanelMascotBaseState::Idle;
    };

    if snapshot.pending_permission_count > 0 {
        return PanelMascotBaseState::Approval;
    }
    if snapshot.pending_question_count > 0 {
        return PanelMascotBaseState::Question;
    }
    if has_status_completion {
        return PanelMascotBaseState::MessageBubble;
    }
    if has_completion_badge {
        return PanelMascotBaseState::Complete;
    }
    if compact_active_session_count(snapshot) > 0 || snapshot.active_session_count > 0 {
        return PanelMascotBaseState::Running;
    }

    PanelMascotBaseState::Idle
}

pub(crate) fn status_priority(status: &str) -> u8 {
    match normalize_status(status).as_str() {
        "waitingapproval" | "waitingquestion" => 0,
        "running" => 1,
        "processing" => 2,
        _ => 3,
    }
}

pub(crate) fn normalize_status(status: &str) -> String {
    status.to_ascii_lowercase()
}

pub(crate) fn format_source(source: &str) -> String {
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

pub(crate) fn format_status(status: &str) -> String {
    match normalize_status(status).as_str() {
        "running" => "Running".to_string(),
        "processing" => "Thinking".to_string(),
        "waitingapproval" => "Approval".to_string(),
        "waitingquestion" => "Question".to_string(),
        "idle" => "Idle".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn session_title(session: &SessionSnapshotView) -> String {
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

pub(crate) fn display_project_name(session: &SessionSnapshotView) -> String {
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

pub(crate) fn compact_title(value: &str, max_length: usize) -> String {
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

pub(crate) fn short_session_id(session_id: &str) -> String {
    session_id
        .split_once('-')
        .map(|(_, tail)| tail.chars().take(6).collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "------".to_string())
}

pub(crate) fn time_ago(last_activity: chrono::DateTime<chrono::Utc>) -> String {
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

pub(crate) fn session_meta_line(session: &SessionSnapshotView) -> String {
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

pub(crate) fn display_snippet(value: Option<&str>, max_chars: usize) -> Option<String> {
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

pub(crate) fn session_prompt_preview(session: &SessionSnapshotView) -> Option<String> {
    display_snippet(session.last_user_prompt.as_deref(), 68)
}

pub(crate) fn session_reply_preview(session: &SessionSnapshotView) -> Option<String> {
    display_snippet(
        session
            .last_assistant_message
            .as_deref()
            .or(session.tool_description.as_deref()),
        92,
    )
}

pub(crate) fn session_tool_preview(
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

pub(crate) fn session_has_visible_card_body(session: &SessionSnapshotView) -> bool {
    session_prompt_preview(session).is_some()
        || session_reply_preview(session).is_some()
        || session_tool_preview(session).is_some()
}

pub(crate) fn is_long_idle_session(session: &SessionSnapshotView) -> bool {
    normalize_status(&session.status) == "idle"
        && (Utc::now() - session.last_activity).num_minutes() > 15
}

pub(crate) fn completion_preview_text(session: &SessionSnapshotView) -> String {
    display_snippet(
        session
            .last_assistant_message
            .as_deref()
            .or(session.tool_description.as_deref()),
        92,
    )
    .unwrap_or_else(|| "Task complete".to_string())
}

fn status_queue_exit_duration() -> Duration {
    Duration::from_millis(PANEL_CARD_EXIT_MS.max(220) + STATUS_QUEUE_EXIT_EXTRA_MS)
}
