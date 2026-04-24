import {
  getStatusQueueKeys,
  setStatusQueueKeys,
  setStatusQueueItems,
} from "../state-helpers.js";
import { buildStatusQueueSceneKeyItem, getStatusQueueSceneCards } from "../renderers/status-surface-scene.js";
import { buildStatusQueueSyncResult } from "./status-queue-sync-result.js";
import { syncLegacyStatusQueue } from "./status-queue-legacy.js";

function syncStatusQueueFromScene(statusSurfaceScene, uiState, timings) {
  const sceneCards = getStatusQueueSceneCards(statusSurfaceScene);
  const previousKeys = new Set(getStatusQueueKeys(uiState).filter((item) => item?.isLive !== false).map((item) => item.key));
  const nextKeys = sceneCards.map((card, index) => buildStatusQueueSceneKeyItem(card, index));
  const addedItems = nextKeys.filter((item) => item.isLive && !previousKeys.has(item.key));

  setStatusQueueKeys(uiState, nextKeys);
  setStatusQueueItems(uiState, []);

  return buildStatusQueueSyncResult({
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
  });
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
