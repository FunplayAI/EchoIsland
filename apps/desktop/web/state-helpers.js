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

export function getStatusSurfaceScene(uiState) {
  return uiState.snapshot.statusSurfaceScene;
}

export function setStatusSurfaceScene(uiState, scene) {
  uiState.snapshot.statusSurfaceScene = scene;
}

export function getSessionSurfaceScene(uiState) {
  return uiState.snapshot.sessionSurfaceScene;
}

export function setSessionSurfaceScene(uiState, scene) {
  uiState.snapshot.sessionSurfaceScene = scene;
}

export function getSettingsSurfaceScene(uiState) {
  return uiState.snapshot.settingsSurfaceScene;
}

export function setSettingsSurfaceScene(uiState, scene) {
  uiState.snapshot.settingsSurfaceScene = scene;
}

export function getSurfaceScene(uiState) {
  return uiState.snapshot.surfaceScene;
}

export function setSurfaceScene(uiState, scene) {
  uiState.snapshot.surfaceScene = scene;
}

export function setSurfaceSceneBundle(uiState, bundle) {
  uiState.snapshot.surfaceScene = bundle?.surfaceScene ?? null;
  uiState.snapshot.statusSurfaceScene = bundle?.statusSurfaceScene ?? null;
  uiState.snapshot.sessionSurfaceScene = bundle?.sessionSurfaceScene ?? null;
  uiState.snapshot.settingsSurfaceScene = bundle?.settingsSurfaceScene ?? null;
}

export function getLastRawSnapshot(uiState) {
  return uiState.snapshot.lastRawSnapshot;
}

export function isCompletionSoundEnabled(uiState) {
  return Boolean(uiState.settings.completionSoundEnabled);
}

export function setCompletionSoundEnabled(uiState, enabled) {
  uiState.settings.completionSoundEnabled = Boolean(enabled);
}

export function isMascotEnabled(uiState) {
  return Boolean(uiState.settings.mascotEnabled);
}

export function setMascotEnabled(uiState, enabled) {
  uiState.settings.mascotEnabled = Boolean(enabled);
}

export function getPreferredDisplayIndex(uiState) {
  return Number(uiState.settings.preferredDisplayIndex ?? 0);
}

export function setPreferredDisplayIndex(uiState, index) {
  uiState.settings.preferredDisplayIndex = Number(index ?? 0);
}

export function getAvailableDisplays(uiState) {
  return Array.isArray(uiState.settings.availableDisplays) ? uiState.settings.availableDisplays : [];
}

export function setAvailableDisplays(uiState, displays) {
  uiState.settings.availableDisplays = Array.isArray(displays) ? displays : [];
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

export function isStatusAutoExpanded(uiState) {
  return Boolean(uiState.surface.statusAutoExpanded);
}

export function setStatusAutoExpanded(uiState, value) {
  uiState.surface.statusAutoExpanded = Boolean(value);
}

export function getCompletionBadgeItems(uiState) {
  return uiState.surface.completionBadgeItems;
}

export function setCompletionBadgeItems(uiState, items) {
  uiState.surface.completionBadgeItems = items ?? [];
}

export function clearCompletionBadgeItems(uiState) {
  uiState.surface.completionBadgeItems = [];
  if (uiState.mascot.state === "complete") {
    uiState.mascot.state = "idle";
  }
}

export function getCompletionBadgeCount(uiState) {
  return uiState.surface.completionBadgeItems.length;
}

export function getStatusQueueItems(uiState) {
  return uiState.surface.statusQueueItems;
}

export function setStatusQueueItems(uiState, items) {
  uiState.surface.statusQueueItems = items;
}

export function getStatusQueueKeys(uiState) {
  return uiState.surface.statusQueueKeys;
}

export function setStatusQueueKeys(uiState, keys) {
  uiState.surface.statusQueueKeys = keys ?? [];
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
