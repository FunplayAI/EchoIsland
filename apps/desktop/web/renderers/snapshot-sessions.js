import { getSessionId } from "./session-snapshot-fallback.js";

export function getSnapshotSessions(snapshot) {
  return Array.isArray(snapshot?.sessions) ? snapshot.sessions : [];
}

export function hasSnapshotSessionSource(snapshot, source) {
  const sourceKey = String(source ?? "").toLowerCase();
  if (!sourceKey) return false;
  return getSnapshotSessions(snapshot).some((session) => String(session?.source ?? "").toLowerCase() === sourceKey);
}

export function indexSnapshotSessions(snapshot) {
  return new Map(getSnapshotSessions(snapshot).map((session) => [getSessionId(session), session]));
}

export function findSnapshotSessionById(snapshot, sessionId) {
  if (!sessionId) return null;
  return indexSnapshotSessions(snapshot).get(sessionId) ?? null;
}
