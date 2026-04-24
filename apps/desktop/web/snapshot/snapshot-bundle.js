import { setSurfaceSceneBundle } from "../state-helpers.js";

export function unpackSnapshotStatusSurfaceBundle(bundle) {
  return {
    rawSnapshot: bundle?.snapshot ?? null,
    surfaceScene: bundle?.surfaceScene ?? null,
    statusSurfaceScene: bundle?.statusSurfaceScene ?? null,
    sessionSurfaceScene: bundle?.sessionSurfaceScene ?? null,
    settingsSurfaceScene: bundle?.settingsSurfaceScene ?? null,
  };
}

export function applySnapshotSurfaceScenes(uiState, bundleState) {
  setSurfaceSceneBundle(uiState, {
    surfaceScene: bundleState?.surfaceScene,
    statusSurfaceScene: bundleState?.statusSurfaceScene,
    sessionSurfaceScene: bundleState?.sessionSurfaceScene,
    settingsSurfaceScene: bundleState?.settingsSurfaceScene,
  });
}
