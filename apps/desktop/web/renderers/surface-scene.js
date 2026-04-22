import { getSurfaceScene } from "../state-helpers.js";

export function getSurfaceSceneMode(uiState) {
  return getSurfaceScene(uiState)?.mode ?? null;
}

export function getSurfaceSceneHeadline(uiState) {
  const scene = getSurfaceScene(uiState);
  if (!scene?.headlineText) {
    return null;
  }
  return {
    text: scene.headlineText,
    emphasized: scene.headlineEmphasized === true,
  };
}
