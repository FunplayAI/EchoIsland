import { getPlatformCapabilities, getSurfaceMode } from "../state-helpers.js";
import { buildDefaultSurfaceEntries, buildQueueSurfaceEntries } from "./session-surface-entries.js";

function appendCardRow(sessionList, entry, index, totalCount) {
  const row = document.createElement("article");
  row.className = entry.row.className;
  row.dataset.cardKey = entry.key ?? `${entry.kind}:${entry.sessionId ?? index}`;
  if (entry.sessionId) {
    row.dataset.sessionId = entry.sessionId;
  }
  row.dataset.status = entry.row.status;
  row.dataset.focusable = entry.row.focusable;
  row.dataset.compact = entry.row.compact;
  row.dataset.exiting = entry.row.exiting ?? "false";
  if (entry.row.completion) {
    row.dataset.completion = entry.row.completion;
  }
  row.style.setProperty("--card-stagger-index", String(index));
  row.style.setProperty("--card-exit-index", String(totalCount - index - 1));
  row.style.setProperty("--card-collapsed-height", entry.row.collapsedHeight);
  row.style.setProperty("--card-target-height", entry.row.targetHeight);
  if (entry.row.minHeight) {
    row.style.minHeight = entry.row.minHeight;
  }
  if (entry.row.height) {
    row.style.height = entry.row.height;
  }
  row.innerHTML = entry.html;
  sessionList.appendChild(row);
}

function captureCardRects(sessionList) {
  return new Map(
    Array.from(sessionList?.children ?? [])
      .map((node) => {
        const key = node instanceof HTMLElement ? node.dataset.cardKey : null;
        if (!key || !(node instanceof HTMLElement)) return null;
        return [key, node.getBoundingClientRect()];
      })
      .filter(Boolean)
  );
}

function animateLayoutShift(sessionList, previousRects) {
  if (!sessionList || !previousRects?.size) return;

  const animations = [];
  for (const node of Array.from(sessionList.children)) {
    if (!(node instanceof HTMLElement)) continue;
    const key = node.dataset.cardKey;
    if (!key) continue;
    const previousRect = previousRects.get(key);
    if (!previousRect) continue;

    const nextRect = node.getBoundingClientRect();
    const deltaY = previousRect.top - nextRect.top;
    const deltaX = previousRect.left - nextRect.left;
    if (Math.abs(deltaX) < 0.5 && Math.abs(deltaY) < 0.5) continue;

    node.style.transition = "none";
    node.style.transform = `translate(${deltaX}px, ${deltaY}px)`;
    node.style.willChange = "transform";
    animations.push(node);
  }

  if (!animations.length) return;

  window.requestAnimationFrame(() => {
    for (const node of animations) {
      node.style.transition = "transform 240ms cubic-bezier(0.22, 0.84, 0.24, 1)";
      node.style.transform = "translate(0px, 0px)";
      const cleanup = () => {
        node.style.transition = "";
        node.style.transform = "";
        node.style.willChange = "";
        node.removeEventListener("transitionend", cleanup);
      };
      node.addEventListener("transitionend", cleanup);
    }
  });
}

export function renderSessions(snapshot, { sessionList, uiState }) {
  if (!sessionList) return;

  const previousRects = captureCardRects(sessionList);
  sessionList.innerHTML = "";

  const platformCapabilities = getPlatformCapabilities(uiState);
  const canFocusTerminal = Boolean(platformCapabilities?.canFocusTerminal);

  if (getSurfaceMode(uiState) === "status") {
    const statusEntries = buildQueueSurfaceEntries(uiState, { canFocusTerminal });
    if (!statusEntries.length) {
      const empty = document.createElement("div");
      empty.className = "session-empty";
      empty.style.setProperty("--card-stagger-index", "0");
      empty.style.setProperty("--card-exit-index", "0");
      empty.style.setProperty("--card-collapsed-height", "34px");
      empty.style.setProperty("--card-target-height", "84px");
      empty.textContent = "No status updates.";
      sessionList.appendChild(empty);
      return;
    }
    statusEntries.forEach((entry, index) => appendCardRow(sessionList, entry, index, statusEntries.length));
    animateLayoutShift(sessionList, previousRects);
    return;
  }

  const entries = buildDefaultSurfaceEntries(snapshot, uiState, { canFocusTerminal });

  if (!entries.length) {
    const empty = document.createElement("div");
    empty.className = "session-empty";
    empty.style.setProperty("--card-stagger-index", "0");
    empty.style.setProperty("--card-exit-index", "0");
    empty.style.setProperty("--card-collapsed-height", "34px");
    empty.style.setProperty("--card-target-height", "84px");
    empty.textContent = "No sessions yet.";
    sessionList.appendChild(empty);
    animateLayoutShift(sessionList, previousRects);
    return;
  }

  entries.forEach((entry, index) => appendCardRow(sessionList, entry, index, entries.length));
  animateLayoutShift(sessionList, previousRects);
}
