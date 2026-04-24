import { getStatusSurfaceScene } from "../state-helpers.js";
import {
  getSessionCompletionPreviewText,
  getSessionId,
} from "./session-snapshot-fallback.js";
import {
  getPrimaryPendingSessionIdWithFallback,
  getPromptAssistSceneSessionIds,
} from "./status-surface-scene.js";
import {
  getCompletionDisplaySessions,
  isCompletionSurfaceActive,
} from "./surface-state.js";
import { findSnapshotSessionById } from "./snapshot-sessions.js";
import {
  getLivePendingSessionIds,
  getPromptAssistFallbackSessions,
  isPromptAssistFallbackSession,
} from "./prompt-assist-fallback.js";

export { getLivePendingSessionIds };

export function isPromptAssistSession(session, uiState, nowMs = Date.now()) {
  return isPromptAssistFallbackSession(session, uiState, nowMs);
}

export function getPromptAssistSessions(snapshot, uiState, limit = 1) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const scenePromptAssistSessionIds = getPromptAssistSceneSessionIds(statusSurfaceScene);
  if (scenePromptAssistSessionIds.length) {
    return scenePromptAssistSessionIds
      .map((sessionId) => findSnapshotSessionById(snapshot, sessionId))
      .filter(Boolean)
      .slice(0, limit);
  }

  return getPromptAssistFallbackSessions(snapshot, uiState, statusSurfaceScene, limit);
}

export function hasPromptAssistSessions(snapshot, uiState) {
  return getPromptAssistSessions(snapshot, uiState, 1).length > 0;
}

export function getPrimaryPromptAssistSessionId(snapshot, uiState) {
  return getSessionId(getPromptAssistSessions(snapshot, uiState, 1)[0]);
}

export function getPrimaryActionSession(snapshot, uiState) {
  if (!snapshot) return null;

  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const pendingSessionId = getPrimaryPendingSessionIdWithFallback(statusSurfaceScene, snapshot);
  if (pendingSessionId) {
    return findSnapshotSessionById(snapshot, pendingSessionId);
  }

  const promptAssistSession = getPromptAssistSessions(snapshot, uiState, 1)[0] ?? null;
  if (promptAssistSession) return promptAssistSession;

  if (isCompletionSurfaceActive(uiState)) {
    return getCompletionDisplaySessions(snapshot, uiState)[0] ?? null;
  }

  return null;
}

export function getCompletionPreviewText(session) {
  return getSessionCompletionPreviewText(session);
}
