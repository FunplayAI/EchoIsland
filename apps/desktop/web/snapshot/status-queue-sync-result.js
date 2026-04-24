export function buildStatusQueueSyncResult({
  addedCount = 0,
  addedApprovalCount = 0,
  addedCompletionCount = 0,
  hasItems = false,
  nextRefreshDelayMs = null,
} = {}) {
  return {
    addedCount,
    addedApprovalCount,
    addedCompletionCount,
    hasItems,
    nextRefreshDelayMs,
  };
}

export function hasAddedStatusQueueItems(statusQueueSync) {
  return Number(statusQueueSync?.addedCount ?? 0) > 0;
}

export function getStatusQueueRefreshDelay(statusQueueSync) {
  return statusQueueSync?.nextRefreshDelayMs ?? null;
}
