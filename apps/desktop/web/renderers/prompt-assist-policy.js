import { normalizeStatus, stripMarkdownDisplay } from "../utils.js";
import { getCompletionDisplaySessions, isCompletionSurfaceActive } from "./surface-state.js";

const PROMPT_ASSIST_RUNNING_MS = 12_000;
const PROMPT_ASSIST_PROCESSING_MS = 18_000;
const PROMPT_ASSIST_RECENT_MS = 20 * 60_000;

export function getLivePendingSessionIds(snapshot) {
  const permissionIds = Array.isArray(snapshot?.pending_permissions)
    ? snapshot.pending_permissions.map((item) => item?.session_id)
    : [snapshot?.pending_permission?.session_id];
  const questionIds = Array.isArray(snapshot?.pending_questions)
    ? snapshot.pending_questions.map((item) => item?.session_id)
    : [snapshot?.pending_question?.session_id];
  return new Set(
    [...permissionIds, ...questionIds].filter((sessionId) => typeof sessionId === "string" && sessionId.trim().length > 0)
  );
}

export function isPromptAssistSession(session, uiState, nowMs = Date.now()) {
  if (String(session?.source ?? "").toLowerCase() !== "codex") return false;
  if (uiState?.snapshot?.codexStatus?.live_capture_ready) return false;

  const status = normalizeStatus(session?.status);
  if (status !== "processing" && status !== "running") return false;

  const lastActivity = new Date(session?.last_activity ?? 0).getTime();
  if (!Number.isFinite(lastActivity) || lastActivity <= 0) return false;

  const ageMs = nowMs - lastActivity;
  const staleMs = status === "running" ? PROMPT_ASSIST_RUNNING_MS : PROMPT_ASSIST_PROCESSING_MS;
  return ageMs >= staleMs && ageMs <= PROMPT_ASSIST_RECENT_MS;
}

export function getPromptAssistSessions(snapshot, uiState, limit = 1) {
  const nowMs = Date.now();
  const livePendingSessionIds = getLivePendingSessionIds(snapshot);
  return (snapshot?.sessions ?? [])
    .filter((session) => !livePendingSessionIds.has(session?.session_id))
    .filter((session) => isPromptAssistSession(session, uiState, nowMs))
    .sort((left, right) => new Date(right.last_activity).getTime() - new Date(left.last_activity).getTime())
    .slice(0, limit);
}

export function hasPromptAssistSessions(snapshot, uiState) {
  return getPromptAssistSessions(snapshot, uiState, 1).length > 0;
}

export function getPrimaryPromptAssistSessionId(snapshot, uiState) {
  return getPromptAssistSessions(snapshot, uiState, 1)[0]?.session_id ?? null;
}

export function getPrimaryActionSession(snapshot, uiState) {
  if (!snapshot) return null;

  const pendingSessionId = snapshot.pending_permission?.session_id ?? snapshot.pending_question?.session_id ?? null;
  if (pendingSessionId) {
    return snapshot.sessions?.find((session) => session.session_id === pendingSessionId) ?? null;
  }

  const promptAssistSession = getPromptAssistSessions(snapshot, uiState, 1)[0] ?? null;
  if (promptAssistSession) return promptAssistSession;

  if (isCompletionSurfaceActive(uiState)) {
    return getCompletionDisplaySessions(snapshot, uiState)[0] ?? null;
  }

  return null;
}

export function getCompletionPreviewText(session) {
  return stripMarkdownDisplay(session?.last_assistant_message ?? session?.tool_description ?? "Task complete");
}

export function wasSessionRecentlyUpdated(session) {
  const timestamp = new Date(session?.last_activity ?? 0).getTime();
  return Number.isFinite(timestamp) && Date.now() - timestamp <= 20_000;
}
