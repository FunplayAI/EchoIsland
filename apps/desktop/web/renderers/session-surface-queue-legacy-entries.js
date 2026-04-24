import { compactTitle, escapeHtml, sessionTitle, shortSessionId, timeAgo } from "../utils.js";
import { estimateMessageCardTargetHeight } from "./panel-measure.js";
import {
  getPendingPermissionBodyText,
  getPendingPermissionTitle,
  getPendingRequestId,
  getPendingSessionId,
  getPendingSourceText,
} from "./pending-snapshot-fallback.js";
import {
  getSessionActivityMs,
  getSessionCompletionPreviewText,
  getSessionId,
  getSessionSourceText,
} from "./session-snapshot-fallback.js";
import { CARD_PRIORITY, parseTimeMs } from "./session-surface-legacy-common.js";
import { getStatusQueueFallbackItems } from "./surface-state.js";

export function buildStatusQueueLegacyEntries(uiState, canFocusTerminal) {
  return getStatusQueueFallbackItems(uiState).map((item) => {
    if (item.kind === "approval") {
      const pending = item.payload;
      const actionDisabled = !item.isLive ? "disabled" : "";
      const bodyText = getPendingPermissionBodyText(pending);
      const title = getPendingPermissionTitle(pending);
      const sourceText = getPendingSourceText(pending);
      const sessionId = getPendingSessionId(pending);
      const requestId = getPendingRequestId(pending);
      return {
        key: item.key ?? `approval:${requestId}`,
        kind: "approval",
        priority: CARD_PRIORITY.approval,
        sessionId: item.sessionId,
        sortTimeMs: parseTimeMs(pending.requested_at),
        html: `
          <div class="session-card-top">
            <div class="session-title-block">
              <div class="session-heading-row">
                <h3 title="${escapeHtml(title)}">${escapeHtml(title)}</h3>
                <span class="badge">${escapeHtml(sourceText)}</span>
              </div>
              <div class="session-meta-line">
                <span class="session-short">#${escapeHtml(shortSessionId(sessionId))}</span>
                <span>${escapeHtml(timeAgo(pending.requested_at))}</span>
              </div>
            </div>
            <span class="status-pill">Approval</span>
          </div>
          <div class="chat-line assistant-line"><span class="chat-prefix">!</span><p>${escapeHtml(bodyText)}</p></div>
          <div class="pending-buttons">
            <button type="button" data-action="allow" data-request-id="${escapeHtml(
              requestId
            )}" ${actionDisabled}>Allow</button>
            <button type="button" data-action="deny" data-request-id="${escapeHtml(
              requestId
            )}" class="danger" ${actionDisabled}>Deny</button>
          </div>
        `,
        row: {
          className: "session-card pending-card",
          status: "waitingapproval",
          focusable: "false",
          compact: "false",
          exiting: item.isRemoving ? "true" : "false",
          collapsedHeight: "52px",
          targetHeight: `${estimateMessageCardTargetHeight("approval", { body: bodyText })}px`,
        },
      };
    }

    const session = item.payload;
    const projectName = sessionTitle(session);
    const assistantText = getSessionCompletionPreviewText(session);
    const sessionId = getSessionId(session);
    const activityMs = getSessionActivityMs(session);
    return {
      key: item.key ?? `completion:${sessionId}`,
      kind: "completion",
      priority: CARD_PRIORITY.completion,
      sessionId,
      sortTimeMs: activityMs,
      html: `
        <div class="session-card-top">
            <div class="session-title-block">
              <div class="session-heading-row">
                <h3 title="${escapeHtml(projectName)}">${escapeHtml(compactTitle(projectName))}</h3>
                <span class="badge">${escapeHtml(getSessionSourceText(session))}</span>
              </div>
              <div class="session-meta-line">
                <span class="session-short">#${escapeHtml(shortSessionId(sessionId))}</span>
                <span>${escapeHtml(timeAgo(activityMs))}</span>
            </div>
          </div>
          <span class="status-pill">Complete</span>
        </div>
        <div class="chat-line assistant-line">
          <span class="chat-prefix">$</span>
          <p>${escapeHtml(assistantText)}</p>
        </div>
      `,
      row: {
        className: "session-card completion-card",
        status: "complete",
        focusable: canFocusTerminal ? "true" : "false",
        compact: "false",
        exiting: item.isRemoving ? "true" : "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("completion", { body: assistantText })}px`,
        completion: "true",
      },
    };
  });
}
