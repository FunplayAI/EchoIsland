import { normalizeStatus } from "../utils.js";
import { wasSessionRecentlyUpdated } from "../renderers.js";

export function detectCompletedSessions(previousSnapshot, snapshot) {
  if (!previousSnapshot) return [];

  const previousById = new Map(previousSnapshot.sessions.map((session) => [session.session_id, session]));
  const completed = [];

  for (const session of snapshot.sessions) {
    const previous = previousById.get(session.session_id);
    if (!previous) continue;

    const previousStatus = normalizeStatus(previous.status);
    const currentStatus = normalizeStatus(session.status);
    const becameIdleFromActive =
      currentStatus === "idle" && (previousStatus === "processing" || previousStatus === "running");
    const idleMessageUpdated =
      currentStatus === "idle" &&
      previousStatus === "idle" &&
      wasSessionRecentlyUpdated(session) &&
      (session.last_assistant_message ?? "") !== (previous.last_assistant_message ?? "") &&
      !!(session.last_assistant_message ?? "").trim();

    if (becameIdleFromActive || idleMessageUpdated) {
      completed.push(session.session_id);
    }
  }

  return completed;
}
