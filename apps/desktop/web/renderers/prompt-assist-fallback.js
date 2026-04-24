import { getCodexStatus } from "../state-helpers.js";
import { getLivePendingSessionIdsFromSnapshot } from "./pending-snapshot-fallback.js";
import {
  getSessionActivityMs,
  getSessionId,
  getSessionSourceKey,
  getSessionStatusKey,
} from "./session-snapshot-fallback.js";
import { getSnapshotSessions } from "./snapshot-sessions.js";
import { getDefaultPendingSceneSessionIds } from "./status-surface-scene.js";

const PROMPT_ASSIST_RUNNING_MS = 12_000;
const PROMPT_ASSIST_PROCESSING_MS = 18_000;
const PROMPT_ASSIST_RECENT_MS = 20 * 60_000;

export function getLivePendingSessionIds(snapshot, statusSurfaceScene = null) {
  const sceneIds = getDefaultPendingSceneSessionIds(statusSurfaceScene);
  const snapshotIds = getLivePendingSessionIdsFromSnapshot(snapshot);
  return new Set([...sceneIds, ...snapshotIds]);
}

export function isPromptAssistFallbackSession(session, uiState, nowMs = Date.now()) {
  if (getSessionSourceKey(session) !== "codex") return false;
  if (getCodexStatus(uiState)?.live_capture_ready) return false;

  const status = getSessionStatusKey(session);
  if (status !== "processing" && status !== "running") return false;

  const lastActivity = getSessionActivityMs(session);
  if (lastActivity <= 0) return false;

  const ageMs = nowMs - lastActivity;
  const staleMs = status === "running" ? PROMPT_ASSIST_RUNNING_MS : PROMPT_ASSIST_PROCESSING_MS;
  return ageMs >= staleMs && ageMs <= PROMPT_ASSIST_RECENT_MS;
}

export function getPromptAssistFallbackSessions(snapshot, uiState, statusSurfaceScene = null, limit = 1) {
  const nowMs = Date.now();
  const livePendingSessionIds = getLivePendingSessionIds(snapshot, statusSurfaceScene);
  return getSnapshotSessions(snapshot)
    .filter((session) => !livePendingSessionIds.has(getSessionId(session)))
    .filter((session) => isPromptAssistFallbackSession(session, uiState, nowMs))
    .sort((left, right) => getSessionActivityMs(right) - getSessionActivityMs(left))
    .slice(0, limit);
}
