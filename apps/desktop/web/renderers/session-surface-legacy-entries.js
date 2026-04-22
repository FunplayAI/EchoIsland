import {
  compactTitle,
  escapeHtml,
  formatSource,
  formatStatus,
  isLongIdleSession,
  sessionTitle,
  shortSessionId,
  statusPriority,
  stripMarkdownDisplay,
  timeAgo,
  toolTone,
} from "../utils.js";
import { estimateCardHeight, estimateMessageCardTargetHeight } from "./panel-measure.js";
import {
  getPendingPermissionPayloads,
  getPendingQuestionPayloads,
} from "./pending-snapshot-fallback.js";
import { getPromptAssistSessions } from "./prompt-assist-policy.js";
import { getDisplayedSessions, getStatusQueueFallbackItems } from "./surface-state.js";

const CARD_PRIORITY = {
  approval: 500,
  question: 400,
  attention: 300,
  completion: 200,
  session: 100,
};

function parseTimeMs(value, fallback = 0) {
  const timestamp = new Date(value ?? 0).getTime();
  return Number.isFinite(timestamp) ? timestamp : fallback;
}

export function buildPendingLegacyEntries(snapshot) {
  const entries = [];
  const pendingPermissions = getPendingPermissionPayloads(snapshot);
  const pendingQuestions = getPendingQuestionPayloads(snapshot);

  pendingPermissions.forEach((pending) => {
    const actionDisabled = pending.display_held ? "disabled" : "";
    const bodyText = stripMarkdownDisplay(pending.tool_description ?? "This action needs your permission.");
    entries.push({
      key: `approval:${pending.request_id}`,
      kind: "approval",
      priority: CARD_PRIORITY.approval,
      sessionId: pending.session_id ?? null,
      sortTimeMs: parseTimeMs(pending.requested_at),
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(pending.tool_name ?? "Permission request")}">${escapeHtml(
                pending.tool_name ?? "Permission request"
              )}</h3>
              <span class="badge">${escapeHtml(formatSource(pending.source))}</span>
            </div>
            <div class="session-meta-line">
              <span class="session-short">#${escapeHtml(shortSessionId(pending.session_id))}</span>
              <span>Approval</span>
            </div>
          </div>
          <span class="status-pill">Approval</span>
        </div>
        <div class="chat-line assistant-line"><span class="chat-prefix">!</span><p>${escapeHtml(bodyText)}</p></div>
        <div class="pending-buttons">
          <button type="button" data-action="allow" data-request-id="${escapeHtml(
            pending.request_id
          )}" ${actionDisabled}>Allow</button>
          <button type="button" data-action="deny" data-request-id="${escapeHtml(
            pending.request_id
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
    const bodyText = stripMarkdownDisplay(pending.text ?? "Need your input.");
    const options = pending.options?.length
      ? `<div class="question-options">${pending.options
          .map(
            (option) =>
              `<button type="button" data-action="answer" data-request-id="${escapeHtml(
                pending.request_id
              )}" data-answer="${escapeHtml(option)}" ${actionDisabled}>${escapeHtml(option)}</button>`
          )
          .join("")}</div>`
      : `<div class="pending-buttons">
           <div class="question-input-row">
             <input id="questionAnswerInput" type="text" placeholder="Type your answer" ${actionDisabled} />
             <button type="button" data-action="answer-from-input" data-request-id="${escapeHtml(
               pending.request_id
             )}" ${actionDisabled}>Submit</button>
           </div>
           <button type="button" data-action="skip-question" data-request-id="${escapeHtml(
             pending.request_id
           )}" class="danger" ${actionDisabled}>Skip</button>
         </div>`;

    entries.push({
      key: `question:${pending.request_id}`,
      kind: "question",
      priority: CARD_PRIORITY.question,
      sessionId: pending.session_id ?? null,
      sortTimeMs: parseTimeMs(pending.requested_at),
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(pending.header ?? "Need your input")}">${escapeHtml(
                pending.header ?? "Need your input"
              )}</h3>
              <span class="badge">${escapeHtml(formatSource(pending.source))}</span>
            </div>
            <div class="session-meta-line">
              <span class="session-short">#${escapeHtml(shortSessionId(pending.session_id))}</span>
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
          options: Array.isArray(pending.options) ? pending.options : [],
        })}px`,
      },
    });
  });

  return entries;
}

export function buildPromptAssistLegacyEntries(snapshot, uiState, canFocusTerminal) {
  return getPromptAssistSessions(snapshot, uiState).map((session) => {
    const title = sessionTitle(session);
    const bodyText = "Approval may be required in the Codex terminal.";
    return {
      key: `attention:${session.session_id}`,
      kind: "attention",
      priority: CARD_PRIORITY.attention,
      sessionId: session.session_id,
      sortTimeMs: parseTimeMs(session.last_activity),
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(title)}">Codex needs attention</h3>
              <span class="badge">Codex</span>
            </div>
            <div class="session-meta-line">
              <span class="session-short">#${escapeHtml(shortSessionId(session.session_id))}</span>
              <span>${escapeHtml(compactTitle(title, 24))}</span>
              <span>${escapeHtml(timeAgo(session.last_activity))}</span>
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

export function buildStatusQueueLegacyEntries(uiState, canFocusTerminal) {
  return getStatusQueueFallbackItems(uiState).map((item) => {
    if (item.kind === "approval") {
      const pending = item.payload;
      const actionDisabled = !item.isLive ? "disabled" : "";
      const bodyText = stripMarkdownDisplay(pending.tool_description ?? "This action needs your permission.");
      return {
        key: item.key ?? `approval:${pending.request_id}`,
        kind: "approval",
        priority: CARD_PRIORITY.approval,
        sessionId: item.sessionId,
        sortTimeMs: parseTimeMs(pending.requested_at),
        html: `
          <div class="session-card-top">
            <div class="session-title-block">
              <div class="session-heading-row">
                <h3 title="${escapeHtml(pending.tool_name ?? "Permission request")}">${escapeHtml(
                  pending.tool_name ?? "Permission request"
                )}</h3>
                <span class="badge">${escapeHtml(formatSource(pending.source))}</span>
              </div>
              <div class="session-meta-line">
                <span class="session-short">#${escapeHtml(shortSessionId(pending.session_id))}</span>
                <span>${escapeHtml(timeAgo(pending.requested_at))}</span>
              </div>
            </div>
            <span class="status-pill">Approval</span>
          </div>
          <div class="chat-line assistant-line"><span class="chat-prefix">!</span><p>${escapeHtml(bodyText)}</p></div>
          <div class="pending-buttons">
            <button type="button" data-action="allow" data-request-id="${escapeHtml(
              pending.request_id
            )}" ${actionDisabled}>Allow</button>
            <button type="button" data-action="deny" data-request-id="${escapeHtml(
              pending.request_id
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
    const assistantText = stripMarkdownDisplay(session.last_assistant_message ?? session.tool_description ?? "Task complete");
    return {
      key: item.key ?? `completion:${session.session_id}`,
      kind: "completion",
      priority: CARD_PRIORITY.completion,
      sessionId: session.session_id,
      sortTimeMs: parseTimeMs(session.last_activity),
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(projectName)}">${escapeHtml(compactTitle(projectName))}</h3>
              <span class="badge">${escapeHtml(formatSource(session.source))}</span>
            </div>
            <div class="session-meta-line">
              <span class="session-short">#${escapeHtml(shortSessionId(session.session_id))}</span>
              <span>${escapeHtml(timeAgo(session.last_activity))}</span>
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

export function buildSessionLegacyEntries(snapshot, uiState, canFocusTerminal, blockedSessionIds) {
  const visibleSessions = getDisplayedSessions(snapshot, uiState).filter((session) => {
    const source = String(session?.source ?? "").toLowerCase();
    if (source !== "opencode") return true;
    return !(
      String(session?.session_id ?? "").startsWith("open-") &&
      !session?.cwd &&
      !session?.project_name &&
      !session?.model &&
      !session?.current_tool &&
      !session?.tool_description &&
      !session?.last_user_prompt &&
      !session?.last_assistant_message
    );
  });

  return visibleSessions
    .filter((session) => !blockedSessionIds.has(session.session_id))
    .map((session) => {
      const projectName = sessionTitle(session);
      const displayTitle = compactTitle(projectName);
      const collapsedIdle = isLongIdleSession(session);
      const estimatedHeight = estimateCardHeight(session);
      const revealTargetHeight = estimatedHeight + (collapsedIdle ? 8 : 20);
      const revealCollapsedHeight = Math.max(
        34,
        Math.min(collapsedIdle ? 52 : 64, Math.round(estimatedHeight * (collapsedIdle ? 0.76 : 0.58)))
      );
      const assistantText = stripMarkdownDisplay(session.last_assistant_message ?? session.tool_description ?? null);
      const userPrompt = stripMarkdownDisplay(session.last_user_prompt ?? "");
      const toolDescription = stripMarkdownDisplay(session.tool_description ?? "working");

      return {
        key: `session:${session.session_id}`,
        kind: "session",
        priority: CARD_PRIORITY.session,
        sessionId: session.session_id,
        statusOrder: statusPriority(session.status),
        sortTimeMs: parseTimeMs(session.last_activity),
        html: `
          <div class="session-card-top">
            <div class="session-title-block">
              <div class="session-heading-row">
                <h3 title="${escapeHtml(projectName)}">${escapeHtml(displayTitle)}</h3>
                <span class="badge">${escapeHtml(formatSource(session.source))}</span>
              </div>
              <div class="session-meta-line">
                <span class="session-short">#${escapeHtml(shortSessionId(session.session_id))}</span>
                ${session.model ? `<span>${escapeHtml(session.model)}</span>` : ""}
                <span>${escapeHtml(timeAgo(session.last_activity))}</span>
              </div>
            </div>
            <span class="status-pill">${escapeHtml(formatStatus(session.status))}</span>
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
            session.current_tool
              ? `<div class="live-tool" data-tone="${escapeHtml(toolTone(session.current_tool))}">
                  <span class="live-tool-name">${escapeHtml(session.current_tool)}</span>
                  <span class="live-tool-desc">${escapeHtml(toolDescription)}</span>
                </div>`
              : ""
          }
          `
          }
        `,
        row: {
          className: "session-card",
          status: String(session.status ?? "").toLowerCase(),
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
