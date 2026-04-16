import { setLog } from "../utils.js";

export function createPendingActions({
  desktopApi,
  eventLog,
  pendingActions,
  refreshSnapshot,
}) {
  async function handlePendingAction(event) {
    const button = event.target.closest("button[data-action]");
    if (!button) return;

    const action = button.dataset.action;
    const requestId = button.dataset.requestId;
    const answer = button.dataset.answer;

    try {
      if (action === "allow") {
        await desktopApi.approvePermission(requestId);
        setLog(eventLog, `Approved permission ${requestId}`, true);
      } else if (action === "deny") {
        await desktopApi.denyPermission(requestId);
        setLog(eventLog, `Denied permission ${requestId}`, true);
      } else if (action === "answer") {
        await desktopApi.answerQuestion(requestId, answer);
        setLog(eventLog, `Answered question ${requestId} with ${answer}`, true);
      } else if (action === "answer-from-input") {
        const input = pendingActions?.querySelector("#questionAnswerInput");
        const inputValue = input?.value?.trim();
        if (!inputValue) {
          setLog(eventLog, "Question answer cannot be empty.", true);
          return;
        }
        await desktopApi.answerQuestion(requestId, inputValue);
        setLog(eventLog, `Answered question ${requestId} with ${inputValue}`, true);
      } else if (action === "skip-question") {
        await desktopApi.skipQuestion(requestId);
        setLog(eventLog, `Skipped question ${requestId}`, true);
      }
      await refreshSnapshot();
    } catch (error) {
      setLog(eventLog, `Action failed: ${error}`, true);
    }
  }

  return { handlePendingAction };
}
