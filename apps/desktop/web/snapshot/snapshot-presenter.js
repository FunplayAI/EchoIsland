import { applyMascot } from "../mascot.js";
import { estimateExpandedPanelHeight, renderPending, renderSessions, updateHeadline } from "../renderers.js";
import {
  getMascotState,
  getInteraction,
  getPanelHeight,
  getSurfaceMode,
  getStatusSurfaceScene,
  isExpanded,
  setPanelHeight,
} from "../state-helpers.js";
import {
  hasDefaultPendingStatusWithFallback,
  shouldShowCompletionGlow,
} from "../renderers/status-surface-scene.js";
import { renderSettingsPanel } from "../renderers/settings-surface-scene.js";

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
    renderSettingsPanel(settingsPanel, uiState);
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

export function applyStatusTone(snapshot, { island, statusChip, uiState }) {
  if (!island) return;
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const hasPendingAttention = hasDefaultPendingStatusWithFallback(statusSurfaceScene, snapshot);

  island.dataset.attention = hasPendingAttention ? "hot" : "calm";
  if (statusChip) {
    statusChip.textContent = snapshot.status;
    statusChip.dataset.status = snapshot.status.toLowerCase();
  }
  island.dataset.empty = snapshot.active_session_count > 0 ? "false" : "true";
}

export function applyCompletionAttention(snapshot, { island, mascotShell, uiState }) {
  applyMascot(snapshot, { mascotShell, uiState });
  if (island) {
    const statusSurfaceScene = getStatusSurfaceScene(uiState);
    const hasCompletionGlow = shouldShowCompletionGlow(statusSurfaceScene);
    island.dataset.completionAttention =
      hasCompletionGlow || getMascotState(uiState) === "complete" ? "true" : "false";
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
    renderPending(snapshot, { pendingActions, pendingSummary, uiState });
  }
  applyStatusTone(snapshot, { island, statusChip, uiState });
  applyCompletionAttention(snapshot, { island, mascotShell, uiState });
}
