import {
  formatSource,
  formatStatus,
  isLongIdleSession,
  normalizeStatus,
  statusPriority,
  stripMarkdownDisplay,
} from "../utils.js";

export function getSessionLastUserPromptRaw(session) {
  return session?.last_user_prompt ?? null;
}

export function getSessionId(session) {
  return session?.session_id ?? null;
}

export function getSessionLastAssistantMessageRaw(session) {
  return session?.last_assistant_message ?? null;
}

export function getSessionUserPromptText(session) {
  return stripMarkdownDisplay(getSessionLastUserPromptRaw(session) ?? "");
}

export function getSessionAssistantText(session) {
  const value = getSessionLastAssistantMessageRaw(session) ?? session?.tool_description ?? null;
  return value ? stripMarkdownDisplay(value) : null;
}

export function getSessionCompletionPreviewText(session, fallback = "Task complete") {
  return stripMarkdownDisplay(getSessionLastAssistantMessageRaw(session) ?? session?.tool_description ?? fallback);
}

export function getSessionToolDescriptionText(session, fallback = "working") {
  return stripMarkdownDisplay(session?.tool_description ?? fallback);
}

export function hasSessionTool(session) {
  return Boolean(session?.current_tool);
}

export function getSessionToolName(session) {
  return String(session?.current_tool ?? "");
}

export function getSessionSourceText(session) {
  return formatSource(session?.source);
}

export function getSessionSourceKey(session) {
  return String(session?.source ?? "").toLowerCase();
}

export function getSessionStatusText(session) {
  return formatStatus(session?.status);
}

export function getSessionStatusKey(session) {
  return normalizeStatus(session?.status);
}

export function getSessionStatusPriority(session) {
  return statusPriority(session?.status);
}

export function getSessionModelText(session) {
  return session?.model ? String(session.model) : null;
}

export function getSessionActivityMs(session) {
  const value = new Date(session?.last_activity ?? 0).getTime();
  return Number.isFinite(value) ? value : 0;
}

export function wasSessionRecentlyUpdated(session) {
  const timestamp = getSessionActivityMs(session);
  return timestamp > 0 && Date.now() - timestamp <= 20_000;
}

export function isPlaceholderOpenCodeSession(session) {
  const source = String(session?.source ?? "").toLowerCase();
  if (source !== "opencode") return false;

  return (
    String(session?.session_id ?? "").startsWith("open-") &&
    !session?.cwd &&
    !session?.project_name &&
    !session?.model &&
      !session?.current_tool &&
      !session?.tool_description &&
      !getSessionLastUserPromptRaw(session) &&
      !getSessionLastAssistantMessageRaw(session)
  );
}

export function estimateFallbackSessionCardHeight(session) {
  if (isFallbackSessionCompact(session)) {
    return 58;
  }

  let height = 46;
  if (getSessionUserPromptText(session)) {
    height += 18;
  }
  if (getSessionAssistantText(session)) {
    height += 30;
  }
  if (hasSessionTool(session)) {
    height += 26;
  }
  return height;
}

export function isFallbackSessionCompact(session) {
  return isLongIdleSession(session);
}
