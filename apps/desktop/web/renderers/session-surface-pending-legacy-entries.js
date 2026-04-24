import { escapeHtml, shortSessionId } from "../utils.js";
import { estimateMessageCardTargetHeight } from "./panel-measure.js";
import {
  getPendingPermissionBodyText,
  getPendingPermissionPayloads,
  getPendingPermissionTitle,
  getPendingQuestionBodyText,
  getPendingQuestionOptions,
  getPendingQuestionPayloads,
  getPendingQuestionTitle,
  getPendingRequestId,
  getPendingSessionId,
  getPendingSourceText,
} from "./pending-snapshot-fallback.js";
import { CARD_PRIORITY, parseTimeMs } from "./session-surface-legacy-common.js";

export function buildPendingLegacyEntries(snapshot) {
  const entries = [];
  const pendingPermissions = getPendingPermissionPayloads(snapshot);
  const pendingQuestions = getPendingQuestionPayloads(snapshot);

  pendingPermissions.forEach((pending) => {
    const actionDisabled = pending.display_held ? "disabled" : "";
    const bodyText = getPendingPermissionBodyText(pending);
    const title = getPendingPermissionTitle(pending);
    const sourceText = getPendingSourceText(pending);
    const requestId = getPendingRequestId(pending);
    const sessionId = getPendingSessionId(pending);
    entries.push({
      key: `approval:${requestId}`,
      kind: "approval",
      priority: CARD_PRIORITY.approval,
      sessionId,
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
              <span>Approval</span>
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
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("approval", { body: bodyText })}px`,
      },
    });
  });

  pendingQuestions.forEach((pending) => {
    const actionDisabled = pending.display_held ? "disabled" : "";
    const bodyText = getPendingQuestionBodyText(pending);
    const title = getPendingQuestionTitle(pending);
    const sourceText = getPendingSourceText(pending);
    const requestId = getPendingRequestId(pending);
    const sessionId = getPendingSessionId(pending);
    const answerOptions = getPendingQuestionOptions(pending);
    const options = answerOptions.length
      ? `<div class="question-options">${answerOptions
          .map(
            (option) =>
              `<button type="button" data-action="answer" data-request-id="${escapeHtml(
                requestId
              )}" data-answer="${escapeHtml(option)}" ${actionDisabled}>${escapeHtml(option)}</button>`
          )
          .join("")}</div>`
      : `<div class="pending-buttons">
           <div class="question-input-row">
             <input id="questionAnswerInput" type="text" placeholder="Type your answer" ${actionDisabled} />
             <button type="button" data-action="answer-from-input" data-request-id="${escapeHtml(
               requestId
             )}" ${actionDisabled}>Submit</button>
           </div>
           <button type="button" data-action="skip-question" data-request-id="${escapeHtml(
             requestId
           )}" class="danger" ${actionDisabled}>Skip</button>
         </div>`;

    entries.push({
      key: `question:${requestId}`,
      kind: "question",
      priority: CARD_PRIORITY.question,
      sessionId,
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
              <span>Question</span>
            </div>
          </div>
          <span class="status-pill">Question</span>
        </div>
        <div class="chat-line assistant-line"><span class="chat-prefix">?</span><p>${escapeHtml(bodyText)}</p></div>
        ${options}
      `,
      row: {
        className: "session-card pending-card",
        status: "waitingquestion",
        focusable: "false",
        compact: "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("question", {
          body: bodyText,
          options: answerOptions,
        })}px`,
      },
    });
  });

  return entries;
}
