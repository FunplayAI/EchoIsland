import { getSessionSurfaceScene, getStatusSurfaceScene } from "../state-helpers.js";
import { buildSessionSurfaceEntries } from "./session-surface-scene.js";
import {
  buildPendingLegacyEntries,
  buildPromptAssistLegacyEntries,
  buildSessionLegacyEntries,
  buildStatusQueueLegacyEntries,
} from "./session-surface-legacy-entries.js";
import { buildStatusSurfaceEntries } from "./status-surface-scene.js";

export function compareSessionSurfaceEntries(left, right) {
  const priorityDiff = right.priority - left.priority;
  if (priorityDiff !== 0) return priorityDiff;

  if (left.kind === "approval" || left.kind === "question") {
    const timeDiff = (left.sortTimeMs ?? Number.NaN) - (right.sortTimeMs ?? Number.NaN);
    if (Number.isFinite(timeDiff) && timeDiff !== 0) return timeDiff;
  } else if (left.kind === "session" && right.kind === "session") {
    const statusDiff = left.statusOrder - right.statusOrder;
    if (statusDiff !== 0) return statusDiff;
    const timeDiff = (right.sortTimeMs ?? Number.NaN) - (left.sortTimeMs ?? Number.NaN);
    if (Number.isFinite(timeDiff) && timeDiff !== 0) return timeDiff;
  } else {
    const timeDiff = (right.sortTimeMs ?? Number.NaN) - (left.sortTimeMs ?? Number.NaN);
    if (Number.isFinite(timeDiff) && timeDiff !== 0) return timeDiff;
  }

  const sortOrderDiff = (left.sortOrder ?? Number.MAX_SAFE_INTEGER) - (right.sortOrder ?? Number.MAX_SAFE_INTEGER);
  if (sortOrderDiff !== 0) return sortOrderDiff;

  return String(left.sessionId ?? "").localeCompare(String(right.sessionId ?? ""));
}

export function buildQueueSurfaceEntries(uiState, { canFocusTerminal = false } = {}) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const sceneStatusEntries = buildStatusSurfaceEntries(statusSurfaceScene, {
    displayMode: "queue",
    canFocusTerminal,
  });
  return sceneStatusEntries.length
    ? sceneStatusEntries
    : buildStatusQueueLegacyEntries(uiState, canFocusTerminal);
}

export function buildDefaultSurfaceEntries(snapshot, uiState, { canFocusTerminal = false } = {}) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const pendingEntries = buildStatusSurfaceEntries(statusSurfaceScene, {
    displayMode: "default_stack",
    canFocusTerminal,
    excludeKinds: ["completion"],
  });
  const fallbackPendingEntries = pendingEntries.length ? [] : buildPendingLegacyEntries(snapshot);
  const sceneOrFallbackEntries = pendingEntries.length ? pendingEntries : fallbackPendingEntries;
  const blockedSessionIds = new Set(sceneOrFallbackEntries.map((entry) => entry.sessionId).filter(Boolean));

  const promptAssistEntries = pendingEntries.length ? [] : buildPromptAssistLegacyEntries(snapshot, uiState, canFocusTerminal);
  promptAssistEntries.forEach((entry) => blockedSessionIds.add(entry.sessionId));

  const sessionSceneEntries = buildSessionSurfaceEntries(getSessionSurfaceScene(uiState), {
    canFocusTerminal,
  });
  const sessionEntries = sessionSceneEntries.length
    ? sessionSceneEntries
    : buildSessionLegacyEntries(snapshot, uiState, canFocusTerminal, blockedSessionIds);

  return [...sceneOrFallbackEntries, ...promptAssistEntries, ...sessionEntries].sort(compareSessionSurfaceEntries);
}
