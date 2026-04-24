import {
  getSessionSurfaceScene,
  getStatusSurfaceScene,
  getStatusQueueItems,
  getSurfaceMode,
} from "../state-helpers.js";
import { getSessionSurfaceSessionIds } from "./session-surface-scene.js";
import {
  getCompletionSceneSessionIds,
  getStatusQueueApprovalCountFromScene,
  getStatusQueueTotalCountFromScene,
} from "./status-surface-scene.js";
import { getSnapshotSessions, indexSnapshotSessions } from "./snapshot-sessions.js";
export {
  findSnapshotSessionById,
  getSnapshotSessions,
  hasSnapshotSessionSource,
  indexSnapshotSessions,
} from "./snapshot-sessions.js";

export function isCompletionSurfaceActive(uiState) {
  return getSurfaceMode(uiState) === "status" && getCompletionDisplaySessionIds(null, uiState).length > 0;
}

export function getDisplayedSessions(snapshot, uiState) {
  const sceneIds = getSessionSurfaceSessionIds(getSessionSurfaceScene(uiState));
  if (!sceneIds.length) {
    return getSnapshotSessions(snapshot);
  }

  const sessionsById = indexSnapshotSessions(snapshot);
  return sceneIds.map((sessionId) => sessionsById.get(sessionId) ?? null).filter(Boolean);
}

export function getCompletionDisplaySessionIds(snapshot, uiState) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const sceneIds = getCompletionSceneSessionIds(statusSurfaceScene);
  if (sceneIds.length) {
    return sceneIds;
  }

  return getStatusQueueItems(uiState)
    .filter((item) => item.kind === "completion")
    .map((item) => item.sessionId ?? item.payload?.session_id ?? item.payload?.sessionId)
    .filter(Boolean);
}

export function getStatusQueueItemCount(uiState) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const sceneCount = getStatusQueueTotalCountFromScene(statusSurfaceScene);
  if (sceneCount > 0) {
    return sceneCount;
  }
  return getStatusQueueItems(uiState).length;
}

export function hasStatusQueueDisplayItems(uiState) {
  return getStatusQueueItemCount(uiState) > 0;
}

export function getStatusQueueApprovalCount(uiState) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const sceneApprovalCount = getStatusQueueApprovalCountFromScene(statusSurfaceScene);
  if (sceneApprovalCount > 0) {
    return sceneApprovalCount;
  }

  return getStatusQueueItems(uiState).filter((item) => item.kind === "approval").length;
}

export function getStatusQueueFallbackItems(uiState) {
  return getStatusQueueItems(uiState);
}

export function getCompletionDisplaySessions(snapshot, uiState) {
  const sessionsById = indexSnapshotSessions(snapshot);

  return getCompletionDisplaySessionIds(snapshot, uiState)
    .map((sessionId) => sessionsById.get(sessionId) ?? null)
    .filter(Boolean);
}
