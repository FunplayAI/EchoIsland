export function wait(ms) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

export function nextFrame() {
  return new Promise((resolve) => window.requestAnimationFrame(() => resolve()));
}

export function setLog(target, message, append = false) {
  if (!target) return;
  if (append) {
    target.textContent = `${target.textContent}\n${message}`.trim();
    target.scrollTop = target.scrollHeight;
    return;
  }
  target.textContent = message;
}

export function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function stripMarkdownDisplay(value) {
  return String(value ?? "")
    .replace(/\r\n/g, "\n")
    .replace(/```[\s\S]*?```/g, (block) =>
      block
        .replace(/^```[^\n]*\n?/, "")
        .replace(/\n?```$/, "")
        .trim()
    )
    .replace(/`([^`]+)`/g, "$1")
    .replace(/!\[([^\]]*)\]\([^)]+\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^>\s?/gm, "")
    .replace(/^#{1,6}\s*/gm, "")
    .replace(/^\s*[-*+]\s+/gm, "")
    .replace(/^\s*\d+\.\s+/gm, "")
    .replace(/[*_~]+/g, "")
    .replace(/\|/g, " ")
    .replace(/\n+/g, " ")
    .replace(/\s{2,}/g, " ")
    .trim();
}

export function shortSessionId(sessionId) {
  return String(sessionId ?? "").replace(/^[^-]+-/, "").slice(0, 6) || "------";
}

export function compactTitle(value, maxLength = 22) {
  const text = String(value ?? "").trim();
  if (text.length <= maxLength) return text;
  const headLength = Math.ceil((maxLength - 1) * 0.58);
  const tailLength = maxLength - 1 - headLength;
  return `${text.slice(0, headLength)}…${text.slice(-tailLength)}`;
}

export function displayProjectName(session) {
  const rawTitle = session.project_name ?? session.cwd ?? "Session";
  const parts = String(rawTitle)
    .trim()
    .split(/[\\/]/)
    .filter(Boolean);
  return parts.at(-1)?.replace(/^[A-Za-z]:$/, "") || "Session";
}

export function formatSource(source) {
  const key = String(source ?? "").toLowerCase();
  if (!key) return "Unknown";
  const map = {
    claude: "Claude",
    codex: "Codex",
    cursor: "Cursor",
    gemini: "Gemini",
    copilot: "Copilot",
    qoder: "Qoder",
    codebuddy: "CodeBuddy",
    opencode: "OpenCode",
    openclaw: "OpenClaw",
    droid: "Factory",
  };
  return map[key] ?? key.charAt(0).toUpperCase() + key.slice(1);
}

export function sessionTitle(session) {
  const projectName = displayProjectName(session);
  if (projectName !== "Session") return projectName;
  return `${formatSource(session.source)} ${shortSessionId(session.session_id)}`;
}

export function formatStatus(status) {
  const key = String(status ?? "").toLowerCase();
  const map = {
    running: "Running",
    processing: "Thinking",
    waitingapproval: "Approval",
    waitingquestion: "Question",
    idle: "Idle",
  };
  return map[key] ?? status ?? "Idle";
}

export function statusPriority(status) {
  switch (String(status ?? "").toLowerCase()) {
    case "waitingapproval":
    case "waitingquestion":
      return 0;
    case "running":
      return 1;
    case "processing":
      return 2;
    case "idle":
    default:
      return 3;
  }
}

export function timeAgo(value) {
  if (!value) return "now";
  const diffMs = Date.now() - new Date(value).getTime();
  if (!Number.isFinite(diffMs) || diffMs <= 0) return "now";
  const minutes = Math.floor(diffMs / 60000);
  if (minutes < 1) return "now";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  return `${days}d`;
}

export function normalizeStatus(status) {
  return String(status ?? "").toLowerCase();
}

export function isLongIdleSession(session) {
  if (normalizeStatus(session?.status) !== "idle") return false;
  const lastActivity = new Date(session?.last_activity ?? 0).getTime();
  if (!Number.isFinite(lastActivity) || lastActivity <= 0) return false;
  return Date.now() - lastActivity > 15 * 60 * 1000;
}

export function toolTone(tool) {
  switch (String(tool ?? "").toLowerCase()) {
    case "bash":
      return "bash";
    case "edit":
    case "write":
      return "edit";
    case "read":
      return "read";
    case "grep":
    case "glob":
      return "search";
    case "agent":
      return "agent";
    default:
      return "default";
  }
}
