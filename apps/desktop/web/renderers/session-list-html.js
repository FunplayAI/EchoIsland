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
import { getCompletionSessionIds, getPlatformCapabilities } from "../state-helpers.js";
import { estimateCardHeight, estimateMessageCardTargetHeight } from "./panel-measure.js";
import { getPromptAssistSessions } from "./prompt-assist-policy.js";
import { getDisplayedSessions } from "./surface-state.js";

function styleVars(entries) {
  return entries
    .filter(([, value]) => value !== undefined && value !== null && value !== "")
    .map(([key, value]) => `${key}:${escapeHtml(value)}`)
    .join(";");
}

function buildPendingCards(snapshot) {
  const cards = [];

  if (snapshot.pending_permission) {
    const pending = snapshot.pending_permission;
    const bodyText = stripMarkdownDisplay(pending.tool_description ?? "This action needs your permission.");
    cards.push({
      type: "approval",
      targetHeight: `${estimateMessageCardTargetHeight("approval", { body: bodyText })}px`,
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
          <button type="button" data-action="allow" data-request-id="${escapeHtml(pending.request_id)}">Allow</button>
          <button type="button" data-action="deny" data-request-id="${escapeHtml(
            pending.request_id
          )}" class="danger">Deny</button>
        </div>
      `,
    });
  }

  if (snapshot.pending_question) {
    const pending = snapshot.pending_question;
    const bodyText = stripMarkdownDisplay(pending.text ?? "Need your input.");
    const options = pending.options?.length
      ? `<div class="question-options">${pending.options
          .map(
            (option) =>
              `<button type="button" data-action="answer" data-request-id="${escapeHtml(
                pending.request_id
              )}" data-answer="${escapeHtml(option)}">${escapeHtml(option)}</button>`
          )
          .join("")}</div>`
      : `<div class="pending-buttons">
           <div class="question-input-row">
             <input id="questionAnswerInput" type="text" placeholder="Type your answer" />
             <button type="button" data-action="answer-from-input" data-request-id="${escapeHtml(
               pending.request_id
             )}">Submit</button>
           </div>
           <button type="button" data-action="skip-question" data-request-id="${escapeHtml(
             pending.request_id
           )}" class="danger">Skip</button>
         </div>`;

    cards.push({
      type: "question",
      targetHeight: `${estimateMessageCardTargetHeight("question", {
        body: bodyText,
        options: Array.isArray(pending.options) ? pending.options : [],
      })}px`,
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
    });
  }

  return cards;
}

function buildPromptAssistCards(snapshot, uiState) {
  return getPromptAssistSessions(snapshot, uiState).map((session) => {
    const title = sessionTitle(session);
    const bodyText = "Approval may be required in the Codex terminal.";
    return {
      session,
      targetHeight: `${estimateMessageCardTargetHeight("attention", { body: bodyText })}px`,
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
    };
  });
}

function visibleSessions(snapshot, uiState) {
  return getDisplayedSessions(snapshot, uiState).filter((session) => {
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
}

function renderPendingCard(card, index, totalCards) {
  const style = styleVars([
    ["--card-stagger-index", String(index)],
    ["--card-exit-index", String(totalCards - index - 1)],
    ["--card-collapsed-height", "52px"],
    ["--card-target-height", card.targetHeight],
  ]);
  const status = card.type === "approval" ? "waitingapproval" : "waitingquestion";

  return `<article class="session-card pending-card" data-status="${status}" data-focusable="false" data-compact="false" style="${style}">${card.html}</article>`;
}

function renderPromptAssistCard(card, renderIndex, exitIndex, canFocusTerminal) {
  const style = styleVars([
    ["--card-stagger-index", String(renderIndex)],
    ["--card-exit-index", String(exitIndex)],
    ["--card-collapsed-height", "52px"],
    ["--card-target-height", card.targetHeight],
  ]);

  return `<article class="session-card prompt-assist-card" data-session-id="${escapeHtml(
    card.session.session_id
  )}" data-status="attention" data-focusable="${canFocusTerminal ? "true" : "false"}" data-compact="false" style="${style}">${card.html}</article>`;
}

function renderEmptyState(staggerIndex) {
  const style = styleVars([
    ["--card-stagger-index", String(staggerIndex)],
    ["--card-exit-index", "0"],
    ["--card-collapsed-height", "34px"],
    ["--card-target-height", "84px"],
  ]);

  return `<div class="session-empty" style="${style}">No sessions yet.</div>`;
}

function renderSessionCard(session, index, sessionsLength, renderIndex, uiState, canFocusTerminal) {
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
  const style = styleVars([
    ["--card-stagger-index", String(renderIndex)],
    ["--card-exit-index", String(sessionsLength - index - 1)],
    ["--card-collapsed-height", `${revealCollapsedHeight}px`],
    ["--card-target-height", `${revealTargetHeight}px`],
    ["min-height", `${estimatedHeight}px`],
    ["height", collapsedIdle ? `${estimatedHeight}px` : "auto"],
  ]);

  return `
    <article
      class="session-card"
      data-session-id="${escapeHtml(session.session_id)}"
      data-status="${escapeHtml(String(session.status ?? "").toLowerCase())}"
      data-compact="${collapsedIdle ? "true" : "false"}"
      data-completion="${getCompletionSessionIds(uiState).includes(session.session_id) ? "true" : "false"}"
      data-focusable="${canFocusTerminal ? "true" : "false"}"
      style="${style}"
    >
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
    </article>
  `;
}

export function renderSessionListHtml(snapshot, uiState) {
  const platformCapabilities = getPlatformCapabilities(uiState);
  const canFocusTerminal = Boolean(platformCapabilities?.canFocusTerminal);
  const pendingCards = buildPendingCards(snapshot);
  const promptAssistCards = buildPromptAssistCards(snapshot, uiState);
  const visible = visibleSessions(snapshot, uiState);

  const sections = [];

  pendingCards.forEach((card, index) => {
    sections.push(renderPendingCard(card, index, pendingCards.length));
  });

  promptAssistCards.forEach((card, index) => {
    const renderIndex = pendingCards.length + index;
    sections.push(
      renderPromptAssistCard(card, renderIndex, promptAssistCards.length - index - 1, canFocusTerminal)
    );
  });

  if (!visible.length) {
    if (!pendingCards.length && !promptAssistCards.length) {
      sections.push(renderEmptyState(pendingCards.length));
    }
    return sections.join("");
  }

  const regularSessions = promptAssistCards.length > 0 ? [] : visible;
  const sessions = [...regularSessions].sort((left, right) => {
    const priorityDiff = statusPriority(left.status) - statusPriority(right.status);
    if (priorityDiff !== 0) return priorityDiff;
    return new Date(right.last_activity).getTime() - new Date(left.last_activity).getTime();
  });

  sessions.forEach((session, index) => {
    const renderIndex = pendingCards.length + promptAssistCards.length + index;
    sections.push(renderSessionCard(session, index, sessions.length, renderIndex, uiState, canFocusTerminal));
  });

  return sections.join("");
}
