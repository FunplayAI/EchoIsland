import { applyMascot } from "../mascot.js";
import { estimateExpandedPanelHeight, renderPending, renderSessions, updateHeadline } from "../renderers.js";
import {
  getInteraction,
  getPanelHeight,
  isExpanded,
  setPanelHeight,
} from "../state-helpers.js";

export async function renderCurrentSurface(snapshot, deps, syncExpanded = true) {
  const { headline, island, islandPanel, sessionList, uiState, syncExpandedPanelHeight } = deps;
  updateHeadline(snapshot, { headline, uiState });
  if (!getInteraction(uiState, "cardExitInProgress")) {
    renderSessions(snapshot, { islandPanel, sessionList, uiState });
  }
  setPanelHeight(uiState, estimateExpandedPanelHeight({ islandPanel, sessionList }));
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

export async function presentSnapshot(snapshot, deps) {
  const {
    uiState,
    headline,
    island,
    islandPanel,
    sessionList,
    pendingActions,
    pendingSummary,
    statusChip,
    mascotShell,
    syncExpandedPanelHeight,
  } = deps;

  await renderCurrentSurface(
    snapshot,
    { headline, island, islandPanel, sessionList, uiState, syncExpandedPanelHeight },
    false
  );
  if (!getInteraction(uiState, "cardExitInProgress")) {
    renderPending(snapshot, { pendingActions, pendingSummary });
  }
  applyStatusTone(snapshot, { island, statusChip });
  applyMascot(snapshot, { mascotShell, uiState });
}
