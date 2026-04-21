import { getInteraction, setInteraction } from "./state-helpers.js";
import { primeNotificationSound } from "./notification-sound.js";

export function bindUiEvents({
  island,
  islandBar,
  islandPanel,
  settingsBtn,
  quitBtn,
  completionSoundToggle,
  mascotToggle,
  displaySelect,
  openReleasePageBtn,
  pendingActions,
  sessionList,
  KEEP_OPEN_SELECTOR,
  uiState,
  clearHoverExpandTimer,
  reconcileHoverState,
  clearHoverCollapseTimer,
  handlePendingAction,
  handleSessionCardClick,
  handleIslandBarClick,
  loadSample,
  refreshSnapshot,
  hideMainWindow,
  openSettingsSurface,
  setCompletionSoundEnabled,
  setMascotEnabled,
  setPreferredDisplayIndex,
  openReleasePage,
  quitApplication,
}) {
  window.addEventListener(
    "pointerdown",
    () => {
      primeNotificationSound();
    },
    { once: true }
  );
  window.addEventListener(
    "keydown",
    () => {
      primeNotificationSound();
    },
    { once: true }
  );
  let lastPointerPoint = null;
  let hoverGeometryFrame = null;

  function hoverDropCompensationPx() {
    const state = island?.dataset.panelState ?? "compact";
    if (!["morphing", "expanding", "tallening", "expanded", "flattening"].includes(state)) {
      return 0;
    }
    return Number.parseFloat(window.getComputedStyle(island).getPropertyValue("--morph-drop-distance")) || 0;
  }

  function barHoverRect() {
    if (!islandBar) return null;
    const rect = islandBar.getBoundingClientRect();
    const drop = hoverDropCompensationPx();
    return {
      left: rect.left,
      right: rect.right,
      top: rect.top - drop,
      bottom: rect.bottom,
    };
  }

  function setPointerInsideBar(nextInside) {
    if (getInteraction(uiState, "pointerInsideBar") === nextInside) return;
    setInteraction(uiState, "pointerInsideBar", nextInside);
    if (nextInside) {
      clearHoverCollapseTimer();
    } else {
      clearHoverExpandTimer();
    }
    reconcileHoverState();
  }

  function syncPointerInsideBar() {
    const rect = barHoverRect();
    if (!rect || !lastPointerPoint) {
      setPointerInsideBar(false);
      return;
    }
    const inside =
      lastPointerPoint.x >= rect.left &&
      lastPointerPoint.x <= rect.right &&
      lastPointerPoint.y >= rect.top &&
      lastPointerPoint.y <= rect.bottom;
    setPointerInsideBar(inside);
  }

  function stopHoverGeometryTracking() {
    if (!hoverGeometryFrame) return;
    window.cancelAnimationFrame(hoverGeometryFrame);
    hoverGeometryFrame = null;
  }

  function startHoverGeometryTracking() {
    if (hoverGeometryFrame) return;
    const tick = () => {
      hoverGeometryFrame = null;
      syncPointerInsideBar();
      if (island?.dataset.transitioning === "true") {
        hoverGeometryFrame = window.requestAnimationFrame(tick);
      }
    };
    hoverGeometryFrame = window.requestAnimationFrame(tick);
  }

  window.addEventListener("pointermove", (event) => {
    lastPointerPoint = { x: event.clientX, y: event.clientY };
    syncPointerInsideBar();
    if (island?.dataset.transitioning === "true") {
      startHoverGeometryTracking();
    }
  });
  window.addEventListener("pointerleave", () => {
    lastPointerPoint = null;
    stopHoverGeometryTracking();
    setPointerInsideBar(false);
  });
  if (island) {
    new MutationObserver(() => {
      syncPointerInsideBar();
      if (island.dataset.transitioning === "true") {
        startHoverGeometryTracking();
      } else {
        stopHoverGeometryTracking();
      }
    }).observe(island, {
      attributes: true,
      attributeFilter: ["data-panel-state", "data-transitioning"],
    });
  }
  document.querySelector("#refreshBtn")?.addEventListener("click", async () => refreshSnapshot());
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
  settingsBtn?.addEventListener("click", async (event) => {
    event.stopPropagation();
    await openSettingsSurface?.();
  });
  completionSoundToggle?.addEventListener("change", async (event) => {
    event.stopPropagation();
    const enabled = !Boolean(event.target?.checked);
    await setCompletionSoundEnabled?.(enabled);
  });
  mascotToggle?.addEventListener("change", async (event) => {
    event.stopPropagation();
    const hidden = Boolean(event.target?.checked);
    await setMascotEnabled?.(hidden);
  });
  displaySelect?.addEventListener("change", async (event) => {
    event.stopPropagation();
    const index = Number.parseInt(String(event.target?.value ?? "0"), 10);
    if (Number.isNaN(index)) return;
    await setPreferredDisplayIndex?.(index);
  });
  openReleasePageBtn?.addEventListener("click", async (event) => {
    event.stopPropagation();
    await openReleasePage?.();
  });
  quitBtn?.addEventListener("click", async (event) => {
    event.stopPropagation();
    await quitApplication();
  });
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
