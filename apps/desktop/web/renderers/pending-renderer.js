import { escapeHtml, formatSource, shortSessionId, stripMarkdownDisplay } from "../utils.js";

function getPendingPermissions(snapshot) {
  const items = Array.isArray(snapshot?.pending_permissions) ? snapshot.pending_permissions : [];
  if (items.length) return items;
  return snapshot?.pending_permission ? [snapshot.pending_permission] : [];
}

function getPendingQuestions(snapshot) {
  const items = Array.isArray(snapshot?.pending_questions) ? snapshot.pending_questions : [];
  if (items.length) return items;
  return snapshot?.pending_question ? [snapshot.pending_question] : [];
}

export function renderPending(snapshot, { pendingActions, pendingSummary }) {
  if (!pendingActions) return;

  pendingActions.innerHTML = "";
  const pendingPermissions = getPendingPermissions(snapshot);
  const pendingQuestions = getPendingQuestions(snapshot);
  if (pendingSummary) {
    pendingSummary.textContent = `${pendingPermissions.length + pendingQuestions.length} waiting`;
  }

  pendingPermissions.forEach((pending) => {
    const actionDisabled = pending.display_held ? "disabled" : "";
    const card = document.createElement("div");
    card.className = "pending-detail";
    card.innerHTML = `
      <span class="label">Approval</span>
      <h3>${escapeHtml(pending.tool_name ?? "Unknown tool")}</h3>
      <p>${escapeHtml(stripMarkdownDisplay(pending.tool_description ?? "No description"))}</p>
      <div class="queue-meta">
        <span class="badge">${escapeHtml(formatSource(pending.source))}</span>
        <span class="session-short">#${escapeHtml(shortSessionId(pending.session_id))}</span>
      </div>
      <div class="pending-buttons">
        <button data-action="allow" data-request-id="${escapeHtml(pending.request_id)}" ${actionDisabled}>Allow</button>
        <button data-action="deny" data-request-id="${escapeHtml(
          pending.request_id
        )}" class="danger" ${actionDisabled}>Deny</button>
      </div>
    `;
    pendingActions.appendChild(card);
  });

  pendingQuestions.forEach((pending) => {
    const actionDisabled = pending.display_held ? "disabled" : "";
    const card = document.createElement("div");
    card.className = "pending-detail";
    const options = pending.options?.length
      ? `<div class="question-options">${pending.options
          .map(
            (option) =>
              `<button data-action="answer" data-request-id="${escapeHtml(
                pending.request_id
              )}" data-answer="${escapeHtml(option)}" ${actionDisabled}>${escapeHtml(option)}</button>`
          )
          .join("")}</div>`
      : `<div class="pending-buttons">
           <div class="question-input-row">
             <input id="questionAnswerInput" type="text" placeholder="Type your answer" ${actionDisabled} />
             <button data-action="answer-from-input" data-request-id="${escapeHtml(
               pending.request_id
             )}" ${actionDisabled}>Submit</button>
           </div>
           <button data-action="skip-question" data-request-id="${escapeHtml(
             pending.request_id
           )}" class="danger" ${actionDisabled}>Skip</button>
         </div>`;

    card.innerHTML = `
      <span class="label">Question</span>
      <h3>${escapeHtml(pending.header ?? "Need your input")}</h3>
      <p>${escapeHtml(stripMarkdownDisplay(pending.text))}</p>
      <div class="queue-meta">
        <span class="badge">${escapeHtml(formatSource(pending.source))}</span>
        <span class="session-short">#${escapeHtml(shortSessionId(pending.session_id))}</span>
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
