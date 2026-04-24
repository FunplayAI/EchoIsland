import { escapeHtml, shortSessionId } from "../utils.js";
import { getStatusSurfaceScene } from "../state-helpers.js";
import {
  getPendingPermissionBodyText,
  getPendingPermissionPayloads,
  getPendingPermissionTitle,
  getPendingQuestionBodyText,
  getPendingQuestionPayloads,
  getPendingQuestionOptions,
  getPendingQuestionTitle,
  getPendingRequestId,
  getPendingSessionId,
  getPendingSourceText,
  mergePendingSnapshotItems,
} from "./pending-snapshot-fallback.js";
import { getPendingSceneCards } from "./status-surface-scene.js";

export function renderPending(snapshot, { pendingActions, pendingSummary, uiState }) {
  if (!pendingActions) return;

  pendingActions.innerHTML = "";
  const sceneCards = uiState ? getPendingSceneCards(getStatusSurfaceScene(uiState)) : null;
  const pendingPermissions = mergePendingSnapshotItems(
    getPendingPermissionPayloads(snapshot),
    sceneCards?.permissions ?? []
  );
  const pendingQuestions = mergePendingSnapshotItems(getPendingQuestionPayloads(snapshot), sceneCards?.questions ?? []);
  if (pendingSummary) {
    pendingSummary.textContent = `${pendingPermissions.length + pendingQuestions.length} waiting`;
  }

  pendingPermissions.forEach((pending) => {
    const actionDisabled = pending.display_held || pending.isLive === false ? "disabled" : "";
    const title = getPendingPermissionTitle(pending);
    const bodyText = getPendingPermissionBodyText(pending);
    const sourceText = getPendingSourceText(pending);
    const sessionId = getPendingSessionId(pending);
    const requestId = getPendingRequestId(pending);
    const card = document.createElement("div");
    card.className = "pending-detail";
    card.innerHTML = `
      <span class="label">Approval</span>
      <h3>${escapeHtml(title)}</h3>
      <p>${escapeHtml(bodyText)}</p>
      <div class="queue-meta">
        <span class="badge">${escapeHtml(sourceText)}</span>
        <span class="session-short">#${escapeHtml(shortSessionId(sessionId))}</span>
      </div>
      <div class="pending-buttons">
        <button data-action="allow" data-request-id="${escapeHtml(requestId)}" ${actionDisabled}>Allow</button>
        <button data-action="deny" data-request-id="${escapeHtml(
          requestId
        )}" class="danger" ${actionDisabled}>Deny</button>
      </div>
    `;
    pendingActions.appendChild(card);
  });

  pendingQuestions.forEach((pending) => {
    const actionDisabled = pending.display_held || pending.isLive === false ? "disabled" : "";
    const title = getPendingQuestionTitle(pending);
    const bodyText = getPendingQuestionBodyText(pending);
    const sourceText = getPendingSourceText(pending);
    const sessionId = getPendingSessionId(pending);
    const requestId = getPendingRequestId(pending);
    const optionsList = getPendingQuestionOptions(pending);
    const card = document.createElement("div");
    card.className = "pending-detail";
    const options = optionsList.length
      ? `<div class="question-options">${optionsList
          .map(
            (option) =>
              `<button data-action="answer" data-request-id="${escapeHtml(
                requestId
              )}" data-answer="${escapeHtml(option)}" ${actionDisabled}>${escapeHtml(option)}</button>`
          )
          .join("")}</div>`
      : `<div class="pending-buttons">
           <div class="question-input-row">
             <input id="questionAnswerInput" type="text" placeholder="Type your answer" ${actionDisabled} />
             <button data-action="answer-from-input" data-request-id="${escapeHtml(
               requestId
             )}" ${actionDisabled}>Submit</button>
           </div>
           <button data-action="skip-question" data-request-id="${escapeHtml(
             requestId
           )}" class="danger" ${actionDisabled}>Skip</button>
         </div>`;

    card.innerHTML = `
      <span class="label">Question</span>
      <h3>${escapeHtml(title)}</h3>
      <p>${escapeHtml(bodyText)}</p>
      <div class="queue-meta">
        <span class="badge">${escapeHtml(sourceText)}</span>
        <span class="session-short">#${escapeHtml(shortSessionId(sessionId))}</span>
      </div>
      ${options}
    `;
    pendingActions.appendChild(card);
  });

  if (!pendingPermissions.length && !pendingQuestions.length) {
    const empty = document.createElement("p");
    empty.className = "hint";
    empty.textContent = "No pending approvals or questions.";
    pendingActions.appendChild(empty);
  }
}
