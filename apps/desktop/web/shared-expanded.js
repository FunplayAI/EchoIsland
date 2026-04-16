import { desktopApi } from "./api.js";
import { applyPlatformTheme } from "./platform-theme.js";
import { renderExpandedPanelHtml } from "./renderers/expanded-panel-renderer.js";
import { getPlatformCapabilities, setLastSnapshot, setPlatformCapabilities } from "./state-helpers.js";
import { uiState } from "./ui-context.js";

const host = document.querySelector("#expandedPanelHost");
let renderCycleToken = 0;
let lastRenderedHtml = "";
let lastReportedHeight = -1;

function setHostState(snapshot) {
  if (!host) return;
  host.dataset.empty = snapshot?.active_session_count > 0 ? "false" : "true";
  host.dataset.status = String(snapshot?.status ?? "idle").toLowerCase();
}

function measureHeight() {
  if (!host) return 0;
  return Math.ceil(host.getBoundingClientRect().height);
}

function reportMeasuredHeight() {
  const height = measureHeight();
  if (!Number.isFinite(height) || height <= 0) return;
  if (Math.abs(height - lastReportedHeight) < 1) return;
  lastReportedHeight = height;
  void desktopApi.setMacosSharedExpandedHeight(height);
}

function scheduleMeasuredHeightReport() {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      reportMeasuredHeight();
    });
  });
}

function applyRenderStage() {
  if (!host) return;

  const cycleToken = String(++renderCycleToken);
  host.dataset.renderCycle = cycleToken;
  host.dataset.renderStage = "pre";

  host.querySelectorAll(".session-card, .session-empty").forEach((element, index) => {
    element.style.setProperty("--shared-card-index", String(index));
  });

  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      if (!host || host.dataset.renderCycle !== cycleToken) return;
      host.dataset.renderStage = "run";
    });
  });
}

export function renderExpandedSnapshot(snapshot) {
  if (!host) return;
  setLastSnapshot(uiState, snapshot);
  const nextHtml = renderExpandedPanelHtml(snapshot, uiState);
  const contentChanged = nextHtml !== lastRenderedHtml;
  if (contentChanged) {
    host.innerHTML = nextHtml;
    lastRenderedHtml = nextHtml;
  }
  setHostState(snapshot);
  if (contentChanged) {
    applyRenderStage();
    scheduleMeasuredHeightReport();
  }
}

export async function refreshExpandedSnapshot() {
  const snapshot = await desktopApi.getSnapshot();
  renderExpandedSnapshot(snapshot);
  return snapshot;
}

async function focusSessionById(sessionId) {
  const capabilities = getPlatformCapabilities(uiState);
  if (!capabilities?.canFocusTerminal || !sessionId) return;

  try {
    await desktopApi.focusSessionTerminal(sessionId);
  } catch (error) {
    console.warn("Failed to focus session terminal", error);
  }
}

async function handleActionClick(button) {
  const action = button?.dataset?.action;
  const requestId = button?.dataset?.requestId;
  const answer = button?.dataset?.answer;

  if (!action || !requestId) return false;

  try {
    if (action === "allow") {
      await desktopApi.approvePermission(requestId);
    } else if (action === "deny") {
      await desktopApi.denyPermission(requestId);
    } else if (action === "answer") {
      await desktopApi.answerQuestion(requestId, answer ?? "");
    } else if (action === "answer-from-input") {
      const input = host?.querySelector("#questionAnswerInput");
      const inputValue = input?.value?.trim();
      if (!inputValue) return true;
      await desktopApi.answerQuestion(requestId, inputValue);
    } else if (action === "skip-question") {
      await desktopApi.skipQuestion(requestId);
    } else {
      return false;
    }

    await refreshExpandedSnapshot();
    return true;
  } catch (error) {
    console.warn(`Shared expanded action failed: ${action}`, error);
    return true;
  }
}

async function handleHostClick(event) {
  if (!(event.target instanceof Element)) return;

  const actionButton = event.target.closest("button[data-action]");
  if (actionButton) {
    await handleActionClick(actionButton);
    return;
  }

  const card = event.target.closest(".session-card[data-session-id]");
  if (!card) return;
  await focusSessionById(card.dataset.sessionId);
}

async function handleHostKeydown(event) {
  if (!(event.target instanceof Element)) return;
  if (event.key !== "Enter" || event.shiftKey || event.isComposing) return;

  const input = event.target.closest("#questionAnswerInput");
  if (!input) return;

  event.preventDefault();
  const submitButton = host?.querySelector('button[data-action="answer-from-input"]');
  if (submitButton) {
    await handleActionClick(submitButton);
  }
}

async function boot() {
  const capabilities = await desktopApi.platformCapabilities();
  setPlatformCapabilities(uiState, capabilities);
  applyPlatformTheme(capabilities);
  document.documentElement.dataset.contentHost = "shared-expanded";
  host?.addEventListener("click", (event) => {
    void handleHostClick(event);
  });
  host?.addEventListener("keydown", (event) => {
    void handleHostKeydown(event);
  });
  window.__TAURI__.event.listen("shared-expanded-snapshot", ({ payload }) => {
    if (payload) {
      renderExpandedSnapshot(payload);
    }
  });
  await refreshExpandedSnapshot();

  window.__CODEISLAND_SHARED_EXPANDED__ = {
    measureHeight,
    refreshSnapshot: refreshExpandedSnapshot,
    renderSnapshot: renderExpandedSnapshot,
  };
}

boot().catch((error) => {
  console.error("Failed to boot shared expanded content", error);
});
