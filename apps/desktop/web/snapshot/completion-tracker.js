import { normalizeStatus } from "../utils.js";
import { wasSessionRecentlyUpdated } from "../renderers.js";
import {
  clearCompletionBadgeItems,
  getCompletionBadgeItems,
  isExpanded,
  isStatusAutoExpanded,
  setCompletionBadgeItems,
} from "../state-helpers.js";

export function detectCompletedSessions(previousSnapshot, snapshot) {
  if (!previousSnapshot) return [];

  const previousById = new Map(previousSnapshot.sessions.map((session) => [session.session_id, session]));
  const completed = [];

  for (const session of snapshot.sessions) {
    const previous = previousById.get(session.session_id);
    if (!previous) continue;

    const previousStatus = normalizeStatus(previous.status);
    const currentStatus = normalizeStatus(session.status);
    const becameIdleFromActive =
      currentStatus === "idle" && (previousStatus === "processing" || previousStatus === "running");
    const idleMessageUpdated =
      currentStatus === "idle" &&
      previousStatus === "idle" &&
      wasSessionRecentlyUpdated(session) &&
      (session.last_assistant_message ?? "") !== (previous.last_assistant_message ?? "") &&
      !!(session.last_assistant_message ?? "").trim();

    if (becameIdleFromActive || idleMessageUpdated) {
      completed.push(session.session_id);
    }
  }

  return completed;
}

function sessionActivityMs(session) {
  const value = new Date(session?.last_activity ?? 0).getTime();
  return Number.isFinite(value) ? value : 0;
}

function buildCompletionBadgeItem(session) {
  return {
    sessionId: session.session_id,
    completedAtMs: sessionActivityMs(session),
    lastUserPrompt: session.last_user_prompt ?? null,
    lastAssistantMessage: session.last_assistant_message ?? null,
  };
}

function hasNewDialogueAfterCompletion(session, item) {
  return (
    sessionActivityMs(session) > Number(item.completedAtMs ?? 0) &&
    (normalizeStatus(session.status) !== "idle" ||
      (session.last_user_prompt ?? null) !== (item.lastUserPrompt ?? null) ||
      (session.last_assistant_message ?? null) !== (item.lastAssistantMessage ?? null))
  );
}

export function syncCompletionBadges(snapshot, completedSessionIds, uiState) {
  if (isExpanded(uiState) && !isStatusAutoExpanded(uiState)) {
    clearCompletionBadgeItems(uiState);
    return;
  }

  const sessionsById = new Map((snapshot.sessions ?? []).map((session) => [session.session_id, session]));
  const nextItems = getCompletionBadgeItems(uiState).filter((item) => {
    const session = sessionsById.get(item.sessionId);
    return session && !hasNewDialogueAfterCompletion(session, item);
  });
  const nextById = new Map(nextItems.map((item) => [item.sessionId, item]));

  for (const sessionId of completedSessionIds ?? []) {
    const session = sessionsById.get(sessionId);
    if (!session) continue;
    nextById.set(sessionId, buildCompletionBadgeItem(session));
  }

  setCompletionBadgeItems(uiState, Array.from(nextById.values()));
}
