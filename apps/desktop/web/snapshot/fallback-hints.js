import { getClaudeStatus, getCodexStatus, getOpenClawStatus } from "../state-helpers.js";

export function shouldShowCodexFallbackHint(snapshot, uiState) {
  const status = getCodexStatus(uiState);
  if (!status || status.live_capture_ready) return false;

  const hasCodexSessions =
    snapshot?.sessions?.some((session) => String(session?.source ?? "").toLowerCase() === "codex") ?? false;

  return Boolean(status.codex_dir_exists || hasCodexSessions);
}

export function shouldShowClaudeFallbackHint(snapshot, uiState) {
  const status = getClaudeStatus(uiState);
  if (!status || status.live_capture_ready) return false;

  const hasClaudeSessions =
    snapshot?.sessions?.some((session) => String(session?.source ?? "").toLowerCase() === "claude") ?? false;

  return Boolean(status.claude_dir_exists || hasClaudeSessions);
}

export function shouldShowOpenClawHint(snapshot, uiState) {
  const status = getOpenClawStatus(uiState);
  if (!status) return false;

  const hasOpenClawSessions =
    snapshot?.sessions?.some((session) => String(session?.source ?? "").toLowerCase() === "openclaw") ?? false;

  return Boolean(hasOpenClawSessions || status.hook_installed || status.openclaw_dir_exists);
}

export function applyModeHint(snapshot, { modeHint, uiState }) {
  const _ = snapshot;
  const __ = uiState;
  if (!modeHint) return;
  modeHint.hidden = true;
  modeHint.removeAttribute("title");
}
