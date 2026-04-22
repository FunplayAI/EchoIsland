import {
  getStatusQueueKeys,
  setStatusQueueKeys,
  setStatusQueueItems,
} from "../state-helpers.js";
import { buildStatusSurfaceCardKey, getStatusSurfaceCardsByMode } from "../renderers/status-surface-scene.js";
import { syncLegacyStatusQueue } from "./status-queue-legacy.js";

function buildQueueKeyFromSceneCard(card, index = 0) {
  return {
    key: buildStatusSurfaceCardKey(card, index),
    kind: card?.kind === "approval" ? "approval" : "completion",
    sessionId: card?.sessionId ?? null,
    requestId: card?.requestId ?? null,
    isLive: card?.isLive !== false,
    isRemoving: card?.isRemoving === true,
    sortOrder: index,
  };
}

function syncStatusQueueFromScene(statusSurfaceScene, uiState, timings) {
  const sceneCards = getStatusSurfaceCardsByMode(statusSurfaceScene, "queue");
  const previousKeys = new Set(getStatusQueueKeys(uiState).filter((item) => item?.isLive !== false).map((item) => item.key));
  const nextKeys = sceneCards.map((card, index) => buildQueueKeyFromSceneCard(card, index));
  const addedItems = nextKeys.filter((item) => item.isLive && !previousKeys.has(item.key));

  setStatusQueueKeys(uiState, nextKeys);
  setStatusQueueItems(uiState, []);

  return {
    addedCount: addedItems.length,
    addedApprovalCount: addedItems.filter((item) => item.kind === "approval").length,
    addedCompletionCount: addedItems.filter((item) => item.kind === "completion").length,
    hasItems: nextKeys.length > 0,
    nextRefreshDelayMs:
      typeof statusSurfaceScene?.queueState?.nextTransitionInMs === "number"
        ? Math.max(
            timings.statusQueue.refreshMinDelayMs,
            statusSurfaceScene.queueState.nextTransitionInMs + timings.statusQueue.refreshLeadMs
          )
        : null,
  };
}

export function syncStatusQueue(
  statusSurfaceScene,
  snapshot,
  previousRawSnapshot,
  completedSessionIds,
  uiState,
  timings,
  nowMs = Date.now()
) {
  if (statusSurfaceScene) {
    return syncStatusQueueFromScene(statusSurfaceScene, uiState, timings);
  }

  setStatusQueueKeys(uiState, []);
  return syncLegacyStatusQueue(snapshot, previousRawSnapshot, completedSessionIds, uiState, timings, nowMs);
}
