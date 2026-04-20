import {
  loadClaudeStatus as loadClaudeStatusState,
  loadCodexStatus as loadCodexStatusState,
  loadOpenClawStatus as loadOpenClawStatusState,
} from "../snapshot-controller.js";
import { applyPlatformTheme } from "../platform-theme.js";
import { setPlatformCapabilities, setPlatformPaths } from "../state-helpers.js";
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
  setPlatformCapabilities(uiState, capabilities);
  setPlatformPaths(uiState, platformPaths);
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
