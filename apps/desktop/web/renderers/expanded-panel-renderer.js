import { renderSessionListHtml } from "./session-list-html.js";

export function renderExpandedPanelHtml(snapshot, uiState) {
  return `
    <section class="panel panel-main panel-sessions-only" data-content-layer="shared">
      <div class="session-list" data-content-slot="session-list">
        ${renderSessionListHtml(snapshot, uiState)}
      </div>
    </section>
  `;
}

export function renderExpandedPanel(snapshot, { islandPanel, sessionList, uiState }) {
  if (islandPanel) {
    islandPanel.dataset.contentRenderer = "shared-html";
    islandPanel.dataset.contentPlatform = document.documentElement.dataset.platform ?? "unknown";
  }

  if (sessionList) {
    sessionList.innerHTML = renderSessionListHtml(snapshot, uiState);
  }
}
