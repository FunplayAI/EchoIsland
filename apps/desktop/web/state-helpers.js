export function getCodexStatus(uiState) {
  return uiState.snapshot.codexStatus;
}

export function setCodexStatus(uiState, status) {
  uiState.snapshot.codexStatus = status;
}

export function getClaudeStatus(uiState) {
  return uiState.snapshot.claudeStatus;
}

export function setClaudeStatus(uiState, status) {
  uiState.snapshot.claudeStatus = status;
}

export function getOpenClawStatus(uiState) {
  return uiState.snapshot.openclawStatus;
}

export function setOpenClawStatus(uiState, status) {
  uiState.snapshot.openclawStatus = status;
}

export function getPlatformCapabilities(uiState) {
  return uiState.snapshot.platformCapabilities;
}

export function setPlatformCapabilities(uiState, capabilities) {
  uiState.snapshot.platformCapabilities = capabilities;
}

export function getPlatformPaths(uiState) {
  return uiState.snapshot.platformPaths;
}

export function setPlatformPaths(uiState, paths) {
  uiState.snapshot.platformPaths = paths;
}

export function getLastSnapshot(uiState) {
  return uiState.snapshot.lastSnapshot;
}

export function setLastSnapshot(uiState, snapshot) {
  uiState.snapshot.lastSnapshot = snapshot;
}

export function getLastRawSnapshot(uiState) {
  return uiState.snapshot.lastRawSnapshot;
}

export function setLastRawSnapshot(uiState, snapshot) {
  uiState.snapshot.lastRawSnapshot = snapshot;
}

export function getSurfaceMode(uiState) {
  return uiState.surface.mode;
}

export function setSurfaceMode(uiState, mode) {
  uiState.surface.mode = mode;
}

export function getCompletionSessionIds(uiState) {
  return uiState.surface.completionSessionIds;
}

export function setCompletionSessionIds(uiState, sessionIds) {
  uiState.surface.completionSessionIds = Array.from(new Set((sessionIds ?? []).filter(Boolean)));
}

export function getStatusQueueItems(uiState) {
  return uiState.surface.statusQueueItems;
}

export function setStatusQueueItems(uiState, items) {
  uiState.surface.statusQueueItems = items;
}

export function hasStatusQueueItems(uiState) {
  return uiState.surface.statusQueueItems.length > 0;
}

export function getPendingPermissionCard(uiState) {
  return uiState.surface.pendingPermissionCard;
}

export function setPendingPermissionCard(uiState, value) {
  uiState.surface.pendingPermissionCard = value;
}

export function getPendingQuestionCard(uiState) {
  return uiState.surface.pendingQuestionCard;
}

export function setPendingQuestionCard(uiState, value) {
  uiState.surface.pendingQuestionCard = value;
}

export function isExpanded(uiState) {
  return Boolean(uiState.window.expanded);
}

export function setExpanded(uiState, expanded) {
  uiState.window.expanded = expanded;
}

export function isTransitioning(uiState) {
  return uiState.window.transitioning;
}

export function setTransitioning(uiState, transitioning) {
  uiState.window.transitioning = transitioning;
}

export function getPanelHeight(uiState) {
  return uiState.window.panelHeight;
}

export function setPanelHeight(uiState, panelHeight) {
  uiState.window.panelHeight = panelHeight;
}

export function getTimer(uiState, key) {
  return uiState.timers[key];
}

export function setTimer(uiState, key, value) {
  uiState.timers[key] = value;
}

export function getInteraction(uiState, key) {
  return uiState.interaction[key];
}

export function setInteraction(uiState, key, value) {
  uiState.interaction[key] = value;
}

export function getMascotSource(uiState) {
  return uiState.mascot.source;
}

export function setMascotSource(uiState, source) {
  uiState.mascot.source = source;
}

export function getMascotState(uiState) {
  return uiState.mascot.state;
}

export function setMascotState(uiState, state) {
  uiState.mascot.state = state;
}
