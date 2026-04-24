import {
  getSessionActivityMs,
  getSessionId,
  getSessionLastAssistantMessageRaw,
  getSessionLastUserPromptRaw,
  getSessionStatusKey,
} from "../renderers/session-snapshot-fallback.js";

export function buildCompletionBadgeItem(session) {
  return {
    sessionId: getSessionId(session),
    completedAtMs: getSessionActivityMs(session),
    lastUserPrompt: getSessionLastUserPromptRaw(session),
    lastAssistantMessage: getSessionLastAssistantMessageRaw(session),
  };
}

export function hasNewDialogueAfterCompletion(session, item) {
  return (
    getSessionActivityMs(session) > Number(item.completedAtMs ?? 0) &&
    (getSessionStatusKey(session) !== "idle" ||
      getSessionLastUserPromptRaw(session) !== (item.lastUserPrompt ?? null) ||
      getSessionLastAssistantMessageRaw(session) !== (item.lastAssistantMessage ?? null))
  );
}

export function retainCompletionBadgeItems(items, sessionsById) {
  return (items ?? []).filter((item) => {
    const session = sessionsById.get(item.sessionId);
    return session && !hasNewDialogueAfterCompletion(session, item);
  });
}

export function mergeCompletedBadgeItems(items, completedSessionIds, findSessionById) {
  const nextById = new Map((items ?? []).map((item) => [item.sessionId, item]));

  for (const sessionId of completedSessionIds ?? []) {
    const session = findSessionById(sessionId);
    if (!session) continue;
    nextById.set(sessionId, buildCompletionBadgeItem(session));
  }

  return Array.from(nextById.values());
}
