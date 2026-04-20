use super::*;

pub(super) fn sync_native_pending_card_visibility(
    state: &mut NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> RuntimeSnapshot {
    let now = Instant::now();
    let next_permission = resolve_native_pending_permission_card(
        displayed_pending_permissions(snapshot).into_iter().next(),
        state.pending_permission_card.as_ref(),
        now,
    );
    let next_question = resolve_native_pending_question_card(
        displayed_pending_questions(snapshot).into_iter().next(),
        state.pending_question_card.as_ref(),
        now,
    );

    state.pending_permission_card = next_permission.clone();
    state.pending_question_card = next_question.clone();

    apply_native_pending_cards_to_snapshot(snapshot, next_permission, next_question)
}

pub(super) fn resolve_native_pending_permission_card(
    current_payload: Option<PendingPermissionView>,
    previous: Option<&NativePendingPermissionCard>,
    now: Instant,
) -> Option<NativePendingPermissionCard> {
    if let Some(payload) = current_payload {
        let started_at = previous
            .filter(|card| card.request_id == payload.request_id)
            .map(|card| card.started_at)
            .unwrap_or(now);
        return Some(NativePendingPermissionCard {
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

    Some(NativePendingPermissionCard {
        request_id: previous.request_id.clone(),
        payload: previous.payload.clone(),
        started_at: previous.started_at,
        last_seen_at: previous.last_seen_at,
        visible_until: keep_visible_until,
    })
}

pub(super) fn resolve_native_pending_question_card(
    current_payload: Option<PendingQuestionView>,
    previous: Option<&NativePendingQuestionCard>,
    now: Instant,
) -> Option<NativePendingQuestionCard> {
    if let Some(payload) = current_payload {
        let started_at = previous
            .filter(|card| card.request_id == payload.request_id)
            .map(|card| card.started_at)
            .unwrap_or(now);
        return Some(NativePendingQuestionCard {
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

    Some(NativePendingQuestionCard {
        request_id: previous.request_id.clone(),
        payload: previous.payload.clone(),
        started_at: previous.started_at,
        last_seen_at: previous.last_seen_at,
        visible_until: keep_visible_until,
    })
}

pub(super) fn apply_native_pending_cards_to_snapshot(
    snapshot: &RuntimeSnapshot,
    pending_permission_card: Option<NativePendingPermissionCard>,
    pending_question_card: Option<NativePendingQuestionCard>,
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

pub(super) fn sync_native_status_queue(
    state: &mut NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> NativeStatusQueueSyncResult {
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
    let previous_queue_keys = state
        .status_queue
        .iter()
        .map(|item| item.key.clone())
        .collect::<HashSet<_>>();
    let previous_approval_count = state
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Approval(_)))
        .count();
    let previous_completion_count = state
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)))
        .count();

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
                next_items.push(NativeStatusQueueItem {
                    key,
                    session_id: pending.session_id.clone(),
                    sort_time: pending.requested_at,
                    expires_at: existing_item.expires_at,
                    is_live: false,
                    is_removing: true,
                    remove_after: existing_item.remove_after,
                    payload: NativeStatusQueuePayload::Approval(pending),
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
        next_items.push(NativeStatusQueueItem {
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
            payload: NativeStatusQueuePayload::Approval(pending),
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
        next_items.push(NativeStatusQueueItem {
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
            payload: NativeStatusQueuePayload::Completion(session),
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
            NativeStatusQueuePayload::Approval(_) => {
                item.is_live = false;
                item.is_removing = true;
                item.remove_after = Some(now + status_queue_exit_duration());
                next_items.push(item);
            }
            NativeStatusQueuePayload::Completion(session) => {
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

    next_items.sort_by(compare_native_status_queue_items);
    next_items.retain(|item| {
        if item.is_removing {
            return item
                .remove_after
                .is_some_and(|remove_after| remove_after > now);
        }
        item.expires_at > now
    });
    let next_queue_keys = next_items
        .iter()
        .map(|item| item.key.clone())
        .collect::<HashSet<_>>();
    let next_approval_count = next_items
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Approval(_)))
        .count();
    let next_completion_count = next_items
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)))
        .count();
    let displayed_permission_count = displayed_pending_permissions(snapshot).len();
    let displayed_question_count = displayed_pending_questions(snapshot).len();
    if added_approvals > 0
        || added_completions > 0
        || previous_queue_keys != next_queue_keys
        || previous_approval_count != next_approval_count
        || previous_completion_count != next_completion_count
        || (snapshot.pending_permission_count > 0 && next_approval_count == 0)
    {
        tracing::debug!(
            snapshot_pending_permission_count = snapshot.pending_permission_count,
            snapshot_pending_question_count = snapshot.pending_question_count,
            displayed_permission_count,
            displayed_question_count,
            previous_approval_count,
            previous_completion_count,
            next_approval_count,
            next_completion_count,
            queue_len = next_items.len(),
            expanded = state.expanded,
            status_auto_expanded = state.status_auto_expanded,
            status_surface_active = state.surface_mode == NativeExpandedSurface::Status,
            added_approvals,
            added_completions,
            "native status queue sync"
        );
    }
    state.status_queue = next_items;
    NativeStatusQueueSyncResult {
        added_approvals,
        added_completions,
    }
}

pub(super) fn sync_native_completion_badge(
    state: &mut NativePanelState,
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

        state
            .completion_badge_items
            .push(NativeCompletionBadgeItem {
                session_id: session.session_id.clone(),
                completed_at: session.last_activity,
                last_user_prompt: session.last_user_prompt.clone(),
                last_assistant_message: session.last_assistant_message.clone(),
            });
    }
}

fn session_has_new_dialogue_after_completion(
    session: &SessionSnapshotView,
    completion: &NativeCompletionBadgeItem,
) -> bool {
    session.last_activity > completion.completed_at
        && (normalize_status(&session.status) != "idle"
            || session.last_user_prompt != completion.last_user_prompt
            || session.last_assistant_message != completion.last_assistant_message)
}

pub(super) fn detect_completed_sessions(
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

pub(super) fn compare_native_status_queue_items(
    left: &NativeStatusQueueItem,
    right: &NativeStatusQueueItem,
) -> std::cmp::Ordering {
    let left_priority = native_status_queue_priority(left);
    let right_priority = native_status_queue_priority(right);
    right_priority
        .cmp(&left_priority)
        .then_with(|| match (&left.payload, &right.payload) {
            (NativeStatusQueuePayload::Approval(_), NativeStatusQueuePayload::Approval(_)) => {
                left.sort_time.cmp(&right.sort_time)
            }
            _ => right.sort_time.cmp(&left.sort_time),
        })
        .then_with(|| left.session_id.cmp(&right.session_id))
}

pub(super) fn native_status_queue_priority(item: &NativeStatusQueueItem) -> u8 {
    match &item.payload {
        NativeStatusQueuePayload::Approval(_) => 2,
        NativeStatusQueuePayload::Completion(_) => 1,
    }
}

pub(super) fn native_status_queue_surface_items() -> Vec<NativeStatusQueueItem> {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                if guard.surface_mode == NativeExpandedSurface::Status
                    && !guard.status_queue.is_empty()
                {
                    guard.status_queue.clone()
                } else {
                    Vec::new()
                }
            })
        })
        .unwrap_or_default()
}
