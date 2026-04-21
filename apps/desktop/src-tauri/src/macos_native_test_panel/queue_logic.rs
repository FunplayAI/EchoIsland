use super::*;

pub(super) fn sync_native_pending_card_visibility(
    state: &mut NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> RuntimeSnapshot {
    let mut core = state.to_core_panel_state();
    let next_snapshot = crate::native_panel_core::sync_pending_card_visibility(&mut core, snapshot);
    state.apply_core_panel_state(core);
    next_snapshot
}

pub(super) fn resolve_native_pending_permission_card(
    current_payload: Option<PendingPermissionView>,
    previous: Option<&NativePendingPermissionCard>,
    now: Instant,
) -> Option<NativePendingPermissionCard> {
    crate::native_panel_core::resolve_pending_permission_card(current_payload, previous, now)
}

pub(super) fn sync_native_status_queue(
    state: &mut NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> NativeStatusQueueSyncResult {
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
    let mut core = state.to_core_panel_state();
    let sync_result = crate::native_panel_core::sync_status_queue(&mut core, snapshot);
    let next_queue_keys = core
        .status_queue
        .iter()
        .map(|item| item.key.clone())
        .collect::<HashSet<_>>();
    let next_approval_count = core
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Approval(_)))
        .count();
    let next_completion_count = core
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)))
        .count();
    let displayed_permission_count = displayed_pending_permissions(snapshot).len();
    let displayed_question_count = displayed_pending_questions(snapshot).len();
    if sync_result.added_approvals > 0
        || sync_result.added_completions > 0
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
            queue_len = core.status_queue.len(),
            expanded = state.expanded,
            status_auto_expanded = state.status_auto_expanded,
            status_surface_active = state.surface_mode == NativeExpandedSurface::Status,
            added_approvals = sync_result.added_approvals,
            added_completions = sync_result.added_completions,
            "native status queue sync"
        );
    }
    state.apply_core_panel_state(core);
    sync_result
}

pub(super) fn sync_native_completion_badge(
    state: &mut NativePanelState,
    snapshot: &RuntimeSnapshot,
    completed_session_ids: &[String],
) {
    let mut core = state.to_core_panel_state();
    crate::native_panel_core::sync_completion_badge(&mut core, snapshot, completed_session_ids);
    state.apply_core_panel_state(core);
}

pub(super) fn detect_completed_sessions(
    previous: &RuntimeSnapshot,
    snapshot: &RuntimeSnapshot,
    now: chrono::DateTime<Utc>,
) -> Vec<String> {
    crate::native_panel_core::detect_completed_sessions(previous, snapshot, now)
}

pub(super) fn compare_native_status_queue_items(
    left: &NativeStatusQueueItem,
    right: &NativeStatusQueueItem,
) -> std::cmp::Ordering {
    crate::native_panel_core::compare_status_queue_items(left, right)
}
