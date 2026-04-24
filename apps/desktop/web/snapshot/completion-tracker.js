import {
  getSessionId,
  getSessionLastAssistantMessageRaw,
  getSessionStatusKey,
  wasSessionRecentlyUpdated,
} from "../renderers/session-snapshot-fallback.js";
import {
  findSnapshotSessionById,
  getSnapshotSessions,
  indexSnapshotSessions,
} from "../renderers/snapshot-sessions.js";
import {
  clearCompletionBadgeItems,
  getCompletionBadgeItems,
  isExpanded,
  isStatusAutoExpanded,
  setCompletionBadgeItems,
} from "../state-helpers.js";
import {
  mergeCompletedBadgeItems,
  retainCompletionBadgeItems,
} from "./completion-badge-runtime.js";

export function detectCompletedSessions(previousSnapshot, snapshot) {
  if (!previousSnapshot) return [];

  const previousById = indexSnapshotSessions(previousSnapshot);
  const completed = [];

  for (const session of getSnapshotSessions(snapshot)) {
    const sessionId = getSessionId(session);
    const previous = previousById.get(sessionId);
    if (!previous) continue;

    const previousStatus = getSessionStatusKey(previous);
    const currentStatus = getSessionStatusKey(session);
    const becameIdleFromActive =
      currentStatus === "idle" && (previousStatus === "processing" || previousStatus === "running");
    const idleMessageUpdated =
      currentStatus === "idle" &&
      previousStatus === "idle" &&
      wasSessionRecentlyUpdated(session) &&
      (getSessionLastAssistantMessageRaw(session) ?? "") !==
        (getSessionLastAssistantMessageRaw(previous) ?? "") &&
      !!(getSessionLastAssistantMessageRaw(session) ?? "").trim();

    if (becameIdleFromActive || idleMessageUpdated) {
      completed.push(sessionId);
    }
  }

  return completed;
}

export function syncCompletionBadges(snapshot, completedSessionIds, uiState) {
  if (isExpanded(uiState) && !isStatusAutoExpanded(uiState)) {
    clearCompletionBadgeItems(uiState);
    return;
  }

  const sessionsById = indexSnapshotSessions(snapshot);
  const retainedItems = retainCompletionBadgeItems(getCompletionBadgeItems(uiState), sessionsById);
  const nextItems = mergeCompletedBadgeItems(retainedItems, completedSessionIds, (sessionId) =>
    findSnapshotSessionById(snapshot, sessionId)
  );
  setCompletionBadgeItems(uiState, nextItems);
}
