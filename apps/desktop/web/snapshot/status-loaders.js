import { setClaudeStatus, setCodexStatus, setOpenClawStatus } from "../state-helpers.js";
import { setLog } from "../utils.js";

export async function loadCodexStatus(api, { uiState, eventLog }) {
  try {
    setCodexStatus(uiState, await api.codexStatus());
  } catch (error) {
    setCodexStatus(uiState, null);
    setLog(eventLog, `Failed to load Codex status: ${error}`, true);
  }
}

export async function loadClaudeStatus(api, { uiState, eventLog }) {
  try {
    setClaudeStatus(uiState, await api.claudeStatus());
  } catch (error) {
    setClaudeStatus(uiState, null);
    setLog(eventLog, `Failed to load Claude status: ${error}`, true);
  }
}

export async function loadOpenClawStatus(api, { uiState, eventLog }) {
  try {
    setOpenClawStatus(uiState, await api.openclawStatus());
  } catch (error) {
    setOpenClawStatus(uiState, null);
    setLog(eventLog, `Failed to load OpenClaw status: ${error}`, true);
  }
}
