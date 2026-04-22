import {
  getTimer,
  getLastRawSnapshot,
  getSurfaceMode,
  isCompletionSoundEnabled,
  isExpanded,
  isTransitioning,
  setInteraction,
  setLastRawSnapshot,
  setLastSnapshot,
  setSessionSurfaceScene,
  setSettingsSurfaceScene,
  setSurfaceScene,
  setStatusSurfaceScene,
  setStatusAutoExpanded,
  setSurfaceMode,
  setTimer,
} from "../state-helpers.js";
import { hasStatusQueueDisplayItems } from "../renderers/surface-state.js";
import { formatSource, formatStatus } from "../utils.js";
import { detectCompletedSessions, syncCompletionBadges } from "./completion-tracker.js";
import { applyModeHint } from "./fallback-hints.js";
import { playNotificationSound } from "../notification-sound.js";
import { applyPendingCardsToSnapshot, syncPendingCardVisibility } from "./pending-card-visibility.js";
import { hasQueueInteraction, resolveSurfaceMode, shouldAutoPopupStatusQueue } from "./queue-mode.js";
import { applyCompletionAttention, applyStatusTone, presentSnapshot } from "./snapshot-presenter.js";
import { syncStatusQueue } from "./status-queue.js";
import { summarizeDefaultStatusSurfaceWithFallback } from "../renderers/status-surface-scene.js";

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
  const statusSurfaceScene = uiState?.snapshot?.statusSurfaceScene ?? null;
  const defaultStatusSummary = summarizeDefaultStatusSurfaceWithFallback(statusSurfaceScene, snapshot);

  if (primaryStatus) {
    primaryStatus.textContent = formatStatus(snapshot.status);
  }
  if (primarySource) {
    primarySource.textContent = formatSource(snapshot.primary_source);
  }
  if (activeCount) {
    activeCount.textContent = String(snapshot.active_session_count);
  }
  if (activeCountExpanded) {
    activeCountExpanded.textContent = String(snapshot.active_session_count);
  }
  if (totalCountCompact) {
    totalCountCompact.textContent = String(snapshot.total_session_count);
  }
  if (totalCountExpanded) {
    totalCountExpanded.textContent = String(snapshot.total_session_count);
  }
  if (totalCount) {
    totalCount.textContent = `${snapshot.total_session_count} total`;
  }
  if (totalCountLabel) {
    totalCountLabel.textContent = `${snapshot.total_session_count} total`;
  }
  if (permissionCount) {
    permissionCount.textContent = String(defaultStatusSummary.approvalCount || snapshot.pending_permission_count);
  }
  if (questionCount) {
    questionCount.textContent = String(defaultStatusSummary.questionCount || snapshot.pending_question_count);
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
  const bundle = await api.getSnapshotStatusSurfaceBundle();
  const rawSnapshot = bundle?.snapshot ?? null;
  const surfaceScene = bundle?.surfaceScene ?? null;
  const statusSurfaceScene = bundle?.statusSurfaceScene ?? null;
  const sessionSurfaceScene = bundle?.sessionSurfaceScene ?? null;
  const settingsSurfaceScene = bundle?.settingsSurfaceScene ?? null;
  const completedSessionIds = detectCompletedSessions(previousRawSnapshot, rawSnapshot);
  syncPendingCardVisibility(rawSnapshot, statusSurfaceScene, uiState, timings);
  const snapshot = applyPendingCardsToSnapshot(rawSnapshot, uiState);
  const statusQueueSync = syncStatusQueue(
    statusSurfaceScene,
    snapshot,
    previousRawSnapshot,
    completedSessionIds,
    uiState,
    timings
  );
  syncCompletionBadges(rawSnapshot, completedSessionIds, uiState);
  scheduleStatusQueueRefresh(uiState, requestRefresh, statusQueueSync.nextRefreshDelayMs);
  setLastRawSnapshot(uiState, rawSnapshot);
  setLastSnapshot(uiState, snapshot);
  setSurfaceScene(uiState, surfaceScene);
  setStatusSurfaceScene(uiState, statusSurfaceScene);
  setSessionSurfaceScene(uiState, sessionSurfaceScene);
  setSettingsSurfaceScene(uiState, settingsSurfaceScene);

  updateSummaryFields(snapshot, deps);
  const currentSurfaceMode = getSurfaceMode(uiState);
  const hasStatusItems = hasStatusQueueDisplayItems(uiState);
  const queueInteractionActive = hasQueueInteraction(uiState);

  if (statusQueueSync.addedCount > 0 && isCompletionSoundEnabled(uiState)) {
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

  if (shouldAutoPopup && statusQueueSync.addedCount > 0 && !isExpanded(uiState) && !isTransitioning(uiState)) {
    await setIslandMode?.(true, true, { autoStatus: true });
    return;
  }

  if (shouldAutoPopup && statusQueueSync.addedCount > 0 && isExpanded(uiState) && !isTransitioning(uiState)) {
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
