import { isLongIdleSession } from "../utils.js";

const MESSAGE_CARD_TEXT_WIDTH = 248;
const MESSAGE_CARD_HEADER_HEIGHT = 62;
const MESSAGE_CARD_TEXT_LINE_HEIGHT = 13;
const MESSAGE_CARD_ACTION_GAP = 12;
const MESSAGE_CARD_BUTTON_HEIGHT = 32;
const MESSAGE_CARD_INPUT_HEIGHT = 38;
const MESSAGE_CARD_BUTTON_ROW_GAP = 8;
const MESSAGE_CARD_BUTTON_HORIZONTAL_GAP = 8;
const MESSAGE_CARD_BUTTONS_WIDTH = 255;

export function estimateCardHeight(session) {
  if (isLongIdleSession(session)) {
    return 58;
  }

  let height = 46;
  if (session.last_user_prompt) {
    height += 18;
  }
  if (session.last_assistant_message ?? session.tool_description) {
    height += 30;
  }
  if (session.current_tool) {
    height += 26;
  }
  return height;
}

function estimateTextWidth(text, fontSize = 10) {
  return Array.from(String(text ?? "")).reduce((width, char) => {
    let factor = 0.6;
    if (/\s/.test(char)) {
      factor = 0.34;
    } else if (/[\u0000-\u007f]/.test(char)) {
      if (/[A-Z]/.test(char)) {
        factor = 0.66;
      } else if (/[`~!@#$%^&*()_\-+=[\]{}\\|;:'",.<>/?]/.test(char)) {
        factor = 0.42;
      }
    } else {
      factor = 1;
    }
    return width + factor * fontSize;
  }, 0);
}

function estimateChatLineCount(text, width = MESSAGE_CARD_TEXT_WIDTH, maxLines = 2) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return 0;

  const lineCount = normalized
    .split(/\n+/)
    .filter(Boolean)
    .reduce((count, line) => count + Math.max(1, Math.ceil(estimateTextWidth(line, 10) / Math.max(width, 1))), 0);

  return Math.min(Math.max(lineCount, 1), Math.max(maxLines, 1));
}

function estimateChatHeight(text, maxLines = 2) {
  return estimateChatLineCount(text, MESSAGE_CARD_TEXT_WIDTH, maxLines) * MESSAGE_CARD_TEXT_LINE_HEIGHT;
}

function estimateButtonWidth(text) {
  return Math.max(68, Math.ceil(estimateTextWidth(text, 12) + 24));
}

function estimateButtonRows(labels) {
  let rows = 1;
  let usedWidth = 0;
  for (const label of labels) {
    const width = estimateButtonWidth(label);
    if (usedWidth > 0 && usedWidth + MESSAGE_CARD_BUTTON_HORIZONTAL_GAP + width > MESSAGE_CARD_BUTTONS_WIDTH) {
      rows += 1;
      usedWidth = width;
      continue;
    }
    usedWidth += usedWidth > 0 ? MESSAGE_CARD_BUTTON_HORIZONTAL_GAP + width : width;
  }
  return rows;
}

function estimateButtonGroupHeight(labels) {
  if (!labels.length) return 0;
  const rows = estimateButtonRows(labels);
  return rows * MESSAGE_CARD_BUTTON_HEIGHT + Math.max(0, rows - 1) * MESSAGE_CARD_BUTTON_ROW_GAP;
}

export function estimateMessageCardTargetHeight(kind, { body = "", options = [] } = {}) {
  const bodyHeight = estimateChatHeight(body, 2);

  switch (kind) {
    case "approval":
      return Math.min(120, Math.max(106, MESSAGE_CARD_HEADER_HEIGHT + bodyHeight + MESSAGE_CARD_ACTION_GAP + MESSAGE_CARD_BUTTON_HEIGHT));
    case "question": {
      const actionsHeight = options.length
        ? estimateButtonGroupHeight(options)
        : MESSAGE_CARD_INPUT_HEIGHT + MESSAGE_CARD_BUTTON_ROW_GAP + MESSAGE_CARD_BUTTON_HEIGHT;
      return Math.min(144, Math.max(112, MESSAGE_CARD_HEADER_HEIGHT + bodyHeight + MESSAGE_CARD_ACTION_GAP + actionsHeight));
    }
    case "attention":
      return Math.min(92, Math.max(76, MESSAGE_CARD_HEADER_HEIGHT + bodyHeight));
    case "completion":
    default:
      return Math.min(92, Math.max(76, MESSAGE_CARD_HEADER_HEIGHT + bodyHeight));
  }
}

export function estimateExpandedPanelHeight({ islandPanel, sessionList, settingsPanel, uiState }) {
  const surfaceMode = uiState?.surface?.mode ?? "default";
  const hasSessionContent = Boolean(sessionList?.children.length);
  const hasSettingsContent = Boolean(settingsPanel && !settingsPanel.hidden);
  if (!islandPanel || (!hasSessionContent && !hasSettingsContent)) {
    return surfaceMode === "settings" ? 176 : 132;
  }

  const expandedWidth =
    Number.parseFloat(window.getComputedStyle(document.documentElement).getPropertyValue("--expanded-bar-width")) || 364;
  const topOffset = Number.parseFloat(window.getComputedStyle(islandPanel).top) || 32;
  const clone = islandPanel.cloneNode(true);
  clone.style.display = "grid";
  clone.style.position = "fixed";
  clone.style.left = "-10000px";
  clone.style.top = "0";
  clone.style.bottom = "auto";
  clone.style.width = `${expandedWidth}px`;
  clone.style.transform = "none";
  clone.style.visibility = "hidden";
  clone.style.pointerEvents = "none";
  clone.style.opacity = "0";
  clone.style.zIndex = "-1";

  document.body.appendChild(clone);
  const panelHeight = Math.ceil(clone.getBoundingClientRect().height);
  clone.remove();

  const cardCount = sessionList?.children.length ?? 0;
  const minimumHeight = surfaceMode === "settings" ? 176 : cardCount <= 1 ? 124 : 160;
  return Math.max(minimumHeight, topOffset + panelHeight);
}
