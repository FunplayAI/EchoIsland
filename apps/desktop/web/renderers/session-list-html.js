import { escapeHtml } from "../utils.js";
import { getPlatformCapabilities, getSurfaceMode } from "../state-helpers.js";
import { buildDefaultSurfaceEntries, buildQueueSurfaceEntries } from "./session-surface-entries.js";

function styleVars(entries) {
  return entries
    .filter(([, value]) => value !== undefined && value !== null && value !== "")
    .map(([key, value]) => `${key}:${escapeHtml(value)}`)
    .join(";");
}

function renderEmptyState(staggerIndex, text) {
  const style = styleVars([
    ["--card-stagger-index", String(staggerIndex)],
    ["--card-exit-index", "0"],
    ["--card-collapsed-height", "34px"],
    ["--card-target-height", "84px"],
  ]);

  return `<div class="session-empty" style="${style}">${escapeHtml(text)}</div>`;
}

function renderEntryCard(entry, renderIndex, totalEntries) {
  const style = styleVars([
    ["--card-stagger-index", String(renderIndex)],
    ["--card-exit-index", String(totalEntries - renderIndex - 1)],
    ["--card-collapsed-height", entry.row.collapsedHeight],
    ["--card-target-height", entry.row.targetHeight],
    ["min-height", entry.row.minHeight],
    ["height", entry.row.height],
  ]);

  return `
    <article
      class="${escapeHtml(entry.row.className)}"
      data-session-id="${escapeHtml(entry.sessionId ?? "")}"
      data-status="${escapeHtml(entry.row.status)}"
      data-compact="${escapeHtml(entry.row.compact)}"
      data-completion="${escapeHtml(entry.row.completion ?? "false")}"
      data-focusable="${escapeHtml(entry.row.focusable)}"
      style="${style}"
    >${entry.html}</article>
  `;
}

export function renderSessionListHtml(snapshot, uiState) {
  const platformCapabilities = getPlatformCapabilities(uiState);
  const canFocusTerminal = Boolean(platformCapabilities?.canFocusTerminal);
  const entries =
    getSurfaceMode(uiState) === "status"
      ? buildQueueSurfaceEntries(uiState, { canFocusTerminal })
      : buildDefaultSurfaceEntries(snapshot, uiState, { canFocusTerminal });

  if (!entries.length) {
    return renderEmptyState(0, getSurfaceMode(uiState) === "status" ? "No status updates." : "No sessions yet.");
  }

  return entries.map((entry, index) => renderEntryCard(entry, index, entries.length)).join("");
}
