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
import { getPlatformCapabilities, getStatusQueueItems, getSurfaceMode } from "../state-helpers.js";
import { estimateCardHeight, estimateMessageCardTargetHeight } from "./panel-measure.js";
import { getPromptAssistSessions } from "./prompt-assist-policy.js";
import { getDisplayedSessions } from "./surface-state.js";

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

function buildPendingEntries(snapshot) {
  const entries = [];
  const pendingPermissions = Array.isArray(snapshot?.pending_permissions)
    ? snapshot.pending_permissions
    : snapshot?.pending_permission
      ? [snapshot.pending_permission]
      : [];
  const pendingQuestions = Array.isArray(snapshot?.pending_questions)
    ? snapshot.pending_questions
    : snapshot?.pending_question
      ? [snapshot.pending_question]
      : [];

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

function buildPromptAssistEntries(snapshot, uiState, canFocusTerminal) {
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

function buildStatusQueueEntries(uiState, canFocusTerminal) {
  return getStatusQueueItems(uiState).map((item) => {
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

function buildSessionEntries(snapshot, uiState, canFocusTerminal, blockedSessionIds) {
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

function compareEntries(left, right) {
  const priorityDiff = right.priority - left.priority;
  if (priorityDiff !== 0) return priorityDiff;

  if (left.kind === "approval" || left.kind === "question") {
    const timeDiff = left.sortTimeMs - right.sortTimeMs;
    if (timeDiff !== 0) return timeDiff;
  } else if (left.kind === "session" && right.kind === "session") {
    const statusDiff = left.statusOrder - right.statusOrder;
    if (statusDiff !== 0) return statusDiff;
    const timeDiff = right.sortTimeMs - left.sortTimeMs;
    if (timeDiff !== 0) return timeDiff;
  } else {
    const timeDiff = right.sortTimeMs - left.sortTimeMs;
    if (timeDiff !== 0) return timeDiff;
  }

  return String(left.sessionId ?? "").localeCompare(String(right.sessionId ?? ""));
}

function appendCardRow(sessionList, entry, index, totalCount) {
  const row = document.createElement("article");
  row.className = entry.row.className;
  row.dataset.cardKey = entry.key ?? `${entry.kind}:${entry.sessionId ?? index}`;
  if (entry.sessionId) {
    row.dataset.sessionId = entry.sessionId;
  }
  row.dataset.status = entry.row.status;
  row.dataset.focusable = entry.row.focusable;
  row.dataset.compact = entry.row.compact;
  row.dataset.exiting = entry.row.exiting ?? "false";
  if (entry.row.completion) {
    row.dataset.completion = entry.row.completion;
  }
  row.style.setProperty("--card-stagger-index", String(index));
  row.style.setProperty("--card-exit-index", String(totalCount - index - 1));
  row.style.setProperty("--card-collapsed-height", entry.row.collapsedHeight);
  row.style.setProperty("--card-target-height", entry.row.targetHeight);
  if (entry.row.minHeight) {
    row.style.minHeight = entry.row.minHeight;
  }
  if (entry.row.height) {
    row.style.height = entry.row.height;
  }
  row.innerHTML = entry.html;
  sessionList.appendChild(row);
}

function captureCardRects(sessionList) {
  return new Map(
    Array.from(sessionList?.children ?? [])
      .map((node) => {
        const key = node instanceof HTMLElement ? node.dataset.cardKey : null;
        if (!key || !(node instanceof HTMLElement)) return null;
        return [key, node.getBoundingClientRect()];
      })
      .filter(Boolean)
  );
}

function animateLayoutShift(sessionList, previousRects) {
  if (!sessionList || !previousRects?.size) return;

  const animations = [];
  for (const node of Array.from(sessionList.children)) {
    if (!(node instanceof HTMLElement)) continue;
    const key = node.dataset.cardKey;
    if (!key) continue;
    const previousRect = previousRects.get(key);
    if (!previousRect) continue;

    const nextRect = node.getBoundingClientRect();
    const deltaY = previousRect.top - nextRect.top;
    const deltaX = previousRect.left - nextRect.left;
    if (Math.abs(deltaX) < 0.5 && Math.abs(deltaY) < 0.5) continue;

    node.style.transition = "none";
    node.style.transform = `translate(${deltaX}px, ${deltaY}px)`;
    node.style.willChange = "transform";
    animations.push(node);
  }

  if (!animations.length) return;

  window.requestAnimationFrame(() => {
    for (const node of animations) {
      node.style.transition = "transform 240ms cubic-bezier(0.22, 0.84, 0.24, 1)";
      node.style.transform = "translate(0px, 0px)";
      const cleanup = () => {
        node.style.transition = "";
        node.style.transform = "";
        node.style.willChange = "";
        node.removeEventListener("transitionend", cleanup);
      };
      node.addEventListener("transitionend", cleanup);
    }
  });
}

export function renderSessions(snapshot, { sessionList, uiState }) {
  if (!sessionList) return;

  const previousRects = captureCardRects(sessionList);
  sessionList.innerHTML = "";

  const platformCapabilities = getPlatformCapabilities(uiState);
  const canFocusTerminal = Boolean(platformCapabilities?.canFocusTerminal);

  if (getSurfaceMode(uiState) === "status") {
    const statusEntries = buildStatusQueueEntries(uiState, canFocusTerminal);
    if (!statusEntries.length) {
      const empty = document.createElement("div");
      empty.className = "session-empty";
      empty.style.setProperty("--card-stagger-index", "0");
      empty.style.setProperty("--card-exit-index", "0");
      empty.style.setProperty("--card-collapsed-height", "34px");
      empty.style.setProperty("--card-target-height", "84px");
      empty.textContent = "No status updates.";
      sessionList.appendChild(empty);
      return;
    }
    statusEntries.forEach((entry, index) => appendCardRow(sessionList, entry, index, statusEntries.length));
    animateLayoutShift(sessionList, previousRects);
    return;
  }

  const pendingEntries = buildPendingEntries(snapshot);
  const blockedSessionIds = new Set(pendingEntries.map((entry) => entry.sessionId).filter(Boolean));

  const promptAssistEntries = buildPromptAssistEntries(snapshot, uiState, canFocusTerminal);
  promptAssistEntries.forEach((entry) => blockedSessionIds.add(entry.sessionId));

  const sessionEntries = buildSessionEntries(snapshot, uiState, canFocusTerminal, blockedSessionIds);
  const entries = [...pendingEntries, ...promptAssistEntries, ...sessionEntries].sort(compareEntries);

  if (!entries.length) {
    const empty = document.createElement("div");
    empty.className = "session-empty";
    empty.style.setProperty("--card-stagger-index", "0");
    empty.style.setProperty("--card-exit-index", "0");
    empty.style.setProperty("--card-collapsed-height", "34px");
    empty.style.setProperty("--card-target-height", "84px");
    empty.textContent = "No sessions yet.";
    sessionList.appendChild(empty);
    animateLayoutShift(sessionList, previousRects);
    return;
  }

  entries.forEach((entry, index) => appendCardRow(sessionList, entry, index, entries.length));
  animateLayoutShift(sessionList, previousRects);
}
