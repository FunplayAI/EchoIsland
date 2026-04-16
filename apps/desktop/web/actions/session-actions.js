import { getPrimaryActionSession } from "../renderers.js";
import { getPlatformCapabilities, setInteraction } from "../state-helpers.js";
import { setLog } from "../utils.js";

export function createSessionActions({
  desktopApi,
  uiState,
  eventLog,
  clearHoverExpandTimer,
  timings,
}) {
  async function focusSessionById(sessionId) {
    const capabilities = getPlatformCapabilities(uiState);
    if (!capabilities?.canFocusTerminal) {
      setLog(eventLog, "当前平台暂不支持会话跳转。", true);
      return;
    }
    if (!sessionId) return;

    try {
      const focused = await desktopApi.focusSessionTerminal(sessionId);
      if (!focused) {
        setLog(eventLog, "当前没有匹配到可聚焦的终端窗口。", true);
      }
    } catch (error) {
      setLog(eventLog, `终端跳转失败: ${error}`, true);
    }
  }

  async function handleSessionCardClick(event) {
    const bindButton = event.target.closest('button[data-action="bind-terminal-tab"]');
    if (bindButton) {
      const capabilities = getPlatformCapabilities(uiState);
      if (!capabilities?.canBindTerminalTab) {
        setLog(eventLog, "当前平台暂不支持终端标签页绑定。", true);
        return;
      }
      const sessionId = bindButton.dataset.sessionId;
      if (!sessionId) return;

      try {
        const title = await desktopApi.bindSessionTerminal(sessionId);
        setLog(eventLog, `已记住当前标签页: ${title}`, true);
      } catch (error) {
        setLog(eventLog, `标签绑定失败: ${error}`, true);
      }
      return;
    }

    const card = event.target.closest(".session-card");
    if (!card) return;
    const sessionId = card.dataset.sessionId;
    await focusSessionById(sessionId);
  }

  async function handleIslandBarClick(event) {
    if (uiState.window.expanded) return;
    if (event.target instanceof Element && event.target.closest("button, input, textarea, select")) {
      return;
    }

    const snapshot = uiState?.snapshot?.lastSnapshot ?? null;
    const session = getPrimaryActionSession(snapshot, uiState);
    if (!session?.session_id) return;

    clearHoverExpandTimer();
    setInteraction(uiState, "suppressHoverExpandUntil", Date.now() + timings.interaction.compactActionHoverSuppressMs);
    await focusSessionById(session.session_id);
  }

  return {
    focusSessionById,
    handleSessionCardClick,
    handleIslandBarClick,
  };
}
