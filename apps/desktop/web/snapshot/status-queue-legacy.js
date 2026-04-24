import { getStatusQueueItems, setStatusQueueItems } from "../state-helpers.js";
import {
  getPendingPermissionPayloads,
  getPendingRequestId,
  getPendingSessionId,
} from "../renderers/pending-snapshot-fallback.js";
import { getSessionId } from "../renderers/session-snapshot-fallback.js";
import { findSnapshotSessionById } from "../renderers/snapshot-sessions.js";
import { buildStatusQueueSyncResult } from "./status-queue-sync-result.js";

function getPendingPermissionIds(snapshot) {
  return new Set(getPendingPermissionPayloads(snapshot).map((item) => getPendingRequestId(item)).filter(Boolean));
}

function buildCompletionItem(session, nowMs, timings, previousItem = null) {
  const sessionId = getSessionId(session);
  return {
    key: `completion:${sessionId}`,
    kind: "completion",
    sessionId,
    payload: session,
    createdAt: previousItem?.createdAt ?? nowMs,
    expiresAt: previousItem?.expiresAt ?? nowMs + timings.statusQueue.completionMs,
    isLive: true,
    isRemoving: false,
    removeAfter: null,
  };
}

function buildApprovalItem(permission, nowMs, timings, previousItem = null) {
  const requestId = getPendingRequestId(permission);
  const sessionId = getPendingSessionId(permission);
  return {
    key: `approval:${requestId}`,
    kind: "approval",
    sessionId,
    requestId,
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

export function syncLegacyStatusQueue(
  snapshot,
  previousRawSnapshot,
  completedSessionIds,
  uiState,
  timings,
  nowMs = Date.now()
) {
  const previousItems = getStatusQueueItems(uiState);
  const previousByKey = new Map(previousItems.map((item) => [item.key, item]));
  const previousLivePermissionIds = getPendingPermissionIds(previousRawSnapshot);
  const nextItems = [];
  let addedCount = 0;
  let addedApprovalCount = 0;
  let addedCompletionCount = 0;

  for (const permission of getPendingPermissionPayloads(snapshot)) {
    const requestId = getPendingRequestId(permission);
    const key = `approval:${requestId}`;
    const previousItem = previousByKey.get(key) ?? null;
    const isNewLivePermission = !previousLivePermissionIds.has(requestId);
    if (!previousItem && !isNewLivePermission) {
      continue;
    }
    if (!previousItem && isNewLivePermission) {
      addedCount += 1;
      addedApprovalCount += 1;
    }
    nextItems.push(buildApprovalItem(permission, nowMs, timings, previousItem));
    previousByKey.delete(key);
  }

  for (const sessionId of completedSessionIds) {
    const session = findSnapshotSessionById(snapshot, sessionId);
    if (!session) continue;
    const key = `completion:${getSessionId(session)}`;
    const previousItem = previousByKey.get(key) ?? null;
    if (!previousItem) {
      addedCount += 1;
      addedCompletionCount += 1;
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
      const latestSession = findSnapshotSessionById(snapshot, leftover.sessionId) ?? leftover.payload;
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

  return buildStatusQueueSyncResult({
    addedCount,
    addedApprovalCount,
    addedCompletionCount,
    hasItems: prunedItems.length > 0,
    nextRefreshDelayMs: nextRefreshAt
      ? Math.max(timings.statusQueue.refreshMinDelayMs, nextRefreshAt - nowMs + timings.statusQueue.refreshLeadMs)
      : null,
  });
}
