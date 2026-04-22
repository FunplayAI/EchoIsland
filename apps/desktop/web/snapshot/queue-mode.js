import { getInteraction, getSurfaceMode } from "../state-helpers.js";
import { hasStatusQueueDisplayItems } from "../renderers/surface-state.js";
import { getSurfaceSceneMode } from "../renderers/surface-scene.js";

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

  const sharedMode = getSurfaceSceneMode(uiState);
  const baseMode = sharedMode === "status" ? "status" : "default";

  if (baseMode === "status" && hasStatusQueueDisplayItems(uiState)) {
    return "status";
  }

  if (hasPanelInteraction(uiState)) {
    return baseMode === "status" && hasStatusQueueDisplayItems(uiState) ? "status" : "default";
  }

  if (isDefaultQueueInteractionActive(uiState)) {
    return "default";
  }

  return baseMode === "status" || hasStatusQueueDisplayItems(uiState) ? "status" : "default";
}

export function shouldAutoPopupStatusQueue(uiState) {
  return hasStatusQueueDisplayItems(uiState) && !hasQueueInteraction(uiState);
}
