use std::time::Instant;

use echoisland_runtime::RuntimeSnapshot;
use objc2_foundation::NSRect;

use super::mascot::NativeMascotRuntime;

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

pub(super) type NativePanelTransitionFrame = crate::native_panel_core::PanelTransitionFrame;

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

#[derive(Clone)]
pub(super) struct NativePanelRenderPayload {
    pub(super) snapshot: RuntimeSnapshot,
    pub(super) expanded: bool,
    pub(super) shared_body_height: Option<f64>,
    pub(super) transitioning: bool,
    pub(super) transition_cards_progress: f64,
    pub(super) transition_cards_entering: bool,
}

pub(super) type NativeStatusQueuePayload = crate::native_panel_core::StatusQueuePayload;
pub(super) type NativeStatusQueueItem = crate::native_panel_core::StatusQueueItem;
pub(super) type NativePendingPermissionCard = crate::native_panel_core::PendingPermissionCardState;
pub(super) type NativePendingQuestionCard = crate::native_panel_core::PendingQuestionCardState;
pub(super) type NativeCompletionBadgeItem = crate::native_panel_core::CompletionBadgeItem;
#[cfg(test)]
pub(super) type NativeStatusQueueSyncResult = crate::native_panel_core::StatusQueueSyncResult;
pub(super) type NativePanelHitAction = crate::native_panel_core::PanelHitAction;

#[derive(Clone)]
pub(super) struct NativeCardHitTarget {
    pub(super) action: NativePanelHitAction,
    pub(super) value: String,
    pub(super) frame: NSRect,
}

pub(super) type NativeExpandedSurface = crate::native_panel_core::ExpandedSurface;
pub(super) type NativeHoverTransition = crate::native_panel_core::HoverTransition;

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

impl NativePanelState {
    pub(super) fn to_core_panel_state(&self) -> crate::native_panel_core::PanelState {
        crate::native_panel_core::PanelState {
            expanded: self.expanded,
            transitioning: self.transitioning,
            skip_next_close_card_exit: self.skip_next_close_card_exit,
            last_raw_snapshot: self.last_raw_snapshot.clone(),
            status_queue: self.status_queue.clone(),
            completion_badge_items: self.completion_badge_items.clone(),
            pending_permission_card: self.pending_permission_card.clone(),
            pending_question_card: self.pending_question_card.clone(),
            status_auto_expanded: self.status_auto_expanded,
            surface_mode: self.surface_mode,
            pointer_inside_since: self.pointer_inside_since,
            pointer_outside_since: self.pointer_outside_since,
        }
    }

    pub(super) fn apply_core_panel_state(&mut self, core: crate::native_panel_core::PanelState) {
        self.expanded = core.expanded;
        self.transitioning = core.transitioning;
        self.skip_next_close_card_exit = core.skip_next_close_card_exit;
        self.last_raw_snapshot = core.last_raw_snapshot;
        self.status_queue = core.status_queue;
        self.completion_badge_items = core.completion_badge_items;
        self.pending_permission_card = core.pending_permission_card;
        self.pending_question_card = core.pending_question_card;
        self.status_auto_expanded = core.status_auto_expanded;
        self.surface_mode = core.surface_mode;
        self.pointer_inside_since = core.pointer_inside_since;
        self.pointer_outside_since = core.pointer_outside_since;
    }
}
