use std::sync::Mutex;
use std::time::Instant;

use super::mascot::NativeMascotRuntime;
use super::panel_globals::{ACTIVE_COUNT_SCROLL_TEXT, NATIVE_TEST_PANEL_STATE};
use super::panel_types::{NativeExpandedSurface, NativePanelState};
use crate::native_panel_renderer::facade::{
    descriptor::NativePanelHostWindowDescriptor, renderer::NativePanelRuntimeSceneCache,
};

pub(super) fn initialize_native_panel_state(
    host_window_descriptor: NativePanelHostWindowDescriptor,
) {
    let _ = NATIVE_TEST_PANEL_STATE.set(Mutex::new(NativePanelState {
        expanded: false,
        transitioning: false,
        transition_cards_progress: 1.0,
        transition_cards_entering: false,
        skip_next_close_card_exit: false,
        pending_transition: None,
        last_raw_snapshot: None,
        last_snapshot: None,
        scene_cache: NativePanelRuntimeSceneCache::default(),
        status_queue: Vec::new(),
        completion_badge_items: Vec::new(),
        pending_permission_card: None,
        pending_question_card: None,
        status_auto_expanded: false,
        surface_mode: NativeExpandedSurface::Default,
        shared_body_height: None,
        host_window_descriptor,
        pointer_inside_since: None,
        pointer_outside_since: None,
        primary_mouse_down: false,
        ignores_mouse_events: true,
        last_focus_click: None,
        pointer_regions: Vec::new(),
        mascot_runtime: NativeMascotRuntime::new(Instant::now()),
    }));
}

pub(super) fn initialize_active_count_scroll_text() {
    let _ = ACTIVE_COUNT_SCROLL_TEXT.set(Mutex::new("0".to_string()));
}
