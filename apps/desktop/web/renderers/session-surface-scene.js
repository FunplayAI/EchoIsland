import { escapeHtml, toolTone } from "../utils.js";

function estimateSceneCardHeight(card) {
  if (card?.compact) {
    return 58;
  }

  let height = 46;
  if (card?.userLine) {
    height += 18;
  }
  if (card?.assistantLine) {
    height += 30;
  }
  if (card?.toolName) {
    height += 26;
  }
  return height;
}

export function getSessionSurfaceCards(sessionSurfaceScene) {
  return Array.isArray(sessionSurfaceScene?.cards) ? sessionSurfaceScene.cards : [];
}

export function getSessionSurfaceSessionIds(sessionSurfaceScene) {
  return getSessionSurfaceCards(sessionSurfaceScene)
    .map((card) => card?.sessionId)
    .filter(Boolean);
}

export function buildSessionSurfaceCardKey(card, index = 0) {
  return `session:${card?.sessionId ?? index}`;
}

export function buildSessionSurfaceEntry(card, { canFocusTerminal = false, index = 0 } = {}) {
  const estimatedHeight = estimateSceneCardHeight(card);
  const revealTargetHeight = estimatedHeight + (card?.compact ? 8 : 20);
  const revealCollapsedHeight = Math.max(
    34,
    Math.min(card?.compact ? 52 : 64, Math.round(estimatedHeight * (card?.compact ? 0.76 : 0.58)))
  );
  const metaItems = Array.isArray(card?.metaItems) ? card.metaItems : [];
  const toolDescription = String(card?.toolDescription ?? "working");

  return {
    key: buildSessionSurfaceCardKey(card, index),
    kind: "session",
    sessionId: card?.sessionId ?? null,
    html: `
      <div class="session-card-top">
        <div class="session-title-block">
          <div class="session-heading-row">
            <h3 title="${escapeHtml(card?.title ?? "")}">${escapeHtml(card?.displayTitle ?? card?.title ?? "")}</h3>
            <span class="badge">${escapeHtml(card?.sourceText ?? "")}</span>
          </div>
          <div class="session-meta-line">
            ${metaItems.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}
          </div>
        </div>
        <span class="status-pill">${escapeHtml(card?.statusText ?? "")}</span>
      </div>
      ${
        card?.compact
          ? ""
          : `
      ${
        card?.userLine
          ? `<div class="chat-line user-line"><span class="chat-prefix">&gt;</span><p>${escapeHtml(
              card.userLine
            )}</p></div>`
          : ""
      }
      ${
        card?.assistantLine
          ? `<div class="chat-line assistant-line"><span class="chat-prefix">$</span><p>${escapeHtml(
              card.assistantLine
            )}</p></div>`
          : ""
      }
      ${
        card?.toolName
          ? `<div class="live-tool" data-tone="${escapeHtml(toolTone(card.toolName))}">
              <span class="live-tool-name">${escapeHtml(card.toolName)}</span>
              <span class="live-tool-desc">${escapeHtml(toolDescription)}</span>
            </div>`
          : ""
      }
      `
      }
    `,
    row: {
      className: "session-card",
      status: String(card?.statusKey ?? "idle"),
      focusable: canFocusTerminal ? "true" : "false",
      compact: card?.compact ? "true" : "false",
      completion: card?.completion ? "true" : undefined,
      collapsedHeight: `${revealCollapsedHeight}px`,
      targetHeight: `${revealTargetHeight}px`,
      minHeight: `${estimatedHeight}px`,
      height: card?.compact ? `${estimatedHeight}px` : "auto",
    },
  };
}

export function buildSessionSurfaceEntries(sessionSurfaceScene, { canFocusTerminal = false } = {}) {
  return getSessionSurfaceCards(sessionSurfaceScene).map((card, index) =>
    buildSessionSurfaceEntry(card, { canFocusTerminal, index })
  );
}
