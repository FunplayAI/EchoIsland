import { escapeHtml } from "../utils.js";
import { estimateMessageCardTargetHeight } from "./panel-measure.js";
import {
  getFirstPendingPermissionSessionId,
  getFirstPendingQuestionSessionId,
} from "./pending-snapshot-fallback.js";

const STATUS_SURFACE_CARD_PRIORITY = {
  approval: 500,
  question: 400,
  prompt_assist: 300,
  completion: 200,
};

function buildQuestionOptionsHtml(answerOptions, requestId, actionDisabled) {
  if (answerOptions.length) {
    return `<div class="question-options">${answerOptions
      .map(
        (option) =>
          `<button type="button" data-action="answer" data-request-id="${escapeHtml(
            requestId
          )}" data-answer="${escapeHtml(option)}" ${actionDisabled}>${escapeHtml(option)}</button>`
      )
      .join("")}</div>`;
  }

  if (!requestId) {
    return "";
  }

  return `<div class="pending-buttons">
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
}

export function getStatusSurfaceCardsByMode(statusSurfaceScene, displayMode) {
  if (!statusSurfaceScene || statusSurfaceScene.displayMode !== displayMode) {
    return [];
  }

  return Array.isArray(statusSurfaceScene.cards) ? statusSurfaceScene.cards : [];
}

export function getPendingSceneCards(statusSurfaceScene) {
  const cards = getStatusSurfaceCardsByMode(statusSurfaceScene, "default_stack");
  return {
    permissions: cards.filter((card) => card?.kind === "approval"),
    questions: cards.filter((card) => card?.kind === "question"),
  };
}

export function getPromptAssistSceneCards(statusSurfaceScene) {
  return getStatusSurfaceCardsByMode(statusSurfaceScene, "default_stack").filter(
    (card) => card?.kind === "prompt_assist"
  );
}

export function getStatusQueueSceneCards(statusSurfaceScene) {
  return getStatusSurfaceCardsByMode(statusSurfaceScene, "queue");
}

export function getStatusSurfaceSessionIdsByKinds(statusSurfaceScene, displayMode, kinds = []) {
  const kindSet = new Set(kinds);
  return getStatusSurfaceCardsByMode(statusSurfaceScene, displayMode)
    .filter((card) => kindSet.has(card?.kind))
    .map((card) => card?.sessionId)
    .filter(Boolean);
}

export function getPromptAssistSceneSessionIds(statusSurfaceScene) {
  return getStatusSurfaceSessionIdsByKinds(statusSurfaceScene, "default_stack", ["prompt_assist"]);
}

export function getDefaultPendingSceneSessionIds(statusSurfaceScene) {
  return getStatusSurfaceSessionIdsByKinds(statusSurfaceScene, "default_stack", ["approval", "question"]);
}

export function getCompletionSceneSessionIds(statusSurfaceScene) {
  return getStatusSurfaceSessionIdsByKinds(statusSurfaceScene, "queue", ["completion"]);
}

export function getStatusQueueTotalCountFromScene(statusSurfaceScene) {
  return Number(statusSurfaceScene?.queueState?.totalCount ?? 0);
}

export function getStatusQueueApprovalCountFromScene(statusSurfaceScene) {
  return getStatusQueueSceneCards(statusSurfaceScene).filter((card) => card?.kind === "approval").length;
}

export function getPrimaryDefaultStatusSessionId(statusSurfaceScene) {
  return getStatusSurfaceCardsByMode(statusSurfaceScene, "default_stack")[0]?.sessionId ?? null;
}

export function getPrimaryPendingSessionIdWithFallback(statusSurfaceScene, snapshot) {
  const sceneSessionId = getPrimaryDefaultStatusSessionId(statusSurfaceScene);
  if (sceneSessionId) {
    return sceneSessionId;
  }

  const fallbackPermissionSessionId = getFirstPendingPermissionSessionId(snapshot);
  if (fallbackPermissionSessionId) {
    return fallbackPermissionSessionId;
  }

  return getFirstPendingQuestionSessionId(snapshot);
}

export function summarizeDefaultStatusSurface(statusSurfaceScene) {
  const defaultState = statusSurfaceScene?.defaultState ?? null;
  if (defaultState) {
    return {
      approvalCount: Number(defaultState.approvalCount ?? 0),
      questionCount: Number(defaultState.questionCount ?? 0),
      promptAssistCount: Number(defaultState.promptAssistCount ?? 0),
    };
  }

  const cards = getStatusSurfaceCardsByMode(statusSurfaceScene, "default_stack");
  return cards.reduce(
    (summary, card) => {
      if (card?.kind === "approval") summary.approvalCount += 1;
      if (card?.kind === "question") summary.questionCount += 1;
      if (card?.kind === "prompt_assist") summary.promptAssistCount += 1;
      return summary;
    },
    {
      approvalCount: 0,
      questionCount: 0,
      promptAssistCount: 0,
    }
  );
}

export function summarizeDefaultStatusSurfaceWithFallback(statusSurfaceScene, snapshot) {
  if (statusSurfaceScene) {
    return summarizeDefaultStatusSurface(statusSurfaceScene);
  }

  return {
    approvalCount: Number(snapshot?.pending_permission_count ?? 0),
    questionCount: Number(snapshot?.pending_question_count ?? 0),
    promptAssistCount: 0,
  };
}

export function summarizeSnapshotHeadline(snapshot) {
  const summary = summarizeDefaultStatusSurfaceWithFallback(null, snapshot);
  if (summary.approvalCount > 0) {
    return summary.approvalCount > 1 ? "Approvals needed" : "Approval needed";
  }
  if (summary.questionCount > 0) {
    return summary.questionCount > 1 ? "Questions waiting" : "Question waiting";
  }

  const activeSessionCount = Number(snapshot?.active_session_count ?? 0);
  if (activeSessionCount > 0) {
    return `${activeSessionCount} active task${activeSessionCount > 1 ? "s" : ""}`;
  }
  return "No active tasks";
}

export function hasDefaultPendingStatus(statusSurfaceScene) {
  const summary = summarizeDefaultStatusSurface(statusSurfaceScene);
  return summary.approvalCount > 0 || summary.questionCount > 0;
}

export function hasDefaultPendingStatusWithFallback(statusSurfaceScene, snapshot) {
  const summary = summarizeDefaultStatusSurfaceWithFallback(statusSurfaceScene, snapshot);
  return summary.approvalCount > 0 || summary.questionCount > 0;
}

export function getPrimaryDefaultStatusKind(statusSurfaceScene) {
  const firstCard = getStatusSurfaceCardsByMode(statusSurfaceScene, "default_stack")[0] ?? null;
  return firstCard?.kind ?? null;
}

export function getCompletionBadgeCountFromScene(statusSurfaceScene) {
  return Number(statusSurfaceScene?.completionBadgeCount ?? 0);
}

export function getCompletionBadgeCountWithFallback(statusSurfaceScene, fallbackCount = 0) {
  const sceneCount = getCompletionBadgeCountFromScene(statusSurfaceScene);
  return sceneCount > 0 ? sceneCount : Number(fallbackCount ?? 0);
}

export function shouldShowCompletionGlow(statusSurfaceScene) {
  return statusSurfaceScene?.showCompletionGlow === true;
}

export function buildStatusSurfaceCardKey(card, index = 0) {
  const requestId = card?.requestId ?? null;
  const sessionId = card?.sessionId ?? null;
  switch (card?.kind) {
    case "approval":
      return `approval:${requestId ?? sessionId ?? index}`;
    case "question":
      return `question:${requestId ?? sessionId ?? index}`;
    case "prompt_assist":
      return `attention:${sessionId ?? index}`;
    case "completion":
      return `completion:${sessionId ?? index}`;
    default:
      return `unknown:${index}`;
  }
}

export function buildStatusQueueSceneKeyItem(card, index = 0) {
  return {
    key: buildStatusSurfaceCardKey(card, index),
    kind: card?.kind === "approval" ? "approval" : "completion",
    sessionId: card?.sessionId ?? null,
    requestId: card?.requestId ?? null,
    isLive: card?.isLive !== false,
    isRemoving: card?.isRemoving === true,
    sortOrder: index,
  };
}

export function buildStatusSurfaceEntry(card, { canFocusTerminal = false, index = 0 } = {}) {
  const bodyText = String(card?.body ?? "");
  const title = String(card?.title ?? "");
  const meta = String(card?.meta ?? "");
  const sourceText = String(card?.sourceText ?? "");
  const statusText = String(card?.statusText ?? "");
  const requestId = card?.requestId ?? null;
  const sessionId = card?.sessionId ?? null;
  const answerOptions = Array.isArray(card?.answerOptions) ? card.answerOptions : [];
  const actionDisabled = card?.isLive === false ? "disabled" : "";
  const cardKeyBase = requestId ?? sessionId ?? index;
  const baseEntry = {
    kind: card?.kind ?? "completion",
    priority: STATUS_SURFACE_CARD_PRIORITY[card?.kind] ?? 0,
    sessionId,
    sortOrder: index,
  };

  if (card?.kind === "approval") {
    return {
      ...baseEntry,
      key: `approval:${cardKeyBase}`,
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(title)}">${escapeHtml(title)}</h3>
              <span class="badge">${escapeHtml(sourceText)}</span>
            </div>
            <div class="session-meta-line">
              <span>${escapeHtml(meta)}</span>
            </div>
          </div>
          <span class="status-pill">${escapeHtml(statusText)}</span>
        </div>
        <div class="chat-line assistant-line"><span class="chat-prefix">!</span><p>${escapeHtml(bodyText)}</p></div>
        ${
          requestId
            ? `<div class="pending-buttons">
                <button type="button" data-action="allow" data-request-id="${escapeHtml(requestId)}" ${actionDisabled}>Allow</button>
                <button type="button" data-action="deny" data-request-id="${escapeHtml(requestId)}" class="danger" ${actionDisabled}>Deny</button>
              </div>`
            : ""
        }
      `,
      row: {
        className: "session-card pending-card",
        status: "waitingapproval",
        focusable: "false",
        compact: "false",
        exiting: card?.isRemoving ? "true" : "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("approval", { body: bodyText })}px`,
      },
    };
  }

  if (card?.kind === "question") {
    return {
      ...baseEntry,
      key: `question:${cardKeyBase}`,
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(title)}">${escapeHtml(title)}</h3>
              <span class="badge">${escapeHtml(sourceText)}</span>
            </div>
            <div class="session-meta-line">
              <span>${escapeHtml(meta)}</span>
            </div>
          </div>
          <span class="status-pill">${escapeHtml(statusText)}</span>
        </div>
        <div class="chat-line assistant-line"><span class="chat-prefix">?</span><p>${escapeHtml(bodyText)}</p></div>
        ${buildQuestionOptionsHtml(answerOptions, requestId, actionDisabled)}
      `,
      row: {
        className: "session-card pending-card",
        status: "waitingquestion",
        focusable: "false",
        compact: "false",
        exiting: card?.isRemoving ? "true" : "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("question", {
          body: bodyText,
          options: answerOptions,
        })}px`,
      },
    };
  }

  if (card?.kind === "prompt_assist") {
    return {
      ...baseEntry,
      key: `attention:${cardKeyBase}`,
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(title)}">${escapeHtml(title)}</h3>
              <span class="badge">${escapeHtml(sourceText)}</span>
            </div>
            <div class="session-meta-line">
              <span>${escapeHtml(meta)}</span>
            </div>
          </div>
          <span class="status-pill">${escapeHtml(statusText)}</span>
        </div>
        <div class="chat-line assistant-line"><span class="chat-prefix">!</span><p>${escapeHtml(bodyText)}</p></div>
      `,
      row: {
        className: "session-card prompt-assist-card",
        status: "attention",
        focusable: canFocusTerminal ? "true" : "false",
        compact: "false",
        exiting: card?.isRemoving ? "true" : "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("attention", { body: bodyText })}px`,
      },
    };
  }

  if (card?.kind === "completion") {
    return {
      ...baseEntry,
      key: `completion:${cardKeyBase}`,
      html: `
        <div class="session-card-top">
          <div class="session-title-block">
            <div class="session-heading-row">
              <h3 title="${escapeHtml(title)}">${escapeHtml(title)}</h3>
              <span class="badge">${escapeHtml(sourceText)}</span>
            </div>
            <div class="session-meta-line">
              <span>${escapeHtml(meta)}</span>
            </div>
          </div>
          <span class="status-pill">${escapeHtml(statusText)}</span>
        </div>
        <div class="chat-line assistant-line"><span class="chat-prefix">$</span><p>${escapeHtml(bodyText)}</p></div>
      `,
      row: {
        className: "session-card",
        status: "idle",
        focusable: canFocusTerminal ? "true" : "false",
        compact: "false",
        exiting: card?.isRemoving ? "true" : "false",
        collapsedHeight: "52px",
        targetHeight: `${estimateMessageCardTargetHeight("completion", { body: bodyText })}px`,
        completion: "true",
      },
    };
  }

  return null;
}

export function buildStatusSurfaceEntries(
  statusSurfaceScene,
  { displayMode, canFocusTerminal = false, excludeKinds = [] } = {}
) {
  const excludedKinds = new Set(excludeKinds);
  return getStatusSurfaceCardsByMode(statusSurfaceScene, displayMode)
    .filter((card) => !excludedKinds.has(card?.kind))
    .map((card, index) => buildStatusSurfaceEntry(card, { canFocusTerminal, index }))
    .filter(Boolean);
}
