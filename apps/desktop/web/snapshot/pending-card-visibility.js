import {
  getPendingPermissionCard,
  getPendingQuestionCard,
  setPendingPermissionCard,
  setPendingQuestionCard,
} from "../state-helpers.js";

function getRawPendingPermissions(snapshot) {
  const items = Array.isArray(snapshot?.pending_permissions) ? snapshot.pending_permissions : [];
  if (items.length) return items;
  return snapshot?.pending_permission ? [snapshot.pending_permission] : [];
}

function getRawPendingQuestions(snapshot) {
  const items = Array.isArray(snapshot?.pending_questions) ? snapshot.pending_questions : [];
  if (items.length) return items;
  return snapshot?.pending_question ? [snapshot.pending_question] : [];
}

function buildPendingCard(type, payload, nowMs, previousCard, minVisibleMs) {
  const requestId = payload?.request_id ?? null;
  if (!requestId) return null;

  const requestKey = `${type}:${requestId}`;
  const startedAt = previousCard?.requestKey === requestKey ? previousCard.startedAt ?? nowMs : nowMs;

  return {
    type,
    requestId,
    requestKey,
    payload,
    isLive: true,
    startedAt,
    lastSeenAt: nowMs,
    visibleUntil: Math.max(previousCard?.visibleUntil ?? 0, startedAt + minVisibleMs),
  };
}

function resolveHeldCard(currentPayload, previousCard, type, timings, nowMs) {
  if (currentPayload?.request_id) {
    return buildPendingCard(type, currentPayload, nowMs, previousCard, timings.pendingCard.minVisibleMs);
  }

  if (!previousCard) {
    return null;
  }

  const keepVisibleUntil = Math.max(
    previousCard.visibleUntil ?? 0,
    (previousCard.lastSeenAt ?? previousCard.startedAt ?? nowMs) + timings.pendingCard.releaseGraceMs
  );
  if (nowMs > keepVisibleUntil) {
    return null;
  }

  return {
    ...previousCard,
    isLive: false,
    visibleUntil: keepVisibleUntil,
  };
}

export function syncPendingCardVisibility(snapshot, uiState, timings, nowMs = Date.now()) {
  const previousPermission = getPendingPermissionCard(uiState);
  const previousQuestion = getPendingQuestionCard(uiState);
  const rawPendingPermissions = getRawPendingPermissions(snapshot);
  const rawPendingQuestions = getRawPendingQuestions(snapshot);

  const nextPermission = resolveHeldCard(rawPendingPermissions[0], previousPermission, "permission", timings, nowMs);
  const nextQuestion = resolveHeldCard(rawPendingQuestions[0], previousQuestion, "question", timings, nowMs);

  setPendingPermissionCard(uiState, nextPermission);
  setPendingQuestionCard(uiState, nextQuestion);

  return {
    permissionStarted:
      Boolean(nextPermission?.requestKey) && nextPermission.requestKey !== previousPermission?.requestKey,
    questionStarted: Boolean(nextQuestion?.requestKey) && nextQuestion.requestKey !== previousQuestion?.requestKey,
    permissionVisible: Boolean(nextPermission),
    questionVisible: Boolean(nextQuestion),
  };
}

export function applyPendingCardsToSnapshot(snapshot, uiState) {
  const pendingPermissionCard = getPendingPermissionCard(uiState);
  const pendingQuestionCard = getPendingQuestionCard(uiState);
  const rawPendingPermissions = getRawPendingPermissions(snapshot);
  const rawPendingQuestions = getRawPendingQuestions(snapshot);

  let pendingPermissions = rawPendingPermissions;
  if (pendingPermissionCard) {
    const heldPermission = {
      ...pendingPermissionCard.payload,
      display_held: !pendingPermissionCard.isLive,
    };
    pendingPermissions = [heldPermission, ...rawPendingPermissions.filter((item) => item?.request_id !== heldPermission.request_id)];
  }

  let pendingQuestions = rawPendingQuestions;
  if (pendingQuestionCard) {
    const heldQuestion = {
      ...pendingQuestionCard.payload,
      display_held: !pendingQuestionCard.isLive,
    };
    pendingQuestions = [heldQuestion, ...rawPendingQuestions.filter((item) => item?.request_id !== heldQuestion.request_id)];
  }

  if (!pendingPermissions.length && !pendingQuestions.length) {
    return snapshot;
  }

  return {
    ...snapshot,
    pending_permission_count: pendingPermissions.length,
    pending_question_count: pendingQuestions.length,
    pending_permission: pendingPermissions[0] ?? null,
    pending_question: pendingQuestions[0] ?? null,
    pending_permissions: pendingPermissions,
    pending_questions: pendingQuestions,
  };
}
