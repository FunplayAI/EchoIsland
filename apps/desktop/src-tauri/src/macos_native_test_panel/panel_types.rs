use std::time::Instant;

use echoisland_runtime::RuntimeSnapshot;
use objc2_foundation::NSRect;

use super::mascot::NativeMascotRuntime;
use crate::native_panel_renderer::facade::{
    descriptor::{NativePanelHostWindowDescriptor, NativePanelPointerRegion},
    host::NativePanelRuntimeHostState,
    interaction::{
        NativePanelClickStateBridge, NativePanelCoreStateBridge,
        NativePanelHostInteractionStateBridge, NativePanelPrimaryPointerStateBridge,
        record_native_panel_focus_click_session, resolve_native_panel_last_focus_click,
    },
    renderer::{
        NativePanelRuntimeSceneCache, NativePanelRuntimeSceneMutableStateBridge,
        NativePanelRuntimeSceneStateBridge,
    },
    runtime::{
        NativePanelRuntimeRenderPayloadState, NativePanelRuntimeRenderPayloadStateBridge,
        resolve_native_panel_runtime_render_payload_for_state,
    },
};

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
pub(super) type NativePanelAnimationDescriptor = crate::native_panel_core::PanelAnimationDescriptor;
pub(super) type NativePanelPendingTransition =
    crate::native_panel_renderer::facade::transition::NativePanelPendingTransition;

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

pub(super) type NativeExpandedSurface = crate::native_panel_core::ExpandedSurface;
#[cfg(test)]
pub(super) type NativeHoverTransition = crate::native_panel_core::HoverTransition;

pub(super) struct NativePanelState {
    pub(super) expanded: bool,
    pub(super) transitioning: bool,
    pub(super) transition_cards_progress: f64,
    pub(super) transition_cards_entering: bool,
    pub(super) skip_next_close_card_exit: bool,
    pub(super) pending_transition: Option<NativePanelPendingTransition>,
    pub(super) last_raw_snapshot: Option<RuntimeSnapshot>,
    pub(super) last_snapshot: Option<RuntimeSnapshot>,
    pub(super) scene_cache: NativePanelRuntimeSceneCache,
    pub(super) status_queue: Vec<NativeStatusQueueItem>,
    pub(super) completion_badge_items: Vec<NativeCompletionBadgeItem>,
    pub(super) pending_permission_card: Option<NativePendingPermissionCard>,
    pub(super) pending_question_card: Option<NativePendingQuestionCard>,
    pub(super) status_auto_expanded: bool,
    pub(super) surface_mode: NativeExpandedSurface,
    pub(super) shared_body_height: Option<f64>,
    pub(super) host_window_descriptor: NativePanelHostWindowDescriptor,
    pub(super) pointer_inside_since: Option<Instant>,
    pub(super) pointer_outside_since: Option<Instant>,
    pub(super) primary_mouse_down: bool,
    pub(super) ignores_mouse_events: bool,
    pub(super) last_focus_click: Option<(String, Instant)>,
    pub(super) pointer_regions: Vec<NativePanelPointerRegion>,
    pub(super) mascot_runtime: NativeMascotRuntime,
}

impl NativePanelState {
    pub(super) fn render_payload_snapshot(&self) -> Option<RuntimeSnapshot> {
        self.scene_cache
            .last_snapshot
            .clone()
            .or_else(|| self.last_snapshot.clone())
    }

    pub(super) fn render_payload(&self) -> Option<NativePanelRenderPayload> {
        resolve_native_panel_runtime_render_payload_for_state(self, |snapshot, payload_state| {
            NativePanelRenderPayload {
                snapshot,
                expanded: payload_state.expanded,
                shared_body_height: payload_state.shared_body_height,
                transitioning: payload_state.transitioning,
                transition_cards_progress: payload_state.transition_cards_progress,
                transition_cards_entering: payload_state.transition_cards_entering,
            }
        })
    }

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

impl NativePanelRuntimeRenderPayloadStateBridge for NativePanelState {
    fn runtime_render_payload_snapshot(&self) -> Option<RuntimeSnapshot> {
        self.render_payload_snapshot()
    }

    fn runtime_render_payload_state(&self) -> NativePanelRuntimeRenderPayloadState {
        NativePanelRuntimeRenderPayloadState {
            expanded: self.expanded,
            shared_body_height: self.shared_body_height,
            transitioning: self.transitioning,
            transition_cards_progress: self.transition_cards_progress,
            transition_cards_entering: self.transition_cards_entering,
        }
    }
}

impl NativePanelRuntimeSceneStateBridge for NativePanelState {
    fn runtime_scene_cache(&self) -> &NativePanelRuntimeSceneCache {
        &self.scene_cache
    }

    fn runtime_scene_current_snapshot(&self) -> Option<&RuntimeSnapshot> {
        self.last_snapshot.as_ref()
    }
}

impl NativePanelRuntimeSceneMutableStateBridge for NativePanelState {
    fn runtime_scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache {
        &mut self.scene_cache
    }

    fn runtime_pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion> {
        &mut self.pointer_regions
    }
}

impl NativePanelCoreStateBridge for NativePanelState {
    fn snapshot_core_panel_state(&self) -> crate::native_panel_core::PanelState {
        self.to_core_panel_state()
    }

    fn apply_core_panel_state(&mut self, core: crate::native_panel_core::PanelState) {
        NativePanelState::apply_core_panel_state(self, core);
    }
}

impl NativePanelClickStateBridge for NativePanelState {
    fn click_expanded(&self) -> bool {
        self.expanded
    }

    fn click_transitioning(&self) -> bool {
        self.transitioning
    }

    fn click_last_focus_click(&self) -> Option<crate::native_panel_core::LastFocusClick<'_>> {
        resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
    }

    fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
        record_native_panel_focus_click_session(&mut self.last_focus_click, session_id, now);
    }
}

impl NativePanelPrimaryPointerStateBridge for NativePanelState {
    fn primary_pointer_down(&self) -> bool {
        self.primary_mouse_down
    }

    fn set_primary_pointer_down(&mut self, down: bool) {
        self.primary_mouse_down = down;
    }
}

impl NativePanelHostInteractionStateBridge for NativePanelState {
    fn host_ignores_mouse_events(&self) -> bool {
        self.ignores_mouse_events
    }

    fn set_host_ignores_mouse_events(&mut self, ignores_mouse_events: bool) {
        self.ignores_mouse_events = ignores_mouse_events;
    }
}

impl NativePanelRuntimeHostState for NativePanelState {
    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
        &mut self.host_window_descriptor
    }

    fn pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion> {
        &mut self.pointer_regions
    }
}
