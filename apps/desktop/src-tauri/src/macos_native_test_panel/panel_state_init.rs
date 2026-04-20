use super::*;

pub(super) fn initialize_native_panel_state() {
    let _ = NATIVE_TEST_PANEL_STATE.set(Mutex::new(NativePanelState {
        expanded: false,
        transitioning: false,
        transition_cards_progress: 1.0,
        transition_cards_entering: false,
        skip_next_close_card_exit: false,
        last_raw_snapshot: None,
        last_snapshot: None,
        status_queue: Vec::new(),
        completion_badge_items: Vec::new(),
        pending_permission_card: None,
        pending_question_card: None,
        status_auto_expanded: false,
        surface_mode: NativeExpandedSurface::Default,
        shared_body_height: None,
        pointer_inside_since: None,
        pointer_outside_since: None,
        primary_mouse_down: false,
        last_focus_click: None,
        card_hit_targets: Vec::new(),
        mascot_runtime: NativeMascotRuntime::new(Instant::now()),
    }));
}

pub(super) fn initialize_active_count_scroll_text() {
    let _ = ACTIVE_COUNT_SCROLL_TEXT.set(Mutex::new("0".to_string()));
}
