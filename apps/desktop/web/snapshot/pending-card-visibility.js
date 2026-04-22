import {
  getPendingPermissionCard,
  getPendingQuestionCard,
  setPendingPermissionCard,
  setPendingQuestionCard,
} from "../state-helpers.js";
import {
  getPendingPermissionPayloads,
  getPendingQuestionPayloads,
} from "../renderers/pending-snapshot-fallback.js";
import { getPendingSceneCards } from "../renderers/status-surface-scene.js";

function selectScenePendingPayload(rawItems, sceneCard) {
  if (!sceneCard) {
    return rawItems[0] ?? null;
  }

  const requestId = sceneCard.requestId ?? null;
  if (!requestId) {
    return rawItems[0] ?? null;
  }

  return rawItems.find((item) => item?.request_id === requestId) ?? rawItems[0] ?? null;
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

export function syncPendingCardVisibility(snapshot, statusSurfaceScene, uiState, timings, nowMs = Date.now()) {
  const previousPermission = getPendingPermissionCard(uiState);
  const previousQuestion = getPendingQuestionCard(uiState);
  const rawPendingPermissions = getPendingPermissionPayloads(snapshot);
  const rawPendingQuestions = getPendingQuestionPayloads(snapshot);
  const sceneCards = getPendingSceneCards(statusSurfaceScene);
  const currentPermission = selectScenePendingPayload(rawPendingPermissions, sceneCards.permissions[0]);
  const currentQuestion = selectScenePendingPayload(rawPendingQuestions, sceneCards.questions[0]);

  const nextPermission = resolveHeldCard(currentPermission, previousPermission, "permission", timings, nowMs);
  const nextQuestion = resolveHeldCard(currentQuestion, previousQuestion, "question", timings, nowMs);

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
  const rawPendingPermissions = getPendingPermissionPayloads(snapshot);
  const rawPendingQuestions = getPendingQuestionPayloads(snapshot);

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
