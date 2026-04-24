import { compactTitle, escapeHtml, sessionTitle, shortSessionId, timeAgo } from "../utils.js";
import { estimateMessageCardTargetHeight } from "./panel-measure.js";
import { getPromptAssistSessions } from "./prompt-assist-policy.js";
import {
  getSessionActivityMs,
  getSessionId,
} from "./session-snapshot-fallback.js";
import { CARD_PRIORITY } from "./session-surface-legacy-common.js";

export function buildPromptAssistLegacyEntries(snapshot, uiState, canFocusTerminal) {
  return getPromptAssistSessions(snapshot, uiState).map((session) => {
    const title = sessionTitle(session);
    const bodyText = "Approval may be required in the Codex terminal.";
    const sessionId = getSessionId(session);
    return {
      key: `attention:${sessionId}`,
      kind: "attention",
      priority: CARD_PRIORITY.attention,
      sessionId,
      sortTimeMs: getSessionActivityMs(session),
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(title)}">Codex needs attention</h3>
              <span class="badge">Codex</span>
            </div>
            <div class="session-meta-line">
              <span class="session-short">#${escapeHtml(shortSessionId(sessionId))}</span>
              <span>${escapeHtml(compactTitle(title, 24))}</span>
              <span>${escapeHtml(timeAgo(getSessionActivityMs(session)))}</span>
            </div>
          </div>
          <span class="status-pill">Check</span>
        </div>
        <div class="chat-line assistant-line">
          <span class="chat-prefix">!</span>
          <p>${bodyText}</p>
        </div>
      `,
      row: {
        className: "session-card prompt-assist-card",
        status: "attention",
        focusable: canFocusTerminal ? "true" : "false",
        compact: "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("attention", { body: bodyText })}px`,
      },
    };
  });
}
