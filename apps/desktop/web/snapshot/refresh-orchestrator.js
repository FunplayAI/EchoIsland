import {
  getStatusSurfaceScene,
  getTimer,
  getLastRawSnapshot,
  getSurfaceMode,
  isCompletionSoundEnabled,
  isExpanded,
  isTransitioning,
  setInteraction,
  setLastRawSnapshot,
  setLastSnapshot,
  setStatusAutoExpanded,
  setSurfaceMode,
  setTimer,
} from "../state-helpers.js";
import { hasStatusQueueDisplayItems } from "../renderers/surface-state.js";
import { buildSnapshotSummary } from "../renderers/snapshot-summary.js";
import { detectCompletedSessions, syncCompletionBadges } from "./completion-tracker.js";
import { applyModeHint } from "./fallback-hints.js";
import { playNotificationSound } from "../notification-sound.js";
import { applyPendingCardsToSnapshot, syncPendingCardVisibility } from "./pending-card-visibility.js";
import { hasQueueInteraction, resolveSurfaceMode, shouldAutoPopupStatusQueue } from "./queue-mode.js";
import { applySnapshotSurfaceScenes, unpackSnapshotStatusSurfaceBundle } from "./snapshot-bundle.js";
import { applyCompletionAttention, applyStatusTone, presentSnapshot } from "./snapshot-presenter.js";
import { getStatusQueueRefreshDelay, hasAddedStatusQueueItems } from "./status-queue-sync-result.js";
import { syncStatusQueue } from "./status-queue.js";

function updateSummaryFields(snapshot, deps) {
  const {
    uiState,
    primaryStatus,
    primarySource,
    activeCount,
    activeCountExpanded,
    totalCountCompact,
    totalCountExpanded,
    totalCount,
    totalCountLabel,
    permissionCount,
    questionCount,
  } = deps;
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const summary = buildSnapshotSummary(snapshot, statusSurfaceScene);

  if (primaryStatus) {
    primaryStatus.textContent = summary.statusText;
  }
  if (primarySource) {
    primarySource.textContent = summary.sourceText;
  }
  if (activeCount) {
    activeCount.textContent = summary.activeCountText;
  }
  if (activeCountExpanded) {
    activeCountExpanded.textContent = summary.activeCountText;
  }
  if (totalCountCompact) {
    totalCountCompact.textContent = summary.totalCountText;
  }
  if (totalCountExpanded) {
    totalCountExpanded.textContent = summary.totalCountText;
  }
  if (totalCount) {
    totalCount.textContent = summary.totalLabelText;
  }
  if (totalCountLabel) {
    totalCountLabel.textContent = summary.totalLabelText;
  }
  if (permissionCount) {
    permissionCount.textContent = summary.approvalCountText;
  }
  if (questionCount) {
    questionCount.textContent = summary.questionCountText;
  }
}

function scheduleStatusQueueRefresh(uiState, refreshSnapshot, delayMs) {
  const existingTimer = getTimer(uiState, "statusQueueRefresh");
  if (existingTimer) {
    window.clearTimeout(existingTimer);
    setTimer(uiState, "statusQueueRefresh", null);
  }

  if (!refreshSnapshot || !delayMs) return;

  setTimer(
    uiState,
    "statusQueueRefresh",
    window.setTimeout(() => {
      setTimer(uiState, "statusQueueRefresh", null);
      void refreshSnapshot();
    }, delayMs)
  );
}

export async function refreshSnapshot(api, deps) {
  const {
    uiState,
    island,
    statusChip,
    modeHint,
    timings,
    refreshSnapshot: requestRefresh,
    setIslandMode,
    syncExpandedPanelHeight,
  } = deps;

  const previousRawSnapshot = getLastRawSnapshot(uiState);
  const bundleState = unpackSnapshotStatusSurfaceBundle(await api.getSnapshotStatusSurfaceBundle());
  const rawSnapshot = bundleState.rawSnapshot;
  const completedSessionIds = detectCompletedSessions(previousRawSnapshot, rawSnapshot);
  syncPendingCardVisibility(rawSnapshot, bundleState.statusSurfaceScene, uiState, timings);
  const snapshot = applyPendingCardsToSnapshot(rawSnapshot, uiState);
  const statusQueueSync = syncStatusQueue(
    bundleState.statusSurfaceScene,
    snapshot,
    previousRawSnapshot,
    completedSessionIds,
    uiState,
    timings
  );
  syncCompletionBadges(rawSnapshot, completedSessionIds, uiState);
  scheduleStatusQueueRefresh(uiState, requestRefresh, getStatusQueueRefreshDelay(statusQueueSync));
  setLastRawSnapshot(uiState, rawSnapshot);
  setLastSnapshot(uiState, snapshot);
  applySnapshotSurfaceScenes(uiState, bundleState);

  updateSummaryFields(snapshot, deps);
  const currentSurfaceMode = getSurfaceMode(uiState);
  const hasStatusItems = hasStatusQueueDisplayItems(uiState);
  const queueInteractionActive = hasQueueInteraction(uiState);
  const queueAddedItems = hasAddedStatusQueueItems(statusQueueSync);

  if (queueAddedItems && isCompletionSoundEnabled(uiState)) {
    void playNotificationSound();
  }

  if (!hasStatusItems && currentSurfaceMode === "status" && isExpanded(uiState) && !isTransitioning(uiState)) {
    setInteraction(uiState, "suppressHoverExpandUntil", Date.now() + timings.statusQueue.autoCloseHoverSuppressMs);
    setStatusAutoExpanded(uiState, false);
    await setIslandMode?.(false, true);
    applyCompletionAttention(snapshot, deps);
    return;
  }

  const nextSurfaceMode = resolveSurfaceMode(uiState);
  if (getSurfaceMode(uiState) !== nextSurfaceMode) {
    setSurfaceMode(uiState, nextSurfaceMode);
  }

  await presentSnapshot(snapshot, deps);
  applyModeHint(snapshot, { modeHint, uiState });

  const shouldAutoPopup = shouldAutoPopupStatusQueue(uiState);

  if (shouldAutoPopup && queueAddedItems && !isExpanded(uiState) && !isTransitioning(uiState)) {
    await setIslandMode?.(true, true, { autoStatus: true });
    return;
  }

  if (shouldAutoPopup && queueAddedItems && isExpanded(uiState) && !isTransitioning(uiState)) {
    setStatusAutoExpanded(uiState, true);
  }

  if (shouldAutoPopup && isExpanded(uiState) && island?.dataset.panelState === "expanded") {
    await syncExpandedPanelHeight(true);
    return;
  }

  if (!hasStatusItems && !queueInteractionActive && isExpanded(uiState) && !isTransitioning(uiState)) {
    setStatusAutoExpanded(uiState, false);
    await setIslandMode?.(false, true);
    applyCompletionAttention(snapshot, deps);
    return;
  }

  if (isExpanded(uiState) && island?.dataset.panelState === "expanded") {
    await syncExpandedPanelHeight(true);
  }
}
