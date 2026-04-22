import { escapeHtml, formatSource, shortSessionId, stripMarkdownDisplay } from "../utils.js";
import { getStatusSurfaceScene } from "../state-helpers.js";
import {
  getPendingPermissionPayloads,
  getPendingQuestionPayloads,
} from "./pending-snapshot-fallback.js";
import { getPendingSceneCards } from "./status-surface-scene.js";

function mergePendingItem(rawItem, sceneCard) {
  if (!rawItem && !sceneCard) return null;
  if (!rawItem) return sceneCard;
  if (!sceneCard) return rawItem;

  return {
    ...rawItem,
    ...sceneCard,
    request_id: rawItem.request_id ?? sceneCard.requestId ?? null,
    requestId: sceneCard.requestId ?? rawItem.request_id ?? null,
    session_id: rawItem.session_id ?? sceneCard.sessionId ?? null,
    sessionId: sceneCard.sessionId ?? rawItem.session_id ?? null,
    source: rawItem.source ?? sceneCard.sourceText ?? null,
    sourceText: sceneCard.sourceText ?? rawItem.sourceText ?? rawItem.source ?? null,
    title: sceneCard.title ?? rawItem.title ?? rawItem.header ?? rawItem.tool_name ?? null,
    body: sceneCard.body ?? rawItem.body ?? rawItem.text ?? rawItem.tool_description ?? null,
    answerOptions: Array.isArray(sceneCard.answerOptions)
      ? sceneCard.answerOptions
      : Array.isArray(rawItem.answerOptions)
        ? rawItem.answerOptions
        : Array.isArray(rawItem.options)
          ? rawItem.options
          : [],
  };
}

function mergePendingItems(rawItems, sceneCards) {
  if (!rawItems.length) {
    return sceneCards;
  }
  if (!sceneCards.length) {
    return rawItems;
  }

  const sceneByRequestId = new Map(
    sceneCards
      .filter((card) => card?.requestId)
      .map((card) => [card.requestId, card])
  );

  return rawItems.map((item) => mergePendingItem(item, sceneByRequestId.get(item?.request_id) ?? null));
}

export function renderPending(snapshot, { pendingActions, pendingSummary, uiState }) {
  if (!pendingActions) return;

  pendingActions.innerHTML = "";
  const sceneCards = uiState ? getPendingSceneCards(getStatusSurfaceScene(uiState)) : null;
  const pendingPermissions = mergePendingItems(getPendingPermissionPayloads(snapshot), sceneCards?.permissions ?? []);
  const pendingQuestions = mergePendingItems(getPendingQuestionPayloads(snapshot), sceneCards?.questions ?? []);
  if (pendingSummary) {
    pendingSummary.textContent = `${pendingPermissions.length + pendingQuestions.length} waiting`;
  }

  pendingPermissions.forEach((pending) => {
    const actionDisabled = pending.display_held || pending.isLive === false ? "disabled" : "";
    const title = pending.tool_name ?? pending.title ?? "Unknown tool";
    const body = pending.tool_description ?? pending.body ?? "No description";
    const source = pending.source ?? pending.sourceText;
    const sessionId = pending.session_id ?? pending.sessionId;
    const requestId = pending.request_id ?? pending.requestId;
    const card = document.createElement("div");
    card.className = "pending-detail";
    card.innerHTML = `
      <span class="label">Approval</span>
      <h3>${escapeHtml(title)}</h3>
      <p>${escapeHtml(stripMarkdownDisplay(body))}</p>
      <div class="queue-meta">
        <span class="badge">${escapeHtml(formatSource(source))}</span>
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
    const title = pending.header ?? pending.title ?? "Need your input";
    const body = pending.text ?? pending.body ?? "";
    const source = pending.source ?? pending.sourceText;
    const sessionId = pending.session_id ?? pending.sessionId;
    const requestId = pending.request_id ?? pending.requestId;
    const optionsList = Array.isArray(pending.options)
      ? pending.options
      : Array.isArray(pending.answerOptions)
        ? pending.answerOptions
        : [];
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
      <p>${escapeHtml(stripMarkdownDisplay(body))}</p>
      <div class="queue-meta">
        <span class="badge">${escapeHtml(formatSource(source))}</span>
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
