use std::time::Instant;

use chrono::Utc;
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

use super::PanelReminderState;

#[derive(Clone, Debug)]
pub(crate) enum StatusQueuePayload {
    Approval(PendingPermissionView),
    Question(PendingQuestionView),
    Completion(SessionSnapshotView),
}

#[derive(Clone, Debug)]
pub(crate) struct StatusQueueItem {
    pub(crate) key: String,
    pub(crate) session_id: String,
    pub(crate) sort_time: chrono::DateTime<Utc>,
    pub(crate) expires_at: Instant,
    pub(crate) is_live: bool,
    pub(crate) is_removing: bool,
    pub(crate) remove_after: Option<Instant>,
    pub(crate) payload: StatusQueuePayload,
}

#[derive(Clone)]
pub(crate) struct PendingPermissionCardState {
    pub(crate) request_id: String,
    pub(crate) payload: PendingPermissionView,
    pub(crate) started_at: Instant,
    pub(crate) last_seen_at: Instant,
    pub(crate) visible_until: Instant,
}

#[derive(Clone)]
pub(crate) struct PendingQuestionCardState {
    pub(crate) request_id: String,
    pub(crate) payload: PendingQuestionView,
    pub(crate) started_at: Instant,
    pub(crate) last_seen_at: Instant,
    pub(crate) visible_until: Instant,
}

#[derive(Clone)]
pub(crate) struct CompletionBadgeItem {
    pub(crate) session_id: String,
    pub(crate) completed_at: chrono::DateTime<Utc>,
    pub(crate) last_user_prompt: Option<String>,
    pub(crate) last_assistant_message: Option<String>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct StatusQueueSyncResult {
    pub(crate) added_approvals: usize,
    pub(crate) added_questions: usize,
    pub(crate) added_completions: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelHitAction {
    FocusSession,
    CycleDisplay,
    ToggleCompletionSound,
    ToggleMascot,
    OpenSettingsLocation,
    OpenReleasePage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum ExpandedSurface {
    #[default]
    Default,
    Status,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HoverTransition {
    Expand,
    Collapse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum PanelMascotBaseState {
    #[default]
    Idle,
    Running,
    Approval,
    Question,
    MessageBubble,
    Complete,
    Sleepy,
    WakeAngry,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelTransitionFrame {
    pub(crate) canvas_height: f64,
    pub(crate) visible_height: f64,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) shoulder_progress: f64,
    pub(crate) drop_progress: f64,
    pub(crate) cards_progress: f64,
}

impl PanelTransitionFrame {
    pub(crate) fn expanded(height: f64) -> Self {
        Self {
            canvas_height: height,
            visible_height: height,
            bar_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        }
    }

    pub(crate) fn collapsed(height: f64) -> Self {
        Self {
            canvas_height: height,
            visible_height: height,
            bar_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StatusSurfaceTransition {
    pub(crate) panel_transition: Option<bool>,
    pub(crate) surface_transition: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct PanelSnapshotSyncResult {
    pub(crate) displayed_snapshot: RuntimeSnapshot,
    pub(crate) reminder: PanelReminderState,
    pub(crate) panel_transition: Option<bool>,
    pub(crate) surface_transition: bool,
}

#[derive(Clone, Default)]
pub(crate) struct PanelState {
    pub(crate) expanded: bool,
    pub(crate) transitioning: bool,
    pub(crate) skip_next_close_card_exit: bool,
    pub(crate) last_raw_snapshot: Option<RuntimeSnapshot>,
    pub(crate) status_queue: Vec<StatusQueueItem>,
    pub(crate) completion_badge_items: Vec<CompletionBadgeItem>,
    pub(crate) pending_permission_card: Option<PendingPermissionCardState>,
    pub(crate) pending_question_card: Option<PendingQuestionCardState>,
    pub(crate) status_auto_expanded: bool,
    pub(crate) surface_mode: ExpandedSurface,
    pub(crate) pointer_inside_since: Option<Instant>,
    pub(crate) pointer_outside_since: Option<Instant>,
}
