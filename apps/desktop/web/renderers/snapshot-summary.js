import { formatSource, formatStatus } from "../utils.js";
import { summarizeDefaultStatusSurfaceWithFallback } from "./status-surface-scene.js";

export function getSnapshotStatusKey(snapshot) {
  return String(snapshot?.status ?? "idle").toLowerCase();
}

export function getSnapshotPrimarySourceKey(snapshot) {
  return String(snapshot?.primary_source ?? "").toLowerCase();
}

export function getSnapshotStatusText(snapshot) {
  return formatStatus(snapshot?.status);
}

export function getSnapshotSourceText(snapshot) {
  return formatSource(snapshot?.primary_source);
}

export function buildSnapshotSummary(snapshot, statusSurfaceScene = null) {
  const defaultStatusSummary = summarizeDefaultStatusSurfaceWithFallback(statusSurfaceScene, snapshot);
  const activeSessionCount = Number(snapshot?.active_session_count ?? 0);
  const totalSessionCount = Number(snapshot?.total_session_count ?? 0);

  return {
    statusText: getSnapshotStatusText(snapshot),
    sourceText: getSnapshotSourceText(snapshot),
    activeCountText: String(activeSessionCount),
    totalCountText: String(totalSessionCount),
    totalLabelText: `${totalSessionCount} total`,
    approvalCountText: String(defaultStatusSummary.approvalCount || Number(snapshot?.pending_permission_count ?? 0)),
    questionCountText: String(defaultStatusSummary.questionCount || Number(snapshot?.pending_question_count ?? 0)),
    hasActiveSessions: activeSessionCount > 0,
    emptyState: activeSessionCount > 0 ? "false" : "true",
  };
}
