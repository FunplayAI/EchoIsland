use std::time::{Duration, Instant};

use crate::native_panel_renderer::facade::{
    descriptor::{
        NativePanelHostWindowDescriptor, NativePanelPointerRegionKind,
        NativePanelRuntimeInputDescriptor, resolve_native_panel_pointer_regions,
    },
    renderer::NativePanelRuntimeSceneCache,
    transition::NativePanelTransitionRequest,
};
use chrono::Utc;
use echoisland_runtime::{PendingPermissionView, RuntimeSnapshot, SessionSnapshotView};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::card_animation::card_content_visibility_phase;
use super::mascot::NativeMascotRuntime;
use super::panel_constants::{
    DEFAULT_COMPACT_PILL_HEIGHT, DEFAULT_COMPACT_PILL_WIDTH, DEFAULT_EXPANDED_PILL_WIDTH,
    DEFAULT_PANEL_CANVAS_WIDTH, EXPANDED_PILL_WIDTH_DELTA, HOVER_DELAY_MS,
    PANEL_CLOSE_MORPH_DELAY_MS, PANEL_CLOSE_SHOULDER_DELAY_MS, PANEL_HEIGHT_MS,
    PANEL_MORPH_DELAY_MS, PANEL_MORPH_MS, PANEL_SHOULDER_HIDE_MS,
    PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS, PENDING_CARD_MIN_VISIBLE_MS,
    PENDING_CARD_RELEASE_GRACE_MS, STATUS_APPROVAL_VISIBLE_SECONDS,
    STATUS_COMPLETION_VISIBLE_SECONDS,
};
use super::panel_geometry::{
    centered_top_frame, island_bar_frame, native_panel_core_layout, panel_transition_canvas_height,
    resolve_native_panel_host_frame, resolve_native_panel_layout, shared_expanded_content_state,
};
use super::panel_hit_testing::native_hover_pill_rect;
use super::panel_interaction::{sync_native_hover_expansion_state, toggle_native_settings_surface};
use super::panel_runtime_dispatch::{
    clear_pending_native_panel_close_transition_in_state,
    defer_native_panel_transition_in_state_if_active,
};
use super::panel_scene_adapter::build_native_panel_scene_for_core_state_with_input;
use super::panel_screen_geometry::{
    expanded_width_for_camera_housing_screen, expanded_width_for_non_camera_housing_screen,
    shell_width_for_non_camera_housing_screen,
};
use super::panel_snapshot::native_panel_render_payload;
use super::panel_snapshot::sync_native_status_surface_policy;
use super::panel_types::{
    NativeCompletionBadgeItem, NativeExpandedSurface, NativeHoverTransition,
    NativePanelGeometryMetrics, NativePanelState, NativePanelTransitionFrame,
    NativePendingPermissionCard, NativeStatusQueueItem, NativeStatusQueuePayload,
    NativeStatusQueueSyncResult,
};
use super::queue_logic::{
    compare_native_status_queue_items, detect_completed_sessions,
    resolve_native_pending_permission_card, sync_native_completion_badge,
    sync_native_pending_card_visibility, sync_native_status_queue,
};
use super::transition_logic::{
    resolve_close_transition_frame, resolve_open_transition_descriptor,
    resolve_open_transition_frame, resolve_surface_switch_transition_frame,
    surface_switch_card_progress,
};
use crate::native_panel_scene::PanelSceneBuildInput;

fn snapshot(active: usize, total: usize) -> RuntimeSnapshot {
    RuntimeSnapshot {
        status: "Idle".to_string(),
        primary_source: "claude".to_string(),
        active_session_count: active,
        total_session_count: total,
        pending_permission_count: 0,
        pending_question_count: 0,
        pending_permission: None,
        pending_question: None,
        pending_permissions: Vec::new(),
        pending_questions: Vec::new(),
        sessions: Vec::new(),
    }
}

fn session(status: &str) -> SessionSnapshotView {
    SessionSnapshotView {
        session_id: "session-1".to_string(),
        source: "claude".to_string(),
        project_name: None,
        cwd: None,
        model: None,
        terminal_app: None,
        terminal_bundle: None,
        host_app: None,
        window_title: None,
        tty: None,
        terminal_pid: None,
        cli_pid: None,
        iterm_session_id: None,
        kitty_window_id: None,
        tmux_env: None,
        tmux_pane: None,
        tmux_client_tty: None,
        status: status.to_string(),
        current_tool: None,
        tool_description: None,
        last_user_prompt: None,
        last_assistant_message: None,
        tool_history_count: 0,
        tool_history: Vec::new(),
        last_activity: Utc::now(),
    }
}

fn pending_permission(request_id: &str, session_id: &str) -> PendingPermissionView {
    PendingPermissionView {
        request_id: request_id.to_string(),
        session_id: session_id.to_string(),
        source: "claude".to_string(),
        tool_name: Some("Bash".to_string()),
        tool_description: Some("Run command".to_string()),
        requested_at: Utc::now(),
    }
}

fn snapshot_with_permission(request_id: &str, session_id: &str) -> RuntimeSnapshot {
    let mut snapshot = snapshot(1, 1);
    let pending = pending_permission(request_id, session_id);
    snapshot.pending_permission_count = 1;
    snapshot.pending_permission = Some(pending.clone());
    snapshot.pending_permissions = vec![pending];
    snapshot.sessions = vec![session("WaitingApproval")];
    snapshot
}

fn panel_state() -> NativePanelState {
    NativePanelState {
        expanded: false,
        transitioning: false,
        transition_cards_progress: 0.0,
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
        host_window_descriptor: NativePanelHostWindowDescriptor::default(),
        pointer_inside_since: None,
        pointer_outside_since: None,
        primary_mouse_down: false,
        ignores_mouse_events: true,
        last_focus_click: None,
        pointer_regions: Vec::new(),
        mascot_runtime: NativeMascotRuntime::new(Instant::now()),
    }
}

#[test]
fn resolved_approval_enters_status_queue_exit_instead_of_waiting_for_expiry() {
    let mut state = panel_state();
    let live_snapshot = snapshot_with_permission("request-1", "session-1");

    assert!(sync_native_status_queue(&mut state, &live_snapshot).added_approvals > 0);
    assert_eq!(state.status_queue.len(), 1);
    state.last_raw_snapshot = Some(live_snapshot);

    let empty_snapshot = snapshot(0, 1);
    assert_eq!(
        sync_native_status_queue(&mut state, &empty_snapshot).added_approvals,
        0
    );

    assert_eq!(state.status_queue.len(), 1);
    assert!(state.status_queue[0].is_removing);
    assert!(state.status_queue[0].remove_after.is_some());
    assert!(matches!(
        state.status_queue[0].payload,
        NativeStatusQueuePayload::Approval(_)
    ));
}

#[test]
fn pending_card_grace_snapshot_does_not_readd_status_approval() {
    let mut state = panel_state();
    let live_snapshot = snapshot_with_permission("request-1", "session-1");
    let held_snapshot = sync_native_pending_card_visibility(&mut state, &live_snapshot);

    assert_eq!(held_snapshot.pending_permission_count, 1);
    state.last_raw_snapshot = Some(live_snapshot);

    let empty_snapshot = snapshot(0, 1);
    let held_after_resolve = sync_native_pending_card_visibility(&mut state, &empty_snapshot);

    assert_eq!(held_after_resolve.pending_permission_count, 1);
    assert_eq!(
        sync_native_status_queue(&mut state, &empty_snapshot).added_approvals,
        0
    );
    assert!(state.status_queue.is_empty());
}

#[test]
fn pending_permission_card_clears_after_grace_window_expires() {
    let now = Instant::now();
    let previous = NativePendingPermissionCard {
        request_id: "request-1".to_string(),
        payload: pending_permission("request-1", "session-1"),
        started_at: now - Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS),
        last_seen_at: now - Duration::from_millis(PENDING_CARD_RELEASE_GRACE_MS + 10),
        visible_until: now - Duration::from_millis(1),
    };

    assert!(resolve_native_pending_permission_card(None, Some(&previous), now).is_none());
}

#[test]
fn completion_badge_tracks_completed_session_until_new_dialogue() {
    let mut state = panel_state();
    let mut previous = snapshot(1, 1);
    previous.sessions = vec![session("Running")];

    let mut current = snapshot(0, 1);
    let mut completed = session("Idle");
    completed.last_assistant_message = Some("Done".to_string());
    current.sessions = vec![completed.clone()];

    let completed_session_ids = detect_completed_sessions(&previous, &current, Utc::now());
    sync_native_completion_badge(&mut state, &current, &completed_session_ids);

    assert_eq!(state.completion_badge_items.len(), 1);
    assert_eq!(
        state.completion_badge_items[0].session_id,
        completed.session_id
    );

    let mut next = current.clone();
    let next_session = next.sessions.first_mut().unwrap();
    next_session.status = "Running".to_string();
    next_session.last_user_prompt = Some("continue".to_string());
    next_session.last_activity = completed.last_activity + chrono::Duration::seconds(1);

    sync_native_completion_badge(&mut state, &next, &[]);

    assert!(state.completion_badge_items.is_empty());
}

#[test]
fn completion_badge_clears_when_island_expands() {
    let mut state = panel_state();
    let mut current = snapshot(0, 1);
    current.sessions = vec![session("Idle")];
    sync_native_completion_badge(
        &mut state,
        &current,
        &[current.sessions[0].session_id.clone()],
    );
    assert_eq!(state.completion_badge_items.len(), 1);

    state.expanded = true;
    sync_native_completion_badge(&mut state, &current, &[]);

    assert!(state.completion_badge_items.is_empty());
}

#[test]
fn completion_badge_stays_during_auto_status_expansion() {
    let mut state = panel_state();
    let mut current = snapshot(0, 1);
    current.sessions = vec![session("Idle")];
    sync_native_completion_badge(
        &mut state,
        &current,
        &[current.sessions[0].session_id.clone()],
    );
    assert_eq!(state.completion_badge_items.len(), 1);

    state.expanded = true;
    state.status_auto_expanded = true;
    state.surface_mode = NativeExpandedSurface::Status;
    sync_native_completion_badge(&mut state, &current, &[]);

    assert_eq!(state.completion_badge_items.len(), 1);
}

#[test]
fn completion_status_queue_auto_expands_status_surface() {
    let mut state = panel_state();
    state
        .completion_badge_items
        .push(NativeCompletionBadgeItem {
            session_id: "session-1".to_string(),
            completed_at: Utc::now(),
            last_user_prompt: Some("ship it".to_string()),
            last_assistant_message: Some("Done".to_string()),
        });
    state.status_queue.push(NativeStatusQueueItem {
        key: "completion:session-1".to_string(),
        session_id: "session-1".to_string(),
        sort_time: Utc::now(),
        expires_at: Instant::now() + Duration::from_secs(STATUS_COMPLETION_VISIBLE_SECONDS),
        is_live: true,
        is_removing: false,
        remove_after: None,
        payload: NativeStatusQueuePayload::Completion(session("Idle")),
    });

    let transition = sync_native_status_surface_policy(
        &mut state,
        NativeStatusQueueSyncResult {
            added_approvals: 0,
            added_questions: 0,
            added_completions: 1,
        },
    );

    assert_eq!(transition.panel_transition, Some(true));
    assert!(!transition.surface_transition);
    assert!(state.expanded);
    assert!(state.status_auto_expanded);
    assert_eq!(state.surface_mode, NativeExpandedSurface::Status);
    assert_eq!(state.completion_badge_items.len(), 1);
}

#[test]
fn status_surface_transition_switches_expanded_panel_into_status_mode() {
    let mut state = panel_state();
    state.expanded = true;
    state.status_queue.push(NativeStatusQueueItem {
        key: "approval:request-1".to_string(),
        session_id: "session-1".to_string(),
        sort_time: Utc::now(),
        expires_at: Instant::now() + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS),
        is_live: true,
        is_removing: false,
        remove_after: None,
        payload: NativeStatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
    });

    let transition = sync_native_status_surface_policy(
        &mut state,
        NativeStatusQueueSyncResult {
            added_approvals: 1,
            added_questions: 0,
            added_completions: 0,
        },
    );

    assert_eq!(transition.panel_transition, None);
    assert!(transition.surface_transition);
    assert!(state.expanded);
    assert!(state.status_auto_expanded);
    assert_eq!(state.surface_mode, NativeExpandedSurface::Status);
}

#[test]
fn status_surface_reverts_to_default_when_queue_drains() {
    let mut state = panel_state();
    state.expanded = true;
    state.surface_mode = NativeExpandedSurface::Status;
    state.status_auto_expanded = true;

    let transition =
        sync_native_status_surface_policy(&mut state, NativeStatusQueueSyncResult::default());

    assert_eq!(transition.panel_transition, Some(false));
    assert!(!transition.surface_transition);
    assert!(!state.expanded);
    assert!(!state.status_auto_expanded);
    assert_eq!(state.surface_mode, NativeExpandedSurface::Default);
    assert!(state.skip_next_close_card_exit);
}

#[test]
fn settings_surface_toggle_cycles_between_default_and_settings() {
    let mut state = panel_state();

    assert!(toggle_native_settings_surface(&mut state));
    assert_eq!(state.surface_mode, NativeExpandedSurface::Settings);
    assert!(!state.status_auto_expanded);

    assert!(toggle_native_settings_surface(&mut state));
    assert_eq!(state.surface_mode, NativeExpandedSurface::Default);
}

#[test]
fn settings_surface_toggle_overrides_status_surface() {
    let mut state = panel_state();
    state.surface_mode = NativeExpandedSurface::Status;
    state.status_auto_expanded = true;

    assert!(toggle_native_settings_surface(&mut state));
    assert_eq!(state.surface_mode, NativeExpandedSurface::Settings);
    assert!(!state.status_auto_expanded);
}

#[test]
fn native_render_payload_captures_snapshot_and_runtime_flags() {
    let mut state = panel_state();
    state.last_snapshot = Some(snapshot(1, 2));
    state.expanded = true;
    state.shared_body_height = Some(132.0);
    state.transitioning = true;
    state.transition_cards_progress = 0.42;
    state.transition_cards_entering = true;

    let payload = native_panel_render_payload(&state).expect("expected render payload");

    assert_eq!(payload.snapshot.active_session_count, 1);
    assert_eq!(payload.snapshot.total_session_count, 2);
    assert!(payload.expanded);
    assert_eq!(payload.shared_body_height, Some(132.0));
    assert!(payload.transitioning);
    assert_eq!(payload.transition_cards_progress, 0.42);
    assert!(payload.transition_cards_entering);
}

#[test]
fn status_queue_sorting_keeps_approvals_first_and_completions_after() {
    let now = Instant::now();
    let earlier = Utc::now() - chrono::Duration::seconds(10);
    let later = Utc::now();
    let mut items = vec![
        NativeStatusQueueItem {
            key: "completion:session-2".to_string(),
            session_id: "session-2".to_string(),
            sort_time: later,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: NativeStatusQueuePayload::Completion(session("Idle")),
        },
        NativeStatusQueueItem {
            key: "approval:request-2".to_string(),
            session_id: "session-2".to_string(),
            sort_time: later,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: NativeStatusQueuePayload::Approval(pending_permission(
                "request-2",
                "session-2",
            )),
        },
        NativeStatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: earlier,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: NativeStatusQueuePayload::Approval(pending_permission(
                "request-1",
                "session-1",
            )),
        },
    ];

    items.sort_by(compare_native_status_queue_items);

    assert!(matches!(
        items[0].payload,
        NativeStatusQueuePayload::Approval(_)
    ));
    assert_eq!(items[0].key, "approval:request-1");
    assert!(matches!(
        items[1].payload,
        NativeStatusQueuePayload::Approval(_)
    ));
    assert_eq!(items[1].key, "approval:request-2");
    assert!(matches!(
        items[2].payload,
        NativeStatusQueuePayload::Completion(_)
    ));
}

#[test]
fn surface_switch_card_progress_starts_above_zero_for_continuity() {
    assert_eq!(
        surface_switch_card_progress(0, 220),
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS
    );
}

#[test]
fn surface_switch_card_progress_reaches_full_visibility() {
    assert_eq!(surface_switch_card_progress(220, 220), 1.0);
    assert_eq!(surface_switch_card_progress(999, 220), 1.0);
    assert_eq!(surface_switch_card_progress(0, 0), 1.0);
}

#[test]
fn entering_content_visibility_waits_for_reveal_delay() {
    assert_eq!(card_content_visibility_phase(0.10, true), 0.0);
    assert_eq!(card_content_visibility_phase(0.18, true), 0.0);
    assert!(card_content_visibility_phase(0.24, true) > 0.0);
}

#[test]
fn exiting_content_visibility_fades_to_zero() {
    assert_eq!(card_content_visibility_phase(0.0, false), 1.0);
    assert!(card_content_visibility_phase(0.30, false) < 1.0);
    assert_eq!(card_content_visibility_phase(1.0, false), 0.0);
}

#[test]
fn shared_content_waits_for_card_content_reveal() {
    let (visible, interactive) =
        shared_expanded_content_state(true, true, 1.0, 1.0, 120.0, false, 0.80);

    assert!(!visible);
    assert!(!interactive);
}

#[test]
fn shared_content_becomes_visible_and_interactive_after_reveal() {
    let (visible, interactive) =
        shared_expanded_content_state(true, true, 1.0, 1.0, 120.0, false, 1.0);

    assert!(visible);
    assert!(interactive);
}

#[test]
fn shared_content_stays_hidden_for_status_surface() {
    let (visible, interactive) =
        shared_expanded_content_state(true, true, 1.0, 1.0, 120.0, true, 1.0);

    assert!(!visible);
    assert!(!interactive);
}

#[test]
fn centered_top_frame_snaps_panel_geometry_to_whole_points() {
    let frame = centered_top_frame(
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1512.0, 982.0)),
        NSSize::new(419.6, 152.4),
    );

    assert_eq!(frame.origin.x.fract(), 0.0);
    assert_eq!(frame.origin.y.fract(), 0.0);
    assert_eq!(frame.size.width, 420.0);
    assert_eq!(frame.size.height, 152.0);
    assert_eq!(frame.origin.y + frame.size.height, 982.0);
}

#[test]
fn transition_canvas_height_uses_max_height_during_animation() {
    assert_eq!(panel_transition_canvas_height(80.0, 164.0), 164.0);
    assert_eq!(panel_transition_canvas_height(196.0, 80.0), 196.0);
    assert_eq!(panel_transition_canvas_height(148.0, 168.0), 168.0);
}

#[test]
fn transition_frame_uses_named_fields_for_progress() {
    let frame = NativePanelTransitionFrame {
        canvas_height: 196.0,
        visible_height: 148.0,
        bar_progress: 0.4,
        height_progress: 0.6,
        shoulder_progress: 0.8,
        drop_progress: 0.3,
        cards_progress: 0.7,
    };

    assert_eq!(frame.canvas_height, 196.0);
    assert_eq!(frame.visible_height, 148.0);
    assert_eq!(frame.bar_progress, 0.4);
    assert_eq!(frame.height_progress, 0.6);
    assert_eq!(frame.shoulder_progress, 0.8);
    assert_eq!(frame.drop_progress, 0.3);
    assert_eq!(frame.cards_progress, 0.7);
}

#[test]
fn non_camera_housing_shell_width_stays_close_to_default_on_1440_screen() {
    let width = shell_width_for_non_camera_housing_screen(1440.0, DEFAULT_COMPACT_PILL_HEIGHT);

    assert_eq!(width, DEFAULT_COMPACT_PILL_WIDTH);
}

#[test]
fn non_camera_housing_expanded_width_matches_web_shell() {
    assert_eq!(
        expanded_width_for_non_camera_housing_screen(),
        DEFAULT_COMPACT_PILL_WIDTH + EXPANDED_PILL_WIDTH_DELTA
    );
}

#[test]
fn camera_housing_expanded_width_is_wider_than_compact_width() {
    let compact_width = 312.0;
    let expanded_width = expanded_width_for_camera_housing_screen(compact_width);

    assert_eq!(expanded_width, compact_width + EXPANDED_PILL_WIDTH_DELTA);
    assert!(expanded_width <= DEFAULT_PANEL_CANVAS_WIDTH);
}

#[test]
fn island_bar_frame_interpolates_width_before_height_growth() {
    let content_size = NSSize::new(420.0, 164.0);
    let compact = island_bar_frame(
        content_size,
        0.0,
        DEFAULT_COMPACT_PILL_WIDTH,
        DEFAULT_EXPANDED_PILL_WIDTH,
        DEFAULT_COMPACT_PILL_HEIGHT,
        0.0,
    );
    let expanding = island_bar_frame(
        content_size,
        0.5,
        DEFAULT_COMPACT_PILL_WIDTH,
        DEFAULT_EXPANDED_PILL_WIDTH,
        DEFAULT_COMPACT_PILL_HEIGHT,
        0.0,
    );
    let expanded = island_bar_frame(
        content_size,
        1.0,
        DEFAULT_COMPACT_PILL_WIDTH,
        DEFAULT_EXPANDED_PILL_WIDTH,
        DEFAULT_COMPACT_PILL_HEIGHT,
        0.0,
    );

    assert_eq!(compact.size.width, DEFAULT_COMPACT_PILL_WIDTH);
    assert!(expanding.size.width > compact.size.width);
    assert!(expanding.size.width < expanded.size.width);
    assert_eq!(expanded.size.width, DEFAULT_EXPANDED_PILL_WIDTH);
}

#[test]
fn static_transition_frames_match_expected_end_states() {
    let expanded = NativePanelTransitionFrame::expanded(164.0);
    let collapsed = NativePanelTransitionFrame::collapsed(80.0);

    assert_eq!(expanded.canvas_height, 164.0);
    assert_eq!(expanded.visible_height, 164.0);
    assert_eq!(expanded.bar_progress, 1.0);
    assert_eq!(expanded.cards_progress, 1.0);
    assert_eq!(collapsed.canvas_height, 80.0);
    assert_eq!(collapsed.visible_height, 80.0);
    assert_eq!(collapsed.bar_progress, 0.0);
    assert_eq!(collapsed.cards_progress, 0.0);
}

#[test]
fn native_panel_layout_clamps_visible_height_to_canvas() {
    let layout = resolve_native_panel_layout(
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1512.0, 982.0)),
        NativePanelGeometryMetrics {
            compact_height: 38.0,
            compact_width: 126.0,
            expanded_width: 356.0,
            panel_width: 356.0,
        },
        120.0,
        220.0,
        1.0,
        1.0,
        0.0,
        1.0,
    );

    assert_eq!(layout.content_frame.size.height, 120.0);
    assert_eq!(layout.expanded_frame.size.height, 120.0);
    assert!(layout.shell_visible);
    assert_eq!(layout.separator_visibility, 0.88);
}

#[test]
fn macos_layout_feeds_shared_pointer_regions() {
    let layout = resolve_native_panel_layout(
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1440.0, 900.0)),
        NativePanelGeometryMetrics {
            compact_height: 38.0,
            compact_width: 253.0,
            expanded_width: 283.0,
            panel_width: 420.0,
        },
        180.0,
        180.0,
        1.0,
        1.0,
        1.0,
        1.0,
    );
    let mut core_state = crate::native_panel_core::PanelState::default();
    core_state.expanded = true;
    core_state.surface_mode = crate::native_panel_core::ExpandedSurface::Settings;
    let input = NativePanelRuntimeInputDescriptor {
        scene_input: PanelSceneBuildInput::default(),
        screen_frame: None,
    };
    let scene =
        build_native_panel_scene_for_core_state_with_input(&snapshot(0, 0), &core_state, &input);

    let regions =
        resolve_native_panel_pointer_regions(native_panel_core_layout(&layout), &scene, None);

    assert!(
        regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CompactBar))
    );
    assert!(
        regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CardsContainer))
    );
    assert!(
        regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::HitTarget(_)))
    );
}

#[test]
fn macos_host_frame_adapter_uses_shared_animation_descriptor() {
    let frame = resolve_native_panel_host_frame(
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1440.0, 900.0)),
        NativePanelGeometryMetrics {
            compact_height: 38.0,
            compact_width: 400.0,
            expanded_width: 700.0,
            panel_width: 700.0,
        },
        crate::native_panel_core::PanelAnimationDescriptor {
            kind: crate::native_panel_core::PanelAnimationKind::Open,
            canvas_height: 180.2,
            visible_height: 140.0,
            width_progress: 0.5,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        },
    );

    assert_eq!(frame.origin.x, 445.0);
    assert_eq!(frame.origin.y, 720.0);
    assert_eq!(frame.size.width, 550.0);
    assert_eq!(frame.size.height, 180.0);
}

#[test]
fn open_transition_sampler_starts_collapsed_and_reveals_cards_later() {
    let frame = resolve_open_transition_frame(0, 164.0, 164.0, 220);

    assert_eq!(frame.canvas_height, 164.0);
    assert_eq!(frame.visible_height, 80.0);
    assert_eq!(frame.bar_progress, 0.0);
    assert_eq!(frame.height_progress, 0.0);
    assert_eq!(frame.shoulder_progress, 0.0);
    assert_eq!(frame.cards_progress, 0.0);
}

#[test]
fn open_transition_contracts_shoulders_before_rounding_top_corners() {
    let shoulder_frame =
        resolve_open_transition_frame(PANEL_SHOULDER_HIDE_MS / 2, 164.0, 164.0, 220);
    assert_eq!(shoulder_frame.bar_progress, 0.0);
    assert!(shoulder_frame.shoulder_progress > 0.0);
    assert!(shoulder_frame.shoulder_progress < 1.0);

    let morph_start = resolve_open_transition_frame(PANEL_MORPH_DELAY_MS, 164.0, 164.0, 220);
    assert_eq!(morph_start.bar_progress, 0.0);
    assert_eq!(morph_start.shoulder_progress, 1.0);
}

#[test]
fn open_transition_expands_width_before_dropping_downward() {
    let width_expanding = resolve_open_transition_frame(
        PANEL_MORPH_DELAY_MS + (PANEL_MORPH_MS / 2),
        164.0,
        164.0,
        220,
    );
    assert!(width_expanding.bar_progress > 0.0);
    assert!(width_expanding.bar_progress < 1.0);
    assert_eq!(width_expanding.height_progress, 0.0);
    assert_eq!(width_expanding.drop_progress, 0.0);

    let height_growing = resolve_open_transition_frame(
        PANEL_MORPH_DELAY_MS + PANEL_MORPH_MS + (PANEL_HEIGHT_MS / 2),
        164.0,
        164.0,
        220,
    );
    assert_eq!(height_growing.bar_progress, 1.0);
    assert!(height_growing.height_progress > 0.0);
    assert!(height_growing.drop_progress > 0.0);
}

#[test]
fn open_transition_descriptor_maps_existing_frame_fields() {
    let descriptor = resolve_open_transition_descriptor(
        PANEL_MORPH_DELAY_MS + (PANEL_MORPH_MS / 2),
        164.0,
        164.0,
        220,
    );

    assert_eq!(
        descriptor.kind,
        crate::native_panel_core::PanelAnimationKind::Open
    );
    assert_eq!(descriptor.canvas_height, 164.0);
    assert_eq!(descriptor.visible_height, 80.0);
    assert!(descriptor.width_progress > 0.0);
    assert!(descriptor.width_progress < 1.0);
    assert_eq!(descriptor.height_progress, 0.0);
    assert_eq!(descriptor.drop_progress, 0.0);
}

#[test]
fn surface_switch_sampler_keeps_shell_fully_open() {
    let frame = resolve_surface_switch_transition_frame(0, 164.0, 120.0, 164.0, 220);

    assert_eq!(frame.bar_progress, 1.0);
    assert_eq!(frame.height_progress, 1.0);
    assert_eq!(frame.shoulder_progress, 1.0);
    assert_eq!(frame.drop_progress, 1.0);
    assert_eq!(
        frame.cards_progress,
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS
    );
}

#[test]
fn close_transition_sampler_reports_completed_exit_after_delays() {
    let frame = resolve_close_transition_frame(999, 164.0, 164.0, 220, 220);

    assert_eq!(frame.canvas_height, 164.0);
    assert_eq!(frame.visible_height, 80.0);
    assert_eq!(frame.bar_progress, 0.0);
    assert_eq!(frame.height_progress, 0.0);
    assert_eq!(frame.shoulder_progress, 0.0);
    assert_eq!(frame.drop_progress, 0.0);
    assert!(frame.cards_progress >= 1.0);
}

#[test]
fn close_transition_squares_top_corners_before_expanding_shoulders() {
    let close_delay_ms = 220;
    let contracting = resolve_close_transition_frame(
        close_delay_ms + PANEL_CLOSE_MORPH_DELAY_MS + 135,
        164.0,
        164.0,
        close_delay_ms,
        220,
    );
    assert!(contracting.bar_progress > 0.0);
    assert!(contracting.bar_progress < 1.0);
    assert_eq!(contracting.shoulder_progress, 1.0);

    let shoulder_frame = resolve_close_transition_frame(
        close_delay_ms + PANEL_CLOSE_SHOULDER_DELAY_MS + 60,
        164.0,
        164.0,
        close_delay_ms,
        220,
    );
    assert_eq!(shoulder_frame.bar_progress, 0.0);
    assert!(shoulder_frame.shoulder_progress > 0.0);
    assert!(shoulder_frame.shoulder_progress < 1.0);
}

#[test]
fn hover_emits_collapse_intent_during_transition_for_runtime_to_defer() {
    let now = Instant::now();
    let mut state = panel_state();
    state.expanded = true;
    state.transitioning = true;
    state.pointer_outside_since =
        Some(now - Duration::from_millis(HOVER_DELAY_MS.saturating_add(100)));

    let transition = sync_native_hover_expansion_state(&mut state, false, now);

    assert_eq!(transition, Some(NativeHoverTransition::Collapse));
    assert!(!state.expanded);
}

#[test]
fn hover_collapse_reuses_existing_timer_after_transition_finishes() {
    let now = Instant::now();
    let mut state = panel_state();
    state.expanded = true;
    state.transitioning = false;
    state.pointer_outside_since =
        Some(now - Duration::from_millis(HOVER_DELAY_MS.saturating_add(100)));

    let transition = sync_native_hover_expansion_state(&mut state, false, now);

    assert_eq!(transition, Some(NativeHoverTransition::Collapse));
    assert!(!state.expanded);
}

#[test]
fn runtime_defers_close_transition_while_animation_is_active() {
    let mut state = panel_state();
    state.transitioning = true;
    let snapshot = snapshot(1, 1);

    assert!(defer_native_panel_transition_in_state_if_active(
        &mut state,
        NativePanelTransitionRequest::Close,
        snapshot.clone(),
    ));

    assert_eq!(
        state.pending_transition,
        Some(super::panel_types::NativePanelPendingTransition {
            request: NativePanelTransitionRequest::Close,
            snapshot,
        })
    );
}

#[test]
fn runtime_defers_any_transition_while_animation_is_active() {
    let mut state = panel_state();
    state.transitioning = true;
    let snapshot = snapshot(1, 1);

    assert!(defer_native_panel_transition_in_state_if_active(
        &mut state,
        NativePanelTransitionRequest::Open,
        snapshot.clone(),
    ));

    assert_eq!(
        state.pending_transition,
        Some(super::panel_types::NativePanelPendingTransition {
            request: NativePanelTransitionRequest::Open,
            snapshot,
        })
    );
}

#[test]
fn runtime_does_not_defer_when_inactive() {
    let mut state = panel_state();
    let snapshot = snapshot(1, 1);

    assert!(!defer_native_panel_transition_in_state_if_active(
        &mut state,
        NativePanelTransitionRequest::Close,
        snapshot,
    ));

    assert!(state.pending_transition.is_none());
}

#[test]
fn runtime_clears_deferred_close_when_pointer_returns_inside() {
    let mut state = panel_state();
    state.pending_transition = Some(super::panel_types::NativePanelPendingTransition {
        request: NativePanelTransitionRequest::Close,
        snapshot: snapshot(1, 1),
    });

    assert!(clear_pending_native_panel_close_transition_in_state(
        &mut state
    ));
    assert!(state.pending_transition.is_none());
}

#[test]
fn native_hover_pill_rect_keeps_top_edge_stable_during_drop() {
    let panel_frame = NSRect::new(NSPoint::new(100.0, 200.0), NSSize::new(420.0, 80.0));
    let pill_frame = NSRect::new(NSPoint::new(80.0, 35.5), NSSize::new(253.0, 40.0));

    let hover_rect = native_hover_pill_rect(panel_frame, pill_frame);

    assert_eq!(hover_rect.origin.y, 235.5);
    assert_eq!(hover_rect.size.height, 44.5);
    assert_eq!(hover_rect.origin.y + hover_rect.size.height, 280.0);
}

#[test]
fn status_auto_hover_keeps_live_status_surface_open_outside() {
    let now = Instant::now();
    let mut state = panel_state();
    let live_snapshot = snapshot_with_permission("request-1", "session-1");
    assert!(sync_native_status_queue(&mut state, &live_snapshot).added_approvals > 0);
    state.expanded = true;
    state.status_auto_expanded = true;
    state.surface_mode = NativeExpandedSurface::Status;
    state.pointer_outside_since =
        Some(now - Duration::from_millis(HOVER_DELAY_MS.saturating_add(100)));

    let transition = sync_native_hover_expansion_state(&mut state, false, now);

    assert_eq!(transition, None);
    assert!(state.expanded);
    assert_eq!(state.surface_mode, NativeExpandedSurface::Status);
}
