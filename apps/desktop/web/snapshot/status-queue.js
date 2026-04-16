import {
  getStatusQueueItems,
  setCompletionSessionIds,
  setStatusQueueItems,
} from "../state-helpers.js";

function getLivePendingPermissions(snapshot) {
  const items = Array.isArray(snapshot?.pending_permissions) ? snapshot.pending_permissions : [];
  if (items.length) return items;
  return snapshot?.pending_permission ? [snapshot.pending_permission] : [];
}

function getPendingPermissionIds(snapshot) {
  return new Set(getLivePendingPermissions(snapshot).map((item) => item?.request_id).filter(Boolean));
}

function buildCompletionItem(session, nowMs, timings, previousItem = null) {
  return {
    key: `completion:${session.session_id}`,
    kind: "completion",
    sessionId: session.session_id,
    payload: session,
    createdAt: previousItem?.createdAt ?? nowMs,
    expiresAt: previousItem?.expiresAt ?? nowMs + timings.statusQueue.completionMs,
    isLive: true,
    isRemoving: false,
    removeAfter: null,
  };
}

function buildApprovalItem(permission, nowMs, timings, previousItem = null) {
  return {
    key: `approval:${permission.request_id}`,
    kind: "approval",
    sessionId: permission.session_id,
    requestId: permission.request_id,
    payload: permission,
    createdAt: previousItem?.createdAt ?? nowMs,
    expiresAt: previousItem?.expiresAt ?? nowMs + timings.statusQueue.approvalMs,
    isLive: true,
    isRemoving: false,
    removeAfter: null,
  };
}

function statusQueueExitDurationMs(timings) {
  return Math.max(timings.statusQueue.exitMinMs, (timings.cardExitDurationMs ?? 220) + timings.statusQueue.exitExtraMs);
}

function sortStatusQueueItems(items) {
  return [...items].sort((left, right) => {
    const priorityLeft = left.kind === "approval" ? 2 : 1;
    const priorityRight = right.kind === "approval" ? 2 : 1;
    if (priorityLeft !== priorityRight) {
      return priorityRight - priorityLeft;
    }

    const leftTime = new Date(left.payload?.requested_at ?? left.payload?.last_activity ?? 0).getTime();
    const rightTime = new Date(right.payload?.requested_at ?? right.payload?.last_activity ?? 0).getTime();

    if (left.kind === "approval") {
      return leftTime - rightTime;
    }
    return rightTime - leftTime;
  });
}

export function syncStatusQueue(snapshot, previousRawSnapshot, completedSessionIds, uiState, timings, nowMs = Date.now()) {
  const previousItems = getStatusQueueItems(uiState);
  const previousByKey = new Map(previousItems.map((item) => [item.key, item]));
  const previousLivePermissionIds = getPendingPermissionIds(previousRawSnapshot);
  const nextItems = [];
  let addedCount = 0;

  for (const permission of getLivePendingPermissions(snapshot)) {
    const key = `approval:${permission.request_id}`;
    const previousItem = previousByKey.get(key) ?? null;
    const isNewLivePermission = !previousLivePermissionIds.has(permission.request_id);
    if (!previousItem && !isNewLivePermission) {
      continue;
    }
    if (!previousItem && isNewLivePermission) {
      addedCount += 1;
    }
    nextItems.push(buildApprovalItem(permission, nowMs, timings, previousItem));
    previousByKey.delete(key);
  }

  for (const sessionId of completedSessionIds) {
    const session = snapshot.sessions.find((item) => item.session_id === sessionId);
    if (!session) continue;
    const key = `completion:${session.session_id}`;
    const previousItem = previousByKey.get(key) ?? null;
    if (!previousItem) {
      addedCount += 1;
    }
    nextItems.push(buildCompletionItem(session, nowMs, timings, previousItem));
    previousByKey.delete(key);
  }

  for (const leftover of previousByKey.values()) {
    if (leftover.isRemoving) {
      if (nowMs < (leftover.removeAfter ?? 0)) {
        nextItems.push(leftover);
      }
      continue;
    }

    if (nowMs >= leftover.expiresAt) {
      nextItems.push({
        ...leftover,
        isLive: false,
        isRemoving: true,
        removeAfter: nowMs + statusQueueExitDurationMs(timings),
      });
      continue;
    }

    if (leftover.kind === "completion") {
      const latestSession =
        snapshot.sessions.find((session) => session.session_id === leftover.sessionId) ?? leftover.payload;
      nextItems.push({
        ...leftover,
        payload: latestSession,
        isLive: false,
        isRemoving: false,
        removeAfter: null,
      });
      continue;
    }

    nextItems.push({
      ...leftover,
      isLive: false,
      isRemoving: false,
      removeAfter: null,
    });
  }

  const prunedItems = sortStatusQueueItems(nextItems).filter((item) =>
    item.isRemoving ? nowMs < (item.removeAfter ?? 0) : nowMs < item.expiresAt
  );
  const nextRefreshAt = prunedItems.reduce((earliest, item) => {
    const transitionAt = item.isRemoving ? item.removeAfter ?? null : item.expiresAt ?? null;
    if (!transitionAt || transitionAt <= nowMs) return earliest;
    return earliest === null ? transitionAt : Math.min(earliest, transitionAt);
  }, null);
  setStatusQueueItems(uiState, prunedItems);
  setCompletionSessionIds(
    uiState,
    prunedItems.filter((item) => item.kind === "completion").map((item) => item.sessionId)
  );

  return {
    addedCount,
    hasItems: prunedItems.length > 0,
    nextRefreshDelayMs: nextRefreshAt
      ? Math.max(timings.statusQueue.refreshMinDelayMs, nextRefreshAt - nowMs + timings.statusQueue.refreshLeadMs)
      : null,
  };
}
