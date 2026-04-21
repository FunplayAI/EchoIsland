import {
  loadClaudeStatus as loadClaudeStatusState,
  loadCodexStatus as loadCodexStatusState,
  loadOpenClawStatus as loadOpenClawStatusState,
} from "../snapshot-controller.js";
import { applyPlatformTheme } from "../platform-theme.js";
import {
  setAvailableDisplays,
  setCompletionSoundEnabled,
  setMascotEnabled,
  setPlatformCapabilities,
  setPlatformPaths,
  setPreferredDisplayIndex,
} from "../state-helpers.js";
import { setLog } from "../utils.js";

export async function bootApp({
  desktopApi,
  uiState,
  ipcAddrInline,
  island,
  eventLog,
  setIslandMode,
  refreshSnapshot,
  bindUiEvents,
  bindUiEventsArgs,
  startMascotLoop,
  mascotCanvas,
  mascotShell,
  mascotCompletionBadge,
}) {
  const ipcAddr = await desktopApi.ipcAddr();
  const capabilities = await desktopApi.platformCapabilities();
  const platformPaths = await desktopApi.platformPaths();
  const appSettings = await desktopApi.getAppSettings();
  const availableDisplays = await desktopApi.getAvailableDisplays();
  setPlatformCapabilities(uiState, capabilities);
  setPlatformPaths(uiState, platformPaths);
  setAvailableDisplays(uiState, availableDisplays);
  setCompletionSoundEnabled(uiState, appSettings.completionSoundEnabled);
  setMascotEnabled(uiState, appSettings.mascotEnabled);
  setPreferredDisplayIndex(uiState, appSettings.preferredDisplayIndex);
  applyPlatformTheme(capabilities, { island });
  if (ipcAddrInline) {
    ipcAddrInline.textContent = ipcAddr;
  }
  if (island) {
    island.dataset.panelState = "compact";
    island.dataset.transitioning = "false";
  }
  await loadCodexStatusState(desktopApi, { uiState, eventLog });
  await loadClaudeStatusState(desktopApi, { uiState, eventLog });
  await loadOpenClawStatusState(desktopApi, { uiState, eventLog });
  await setIslandMode(false, true);
  await refreshSnapshot();

  bindUiEvents(bindUiEventsArgs);

  window.__TAURI__.event.listen("ipc-error", ({ payload }) => {
    setLog(eventLog, `IPC error: ${payload}`, true);
  });

  window.__TAURI__.event.listen("tray-refresh", async () => {
    await refreshSnapshot();
    setLog(eventLog, "Tray requested refresh.", true);
  });

  startMascotLoop({ mascotCanvas, mascotShell, mascotCompletionBadge, uiState });
  setInterval(refreshSnapshot, 1500);
}
