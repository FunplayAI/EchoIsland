import { isLongIdleSession } from "../utils.js";

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

export function estimateExpandedPanelHeight({ islandPanel, sessionList }) {
  if (!islandPanel || !sessionList || !sessionList.children.length) {
    return 132;
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

  const cardCount = sessionList.children.length;
  const minimumHeight = cardCount <= 1 ? 124 : 160;
  return Math.max(minimumHeight, topOffset + panelHeight);
}
