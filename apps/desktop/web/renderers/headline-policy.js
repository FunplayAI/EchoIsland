import { getCompletionPreviewText, hasPromptAssistSessions } from "./prompt-assist-policy.js";
import { getCompletionDisplaySessions, isCompletionSurfaceActive } from "./surface-state.js";
import { getSurfaceMode, getStatusQueueItems } from "../state-helpers.js";

function summarizeHeadline(snapshot) {
  if (snapshot.pending_permission_count > 0) {
    return snapshot.pending_permission_count > 1 ? "Approvals needed" : "Approval needed";
  }
  if (snapshot.pending_question_count > 0) {
    return snapshot.pending_question_count > 1 ? "Questions waiting" : "Question waiting";
  }
  if (snapshot.active_session_count > 0) {
    return `${snapshot.active_session_count} active task${snapshot.active_session_count > 1 ? "s" : ""}`;
  }
  return "No active tasks";
}

export function updateHeadline(snapshot, { headline, uiState }) {
  if (!headline) return;
  if (getSurfaceMode(uiState) === "status") {
    const statusQueueItems = getStatusQueueItems(uiState);
    const approvalCount = statusQueueItems.filter((item) => item.kind === "approval").length;
    if (approvalCount > 0) {
      headline.textContent = approvalCount > 1 ? "Approvals waiting" : "Approval waiting";
      return;
    }
  }
  if (snapshot.pending_permission_count > 0 || snapshot.pending_question_count > 0) {
    headline.textContent = summarizeHeadline(snapshot);
    return;
  }
  if (hasPromptAssistSessions(snapshot, uiState)) {
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
  headline.textContent = summarizeHeadline(snapshot);
}
