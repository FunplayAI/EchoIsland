import { desktopApi } from "./api.js";
import { bindUiEvents } from "./bindings.js";
import { startMascotLoop } from "./mascot.js";
import { createPanelController } from "./panel-controller.js";
import { renderCurrentSurface as renderCurrentSurfaceState, refreshSnapshot as refreshSnapshotState } from "./snapshot-controller.js";
import {
  getSurfaceMode,
  setCompletionSoundEnabled,
  setMascotEnabled,
  setPreferredDisplayIndex,
  setSurfaceMode,
} from "./state-helpers.js";
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
  settingsPanel,
  completionSoundToggle,
  mascotToggle,
  displaySelect,
  openReleasePageBtn,
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
      { headline, island, islandPanel, sessionList, settingsPanel, uiState, syncExpandedPanelHeight },
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
  let setIslandMode;

  ({
    clearHoverExpandTimer,
    clearHoverCollapseTimer,
    reconcileHoverState,
    syncExpandedPanelHeight,
    setIslandMode,
  } = panelController);

  const snapshotDeps = {
    uiState,
    headline,
    island,
    islandPanel,
    sessionList,
    settingsPanel,
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
      island,
      islandBar,
      islandPanel,
      settingsBtn,
      quitBtn,
      completionSoundToggle,
      mascotToggle,
      displaySelect,
      openReleasePageBtn,
      pendingActions,
      sessionList,
      KEEP_OPEN_SELECTOR,
      uiState,
      clearHoverExpandTimer,
      reconcileHoverState,
      clearHoverCollapseTimer,
      handlePendingAction,
      handleSessionCardClick,
      handleIslandBarClick,
      openSettingsSurface: async () => {
        const snapshot = uiState.snapshot.lastSnapshot;
        if (!snapshot) return;
        setSurfaceMode(uiState, getSurfaceMode(uiState) === "settings" ? "default" : "settings");
        await renderCurrentSurface(snapshot, true);
      },
      setCompletionSoundEnabled: async (enabled) => {
        const settings = await desktopApi.setCompletionSoundEnabled(enabled);
        setCompletionSoundEnabled(uiState, settings.completionSoundEnabled);
        if (completionSoundToggle) {
          completionSoundToggle.checked = !settings.completionSoundEnabled;
        }
      },
      setMascotEnabled: async (hidden) => {
        const settings = await desktopApi.setMascotEnabled(!hidden);
        setMascotEnabled(uiState, settings.mascotEnabled);
        if (mascotToggle) {
          mascotToggle.checked = !settings.mascotEnabled;
        }
        if (uiState.snapshot.lastSnapshot) {
          await renderCurrentSurface(uiState.snapshot.lastSnapshot, true);
        }
      },
      setPreferredDisplayIndex: async (index) => {
        const settings = await desktopApi.setPreferredDisplayIndex(index);
        setPreferredDisplayIndex(uiState, settings.preferredDisplayIndex);
        if (displaySelect) {
          displaySelect.value = String(settings.preferredDisplayIndex);
        }
      },
      loadSample: (fileName) => loadSample(fileName, refreshSnapshot),
      refreshSnapshot,
      hideMainWindow: () => desktopApi.hideMainWindow(),
      openSettingsLocation: () => desktopApi.openSettingsLocation(),
      openReleasePage: () => desktopApi.openReleasePage(),
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
