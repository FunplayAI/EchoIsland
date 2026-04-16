function resolveShell(capabilities) {
  const explicitShell = globalThis.__CODEISLAND_SHELL__;

  if (typeof explicitShell === "string" && explicitShell.trim()) {
    return explicitShell.trim().toLowerCase();
  }

  return "webview";
}

export function applyPlatformTheme(capabilities, { island } = {}) {
  const root = document.documentElement;
  const platform = String(capabilities?.platform ?? "unknown").toLowerCase();
  const backend = String(capabilities?.platformBackend ?? "unknown").toLowerCase();
  const shell = resolveShell(capabilities);

  root.dataset.platform = platform;
  root.dataset.platformBackend = backend;
  root.dataset.shell = shell;

  if (island) {
    island.dataset.platform = platform;
    island.dataset.platformBackend = backend;
    island.dataset.shell = shell;
  }
}
