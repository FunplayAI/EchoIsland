use super::*;

#[derive(Clone, Copy)]
pub(super) struct NativePanelHandles {
    pub(super) panel: usize,
    pub(super) content_view: usize,
    pub(super) left_shoulder: usize,
    pub(super) right_shoulder: usize,
    pub(super) pill_view: usize,
    pub(super) expanded_container: usize,
    pub(super) cards_container: usize,
    pub(super) completion_glow: usize,
    pub(super) top_highlight: usize,
    pub(super) body_separator: usize,
    pub(super) settings_button: usize,
    pub(super) quit_button: usize,
    pub(super) mascot_shell: usize,
    pub(super) mascot_body: usize,
    pub(super) mascot_left_eye: usize,
    pub(super) mascot_right_eye: usize,
    pub(super) mascot_mouth: usize,
    pub(super) mascot_bubble: usize,
    pub(super) mascot_sleep_label: usize,
    pub(super) mascot_completion_badge: usize,
    pub(super) mascot_completion_badge_label: usize,
    pub(super) headline: usize,
    pub(super) active_count_clip: usize,
    pub(super) active_count: usize,
    pub(super) active_count_next: usize,
    pub(super) slash: usize,
    pub(super) total_count: usize,
}

#[derive(Clone, Copy)]
pub(super) struct CardAnimationLayout {
    pub(super) frame: NSRect,
    pub(super) collapsed_height: f64,
}

#[derive(Clone, Copy)]
pub(super) struct NativePanelTransitionFrame {
    pub(super) canvas_height: f64,
    pub(super) visible_height: f64,
    pub(super) bar_progress: f64,
    pub(super) height_progress: f64,
    pub(super) shoulder_progress: f64,
    pub(super) drop_progress: f64,
    pub(super) cards_progress: f64,
}

#[derive(Clone, Copy)]
pub(super) struct NativePanelGeometryMetrics {
    pub(super) compact_height: f64,
    pub(super) compact_width: f64,
    pub(super) expanded_width: f64,
    pub(super) panel_width: f64,
}

#[derive(Clone, Copy)]
pub(super) struct NativePanelLayout {
    pub(super) panel_frame: NSRect,
    pub(super) content_frame: NSRect,
    pub(super) pill_frame: NSRect,
    pub(super) left_shoulder_frame: NSRect,
    pub(super) right_shoulder_frame: NSRect,
    pub(super) expanded_frame: NSRect,
    pub(super) cards_frame: NSRect,
    pub(super) separator_frame: NSRect,
    pub(super) shared_content_frame: NSRect,
    pub(super) shell_visible: bool,
    pub(super) separator_visibility: f64,
}

impl NativePanelTransitionFrame {
    pub(super) fn expanded(height: f64) -> Self {
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

    pub(super) fn collapsed(height: f64) -> Self {
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

#[derive(Clone)]
pub(super) enum NativeStatusQueuePayload {
    Approval(PendingPermissionView),
    Completion(SessionSnapshotView),
}

#[derive(Clone)]
pub(super) struct NativeStatusQueueItem {
    pub(super) key: String,
    pub(super) session_id: String,
    pub(super) sort_time: chrono::DateTime<Utc>,
    pub(super) expires_at: Instant,
    pub(super) is_live: bool,
    pub(super) is_removing: bool,
    pub(super) remove_after: Option<Instant>,
    pub(super) payload: NativeStatusQueuePayload,
}

#[derive(Clone)]
pub(super) struct NativePendingPermissionCard {
    pub(super) request_id: String,
    pub(super) payload: PendingPermissionView,
    pub(super) started_at: Instant,
    pub(super) last_seen_at: Instant,
    pub(super) visible_until: Instant,
}

#[derive(Clone)]
pub(super) struct NativePendingQuestionCard {
    pub(super) request_id: String,
    pub(super) payload: PendingQuestionView,
    pub(super) started_at: Instant,
    pub(super) last_seen_at: Instant,
    pub(super) visible_until: Instant,
}

#[derive(Clone)]
pub(super) struct NativeCompletionBadgeItem {
    pub(super) session_id: String,
    pub(super) completed_at: chrono::DateTime<Utc>,
    pub(super) last_user_prompt: Option<String>,
    pub(super) last_assistant_message: Option<String>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct NativeStatusQueueSyncResult {
    pub(super) added_approvals: usize,
    pub(super) added_completions: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativePanelHitAction {
    FocusSession,
    CycleDisplay,
    ToggleCompletionSound,
    ToggleMascot,
    OpenReleasePage,
}

#[derive(Clone)]
pub(super) struct NativeCardHitTarget {
    pub(super) action: NativePanelHitAction,
    pub(super) value: String,
    pub(super) frame: NSRect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativeExpandedSurface {
    Default,
    Status,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativeHoverTransition {
    Expand,
    Collapse,
}

pub(super) struct NativePanelState {
    pub(super) expanded: bool,
    pub(super) transitioning: bool,
    pub(super) transition_cards_progress: f64,
    pub(super) transition_cards_entering: bool,
    pub(super) skip_next_close_card_exit: bool,
    pub(super) last_raw_snapshot: Option<RuntimeSnapshot>,
    pub(super) last_snapshot: Option<RuntimeSnapshot>,
    pub(super) status_queue: Vec<NativeStatusQueueItem>,
    pub(super) completion_badge_items: Vec<NativeCompletionBadgeItem>,
    pub(super) pending_permission_card: Option<NativePendingPermissionCard>,
    pub(super) pending_question_card: Option<NativePendingQuestionCard>,
    pub(super) status_auto_expanded: bool,
    pub(super) surface_mode: NativeExpandedSurface,
    pub(super) shared_body_height: Option<f64>,
    pub(super) pointer_inside_since: Option<Instant>,
    pub(super) pointer_outside_since: Option<Instant>,
    pub(super) primary_mouse_down: bool,
    pub(super) last_focus_click: Option<(String, Instant)>,
    pub(super) card_hit_targets: Vec<NativeCardHitTarget>,
    pub(super) mascot_runtime: NativeMascotRuntime,
}
