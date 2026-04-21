import { getInteraction, getSurfaceMode, hasStatusQueueItems } from "../state-helpers.js";

function hasPanelInteraction(uiState) {
  return (
    getInteraction(uiState, "pointerInsidePanel") ||
    getInteraction(uiState, "panelHasInteractiveFocus") ||
    getInteraction(uiState, "panelPointerActive")
  );
}

export function isDefaultQueueInteractionActive(uiState) {
  return getInteraction(uiState, "pointerInsideBar");
}

export function hasQueueInteraction(uiState) {
  return isDefaultQueueInteractionActive(uiState) || hasPanelInteraction(uiState);
}

export function resolveSurfaceMode(uiState) {
  if (getSurfaceMode(uiState) === "settings") {
    return "settings";
  }

  if (getSurfaceMode(uiState) === "status" && hasStatusQueueItems(uiState)) {
    return "status";
  }

  if (hasPanelInteraction(uiState)) {
    return getSurfaceMode(uiState) === "status" && hasStatusQueueItems(uiState) ? "status" : "default";
  }

  if (isDefaultQueueInteractionActive(uiState)) {
    return "default";
  }

  return hasStatusQueueItems(uiState) ? "status" : "default";
}

export function shouldAutoPopupStatusQueue(uiState) {
  return hasStatusQueueItems(uiState) && !hasQueueInteraction(uiState);
}
