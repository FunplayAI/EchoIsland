import { getClaudeStatus, getCodexStatus, getOpenClawStatus } from "../state-helpers.js";
import { hasSnapshotSessionSource } from "../renderers/snapshot-sessions.js";

export function shouldShowCodexFallbackHint(snapshot, uiState) {
  const status = getCodexStatus(uiState);
  if (!status || status.live_capture_ready) return false;
  return Boolean(status.codex_dir_exists || hasSnapshotSessionSource(snapshot, "codex"));
}

export function shouldShowClaudeFallbackHint(snapshot, uiState) {
  const status = getClaudeStatus(uiState);
  if (!status || status.live_capture_ready) return false;
  return Boolean(status.claude_dir_exists || hasSnapshotSessionSource(snapshot, "claude"));
}

export function shouldShowOpenClawHint(snapshot, uiState) {
  const status = getOpenClawStatus(uiState);
  if (!status) return false;
  return Boolean(hasSnapshotSessionSource(snapshot, "openclaw") || status.hook_installed || status.openclaw_dir_exists);
}

export function applyModeHint(snapshot, { modeHint, uiState }) {
  const _ = snapshot;
  const __ = uiState;
  if (!modeHint) return;
  modeHint.hidden = true;
  modeHint.removeAttribute("title");
}
