import { applyMascot } from "../mascot.js";
import { estimateExpandedPanelHeight, renderPending, renderSessions, updateHeadline } from "../renderers.js";
import {
  getMascotState,
  getInteraction,
  getPanelHeight,
  getSurfaceMode,
  getAvailableDisplays,
  getPreferredDisplayIndex,
  isCompletionSoundEnabled,
  isMascotEnabled,
  isExpanded,
  setPanelHeight,
} from "../state-helpers.js";

export async function renderCurrentSurface(snapshot, deps, syncExpanded = true) {
  const { headline, island, islandPanel, sessionList, settingsPanel, uiState, syncExpandedPanelHeight } = deps;
  const surfaceMode = getSurfaceMode(uiState);
  updateHeadline(snapshot, { headline, uiState });
  if (!getInteraction(uiState, "cardExitInProgress") && surfaceMode !== "settings") {
    renderSessions(snapshot, { islandPanel, sessionList, uiState });
  }
  if (sessionList) {
    sessionList.hidden = surfaceMode === "settings";
    sessionList.setAttribute("aria-hidden", surfaceMode === "settings" ? "true" : "false");
  }
  if (settingsPanel) {
    settingsPanel.hidden = surfaceMode !== "settings";
    settingsPanel.setAttribute("aria-hidden", surfaceMode === "settings" ? "false" : "true");
    const toggle = settingsPanel.querySelector("#completionSoundToggle");
    if (toggle) {
      toggle.checked = !isCompletionSoundEnabled(uiState);
    }
    const mascotToggle = settingsPanel.querySelector("#mascotToggle");
    if (mascotToggle) {
      mascotToggle.checked = !isMascotEnabled(uiState);
    }
    const displaySelect = settingsPanel.querySelector("#displaySelect");
    if (displaySelect) {
      const displays = getAvailableDisplays(uiState);
      const selectedIndex = getPreferredDisplayIndex(uiState);
      displaySelect.innerHTML = displays
        .map(
          (display) =>
            `<option value="${display.index}" ${display.index === selectedIndex ? "selected" : ""}>${display.name} (${display.width}×${display.height})</option>`
        )
        .join("");
    }
  }
  if (island) {
    island.dataset.surface = surfaceMode;
  }
  setPanelHeight(uiState, estimateExpandedPanelHeight({ islandPanel, sessionList, settingsPanel, uiState }));
  document.documentElement.style.setProperty("--menu-bar-height", `${getPanelHeight(uiState)}px`);
  if (syncExpanded && isExpanded(uiState) && island?.dataset.panelState === "expanded") {
    await syncExpandedPanelHeight(true);
  }
}

export function applyStatusTone(snapshot, { island, statusChip }) {
  if (!island) return;

  island.dataset.attention =
    snapshot.pending_permission_count > 0 || snapshot.pending_question_count > 0 ? "hot" : "calm";
  if (statusChip) {
    statusChip.textContent = snapshot.status;
    statusChip.dataset.status = snapshot.status.toLowerCase();
  }
  island.dataset.empty = snapshot.active_session_count > 0 ? "false" : "true";
}

export function applyCompletionAttention(snapshot, { island, mascotShell, uiState }) {
  applyMascot(snapshot, { mascotShell, uiState });
  if (island) {
    island.dataset.completionAttention = getMascotState(uiState) === "complete" ? "true" : "false";
  }
}

export async function presentSnapshot(snapshot, deps) {
  const {
    uiState,
    headline,
    island,
    islandPanel,
    sessionList,
    settingsPanel,
    pendingActions,
    pendingSummary,
    statusChip,
    mascotShell,
    syncExpandedPanelHeight,
  } = deps;

  await renderCurrentSurface(
    snapshot,
    { headline, island, islandPanel, sessionList, settingsPanel, uiState, syncExpandedPanelHeight },
    false
  );
  if (!getInteraction(uiState, "cardExitInProgress")) {
    renderPending(snapshot, { pendingActions, pendingSummary });
  }
  applyStatusTone(snapshot, { island, statusChip });
  applyCompletionAttention(snapshot, { island, mascotShell, uiState });
}
