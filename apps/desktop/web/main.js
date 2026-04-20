import { desktopApi } from "./api.js";
import { bindUiEvents } from "./bindings.js";
import { startMascotLoop } from "./mascot.js";
import { createPanelController } from "./panel-controller.js";
import { renderCurrentSurface as renderCurrentSurfaceState, refreshSnapshot as refreshSnapshotState } from "./snapshot-controller.js";
import { elements, KEEP_OPEN_SELECTOR, timings, uiState } from "./ui-context.js";
import { setLog } from "./utils.js";
import { createPendingActions } from "./actions/pending-actions.js";
import { createSessionActions } from "./actions/session-actions.js";
import { bootApp } from "./app/bootstrap.js";

const {
  island,
  islandBar,
  islandPanel,
  settingsBtn,
  quitBtn,
  mascotCanvas,
  mascotShell,
  mascotCompletionBadge,
  headline,
  primaryStatus,
  primarySource,
  activeCount,
  activeCountExpanded,
  modeHint,
  totalCountCompact,
  totalCountExpanded,
  totalCount,
  totalCountLabel,
  permissionCount,
  questionCount,
  sessionList,
  eventLog,
  ipcAddrInline,
  pendingActions,
  pendingSummary,
  statusChip,
} = elements;

async function loadSample(fileName, refreshSnapshot) {
  const response = await desktopApi.ingestSample(fileName);
  setLog(eventLog, JSON.stringify(response, null, 2), true);
  await refreshSnapshot();
}

async function main() {
  let syncExpandedPanelHeight;
  let refreshInFlight = null;
  let refreshQueued = false;

  async function renderCurrentSurface(snapshot, syncExpanded = true) {
    return renderCurrentSurfaceState(
      snapshot,
      { headline, island, islandPanel, sessionList, uiState, syncExpandedPanelHeight },
      syncExpanded
    );
  }

  const panelController = createPanelController({
    island,
    sessionList,
    timings,
    uiState,
    desktopApi,
    renderCurrentSurface,
  });

  let clearHoverExpandTimer;
  let clearHoverCollapseTimer;
  let reconcileHoverState;
  let scheduleHoverExpand;
  let setIslandMode;

  ({
    clearHoverExpandTimer,
    clearHoverCollapseTimer,
    reconcileHoverState,
    scheduleHoverExpand,
    syncExpandedPanelHeight,
    setIslandMode,
  } = panelController);

  const snapshotDeps = {
    uiState,
    headline,
    island,
    islandPanel,
    sessionList,
    primaryStatus,
    primarySource,
    activeCount,
    activeCountExpanded,
    modeHint,
    totalCountCompact,
    totalCountExpanded,
    totalCount,
    totalCountLabel,
    permissionCount,
    questionCount,
    timings,
    pendingActions,
    pendingSummary,
    statusChip,
    mascotShell,
    eventLog,
    refreshSnapshot: () => refreshSnapshot(),
    syncExpandedPanelHeight,
    setIslandMode: (expanded, syncWindow, options) => setIslandMode(expanded, syncWindow, options),
  };

  async function runRefreshSnapshot() {
    await refreshSnapshotState(desktopApi, snapshotDeps);
  }

  async function refreshSnapshot() {
    if (refreshInFlight) {
      refreshQueued = true;
      return refreshInFlight;
    }

    refreshInFlight = (async () => {
      do {
        refreshQueued = false;
        await runRefreshSnapshot();
      } while (refreshQueued);
    })().finally(() => {
      refreshInFlight = null;
    });

    return refreshInFlight;
  }

  const { handlePendingAction } = createPendingActions({
    desktopApi,
    eventLog,
    pendingActions,
    refreshSnapshot,
  });

  const { handleSessionCardClick, handleIslandBarClick } = createSessionActions({
    desktopApi,
    uiState,
    eventLog,
    clearHoverExpandTimer,
    timings,
  });

  await bootApp({
    desktopApi,
    uiState,
    ipcAddrInline,
    island,
    eventLog,
    setIslandMode,
    refreshSnapshot,
    bindUiEvents,
    bindUiEventsArgs: {
      islandBar,
      islandPanel,
      settingsBtn,
      quitBtn,
      pendingActions,
      sessionList,
      KEEP_OPEN_SELECTOR,
      uiState,
      scheduleHoverExpand,
      clearHoverExpandTimer,
      reconcileHoverState,
      clearHoverCollapseTimer,
      handlePendingAction,
      handleSessionCardClick,
      handleIslandBarClick,
      loadSample: (fileName) => loadSample(fileName, refreshSnapshot),
      refreshSnapshot,
      hideMainWindow: () => desktopApi.hideMainWindow(),
      openSettingsLocation: () => desktopApi.openSettingsLocation(),
      quitApplication: () => desktopApi.quitApplication(),
    },
    startMascotLoop,
    mascotCanvas,
    mascotShell,
    mascotCompletionBadge,
  });
}

main().catch((error) => {
  setLog(eventLog, `Failed to boot app: ${error}`);
});
