import {
  clearCompletionBadgeItems,
  getSurfaceMode,
  getInteraction,
  getLastSnapshot,
  getPanelHeight,
  getPlatformCapabilities,
  hasStatusQueueItems,
  getTimer,
  isExpanded,
  isTransitioning,
  setExpanded,
  setInteraction,
  setSurfaceMode,
  setTimer,
  setTransitioning,
} from "./state-helpers.js";
import { resolveSurfaceMode } from "./snapshot/queue-mode.js";
import { nextFrame, wait } from "./utils.js";

export function createPanelController({
  island,
  sessionList,
  timings,
  uiState,
  desktopApi,
  renderCurrentSurface,
}) {
  function shouldSyncWindowSurface() {
    return Boolean(getPlatformCapabilities(uiState)?.canShapeWindowRegion);
  }

  function hasPendingInteraction() {
    const snapshot = getLastSnapshot(uiState);
    return Number(snapshot?.pending_permission_count ?? 0) > 0 || Number(snapshot?.pending_question_count ?? 0) > 0;
  }

  function clearHoverExpandTimer() {
    if (getTimer(uiState, "hoverExpand")) {
      window.clearTimeout(getTimer(uiState, "hoverExpand"));
      setTimer(uiState, "hoverExpand", null);
    }
  }

  function clearHoverCollapseTimer() {
    if (getTimer(uiState, "hoverCollapse")) {
      window.clearTimeout(getTimer(uiState, "hoverCollapse"));
      setTimer(uiState, "hoverCollapse", null);
    }
  }

  function clearCardRevealTimer() {
    if (getTimer(uiState, "cardReveal")) {
      window.clearTimeout(getTimer(uiState, "cardReveal"));
      setTimer(uiState, "cardReveal", null);
    }
  }

  function resetSessionCardReveal() {
    clearCardRevealTimer();
    delete island?.dataset.cardReveal;
  }

  function startSessionCardReveal() {
    const cardCount = sessionList?.children.length ?? 0;
    if (!cardCount || !island) return;

    clearCardRevealTimer();
    island.dataset.cardReveal = "pre";

    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(() => {
        if (island.dataset.panelState !== "expanded") return;
        island.dataset.cardReveal = "run";
        const totalDuration =
          timings.cardRevealDurationMs + Math.max(0, cardCount - 1) * timings.cardRevealStaggerMs + 180;
        setTimer(
          uiState,
          "cardReveal",
          window.setTimeout(() => {
            if (island.dataset.cardReveal === "run") {
              delete island.dataset.cardReveal;
            }
            setTimer(uiState, "cardReveal", null);
          }, totalDuration)
        );
      });
    });
  }

  async function playSessionCardExit() {
    const cardCount = sessionList?.children.length ?? 0;
    if (!cardCount || island?.dataset.panelState !== "expanded" || getInteraction(uiState, "cardExitInProgress")) {
      return;
    }

    clearCardRevealTimer();
    setInteraction(uiState, "cardExitInProgress", true);
    island.dataset.cardReveal = "exit";
    try {
      const totalDuration = timings.cardExitDurationMs + Math.max(0, cardCount - 1) * timings.cardExitStaggerMs + 40;
      await wait(totalDuration);
    } finally {
      setInteraction(uiState, "cardExitInProgress", false);
    }
  }

  function shouldKeepIslandExpanded() {
    return (
      hasPendingInteraction() ||
      hasStatusQueueItems(uiState) ||
      getInteraction(uiState, "pointerInsideBar") ||
      getInteraction(uiState, "pointerInsidePanel") ||
      getInteraction(uiState, "panelHasInteractiveFocus") ||
      getInteraction(uiState, "panelPointerActive")
    );
  }

  async function syncDisplayedQueue(syncWindow = true) {
    const snapshot = getLastSnapshot(uiState);
    if (!snapshot) return;

    const nextMode = resolveSurfaceMode(uiState);
    if (getSurfaceMode(uiState) !== nextMode) {
      setSurfaceMode(uiState, nextMode);
      await renderCurrentSurface(snapshot, false);
      if (isExpanded(uiState) && syncWindow) {
        await syncExpandedPanelHeight(true);
      }
    }
  }

  function reconcileHoverState() {
    void syncDisplayedQueue(false);
    if (getInteraction(uiState, "pointerInsideBar")) {
      scheduleHoverExpand();
      return;
    }
    if (shouldKeepIslandExpanded()) {
      clearHoverCollapseTimer();
      if (!isExpanded(uiState) && !isTransitioning(uiState)) {
        void setIslandMode(true, true);
      }
      return;
    }
    scheduleHoverCollapse();
  }

  function scheduleHoverExpand() {
    clearHoverCollapseTimer();
    if (isExpanded(uiState) || isTransitioning(uiState)) return;
    if (Number(getInteraction(uiState, "suppressHoverExpandUntil") ?? 0) > Date.now()) return;
    clearHoverExpandTimer();
    setTimer(
      uiState,
      "hoverExpand",
      window.setTimeout(async () => {
        setTimer(uiState, "hoverExpand", null);
        if (!getInteraction(uiState, "pointerInsideBar") || isExpanded(uiState) || isTransitioning(uiState)) {
          return;
        }
        await setIslandMode(true, true);
      }, timings.hoverExpandDelayMs)
    );
  }

  function scheduleHoverCollapse() {
    clearHoverExpandTimer();
    if (!isExpanded(uiState) || isTransitioning(uiState)) return;
    clearHoverCollapseTimer();
    setTimer(
      uiState,
      "hoverCollapse",
      window.setTimeout(async () => {
        setTimer(uiState, "hoverCollapse", null);
        if (shouldKeepIslandExpanded() || !isExpanded(uiState) || isTransitioning(uiState)) {
          return;
        }
        await setIslandMode(false, true);
      }, timings.hoverCollapseDelayMs)
    );
  }

  async function syncExpandedPanelHeight(syncWindow = true) {
    const desiredHeight = getPanelHeight(uiState);
    document.documentElement.style.setProperty("--menu-bar-height", `${desiredHeight}px`);
    if (syncWindow && isExpanded(uiState) && shouldSyncWindowSurface()) {
      await desktopApi.setIslandPanelStagePassive(desiredHeight + 20);
    }
  }

  async function setIslandMode(expanded, syncWindow = true) {
    if (!island || isTransitioning(uiState)) {
      return;
    }
    if (isExpanded(uiState) === expanded && island.dataset.panelState === (expanded ? "expanded" : "compact")) {
      return;
    }

    setTransitioning(uiState, true);
    island.dataset.transitioning = "true";
    const syncWindowSurface = syncWindow && shouldSyncWindowSurface();

    try {
      if (expanded) {
        setExpanded(uiState, true);
        clearCompletionBadgeItems(uiState);
        if (syncWindowSurface) {
          await desktopApi.setIslandBarStagePassive();
        }
        island.dataset.mode = "compact";
        island.dataset.panelState = "morphing";
        await wait(timings.shoulderTransitionMs + Math.max(timings.morphTransitionMs, timings.expandTransitionMs));
        document.documentElement.style.setProperty("--menu-bar-height", `${getPanelHeight(uiState)}px`);
        if (syncWindowSurface) {
          await desktopApi.setIslandPanelStagePassive(getPanelHeight(uiState) + 20);
        }
        await nextFrame();
        island.dataset.panelState = "tallening";
        await wait(timings.heightTransitionMs);
        island.dataset.panelState = "expanded";
        startSessionCardReveal();
        return;
      }

      await playSessionCardExit();
      resetSessionCardReveal();
      island.dataset.panelState = "flattening";
      await wait(timings.heightTransitionMs);
      if (syncWindowSurface) {
        await desktopApi.setIslandBarStagePassive();
      }
      await nextFrame();
      island.dataset.panelState = "contracting";
      await wait(timings.expandTransitionMs);
      island.dataset.panelState = "collapsing";
      await wait(timings.shoulderTransitionMs);
      setExpanded(uiState, false);
      island.dataset.mode = "compact";
      island.dataset.panelState = "compact";
      if (syncWindowSurface) {
        await desktopApi.setIslandExpandedPassive(false);
      }
    } finally {
      setTransitioning(uiState, false);
      island.dataset.transitioning = "false";
      reconcileHoverState();
    }
  }

  return {
    clearHoverExpandTimer,
    clearHoverCollapseTimer,
    clearCardRevealTimer,
    resetSessionCardReveal,
    startSessionCardReveal,
    playSessionCardExit,
    shouldKeepIslandExpanded,
    reconcileHoverState,
    scheduleHoverExpand,
    scheduleHoverCollapse,
    syncExpandedPanelHeight,
    setIslandMode,
  };
}
