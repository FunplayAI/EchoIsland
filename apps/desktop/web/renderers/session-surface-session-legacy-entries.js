import { compactTitle, escapeHtml, sessionTitle, shortSessionId, timeAgo, toolTone } from "../utils.js";
import {
  estimateFallbackSessionCardHeight,
  getSessionActivityMs,
  getSessionAssistantText,
  getSessionId,
  getSessionModelText,
  getSessionSourceText,
  getSessionStatusKey,
  getSessionStatusPriority,
  getSessionStatusText,
  getSessionToolDescriptionText,
  getSessionToolName,
  getSessionUserPromptText,
  hasSessionTool,
  isFallbackSessionCompact,
  isPlaceholderOpenCodeSession,
} from "./session-snapshot-fallback.js";
import { CARD_PRIORITY } from "./session-surface-legacy-common.js";
import { getDisplayedSessions } from "./surface-state.js";

export function buildSessionLegacyEntries(snapshot, uiState, canFocusTerminal, blockedSessionIds) {
  const visibleSessions = getDisplayedSessions(snapshot, uiState).filter(
    (session) => !isPlaceholderOpenCodeSession(session)
  );

  return visibleSessions
    .filter((session) => !blockedSessionIds.has(getSessionId(session)))
    .map((session) => {
      const sessionId = getSessionId(session);
      const activityMs = getSessionActivityMs(session);
      const projectName = sessionTitle(session);
      const displayTitle = compactTitle(projectName);
      const collapsedIdle = isFallbackSessionCompact(session);
      const estimatedHeight = estimateFallbackSessionCardHeight(session);
      const revealTargetHeight = estimatedHeight + (collapsedIdle ? 8 : 20);
      const revealCollapsedHeight = Math.max(
        34,
        Math.min(collapsedIdle ? 52 : 64, Math.round(estimatedHeight * (collapsedIdle ? 0.76 : 0.58)))
      );
      const assistantText = getSessionAssistantText(session);
      const userPrompt = getSessionUserPromptText(session);
      const toolDescription = getSessionToolDescriptionText(session);
      const sourceText = getSessionSourceText(session);
      const modelText = getSessionModelText(session);
      const statusKey = getSessionStatusKey(session);
      const statusText = getSessionStatusText(session);
      const toolName = getSessionToolName(session);

      return {
        key: `session:${sessionId}`,
        kind: "session",
        priority: CARD_PRIORITY.session,
        sessionId,
        statusOrder: getSessionStatusPriority(session),
        sortTimeMs: activityMs,
        html: `
          <div class="session-card-top">
            <div class="session-title-block">
              <div class="session-heading-row">
                <h3 title="${escapeHtml(projectName)}">${escapeHtml(displayTitle)}</h3>
                <span class="badge">${escapeHtml(sourceText)}</span>
              </div>
              <div class="session-meta-line">
                <span class="session-short">#${escapeHtml(shortSessionId(sessionId))}</span>
                ${modelText ? `<span>${escapeHtml(modelText)}</span>` : ""}
                <span>${escapeHtml(timeAgo(activityMs))}</span>
              </div>
            </div>
            <span class="status-pill">${escapeHtml(statusText)}</span>
          </div>
          ${
            collapsedIdle
              ? ""
              : `
          ${
            userPrompt
              ? `<div class="chat-line user-line"><span class="chat-prefix">&gt;</span><p>${escapeHtml(
                  userPrompt
                )}</p></div>`
              : ""
          }
          ${
            assistantText
              ? `<div class="chat-line assistant-line"><span class="chat-prefix">$</span><p>${escapeHtml(
                  assistantText
                )}</p></div>`
              : ""
          }
          ${
            hasSessionTool(session)
              ? `<div class="live-tool" data-tone="${escapeHtml(toolTone(toolName))}">
                  <span class="live-tool-name">${escapeHtml(toolName)}</span>
                  <span class="live-tool-desc">${escapeHtml(toolDescription)}</span>
                </div>`
              : ""
          }
          `
          }
        `,
        row: {
          className: "session-card",
          status: statusKey,
          focusable: canFocusTerminal ? "true" : "false",
          compact: collapsedIdle ? "true" : "false",
          collapsedHeight: `${revealCollapsedHeight}px`,
          targetHeight: `${revealTargetHeight}px`,
          minHeight: `${estimatedHeight}px`,
          height: collapsedIdle ? `${estimatedHeight}px` : "auto",
        },
      };
    });
}
