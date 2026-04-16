import { getCompletionSessionIds, getStatusQueueItems, getSurfaceMode } from "../state-helpers.js";

export function isCompletionSurfaceActive(uiState) {
  return getSurfaceMode(uiState) === "status" && getCompletionSessionIds(uiState).length > 0;
}

export function getDisplayedSessions(snapshot, uiState) {
  return snapshot.sessions;
}

export function getCompletionDisplaySessions(snapshot, uiState) {
  return getStatusQueueItems(uiState)
    .filter((item) => item.kind === "completion")
    .map((item) => item.payload)
    .filter(Boolean);
}
