import {
  getSessionSurfaceScene,
  getStatusSurfaceScene,
  getStatusQueueItems,
  getSurfaceMode,
} from "../state-helpers.js";
import { getSessionSurfaceSessionIds } from "./session-surface-scene.js";
import { getCompletionSceneSessionIds, getStatusSurfaceCardsByMode } from "./status-surface-scene.js";

export function isCompletionSurfaceActive(uiState) {
  return getSurfaceMode(uiState) === "status" && getCompletionDisplaySessionIds(null, uiState).length > 0;
}

export function indexSnapshotSessions(snapshot) {
  const sessions = Array.isArray(snapshot?.sessions) ? snapshot.sessions : [];
  return new Map(sessions.map((session) => [session?.session_id, session]));
}

export function findSnapshotSessionById(snapshot, sessionId) {
  if (!sessionId) return null;
  return indexSnapshotSessions(snapshot).get(sessionId) ?? null;
}

export function getDisplayedSessions(snapshot, uiState) {
  const sceneIds = getSessionSurfaceSessionIds(getSessionSurfaceScene(uiState));
  if (!sceneIds.length) {
    return Array.isArray(snapshot?.sessions) ? snapshot.sessions : [];
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
  const sceneCount = Number(statusSurfaceScene?.queueState?.totalCount ?? 0);
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
  const sceneApprovalCount = getStatusSurfaceCardsByMode(statusSurfaceScene, "queue").filter(
    (card) => card?.kind === "approval"
  ).length;
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
