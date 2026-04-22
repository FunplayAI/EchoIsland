export function getPendingPermissionPayloads(snapshot) {
  const items = Array.isArray(snapshot?.pending_permissions) ? snapshot.pending_permissions : [];
  if (items.length) return items;
  return snapshot?.pending_permission ? [snapshot.pending_permission] : [];
}

export function getPendingQuestionPayloads(snapshot) {
  const items = Array.isArray(snapshot?.pending_questions) ? snapshot.pending_questions : [];
  if (items.length) return items;
  return snapshot?.pending_question ? [snapshot.pending_question] : [];
}

export function getFirstPendingPermissionSessionId(snapshot) {
  return getPendingPermissionPayloads(snapshot)[0]?.session_id ?? null;
}

export function getFirstPendingQuestionSessionId(snapshot) {
  return getPendingQuestionPayloads(snapshot)[0]?.session_id ?? null;
}

export function getLivePendingSessionIdsFromSnapshot(snapshot) {
  return new Set(
    [...getPendingPermissionPayloads(snapshot), ...getPendingQuestionPayloads(snapshot)]
      .map((item) => item?.session_id)
      .filter((sessionId) => typeof sessionId === "string" && sessionId.trim().length > 0)
  );
}
