import { getCompletionPreviewText, hasPromptAssistSessions } from "./prompt-assist-policy.js";
import {
  getCompletionDisplaySessions,
  getStatusQueueApprovalCount,
  isCompletionSurfaceActive,
} from "./surface-state.js";
import { getStatusSurfaceScene, getSurfaceMode } from "../state-helpers.js";
import { getSurfaceSceneHeadline } from "./surface-scene.js";
import {
  summarizeDefaultStatusSurfaceWithFallback,
  summarizeSnapshotHeadline,
} from "./status-surface-scene.js";

export function updateHeadline(snapshot, { headline, uiState }) {
  if (!headline) return;
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const sharedHeadline = getSurfaceSceneHeadline(uiState);

  if (getSurfaceMode(uiState) === "status") {
    const approvalCount = getStatusQueueApprovalCount(uiState);
    if (approvalCount > 0) {
      headline.textContent = approvalCount > 1 ? "Approvals waiting" : "Approval waiting";
      return;
    }
  }
  const defaultStatusSummary = summarizeDefaultStatusSurfaceWithFallback(statusSurfaceScene, snapshot);
  if (defaultStatusSummary.approvalCount > 0) {
    headline.textContent = defaultStatusSummary.approvalCount > 1 ? "Approvals needed" : "Approval needed";
    return;
  }
  if (defaultStatusSummary.questionCount > 0) {
    headline.textContent = defaultStatusSummary.questionCount > 1 ? "Questions waiting" : "Question waiting";
    return;
  }
  if (defaultStatusSummary.promptAssistCount > 0 || hasPromptAssistSessions(snapshot, uiState)) {
    headline.textContent = "Codex needs attention";
    return;
  }
  if (isCompletionSurfaceActive(uiState)) {
    const displayedSessions = getCompletionDisplaySessions(snapshot, uiState);
    const count = displayedSessions.length;
    headline.textContent =
      count > 1 ? `${count} tasks complete` : displayedSessions[0] ? getCompletionPreviewText(displayedSessions[0]) : "Task complete";
    return;
  }
  if (sharedHeadline?.text) {
    headline.textContent = sharedHeadline.text;
    return;
  }
  headline.textContent = summarizeSnapshotHeadline(snapshot);
}
