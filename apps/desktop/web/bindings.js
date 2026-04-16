import { getInteraction, setInteraction } from "./state-helpers.js";

export function bindUiEvents({
  islandBar,
  islandPanel,
  pendingActions,
  sessionList,
  KEEP_OPEN_SELECTOR,
  uiState,
  scheduleHoverExpand,
  clearHoverExpandTimer,
  reconcileHoverState,
  clearHoverCollapseTimer,
  handlePendingAction,
  handleSessionCardClick,
  handleIslandBarClick,
  loadSample,
  refreshSnapshot,
  hideMainWindow,
}) {
  document.querySelector("#refreshBtn")?.addEventListener("click", async () => refreshSnapshot());
  islandBar?.addEventListener("pointerenter", () => {
    setInteraction(uiState, "pointerInsideBar", true);
    reconcileHoverState();
    scheduleHoverExpand();
  });
  islandBar?.addEventListener("pointerleave", () => {
    setInteraction(uiState, "pointerInsideBar", false);
    clearHoverExpandTimer();
    reconcileHoverState();
  });
  islandBar?.addEventListener("click", handleIslandBarClick);
  islandPanel?.addEventListener("pointerenter", () => {
    setInteraction(uiState, "pointerInsidePanel", true);
    clearHoverCollapseTimer();
    reconcileHoverState();
  });
  islandPanel?.addEventListener("pointerleave", () => {
    setInteraction(uiState, "pointerInsidePanel", false);
    reconcileHoverState();
  });
  islandPanel?.addEventListener("pointerdown", (event) => {
    if (event.target instanceof Element && event.target.closest(KEEP_OPEN_SELECTOR)) {
      setInteraction(uiState, "panelPointerActive", true);
      clearHoverCollapseTimer();
      reconcileHoverState();
    }
  });
  window.addEventListener("pointerup", () => {
    if (!getInteraction(uiState, "panelPointerActive")) return;
    setInteraction(uiState, "panelPointerActive", false);
    reconcileHoverState();
  });
  islandPanel?.addEventListener("focusin", (event) => {
    if (event.target instanceof Element && event.target.closest(KEEP_OPEN_SELECTOR)) {
      setInteraction(uiState, "panelHasInteractiveFocus", true);
      clearHoverCollapseTimer();
      reconcileHoverState();
    }
  });
  islandPanel?.addEventListener("focusout", () => {
    window.setTimeout(() => {
      setInteraction(uiState, "panelHasInteractiveFocus", !!islandPanel?.matches(":focus-within"));
      reconcileHoverState();
    }, 0);
  });
  document.querySelector("#hideBtn")?.addEventListener("click", async () => hideMainWindow());
  document
    .querySelector("#loadCodexBtn")
    ?.addEventListener("click", async () => loadSample("codex_session_start.json"));
  document
    .querySelector("#loadClaudeBtn")
    ?.addEventListener("click", async () => loadSample("claude_permission_request.json"));
  document
    .querySelector("#loadQuestionBtn")
    ?.addEventListener("click", async () => loadSample("claude_question_request.json"));
  pendingActions?.addEventListener("click", handlePendingAction);
  sessionList?.addEventListener("click", async (event) => {
    if (event.target instanceof Element) {
      const actionButton = event.target.closest("button[data-action]");
      const action = actionButton?.dataset.action;
      if (action && ["allow", "deny", "answer", "answer-from-input", "skip-question"].includes(action)) {
        await handlePendingAction(event);
        return;
      }
    }
    await handleSessionCardClick(event);
  });
}
