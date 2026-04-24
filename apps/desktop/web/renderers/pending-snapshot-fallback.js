import { formatSource, stripMarkdownDisplay } from "../utils.js";

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

export function getPendingRequestId(pending) {
  return pending?.request_id ?? pending?.requestId ?? null;
}

export function getPendingSessionId(pending) {
  return pending?.session_id ?? pending?.sessionId ?? null;
}

export function getPendingSourceText(pending) {
  return formatSource(pending?.sourceText ?? pending?.source);
}

export function getPendingPermissionTitle(pending) {
  return pending?.tool_name ?? pending?.title ?? "Permission request";
}

export function getPendingQuestionTitle(pending) {
  return pending?.header ?? pending?.title ?? "Need your input";
}

export function getPendingPermissionBodyText(pending) {
  return stripMarkdownDisplay(pending?.tool_description ?? pending?.body ?? "This action needs your permission.");
}

export function getPendingQuestionBodyText(pending) {
  return stripMarkdownDisplay(pending?.text ?? pending?.body ?? "Need your input.");
}

export function getPendingQuestionOptions(pending) {
  if (Array.isArray(pending?.answerOptions)) {
    return pending.answerOptions;
  }
  if (Array.isArray(pending?.options)) {
    return pending.options;
  }
  return [];
}

export function mergePendingSnapshotItem(rawItem, sceneCard) {
  if (!rawItem && !sceneCard) return null;
  if (!rawItem) return sceneCard;
  if (!sceneCard) return rawItem;

  return {
    ...rawItem,
    ...sceneCard,
    request_id: rawItem.request_id ?? sceneCard.requestId ?? null,
    requestId: sceneCard.requestId ?? rawItem.request_id ?? null,
    session_id: rawItem.session_id ?? sceneCard.sessionId ?? null,
    sessionId: sceneCard.sessionId ?? rawItem.session_id ?? null,
    source: rawItem.source ?? sceneCard.sourceText ?? null,
    sourceText: sceneCard.sourceText ?? rawItem.sourceText ?? rawItem.source ?? null,
    title: sceneCard.title ?? rawItem.title ?? rawItem.header ?? rawItem.tool_name ?? null,
    body: sceneCard.body ?? rawItem.body ?? rawItem.text ?? rawItem.tool_description ?? null,
    answerOptions: Array.isArray(sceneCard.answerOptions)
      ? sceneCard.answerOptions
      : Array.isArray(rawItem.answerOptions)
        ? rawItem.answerOptions
        : Array.isArray(rawItem.options)
          ? rawItem.options
          : [],
  };
}

export function mergePendingSnapshotItems(rawItems, sceneCards) {
  if (!rawItems.length) {
    return sceneCards;
  }
  if (!sceneCards.length) {
    return rawItems;
  }

  const sceneByRequestId = new Map(sceneCards.filter((card) => card?.requestId).map((card) => [card.requestId, card]));
  return rawItems.map((item) => mergePendingSnapshotItem(item, sceneByRequestId.get(item?.request_id) ?? null));
}
