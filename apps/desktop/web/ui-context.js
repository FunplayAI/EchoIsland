export const invoke = async (cmd, args = {}) => window.__TAURI__.core.invoke(cmd, args);

export const elements = {
  island: document.querySelector("#island"),
  islandBar: document.querySelector("#islandBar"),
  islandPanel: document.querySelector(".island-panel"),
  settingsBtn: document.querySelector("#settingsBtn"),
  quitBtn: document.querySelector("#quitBtn"),
  panelSessionsOnly: document.querySelector(".panel-sessions-only"),
  mascotShell: document.querySelector("#mascotShell"),
  mascotCanvas: document.querySelector("#mascotCanvas"),
  mascotCompletionBadge: document.querySelector("#mascotCompletionBadge"),
  headline: document.querySelector("#headline"),
  primaryStatus: document.querySelector("#primaryStatus"),
  primarySource: document.querySelector("#primarySource"),
  activeCount: document.querySelector("#activeCount"),
  activeCountExpanded: document.querySelector("#activeCountExpanded"),
  modeHint: document.querySelector("#modeHint"),
  totalCountCompact: document.querySelector("#totalCountCompact"),
  totalCountExpanded: document.querySelector("#totalCountExpanded"),
  totalCount: document.querySelector("#totalCount"),
  totalCountLabel: document.querySelector("#totalCountLabel"),
  permissionCount: document.querySelector("#permissionCount"),
  questionCount: document.querySelector("#questionCount"),
  sessionList: document.querySelector("#sessionList"),
  eventLog: document.querySelector("#eventLog"),
  ipcAddrInline: document.querySelector("#ipcAddrInline"),
  pendingActions: document.querySelector("#pendingActions"),
  pendingSummary: document.querySelector("#pendingSummary"),
  statusChip: document.querySelector("#statusChip"),
};

export const uiState = {
  window: {
    expanded: null,
    transitioning: false,
    panelHeight: 560,
  },
  mascot: {
    source: "claude",
    state: "idle",
  },
  snapshot: {
    lastSnapshot: null,
    lastRawSnapshot: null,
    codexStatus: null,
    claudeStatus: null,
    openclawStatus: null,
    platformCapabilities: null,
    platformPaths: null,
  },
  surface: {
    mode: "default",
    completionSessionIds: [],
    completionBadgeItems: [],
    pendingPermissionCard: null,
    pendingQuestionCard: null,
    statusQueueItems: [],
  },
  timers: {
    statusQueueRefresh: null,
    hoverExpand: null,
    hoverCollapse: null,
    cardReveal: null,
  },
  interaction: {
    pointerInsideBar: false,
    pointerInsidePanel: false,
    panelHasInteractiveFocus: false,
    panelPointerActive: false,
    cardExitInProgress: false,
    suppressHoverExpandUntil: 0,
  },
};

export const timings = {
  morphTransitionMs: 270,
  expandTransitionMs: 270,
  shoulderTransitionMs: 120,
  heightTransitionMs: 270,
  hoverExpandDelayMs: 500,
  hoverCollapseDelayMs: 500,
  cardRevealStaggerMs: 70,
  cardRevealDurationMs: 320,
  cardExitStaggerMs: 38,
  cardExitDurationMs: 220,
  statusQueue: {
    completionMs: 10000,
    approvalMs: 30000,
    exitMinMs: 220,
    exitExtraMs: 80,
    refreshLeadMs: 12,
    refreshMinDelayMs: 16,
    autoCloseHoverSuppressMs: 900,
  },
  pendingCard: {
    minVisibleMs: 2200,
    releaseGraceMs: 1600,
  },
  interaction: {
    compactActionHoverSuppressMs: 900,
  },
};

export const KEEP_OPEN_SELECTOR =
  'button, input, textarea, select, [contenteditable="true"], [data-keep-open="true"]';
