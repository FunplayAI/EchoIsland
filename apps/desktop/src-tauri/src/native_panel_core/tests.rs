use std::time::{Duration, Instant};

use chrono::Utc;
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

use super::*;

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

fn pending_question(request_id: &str, session_id: &str) -> PendingQuestionView {
    PendingQuestionView {
        request_id: request_id.to_string(),
        session_id: session_id.to_string(),
        source: "claude".to_string(),
        header: Some("Pick one".to_string()),
        text: "Choose the deployment target".to_string(),
        options: vec!["Local".to_string(), "Staging".to_string()],
        requested_at: Utc::now(),
    }
}

fn snapshot_with_question(request_id: &str, session_id: &str) -> RuntimeSnapshot {
    let mut snapshot = snapshot(1, 1);
    let pending = pending_question(request_id, session_id);
    snapshot.pending_question_count = 1;
    snapshot.pending_question = Some(pending.clone());
    snapshot.pending_questions = vec![pending];
    snapshot.sessions = vec![session("WaitingQuestion")];
    snapshot
}

#[test]
fn pending_permission_card_clears_after_grace_window_expires() {
    let now = Instant::now();
    let previous = PendingPermissionCardState {
        request_id: "request-1".to_string(),
        payload: pending_permission("request-1", "session-1"),
        started_at: now - Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS),
        last_seen_at: now - Duration::from_millis(PENDING_CARD_RELEASE_GRACE_MS + 10),
        visible_until: now - Duration::from_millis(1),
    };

    assert!(resolve_pending_permission_card(None, Some(&previous), now).is_none());
}

#[test]
fn pending_card_grace_snapshot_does_not_readd_status_approval() {
    let mut state = PanelState::default();
    let live_snapshot = snapshot_with_permission("request-1", "session-1");
    let held_snapshot = sync_pending_card_visibility(&mut state, &live_snapshot);

    assert_eq!(held_snapshot.pending_permission_count, 1);
    state.last_raw_snapshot = Some(live_snapshot);

    let empty_snapshot = snapshot(0, 1);
    let held_after_resolve = sync_pending_card_visibility(&mut state, &empty_snapshot);

    assert_eq!(held_after_resolve.pending_permission_count, 1);
    assert_eq!(
        sync_status_queue(&mut state, &empty_snapshot).added_approvals,
        0
    );
    assert!(state.status_queue.is_empty());
}

#[test]
fn completion_badge_tracks_completed_session_until_new_dialogue() {
    let mut state = PanelState::default();
    let mut previous = snapshot(1, 1);
    previous.sessions = vec![session("Running")];

    let mut current = snapshot(0, 1);
    let mut completed = session("Idle");
    completed.last_assistant_message = Some("Done".to_string());
    current.sessions = vec![completed.clone()];

    let completed_session_ids = detect_completed_sessions(&previous, &current, Utc::now());
    sync_completion_badge(&mut state, &current, &completed_session_ids);

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

    sync_completion_badge(&mut state, &next, &[]);

    assert!(state.completion_badge_items.is_empty());
}

#[test]
fn status_surface_transition_switches_expanded_panel_into_status_mode() {
    let mut state = PanelState {
        expanded: true,
        status_queue: vec![StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now() + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        }],
        ..PanelState::default()
    };

    let transition = sync_status_surface_policy(
        &mut state,
        StatusQueueSyncResult {
            added_approvals: 1,
            added_questions: 0,
            added_completions: 0,
        },
    );

    assert_eq!(transition.panel_transition, None);
    assert!(transition.surface_transition);
    assert!(state.status_auto_expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Status);
}

#[test]
fn status_surface_policy_marks_new_status_for_reopen_during_close_transition() {
    let mut state = PanelState {
        expanded: false,
        transitioning: true,
        status_queue: vec![StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now() + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        }],
        surface_mode: ExpandedSurface::Default,
        ..PanelState::default()
    };

    let transition = sync_status_surface_policy(
        &mut state,
        StatusQueueSyncResult {
            added_approvals: 1,
            added_questions: 0,
            added_completions: 0,
        },
    );

    assert_eq!(transition.panel_transition, None);
    assert!(!transition.surface_transition);
    assert!(!state.expanded);
    assert!(state.status_auto_expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Status);
}

#[test]
fn pending_status_reopen_after_transition_expands_auto_status_surface_once() {
    let mut state = PanelState {
        expanded: false,
        transitioning: false,
        status_auto_expanded: true,
        surface_mode: ExpandedSurface::Status,
        status_queue: vec![StatusQueueItem {
            key: "question:question-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now() + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Question(pending_question("question-1", "session-1")),
        }],
        ..PanelState::default()
    };

    assert!(take_pending_status_reopen_after_transition(&mut state));
    assert!(state.expanded);
    assert!(!take_pending_status_reopen_after_transition(&mut state));
}

#[test]
fn snapshot_sync_emits_message_sound_and_panel_transition_for_new_status() {
    let mut state = PanelState::default();
    let raw_snapshot = snapshot_with_permission("request-1", "session-1");

    let result = sync_panel_snapshot_state(&mut state, &raw_snapshot, Utc::now());

    assert!(result.reminder.play_sound);
    assert!(result.reminder.show_status_card);
    assert_eq!(result.panel_transition, Some(true));
    assert!(!result.surface_transition);
    assert_eq!(result.displayed_snapshot.pending_permission_count, 1);
    assert!(state.last_raw_snapshot.is_some());
    assert_eq!(state.surface_mode, ExpandedSurface::Status);
}

#[test]
fn snapshot_sync_auto_expands_status_surface_for_new_question() {
    let mut state = PanelState::default();
    let raw_snapshot = snapshot_with_question("question-1", "session-1");

    let result = sync_panel_snapshot_state(&mut state, &raw_snapshot, Utc::now());

    assert!(result.reminder.play_sound);
    assert!(result.reminder.show_status_card);
    assert_eq!(result.panel_transition, Some(true));
    assert_eq!(state.surface_mode, ExpandedSurface::Status);
    assert!(state.status_queue.iter().any(|item| {
        matches!(
            &item.payload,
            StatusQueuePayload::Question(question) if question.request_id == "question-1"
        )
    }));
}

#[test]
fn status_surface_reverts_to_default_when_queue_drains() {
    let mut state = PanelState {
        expanded: true,
        status_auto_expanded: true,
        surface_mode: ExpandedSurface::Status,
        ..PanelState::default()
    };

    let transition = sync_status_surface_policy(&mut state, StatusQueueSyncResult::default());

    assert_eq!(transition.panel_transition, Some(false));
    assert!(!transition.surface_transition);
    assert!(!state.expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Default);
    assert!(state.skip_next_close_card_exit);
}

#[test]
fn settings_surface_toggle_cycles_between_default_and_settings() {
    let mut state = PanelState::default();

    assert!(toggle_settings_surface(&mut state));
    assert_eq!(state.surface_mode, ExpandedSurface::Settings);

    assert!(toggle_settings_surface(&mut state));
    assert_eq!(state.surface_mode, ExpandedSurface::Default);
}

#[test]
fn settings_row_actions_preserve_semantics() {
    assert_eq!(settings_row_action(0), Some(PanelHitAction::CycleDisplay));
    assert_eq!(
        settings_row_action(1),
        Some(PanelHitAction::ToggleCompletionSound)
    );
    assert_eq!(settings_row_action(2), Some(PanelHitAction::ToggleMascot));
    assert_eq!(
        settings_row_action(3),
        Some(PanelHitAction::OpenReleasePage)
    );
    assert_eq!(settings_row_action(4), None);
}

#[test]
fn preferred_panel_display_uses_key_before_index() {
    let displays = vec![
        panel_display_key(PanelDisplayGeometry {
            x: 0,
            y: 0,
            width: 1440,
            height: 900,
        }),
        panel_display_key(PanelDisplayGeometry {
            x: 1440,
            y: 0,
            width: 1512,
            height: 982,
        }),
    ];

    assert_eq!(
        resolve_preferred_panel_display_index(&displays, Some(&displays[1]), 0, Some(0)),
        Some(1)
    );
}

#[test]
fn preferred_panel_display_falls_back_to_index_then_main_then_first() {
    let displays = vec![
        "Display|0|0|1440|900".to_string(),
        "Display|1440|0|1512|982".to_string(),
    ];

    assert_eq!(
        resolve_preferred_panel_display_index(&displays, Some("missing"), 1, Some(0)),
        Some(1)
    );
    assert_eq!(
        resolve_preferred_panel_display_index(&displays, Some("missing"), 9, Some(1)),
        Some(1)
    );
    assert_eq!(
        resolve_preferred_panel_display_index(&displays, Some("missing"), 9, Some(9)),
        Some(0)
    );
    assert_eq!(
        resolve_preferred_panel_display_index(&[], Some("missing"), 0, Some(0)),
        None
    );
}

#[test]
fn click_action_prioritizes_settings_before_quit_and_cards() {
    let now = Instant::now();
    let resolution = resolve_panel_click_action(PanelClickInput {
        primary_click_started: true,
        expanded: true,
        transitioning: false,
        settings_button_hit: true,
        quit_button_hit: true,
        cards_visible: true,
        card_target: Some(PanelHitTarget {
            action: PanelHitAction::FocusSession,
            value: "session-1".to_string(),
        }),
        last_focus_click: None,
        now,
        focus_debounce_ms: 600,
    });

    assert_eq!(
        resolution.command,
        PanelInteractionCommand::ToggleSettingsSurface
    );
    assert_eq!(resolution.focus_click_to_record, None);
}

#[test]
fn click_action_allows_edge_actions_during_open_transition() {
    let now = Instant::now();
    let settings = resolve_panel_click_action(PanelClickInput {
        primary_click_started: true,
        expanded: true,
        transitioning: true,
        settings_button_hit: true,
        quit_button_hit: false,
        cards_visible: true,
        card_target: Some(PanelHitTarget {
            action: PanelHitAction::FocusSession,
            value: "session-1".to_string(),
        }),
        last_focus_click: None,
        now,
        focus_debounce_ms: 600,
    });
    let card = resolve_panel_click_action(PanelClickInput {
        primary_click_started: true,
        expanded: true,
        transitioning: true,
        settings_button_hit: false,
        quit_button_hit: false,
        cards_visible: true,
        card_target: Some(PanelHitTarget {
            action: PanelHitAction::FocusSession,
            value: "session-1".to_string(),
        }),
        last_focus_click: None,
        now,
        focus_debounce_ms: 600,
    });

    assert_eq!(
        settings.command,
        PanelInteractionCommand::ToggleSettingsSurface
    );
    assert_eq!(settings.focus_click_to_record, None);
    assert_eq!(card.command, PanelInteractionCommand::None);
    assert_eq!(card.focus_click_to_record, None);
}

#[test]
fn click_action_records_focus_session_and_suppresses_duplicates() {
    let now = Instant::now();
    let target = PanelHitTarget {
        action: PanelHitAction::FocusSession,
        value: "session-1".to_string(),
    };
    let first = resolve_panel_click_action(PanelClickInput {
        primary_click_started: true,
        expanded: true,
        transitioning: false,
        settings_button_hit: false,
        quit_button_hit: false,
        cards_visible: true,
        card_target: Some(target.clone()),
        last_focus_click: None,
        now,
        focus_debounce_ms: 600,
    });

    assert_eq!(
        first.command,
        PanelInteractionCommand::HitTarget(target.clone())
    );
    assert_eq!(first.focus_click_to_record, Some("session-1".to_string()));

    let duplicate = resolve_panel_click_action(PanelClickInput {
        primary_click_started: true,
        expanded: true,
        transitioning: false,
        settings_button_hit: false,
        quit_button_hit: false,
        cards_visible: true,
        card_target: Some(target),
        last_focus_click: Some(LastFocusClick {
            session_id: "session-1",
            clicked_at: now,
        }),
        now,
        focus_debounce_ms: 600,
    });

    assert_eq!(duplicate.command, PanelInteractionCommand::None);
    assert_eq!(duplicate.focus_click_to_record, None);
}

#[test]
fn hover_state_expands_after_inside_delay_and_clears_badges() {
    let now = Instant::now();
    let mut state = PanelState {
        pointer_inside_since: Some(now - Duration::from_millis(600)),
        completion_badge_items: vec![CompletionBadgeItem {
            session_id: "session-1".to_string(),
            completed_at: Utc::now(),
            last_user_prompt: None,
            last_assistant_message: Some("Done".to_string()),
        }],
        status_auto_expanded: true,
        surface_mode: ExpandedSurface::Status,
        ..PanelState::default()
    };

    let transition = sync_hover_expansion_state(&mut state, true, now, 500);

    assert_eq!(transition, Some(HoverTransition::Expand));
    assert!(state.expanded);
    assert!(state.completion_badge_items.is_empty());
    assert!(!state.status_auto_expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Default);
}

#[test]
fn hover_state_collapses_after_outside_delay_when_not_transitioning() {
    let now = Instant::now();
    let mut state = PanelState {
        expanded: true,
        pointer_outside_since: Some(now - Duration::from_millis(600)),
        ..PanelState::default()
    };

    let transition = sync_hover_expansion_state(&mut state, false, now, 500);

    assert_eq!(transition, Some(HoverTransition::Collapse));
    assert!(!state.expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Default);
}

#[test]
fn hover_state_reopens_during_close_transition_after_inside_delay() {
    let now = Instant::now();
    let mut state = PanelState {
        expanded: false,
        transitioning: true,
        pointer_inside_since: Some(now - Duration::from_millis(600)),
        ..PanelState::default()
    };

    let transition = sync_hover_expansion_state(&mut state, true, now, 500);

    assert_eq!(transition, Some(HoverTransition::Expand));
    assert!(state.expanded);
}

#[test]
fn hover_state_recloses_during_open_transition_after_outside_delay() {
    let now = Instant::now();
    let mut state = PanelState {
        expanded: true,
        transitioning: true,
        pointer_outside_since: Some(now - Duration::from_millis(600)),
        ..PanelState::default()
    };

    let transition = sync_hover_expansion_state(&mut state, false, now, 500);

    assert_eq!(transition, Some(HoverTransition::Collapse));
    assert!(!state.expanded);
}

#[test]
fn hover_state_keeps_auto_status_surface_open_outside() {
    let now = Instant::now();
    let mut state = PanelState {
        expanded: true,
        status_auto_expanded: true,
        surface_mode: ExpandedSurface::Status,
        pointer_outside_since: Some(now - Duration::from_millis(600)),
        status_queue: vec![StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: now + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        }],
        ..PanelState::default()
    };

    let transition = sync_hover_expansion_state(&mut state, false, now, 500);

    assert_eq!(transition, None);
    assert!(state.expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Status);
}

#[test]
fn status_surface_policy_switches_hover_expanded_panel_to_new_status_message() {
    let now = Instant::now();
    let mut state = PanelState {
        expanded: true,
        pointer_inside_since: Some(now - Duration::from_millis(HOVER_DELAY_MS + 100)),
        surface_mode: ExpandedSurface::Default,
        status_queue: vec![StatusQueueItem {
            key: "completion:session-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: now + Duration::from_secs(STATUS_COMPLETION_VISIBLE_SECONDS),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Completion(session("Idle")),
        }],
        ..PanelState::default()
    };

    let transition = sync_status_surface_policy(
        &mut state,
        StatusQueueSyncResult {
            added_approvals: 0,
            added_questions: 0,
            added_completions: 1,
        },
    );

    assert_eq!(transition.panel_transition, None);
    assert!(transition.surface_transition);
    assert!(state.expanded);
    assert!(state.status_auto_expanded);
    assert_eq!(state.surface_mode, ExpandedSurface::Status);
}

#[test]
fn compact_active_session_count_ignores_idle_sessions() {
    let mut snapshot = snapshot(2, 3);
    snapshot.sessions = vec![session("Running"), session("Idle"), session("Processing")];
    snapshot.sessions[1].session_id = "session-2".to_string();
    snapshot.sessions[2].session_id = "session-3".to_string();

    assert_eq!(compact_active_session_count(&snapshot), 2);
}

#[test]
fn mascot_base_state_preserves_priority_order() {
    let mut snapshot = snapshot(1, 1);
    snapshot.sessions = vec![session("Running")];

    assert_eq!(
        resolve_mascot_base_state(Some(&snapshot), false, false),
        PanelMascotBaseState::Running
    );

    assert_eq!(
        resolve_mascot_base_state(Some(&snapshot), false, true),
        PanelMascotBaseState::Complete
    );

    assert_eq!(
        resolve_mascot_base_state(Some(&snapshot), true, true),
        PanelMascotBaseState::MessageBubble
    );

    snapshot.pending_question_count = 1;
    assert_eq!(
        resolve_mascot_base_state(Some(&snapshot), true, true),
        PanelMascotBaseState::Question
    );

    snapshot.pending_permission_count = 1;
    assert_eq!(
        resolve_mascot_base_state(Some(&snapshot), true, true),
        PanelMascotBaseState::Approval
    );
}

#[test]
fn reminder_state_unifies_badge_glow_and_mascot_semantics() {
    let mut snapshot = snapshot(1, 1);
    snapshot.sessions = vec![session("Running")];
    let state = PanelState {
        completion_badge_items: vec![CompletionBadgeItem {
            session_id: "session-1".to_string(),
            completed_at: Utc::now(),
            last_user_prompt: None,
            last_assistant_message: Some("Done".to_string()),
        }],
        ..PanelState::default()
    };

    let reminder = resolve_panel_reminder_state(&state, Some(&snapshot));

    assert_eq!(reminder.completion_badge_count, 1);
    assert!(reminder.show_completion_glow);
    assert!(!reminder.has_status_completion);
    assert_eq!(reminder.mascot_base_state, PanelMascotBaseState::Complete);
}

#[test]
fn settings_surface_card_height_grows_with_row_count() {
    assert_eq!(resolve_settings_surface_card_height(4), 206.0);
    assert_eq!(resolve_settings_surface_card_height(5), 244.0);
}

#[test]
fn panel_style_resolver_hides_actions_before_threshold() {
    let resolved = resolve_panel_style(PanelStyleResolverInput {
        shell_visible: true,
        separator_visibility: 0.5,
        shared_visible: false,
        bar_progress: 0.3,
        height_progress: 0.0,
        headline_emphasized: true,
        edge_actions_visible: false,
        compact_pill_radius: 20.0,
        panel_morph_pill_radius: 24.0,
        expanded_panel_radius: 28.0,
    });

    assert!(resolved.highlight_alpha > 0.0);
    assert!(resolved.actions_hidden);
    assert_eq!(resolved.action_alpha, 0.0);
    assert!(!resolved.use_compact_corner_mask);
}

#[test]
fn panel_style_resolver_reveals_actions_and_morphs_shell() {
    let resolved = resolve_panel_style(PanelStyleResolverInput {
        shell_visible: true,
        separator_visibility: 0.5,
        shared_visible: true,
        bar_progress: 1.0,
        height_progress: 1.0,
        headline_emphasized: false,
        edge_actions_visible: true,
        compact_pill_radius: 20.0,
        panel_morph_pill_radius: 24.0,
        expanded_panel_radius: 28.0,
    });

    assert!(resolved.cards_hidden);
    assert!(!resolved.actions_hidden);
    assert_eq!(resolved.action_alpha, 1.0);
    assert_eq!(resolved.action_scale, 1.0);
    assert_eq!(resolved.pill_corner_radius, 24.0);
    assert_eq!(resolved.pill_border_width, 0.0);
    assert_eq!(resolved.expanded_corner_radius, 28.0);
    assert!(!resolved.use_compact_corner_mask);
}

#[test]
fn render_progress_clamps_transition_frame_values() {
    let progress = resolve_panel_render_progress(PanelTransitionFrame {
        canvas_height: 120.0,
        visible_height: 120.0,
        bar_progress: -0.4,
        height_progress: 1.4,
        shoulder_progress: 0.5,
        drop_progress: 2.0,
        cards_progress: 0.0,
    });

    assert_eq!(
        progress,
        PanelRenderProgress {
            bar: 0.0,
            height: 1.0,
            shoulder: 0.5,
            drop: 1.0,
        }
    );
}

#[test]
fn centered_top_frame_snaps_geometry_to_whole_points() {
    let frame = resolve_centered_top_frame(
        PanelRect {
            x: 0.0,
            y: 0.0,
            width: 1512.0,
            height: 982.0,
        },
        PanelSize {
            width: 419.6,
            height: 152.4,
        },
    );

    assert_eq!(
        frame,
        PanelRect {
            x: 546.0,
            y: 830.0,
            width: 420.0,
            height: 152.0,
        }
    );
}

#[test]
fn centered_top_frame_preserves_left_edge_when_panel_is_wider_than_screen() {
    let frame = resolve_centered_top_frame(
        PanelRect {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 800.0,
        },
        PanelSize {
            width: 420.0,
            height: 80.0,
        },
    );

    assert_eq!(frame.x, 100.0);
    assert_eq!(frame.y, 920.0);
    assert_eq!(frame.width, 420.0);
    assert_eq!(frame.height, 80.0);
}

#[test]
fn rect_helpers_compose_and_hit_test_without_platform_types() {
    let parent = PanelRect {
        x: 100.0,
        y: 200.0,
        width: 420.0,
        height: 180.0,
    };
    let child = PanelRect {
        x: 20.0,
        y: 12.0,
        width: 80.0,
        height: 30.0,
    };
    let absolute = absolute_rect(parent, child);

    assert_eq!(
        absolute,
        PanelRect {
            x: 120.0,
            y: 212.0,
            width: 80.0,
            height: 30.0,
        }
    );
    assert_eq!(compose_local_rect(parent, child), absolute);
    assert!(point_in_rect(PanelPoint { x: 200.0, y: 242.0 }, absolute));
    assert!(!point_in_rect(PanelPoint { x: 200.1, y: 242.0 }, absolute));
}

#[test]
fn rect_nearly_equal_uses_tolerance_for_all_edges() {
    assert!(rects_nearly_equal(
        PanelRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        },
        PanelRect {
            x: 10.4,
            y: 20.4,
            width: 30.4,
            height: 40.4,
        },
        0.5,
    ));
    assert!(!rects_nearly_equal(
        PanelRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        },
        PanelRect {
            x: 10.5,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        },
        0.5,
    ));
}

#[test]
fn island_bar_frame_interpolates_width_and_preserves_top_offset() {
    let compact = resolve_island_bar_frame(
        PanelSize {
            width: 420.0,
            height: 164.0,
        },
        0.0,
        253.0,
        366.0,
        40.0,
        4.5,
    );
    let expanded = resolve_island_bar_frame(
        PanelSize {
            width: 420.0,
            height: 164.0,
        },
        1.0,
        253.0,
        366.0,
        40.0,
        4.5,
    );

    assert_eq!(compact.width, 253.0);
    assert_eq!(expanded.width, 366.0);
    assert_eq!(compact.y, 119.5);
    assert_eq!(expanded.y, 119.5);
}

#[test]
fn expanded_background_frame_clamps_height_and_interpolates_width() {
    let frame = resolve_expanded_background_frame(
        PanelSize {
            width: 420.0,
            height: 164.0,
        },
        500.0,
        0.5,
        1.0,
        253.0,
        366.0,
        40.0,
        4.5,
        80.0,
    );

    assert_eq!(frame.width, 309.5);
    assert_eq!(frame.height, 159.5);
    assert_eq!(frame.y, 0.0);
}

#[test]
fn expanded_body_frames_match_insets() {
    let container = PanelRect {
        x: 12.0,
        y: 0.0,
        width: 366.0,
        height: 164.0,
    };
    let cards = resolve_expanded_cards_frame(container, 40.0, 8.0, 10.0, 14.0);
    let separator = resolve_expanded_separator_frame(container, 40.0, 14.0);

    assert_eq!(
        cards,
        PanelRect {
            x: 14.0,
            y: 10.0,
            width: 338.0,
            height: 106.0,
        }
    );
    assert_eq!(
        separator,
        PanelRect {
            x: 14.0,
            y: 123.5,
            width: 338.0,
            height: 1.0,
        }
    );
}

#[test]
fn compact_bar_content_layout_centers_headline_and_keeps_counts_trailing() {
    let compact = resolve_compact_bar_content_layout(CompactBarContentLayoutInput {
        bar_width: 253.0,
        bar_height: 37.0,
    });
    let expanded = resolve_compact_bar_content_layout(CompactBarContentLayoutInput {
        bar_width: 283.0,
        bar_height: 37.0,
    });

    assert_eq!(compact.mascot_center_x, 22.0);
    assert_eq!(compact.headline_x, 48.5);
    assert_eq!(compact.headline_width, 156.0);
    assert_eq!(compact.headline_center_x, 126.5);
    assert_eq!(compact.active_x, 208.0);
    assert_eq!(compact.slash_x, 217.0);
    assert_eq!(compact.total_x, 229.0);

    assert_eq!(expanded.headline_x, 63.5);
    assert_eq!(expanded.headline_width, 156.0);
    assert_eq!(expanded.headline_center_x, 141.5);
    assert_eq!(expanded.active_x, 238.0);
    assert_eq!(expanded.slash_x, 247.0);
    assert_eq!(expanded.total_x, 259.0);
}

#[test]
fn active_count_marquee_keeps_single_digit_static() {
    let frame = resolve_active_count_marquee_frame(ActiveCountMarqueeInput {
        text: "7",
        elapsed_ms: ACTIVE_COUNT_SCROLL_HOLD_MS + 120,
    });

    assert_eq!(frame.current, "7");
    assert_eq!(frame.next, "7");
    assert!(!frame.show_next);
    assert_eq!(frame.scroll_offset, 0.0);
}

#[test]
fn active_count_marquee_holds_then_scrolls_between_digits() {
    let held = resolve_active_count_marquee_frame(ActiveCountMarqueeInput {
        text: "23",
        elapsed_ms: ACTIVE_COUNT_SCROLL_HOLD_MS - 1,
    });
    let moving = resolve_active_count_marquee_frame(ActiveCountMarqueeInput {
        text: "23",
        elapsed_ms: ACTIVE_COUNT_SCROLL_HOLD_MS + (ACTIVE_COUNT_SCROLL_MOVE_MS / 2),
    });
    let moved = resolve_active_count_marquee_frame(ActiveCountMarqueeInput {
        text: "23",
        elapsed_ms: ACTIVE_COUNT_SCROLL_HOLD_MS + ACTIVE_COUNT_SCROLL_MOVE_MS + 1,
    });

    assert_eq!(held.current, "2");
    assert_eq!(held.next, "3");
    assert!(held.show_next);
    assert_eq!(held.scroll_offset, 0.0);
    assert_eq!(moving.current, "2");
    assert_eq!(moving.next, "3");
    assert!(moving.scroll_offset > 0.0);
    assert!(moving.scroll_offset < ACTIVE_COUNT_SCROLL_TRAVEL);
    assert_eq!(moved.scroll_offset, ACTIVE_COUNT_SCROLL_TRAVEL);
}

#[test]
fn mascot_visual_frame_animates_idle_breath_and_blink() {
    let idle = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Idle,
        elapsed_ms: 0,
    });
    let breathing = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Idle,
        elapsed_ms: 900,
    });
    let blinking = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Idle,
        elapsed_ms: 4535,
    });

    assert_eq!(idle.offset_y, 0.0);
    assert!(breathing.scale_x > idle.scale_x);
    assert!(blinking.eye_open < idle.eye_open);
}

#[test]
fn mascot_visual_frame_gives_message_bubble_a_visible_bob() {
    let start = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::MessageBubble,
        elapsed_ms: 0,
    });
    let bobbing = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::MessageBubble,
        elapsed_ms: 480,
    });

    assert!(bobbing.offset_y > start.offset_y);
    assert!(bobbing.scale_x >= 1.0);
    assert_eq!(bobbing.eye_open, 1.0);
}

#[test]
fn mascot_visual_frame_matches_mac_motion_phase_and_horizontal_sway() {
    let running = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Running,
        elapsed_ms: 250,
    });
    let message = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::MessageBubble,
        elapsed_ms: 0,
    });

    assert!(running.offset_x.abs() > 0.01);
    assert!((message.offset_y - 0.8).abs() < 0.001);
}

#[test]
fn mascot_visual_frame_uses_mac_style_state_specific_blink_floor() {
    let elapsed_ms = 4535;
    let idle = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Idle,
        elapsed_ms,
    });
    let running = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Running,
        elapsed_ms,
    });
    let approval = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Approval,
        elapsed_ms,
    });
    let question = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Question,
        elapsed_ms,
    });
    let complete = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Complete,
        elapsed_ms,
    });

    assert!(idle.eye_open < 0.2);
    assert!(running.eye_open >= 0.72);
    assert!(approval.eye_open >= 0.34);
    assert!(question.eye_open >= 0.48);
    assert!(complete.eye_open >= 0.72);
}

#[test]
fn mascot_visual_frame_supports_mac_sleepy_and_wake_angry_states() {
    let sleepy_start = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Sleepy,
        elapsed_ms: 0,
    });
    let sleepy_nod = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Sleepy,
        elapsed_ms: 4550,
    });
    let wake_start = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::WakeAngry,
        elapsed_ms: 0,
    });
    let wake_faded = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::WakeAngry,
        elapsed_ms: 900,
    });

    assert!(sleepy_start.scale_y < 0.96);
    assert!(sleepy_start.eye_open < 1.0);
    assert!(sleepy_nod.offset_y < 0.0);
    assert!(wake_start.offset_x.abs() < 0.001);
    assert!(wake_start.scale_x > 1.04);
    assert!(wake_faded.scale_x < wake_start.scale_x);
}

#[test]
fn mascot_visual_frame_transition_smoothsteps_motion_fields() {
    let start = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Idle,
        elapsed_ms: 0,
    });
    let target = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Running,
        elapsed_ms: 0,
    });
    let halfway = resolve_mascot_visual_frame_transition(MascotVisualFrameTransitionInput {
        start,
        target,
        elapsed_ms: 120,
        duration_ms: 240,
    });
    let done = resolve_mascot_visual_frame_transition(MascotVisualFrameTransitionInput {
        start,
        target,
        elapsed_ms: 240,
        duration_ms: 240,
    });

    assert!((halfway.scale_x - ((start.scale_x + target.scale_x) / 2.0)).abs() < 0.001);
    assert!(halfway.shadow_opacity > start.shadow_opacity);
    assert!(halfway.shadow_opacity < target.shadow_opacity);
    assert_eq!(done, target);
}

#[test]
fn mascot_visual_frame_transition_zero_duration_jumps_to_target() {
    let start = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Idle,
        elapsed_ms: 0,
    });
    let target = resolve_mascot_visual_frame(MascotVisualFrameInput {
        state: PanelMascotBaseState::Complete,
        elapsed_ms: 300,
    });

    assert_eq!(
        resolve_mascot_visual_frame_transition(MascotVisualFrameTransitionInput {
            start,
            target,
            elapsed_ms: 0,
            duration_ms: 0,
        }),
        target
    );
}

#[test]
fn panel_layout_clamps_visible_height_and_resolves_child_frames() {
    let layout = resolve_panel_layout(PanelLayoutInput {
        screen_frame: PanelRect {
            x: 0.0,
            y: 0.0,
            width: 1512.0,
            height: 982.0,
        },
        metrics: PanelGeometryMetrics {
            compact_height: 38.0,
            compact_width: 126.0,
            expanded_width: 356.0,
            panel_width: 356.0,
        },
        canvas_height: 120.0,
        visible_height: 220.0,
        bar_progress: 1.0,
        height_progress: 1.0,
        drop_progress: 0.0,
        content_visibility: 1.0,
        collapsed_height: 80.0,
        drop_distance: 8.0,
        content_top_gap: 8.0,
        content_bottom_inset: 10.0,
        cards_side_inset: 14.0,
        shoulder_size: 6.0,
        separator_side_inset: 14.0,
    });

    assert_eq!(layout.panel_frame.x, 578.0);
    assert_eq!(layout.content_frame.height, 120.0);
    assert_eq!(layout.expanded_frame.height, 120.0);
    assert_eq!(layout.cards_frame.height, 64.0);
    assert_eq!(layout.left_shoulder_frame.x, -6.0);
    assert_eq!(layout.right_shoulder_frame.x, 356.0);
    assert!(layout.shell_visible);
    assert_eq!(layout.separator_visibility, 0.88);
}

#[test]
fn panel_screen_widths_match_non_camera_housing_defaults() {
    let input = PanelScreenWidthInput {
        top_area: PanelScreenTopArea {
            screen_width: 1440.0,
            auxiliary_left_width: 0.0,
            auxiliary_right_width: 0.0,
        },
        compact_height: 37.0,
        default_compact_width: 253.0,
        expanded_width_delta: 30.0,
        default_expanded_width: 283.0,
        default_canvas_width: 420.0,
    };

    assert!(!resolve_panel_screen_has_camera_housing(input.top_area));
    assert_eq!(resolve_panel_notch_width(input.top_area), 240.0);
    assert_eq!(resolve_panel_shell_width(input), 253.0);
    assert_eq!(resolve_panel_expanded_width(input), 283.0);
    assert_eq!(resolve_panel_canvas_width(input), 420.0);
}

#[test]
fn panel_screen_widths_expand_around_camera_housing() {
    let input = PanelScreenWidthInput {
        top_area: PanelScreenTopArea {
            screen_width: 1512.0,
            auxiliary_left_width: 651.0,
            auxiliary_right_width: 651.0,
        },
        compact_height: 37.0,
        default_compact_width: 253.0,
        expanded_width_delta: 30.0,
        default_expanded_width: 283.0,
        default_canvas_width: 420.0,
    };

    assert!(resolve_panel_screen_has_camera_housing(input.top_area));
    assert_eq!(resolve_panel_notch_width(input.top_area), 210.0);
    assert_eq!(resolve_panel_shell_width(input), 320.144);
    assert_eq!(resolve_panel_expanded_width(input), 350.144);
    assert_eq!(resolve_panel_canvas_width(input), 420.0);
}

#[test]
fn panel_screen_width_fallbacks_preserve_existing_clamps() {
    assert_eq!(resolve_fallback_panel_expanded_width(500.0, 253.0), 253.0);
    assert_eq!(resolve_fallback_panel_expanded_width(200.0, 253.0), 200.0);
    assert_eq!(resolve_fallback_panel_expanded_width(0.0, 253.0), 1.0);
    assert_eq!(resolve_fallback_panel_canvas_width(300.0, 420.0), 420.0);
    assert_eq!(resolve_fallback_panel_canvas_width(500.0, 420.0), 500.0);
}

#[test]
fn expanded_cards_width_never_goes_negative() {
    assert_eq!(resolve_expanded_cards_width(366.0, 14.0), 338.0);
    assert_eq!(resolve_expanded_cards_width(20.0, 14.0), 0.0);
}

#[test]
fn native_panel_host_frame_interpolates_width_and_uses_canvas_height() {
    let frame = resolve_native_panel_host_frame(
        PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.2,
            visible_height: 140.0,
            width_progress: 0.5,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        },
        PanelRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        },
        400.0,
        700.0,
    );

    assert_eq!(
        frame,
        PanelRect {
            x: 445.0,
            y: 720.0,
            width: 550.0,
            height: 180.0,
        }
    );
}

#[test]
fn expanded_total_height_prefers_larger_shared_height_and_caps_body() {
    assert_eq!(
        resolve_expanded_total_height(84.0, Some(124.0), 40.0, 8.0, 10.0, 220.0),
        182.0
    );
    assert_eq!(
        resolve_expanded_total_height(84.0, Some(260.0), 40.0, 8.0, 10.0, 220.0),
        278.0
    );
}

#[test]
fn panel_transition_canvas_height_uses_largest_height() {
    assert_eq!(
        resolve_panel_transition_canvas_height(80.0, 164.0, 80.0),
        164.0
    );
    assert_eq!(
        resolve_panel_transition_canvas_height(196.0, 80.0, 80.0),
        196.0
    );
    assert_eq!(
        resolve_panel_transition_canvas_height(20.0, 30.0, 80.0),
        80.0
    );
}

#[test]
fn next_stacked_card_frame_applies_gap_and_overhang() {
    let mut cursor_y = 180.0;
    let first = resolve_next_stacked_card_frame(&mut cursor_y, false, 60.0, 320.0, 12.0, 4.0);
    let second = resolve_next_stacked_card_frame(&mut cursor_y, true, 70.0, 320.0, 12.0, 4.0);
    let missing = resolve_next_stacked_card_frame(&mut cursor_y, true, 80.0, 320.0, 12.0, 4.0);

    assert_eq!(
        first,
        Some(PanelRect {
            x: -4.0,
            y: 120.0,
            width: 328.0,
            height: 60.0,
        })
    );
    assert_eq!(
        second,
        Some(PanelRect {
            x: -4.0,
            y: 38.0,
            width: 328.0,
            height: 70.0,
        })
    );
    assert_eq!(missing, None);
    assert_eq!(cursor_y, 26.0);
}

fn test_card_metrics() -> PanelCardMetricConstants {
    PanelCardMetricConstants {
        card_inset_x: 10.0,
        chat_prefix_width: 15.0,
        chat_line_height: 14.0,
        header_height: 52.0,
        content_bottom_inset: 6.0,
        chat_gap: 4.0,
        tool_gap: 7.0,
        pending_action_y: 9.0,
        pending_action_height: 18.0,
        pending_action_gap: 6.0,
    }
}

#[test]
fn card_metrics_estimate_text_and_body_height() {
    let metrics = test_card_metrics();

    assert_eq!(resolve_card_chat_body_width(390.0, metrics), 355.0);
    assert!((resolve_estimated_text_width("Aa, 中", 10.0) - 30.2).abs() < 0.0001);
    assert_eq!(
        resolve_estimated_chat_line_count("short\nsecond line", 60.0, 3),
        3
    );
    assert_eq!(
        resolve_estimated_chat_body_height("short\nsecond line", 60.0, 3, metrics),
        42.0
    );
}

#[test]
fn card_metrics_resolve_card_heights() {
    let metrics = test_card_metrics();

    assert_eq!(
        resolve_pending_like_card_height("needs approval", 92.0, 120.0, 355.0, metrics),
        105.0
    );
    assert_eq!(resolve_session_card_collapsed_height(100.0, true), 52.0);
    assert_eq!(resolve_session_card_collapsed_height(100.0, false), 58.0);

    let content_height = resolve_session_card_content_height(SessionCardContentInput {
        prompt: Some("prompt"),
        reply: Some("reply"),
        has_tool: true,
        default_body_width: 355.0,
        metrics,
    });
    assert_eq!(content_height, 67.0);
    assert_eq!(
        resolve_session_card_height(false, true, content_height, metrics),
        119.0
    );
    assert_eq!(
        resolve_session_card_height(true, true, content_height, metrics),
        58.0
    );
    assert_eq!(
        resolve_completion_card_height("Task complete", 355.0, metrics),
        76.0
    );
}

#[test]
fn stacked_cards_total_height_uses_empty_height_and_gap_sum() {
    assert_eq!(resolve_stacked_cards_total_height(&[], 12.0, 84.0), 84.0);
    assert_eq!(
        resolve_stacked_cards_total_height(&[92.0, 108.0, 76.0], 12.0, 84.0),
        300.0
    );
}

#[test]
fn panel_render_layer_style_state_preserves_render_flags() {
    let state = resolve_panel_render_layer_style_state(PanelRenderLayerStyleInput {
        shell_visible: true,
        separator_visibility: 0.42,
        shared_visible: false,
        bar_progress: 0.7,
        height_progress: 0.8,
        shoulder_progress: 0.25,
        headline_emphasized: true,
        edge_actions_visible: true,
    });

    assert_eq!(
        state,
        PanelRenderLayerStyleState {
            shell_visible: true,
            separator_visibility: 0.42,
            shared_visible: false,
            bar_progress: 0.7,
            height_progress: 0.8,
            shoulder_progress: 0.25,
            headline_emphasized: true,
            edge_actions_visible: true,
        }
    );
}

#[test]
fn shared_body_height_decision_ignores_sub_threshold_updates() {
    let decision = resolve_shared_body_height_decision(SharedBodyHeightDecisionInput {
        current_height: Some(120.0),
        requested_height: 120.4,
        has_snapshot: true,
        update_threshold: 1.0,
    });

    assert_eq!(
        decision,
        SharedBodyHeightDecision {
            next_height: 120.4,
            should_update: false,
            should_rerender: false,
        }
    );
}

#[test]
fn shared_body_height_decision_clamps_and_rerenders_when_snapshot_exists() {
    let decision = resolve_shared_body_height_decision(SharedBodyHeightDecisionInput {
        current_height: Some(12.0),
        requested_height: -4.0,
        has_snapshot: true,
        update_threshold: 1.0,
    });

    assert_eq!(
        decision,
        SharedBodyHeightDecision {
            next_height: 0.0,
            should_update: true,
            should_rerender: true,
        }
    );
}

#[test]
fn status_queue_sorting_keeps_approvals_first_and_completions_after() {
    let now = Instant::now();
    let earlier = Utc::now() - chrono::Duration::seconds(10);
    let middle = Utc::now() - chrono::Duration::seconds(5);
    let later = Utc::now();
    let mut items = vec![
        StatusQueueItem {
            key: "completion:session-2".to_string(),
            session_id: "session-2".to_string(),
            sort_time: later,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Completion(session("Idle")),
        },
        StatusQueueItem {
            key: "approval:request-2".to_string(),
            session_id: "session-2".to_string(),
            sort_time: later,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-2", "session-2")),
        },
        StatusQueueItem {
            key: "question:question-1".to_string(),
            session_id: "session-3".to_string(),
            sort_time: middle,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Question(pending_question("question-1", "session-3")),
        },
        StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: earlier,
            expires_at: now,
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        },
    ];

    items.sort_by(compare_status_queue_items);

    assert_eq!(items[0].key, "approval:request-1");
    assert_eq!(items[1].key, "question:question-1");
    assert_eq!(items[2].key, "approval:request-2");
    assert!(matches!(
        items[3].payload,
        StatusQueuePayload::Completion(_)
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
fn animation_timeline_samples_match_existing_transition_descriptors() {
    let open = PanelAnimationTimeline::open(80.0, 164.0, 3);
    assert_eq!(
        open.total_ms(),
        PANEL_OPEN_TOTAL_MS
            + card_transition_total_ms(3, PANEL_CARD_REVEAL_MS, PANEL_CARD_REVEAL_STAGGER_MS)
    );
    assert_eq!(
        open.sample(120),
        resolve_open_transition_descriptor(
            120,
            panel_transition_canvas_height(80.0, 164.0),
            164.0,
            card_transition_total_ms(3, PANEL_CARD_REVEAL_MS, PANEL_CARD_REVEAL_STAGGER_MS),
        )
    );

    let surface_switch = PanelAnimationTimeline::surface_switch(120.0, 164.0, 2);
    assert_eq!(
        surface_switch.sample(80),
        resolve_surface_switch_transition_descriptor(
            80,
            panel_transition_canvas_height(120.0, 164.0),
            120.0,
            164.0,
            card_transition_total_ms(
                2,
                PANEL_SURFACE_SWITCH_CARD_REVEAL_MS,
                PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS
            ),
        )
    );

    let close = PanelAnimationTimeline::close(164.0, 2);
    assert_eq!(
        close.total_ms(),
        card_transition_total_ms(2, PANEL_CARD_EXIT_MS, PANEL_CARD_EXIT_STAGGER_MS)
            + PANEL_CARD_EXIT_SETTLE_MS
            + PANEL_CLOSE_TOTAL_MS
    );
    assert_eq!(close.sample(0).kind, PanelAnimationKind::Close);
}

#[test]
fn animation_descriptor_clamps_transition_values_and_preserves_kind() {
    let descriptor = resolve_panel_animation_descriptor(
        PanelAnimationKind::Open,
        PanelTransitionFrame {
            canvas_height: 180.0,
            visible_height: 140.0,
            bar_progress: 1.4,
            height_progress: -0.2,
            shoulder_progress: 0.5,
            drop_progress: 2.0,
            cards_progress: -1.0,
        },
    );

    assert_eq!(descriptor.kind, PanelAnimationKind::Open);
    assert_eq!(descriptor.canvas_height, 180.0);
    assert_eq!(descriptor.visible_height, 140.0);
    assert_eq!(descriptor.width_progress, 1.0);
    assert_eq!(descriptor.height_progress, 0.0);
    assert_eq!(descriptor.shoulder_progress, 0.5);
    assert_eq!(descriptor.drop_progress, 1.0);
    assert_eq!(descriptor.cards_progress, 0.0);
}

#[test]
fn panel_cards_visibility_progress_treats_close_progress_as_exit_progress() {
    let close = resolve_panel_animation_descriptor(
        PanelAnimationKind::Close,
        PanelTransitionFrame {
            canvas_height: 180.0,
            visible_height: 140.0,
            bar_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 0.25,
        },
    );
    let open = resolve_panel_animation_descriptor(
        PanelAnimationKind::Open,
        PanelTransitionFrame {
            cards_progress: 0.25,
            ..PanelTransitionFrame {
                canvas_height: 180.0,
                visible_height: 140.0,
                bar_progress: 1.0,
                height_progress: 1.0,
                shoulder_progress: 1.0,
                drop_progress: 1.0,
                cards_progress: 0.0,
            }
        },
    );

    assert_eq!(resolve_panel_cards_visibility_progress(close), 0.75);
    assert_eq!(resolve_panel_cards_visibility_progress(open), 0.25);
}
