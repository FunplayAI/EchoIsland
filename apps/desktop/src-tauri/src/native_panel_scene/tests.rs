use std::time::Instant;

use chrono::Utc;
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

use crate::native_panel_core::{
    CompletionBadgeItem, ExpandedSurface, PanelHitAction, PanelSettingsState, PanelState,
    StatusQueueItem, StatusQueuePayload,
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
        project_name: Some("EchoIsland".to_string()),
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
        tool_description: Some("Build scene".to_string()),
        last_user_prompt: None,
        last_assistant_message: Some("Done".to_string()),
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

fn pending_question(session_id: &str) -> PendingQuestionView {
    PendingQuestionView {
        request_id: "question-1".to_string(),
        session_id: session_id.to_string(),
        source: "claude".to_string(),
        header: Some("Pick one".to_string()),
        text: "Choose the deployment target".to_string(),
        options: vec![
            "Local".to_string(),
            "Staging".to_string(),
            "Production".to_string(),
            "Other".to_string(),
        ],
        requested_at: Utc::now(),
    }
}

#[test]
fn scene_builder_emits_compact_bar_content() {
    let mut snapshot = snapshot(2, 5);
    snapshot.sessions = vec![session("Running"), session("Processing")];
    snapshot.sessions[1].session_id = "session-2".to_string();

    let scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert_eq!(scene.compact_bar.headline.text, "2 active tasks");
    assert_eq!(scene.compact_bar.active_count, "2");
    assert_eq!(scene.compact_bar.total_count, "5");
    assert!(!scene.compact_bar.actions_visible);
}

#[test]
fn scene_builder_emits_default_status_surface_scene_cards() {
    let mut snapshot = snapshot(1, 1);
    let pending = pending_permission("request-1", "session-1");
    snapshot.pending_permission_count = 1;
    snapshot.pending_permission = Some(pending.clone());
    snapshot.pending_permissions = vec![pending];
    let mut codex = session("Running");
    codex.session_id = "session-2".to_string();
    codex.source = "codex".to_string();
    codex.last_activity = Utc::now() - chrono::Duration::seconds(20);
    snapshot.sessions = vec![codex];

    let scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert_eq!(
        scene.status_surface.display_mode,
        StatusSurfaceDisplayMode::DefaultStack
    );
    assert_eq!(scene.status_surface.cards.len(), 2);
    assert!(matches!(
        scene.status_surface.cards[0].kind,
        StatusCardSceneKind::Approval
    ));
    assert!(matches!(
        scene.status_surface.cards[1].kind,
        StatusCardSceneKind::PromptAssist
    ));
}

#[test]
fn scene_builder_emits_shared_session_surface_cards() {
    let mut snapshot = snapshot(2, 2);
    let mut running = session("Running");
    running.session_id = "session-1".to_string();
    running.last_user_prompt = Some("Open the logs".to_string());
    running.last_assistant_message = Some("Reading latest output".to_string());
    running.current_tool = Some("Read".to_string());
    running.tool_description = Some("Scanning log files".to_string());

    let mut idle = session("Idle");
    idle.session_id = "session-2".to_string();
    idle.last_user_prompt = None;
    idle.last_assistant_message = None;
    idle.current_tool = None;
    idle.tool_description = None;
    idle.last_activity = Utc::now() - chrono::Duration::minutes(20);

    snapshot.sessions = vec![running, idle];

    let scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert_eq!(scene.session_surface.cards.len(), 2);
    assert_eq!(scene.session_surface.cards[0].status_key, "running");
    assert_eq!(
        scene.session_surface.cards[0].user_line.as_deref(),
        Some("Open the logs")
    );
    assert_eq!(
        scene.session_surface.cards[0].assistant_line.as_deref(),
        Some("Reading latest output")
    );
    assert_eq!(
        scene.session_surface.cards[0].tool_name.as_deref(),
        Some("Read")
    );
    assert!(!scene.session_surface.cards[0].compact);
    assert!(scene.session_surface.cards[1].compact);
}

#[test]
fn scene_builder_emits_shared_surface_scene_state() {
    let mut snapshot = snapshot(1, 1);
    snapshot.sessions = vec![session("Running")];

    let default_scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );
    assert_eq!(default_scene.surface_scene.mode, SurfaceSceneMode::Default);
    assert_eq!(default_scene.surface_scene.headline_text, "1 active task");
    assert!(!default_scene.surface_scene.headline_emphasized);

    let status_scene = build_panel_scene(
        &PanelState {
            expanded: true,
            surface_mode: ExpandedSurface::Status,
            status_queue: vec![StatusQueueItem {
                key: "approval:request-1".to_string(),
                session_id: "session-1".to_string(),
                sort_time: Utc::now(),
                expires_at: Instant::now(),
                is_live: true,
                is_removing: false,
                remove_after: None,
                payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
            }],
            ..PanelState::default()
        },
        &snapshot,
        &PanelSceneBuildInput::default(),
    );
    assert_eq!(status_scene.surface_scene.mode, SurfaceSceneMode::Status);
    assert_eq!(status_scene.surface_scene.headline_text, "Approval waiting");
    assert!(status_scene.surface_scene.headline_emphasized);
    assert!(!status_scene.surface_scene.edge_actions_visible);
}

#[test]
fn scene_builder_uses_request_headline_for_mixed_approval_and_question_queue() {
    let snapshot = snapshot(1, 1);
    let scene = build_panel_scene(
        &PanelState {
            expanded: true,
            surface_mode: ExpandedSurface::Status,
            status_queue: vec![
                StatusQueueItem {
                    key: "approval:request-1".to_string(),
                    session_id: "session-1".to_string(),
                    sort_time: Utc::now(),
                    expires_at: Instant::now(),
                    is_live: true,
                    is_removing: false,
                    remove_after: None,
                    payload: StatusQueuePayload::Approval(pending_permission(
                        "request-1",
                        "session-1",
                    )),
                },
                StatusQueueItem {
                    key: "question:question-1".to_string(),
                    session_id: "session-2".to_string(),
                    sort_time: Utc::now(),
                    expires_at: Instant::now(),
                    is_live: true,
                    is_removing: false,
                    remove_after: None,
                    payload: StatusQueuePayload::Question(pending_question("session-2")),
                },
            ],
            ..PanelState::default()
        },
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert_eq!(scene.compact_bar.headline.text, "Requests waiting");
    assert_eq!(scene.surface_scene.headline_text, "Requests waiting");
    assert_eq!(scene.cards.len(), 2);
    assert!(matches!(scene.cards[0], SceneCard::StatusApproval { .. }));
    assert!(matches!(scene.cards[1], SceneCard::StatusQuestion { .. }));
}

#[test]
fn scene_builder_hides_edge_actions_for_status_message_surface() {
    let scene = build_panel_scene(
        &PanelState {
            expanded: true,
            surface_mode: ExpandedSurface::Status,
            status_queue: vec![StatusQueueItem {
                key: "completion:session-1".to_string(),
                session_id: "session-1".to_string(),
                sort_time: Utc::now(),
                expires_at: Instant::now(),
                is_live: true,
                is_removing: false,
                remove_after: None,
                payload: StatusQueuePayload::Completion(session("Idle")),
            }],
            ..PanelState::default()
        },
        &snapshot(1, 1),
        &PanelSceneBuildInput::default(),
    );

    assert!(!scene.compact_bar.actions_visible);
    assert!(!scene.surface_scene.edge_actions_visible);
}

#[test]
fn scene_builder_emits_queue_status_surface_scene_state() {
    let state = PanelState {
        expanded: false,
        surface_mode: ExpandedSurface::Status,
        completion_badge_items: vec![CompletionBadgeItem {
            session_id: "session-2".to_string(),
            completed_at: Utc::now(),
            last_user_prompt: None,
            last_assistant_message: Some("Done".to_string()),
        }],
        status_queue: vec![
            StatusQueueItem {
                key: "approval:request-1".to_string(),
                session_id: "session-1".to_string(),
                sort_time: Utc::now(),
                expires_at: Instant::now(),
                is_live: true,
                is_removing: false,
                remove_after: None,
                payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
            },
            StatusQueueItem {
                key: "completion:session-2".to_string(),
                session_id: "session-2".to_string(),
                sort_time: Utc::now(),
                expires_at: Instant::now(),
                is_live: false,
                is_removing: true,
                remove_after: Some(Instant::now()),
                payload: StatusQueuePayload::Completion(session("Idle")),
            },
        ],
        ..PanelState::default()
    };

    let scene = build_panel_scene(&state, &snapshot(1, 1), &PanelSceneBuildInput::default());

    assert_eq!(
        scene.status_surface.display_mode,
        StatusSurfaceDisplayMode::Queue
    );
    assert_eq!(scene.status_surface.cards.len(), 2);
    assert_eq!(scene.status_surface.queue_state.total_count, 2);
    assert_eq!(scene.status_surface.queue_state.live_count, 1);
    assert_eq!(scene.status_surface.queue_state.removing_count, 1);
    assert_eq!(scene.status_surface.completion_badge_count, 1);
    assert!(scene.status_surface.show_completion_glow);
}

#[test]
fn scene_builder_sanitizes_completion_message_for_compact_headline() {
    let mut completion = session("Idle");
    completion.last_assistant_message = Some(
        "apps/desktop/src-tauri/src/windows_native_panel/host_runtime.rs\nsrc/native_panel_renderer"
            .to_string(),
    );
    let state = PanelState {
        status_queue: vec![StatusQueueItem {
            key: "completion:session-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now(),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Completion(completion),
        }],
        ..PanelState::default()
    };

    let scene = build_panel_scene(&state, &snapshot(0, 1), &PanelSceneBuildInput::default());

    assert!(!scene.compact_bar.headline.text.contains('\n'));
    assert!(
        scene
            .compact_bar
            .headline
            .text
            .starts_with("apps/desktop/src-tauri")
    );
    assert!(scene.compact_bar.headline.text.chars().count() <= 42);
}

#[test]
fn scene_builder_formats_empty_compact_bar_content() {
    let mut snapshot = snapshot(0, 1);
    snapshot.sessions = vec![session("Idle")];

    let scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert_eq!(scene.compact_bar.headline.text, "No active tasks");
    assert_eq!(scene.compact_bar.active_count, "0");
    assert_eq!(scene.compact_bar.total_count, "1");
}

#[test]
fn scene_builder_emits_settings_rows_and_value_badges() {
    let state = PanelState {
        expanded: true,
        surface_mode: ExpandedSurface::Settings,
        ..PanelState::default()
    };
    let input = PanelSceneBuildInput {
        display_options: vec![
            crate::native_panel_scene::panel_display_option_state(
                0,
                "display-1",
                "Built-in",
                3024,
                1964,
            ),
            crate::native_panel_scene::panel_display_option_state(
                1,
                "display-2",
                "Studio Display",
                2560,
                1440,
            ),
            crate::native_panel_scene::panel_display_option_state(
                2,
                "display-3",
                "Projector",
                1920,
                1080,
            ),
        ],
        settings: PanelSettingsState {
            selected_display_index: 1,
            completion_sound_enabled: false,
            mascot_enabled: true,
            debug_mode_enabled: false,
        },
        app_version: "0.5.0".to_string(),
    };

    let scene = build_panel_scene(&state, &snapshot(0, 0), &input);

    let SceneCard::Settings { version, rows, .. } = &scene.cards[0] else {
        panic!("expected settings card");
    };
    assert!(scene.compact_bar.actions_visible);
    assert_eq!(version.text, "v0.5.0");
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].value.text, "Studio Display · 2560×1440");
    assert_eq!(rows[1].value.text, "Off");
    assert_eq!(rows[2].value.text, "On");
    assert_eq!(rows[3].action, PanelHitAction::OpenReleasePage);
    assert_eq!(scene.settings_surface.title, "Settings");
    assert_eq!(scene.settings_surface.version_text, "EchoIsland v0.5.0");
    assert_eq!(scene.settings_surface.rows[0].id, "island_display");
    assert_eq!(scene.settings_surface.rows[1].label, "Mute Sound");
    assert_eq!(scene.settings_surface.rows[1].checked, Some(true));
    assert_eq!(scene.settings_surface.rows[2].checked, Some(false));
    assert_eq!(scene.settings_surface.rows[3].id, "update");
    assert_eq!(scene.settings_surface.rows[3].label, "Update & Upgrade");
}

#[test]
fn shell_scene_state_exposes_compact_bar_runtime_semantics() {
    let state = PanelState {
        expanded: true,
        surface_mode: ExpandedSurface::Status,
        status_queue: vec![StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now(),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        }],
        ..PanelState::default()
    };

    let scene = build_panel_scene(&state, &snapshot(1, 1), &PanelSceneBuildInput::default());
    let shell = resolve_panel_shell_scene_state(&scene);

    assert!(shell.headline_emphasized);
    assert!(!shell.edge_actions_visible);
}

#[test]
fn scene_builder_emits_pending_and_session_card_descriptors() {
    let mut snapshot = snapshot(1, 1);
    let pending = pending_permission("request-1", "session-1");
    snapshot.pending_permission_count = 1;
    snapshot.pending_permission = Some(pending.clone());
    snapshot.pending_permissions = vec![pending];
    snapshot.sessions = vec![session("Running")];

    let scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert!(matches!(
        scene.cards[0],
        SceneCard::PendingPermission { .. }
    ));
    assert!(
        scene
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::Empty)
                || matches!(card, SceneCard::PendingPermission { .. }))
    );
    assert_eq!(scene.hit_targets[0].action, PanelHitAction::FocusSession);
}

#[test]
fn scene_builder_emits_prompt_assist_card_descriptor() {
    let mut snapshot = snapshot(1, 1);
    let mut codex = session("Running");
    codex.source = "codex".to_string();
    codex.last_activity = Utc::now() - chrono::Duration::seconds(20);
    snapshot.sessions = vec![codex];

    let scene = build_panel_scene(
        &PanelState::default(),
        &snapshot,
        &PanelSceneBuildInput::default(),
    );

    assert!(
        scene
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::PromptAssist { .. }))
    );
}

#[test]
fn shared_completion_status_card_scene_formats_platform_neutral_fields() {
    let scene = build_completion_status_card_scene(&session("Idle"));

    assert_eq!(scene.kind, StatusCardSceneKind::Completion);
    assert_eq!(scene.status_text, "Complete");
    assert_eq!(scene.source_text, "Claude");
    assert_eq!(scene.body, "Done");
    assert!(scene.action_hint.is_none());
}

#[test]
fn shared_pending_question_status_card_scene_uses_compact_option_hint() {
    let scene = build_pending_question_status_card_scene(&pending_question("session-1"));

    assert_eq!(scene.kind, StatusCardSceneKind::Question);
    assert_eq!(scene.status_text, "Question");
    assert_eq!(scene.source_text, "Claude");
    assert_eq!(
        scene.action_hint.as_deref(),
        Some("Local / Staging / Production / …")
    );
}

#[test]
fn shared_status_queue_scene_reuses_approval_card_builder() {
    let item = StatusQueueItem {
        key: "approval:request-1".to_string(),
        session_id: "session-1".to_string(),
        sort_time: Utc::now(),
        expires_at: Instant::now(),
        is_live: true,
        is_removing: false,
        remove_after: None,
        payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
    };

    let scene = build_status_queue_status_card_scene(&item);

    assert_eq!(scene.kind, StatusCardSceneKind::Approval);
    assert_eq!(scene.status_text, "Approval");
    assert_eq!(scene.source_text, "Claude");
}

#[test]
fn shared_status_queue_scene_reuses_completion_card_builder() {
    let item = StatusQueueItem {
        key: "completion:session-1".to_string(),
        session_id: "session-1".to_string(),
        sort_time: Utc::now(),
        expires_at: Instant::now(),
        is_live: true,
        is_removing: false,
        remove_after: None,
        payload: StatusQueuePayload::Completion(session("Idle")),
    };

    let scene = build_status_queue_status_card_scene(&item);

    assert_eq!(scene.kind, StatusCardSceneKind::Completion);
    assert_eq!(scene.status_text, "Complete");
    assert_eq!(scene.body, "Done");
}

#[test]
fn scene_builder_emits_status_and_completion_descriptors() {
    let state = PanelState {
        expanded: true,
        surface_mode: ExpandedSurface::Status,
        status_queue: vec![StatusQueueItem {
            key: "completion:session-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now(),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Completion(session("Idle")),
        }],
        ..PanelState::default()
    };

    let scene = build_panel_scene(&state, &snapshot(0, 1), &PanelSceneBuildInput::default());

    assert!(matches!(scene.cards[0], SceneCard::StatusCompletion { .. }));
    assert_eq!(scene.hit_targets[0].action, PanelHitAction::FocusSession);
}

#[test]
fn scene_builder_emits_settings_row_and_session_hit_targets() {
    let settings_state = PanelState {
        expanded: true,
        surface_mode: ExpandedSurface::Settings,
        ..PanelState::default()
    };
    let settings_scene = build_panel_scene(
        &settings_state,
        &snapshot(0, 0),
        &PanelSceneBuildInput::default(),
    );
    assert_eq!(settings_scene.hit_targets.len(), 4);
    assert_eq!(
        settings_scene.hit_targets[0].action,
        PanelHitAction::CycleDisplay
    );

    let mut default_snapshot = snapshot(1, 1);
    default_snapshot.sessions = vec![session("Running")];
    let default_scene = build_panel_scene(
        &PanelState::default(),
        &default_snapshot,
        &PanelSceneBuildInput::default(),
    );
    assert!(
        default_scene
            .hit_targets
            .iter()
            .any(|target| target.action == PanelHitAction::FocusSession)
    );
}

#[test]
fn scene_builder_emits_completion_glow_when_badge_is_waiting() {
    let state = PanelState {
        completion_badge_items: vec![CompletionBadgeItem {
            session_id: "session-1".to_string(),
            completed_at: Utc::now(),
            last_user_prompt: None,
            last_assistant_message: Some("Done".to_string()),
        }],
        ..PanelState::default()
    };

    let scene = build_panel_scene(&state, &snapshot(0, 0), &PanelSceneBuildInput::default());

    assert_eq!(
        scene.glow,
        Some(SceneGlow {
            style: SceneGlowStyle::Completion,
            opacity: 0.78
        })
    );
    assert_eq!(scene.mascot_pose, SceneMascotPose::Complete);
}

#[test]
fn scene_card_height_input_preserves_variant_payload_semantics() {
    let session = session("Running");
    let pending = pending_permission("request-1", "session-1");
    let status_item = StatusQueueItem {
        key: "completion:session-1".to_string(),
        session_id: "session-1".to_string(),
        sort_time: Utc::now(),
        expires_at: Instant::now(),
        is_live: true,
        is_removing: false,
        remove_after: None,
        payload: StatusQueuePayload::Completion(session.clone()),
    };

    assert!(matches!(
        resolve_scene_card_height_input(&SceneCard::Settings {
            title: "Settings".to_string(),
            version: SceneBadge {
                text: "v0.5.0".to_string(),
                emphasized: false,
            },
            rows: Vec::new(),
        }),
        SceneCardHeightInput::Settings { row_count: 0 }
    ));
    assert!(matches!(
        resolve_scene_card_height_input(&SceneCard::PendingPermission {
            pending: pending.clone(),
            count: 1,
        }),
        SceneCardHeightInput::PendingPermission(item) if item.request_id == pending.request_id
    ));
    assert!(matches!(
        resolve_scene_card_height_input(&SceneCard::PromptAssist {
            session: session.clone(),
        }),
        SceneCardHeightInput::PromptAssist(item) if item.session_id == session.session_id
    ));
    assert!(matches!(
        resolve_scene_card_height_input(&SceneCard::Session {
            session: session.clone(),
            title: "EchoIsland".to_string(),
            status: SceneBadge {
                text: "Running".to_string(),
                emphasized: false,
            },
            snippet: None,
        }),
        SceneCardHeightInput::Session(item) if item.session_id == session.session_id
    ));
    assert!(matches!(
        resolve_scene_card_height_input(&SceneCard::StatusCompletion {
            item: status_item.clone(),
        }),
        SceneCardHeightInput::StatusItem(item) if item.session_id == status_item.session_id
    ));
    assert!(matches!(
        resolve_scene_card_height_input(&SceneCard::Empty),
        SceneCardHeightInput::Empty
    ));
}

#[test]
fn scene_cards_total_height_delegates_card_height_resolution() {
    let scene = PanelScene {
        surface: ExpandedSurface::Default,
        compact_bar: CompactBarScene {
            headline: SceneText {
                text: "No active tasks".to_string(),
                emphasized: false,
            },
            active_count: "0".to_string(),
            total_count: "0".to_string(),
            completion_count: 0,
            actions_visible: false,
        },
        surface_scene: SurfaceScene {
            mode: SurfaceSceneMode::Default,
            headline_text: "No active tasks".to_string(),
            headline_emphasized: false,
            edge_actions_visible: false,
        },
        status_surface: StatusSurfaceScene {
            cards: Vec::new(),
            display_mode: StatusSurfaceDisplayMode::Hidden,
            default_state: StatusSurfaceDefaultState::default(),
            queue_state: StatusSurfaceQueueState::default(),
            completion_badge_count: 0,
            show_completion_glow: false,
        },
        session_surface: SessionSurfaceScene { cards: Vec::new() },
        settings_surface: build_settings_surface_scene(
            &[crate::native_panel_scene::fallback_panel_display_option()],
            PanelSettingsState::default(),
            env!("CARGO_PKG_VERSION"),
        ),
        cards: vec![SceneCard::Empty, SceneCard::Empty, SceneCard::Empty],
        glow: None,
        mascot_pose: SceneMascotPose::Idle,
        debug_mode_enabled: false,
        hit_targets: Vec::new(),
        nodes: Vec::new(),
    };

    assert_eq!(
        resolve_scene_cards_total_height(&scene, |_| 84.0, 12.0, 84.0),
        276.0
    );
    assert_eq!(
        resolve_scene_cards_total_height(
            &PanelScene {
                cards: Vec::new(),
                ..scene
            },
            |_| 84.0,
            12.0,
            84.0
        ),
        84.0
    );
}

#[test]
fn runtime_shell_scene_state_defaults_without_snapshot() {
    let state = PanelState {
        expanded: true,
        transitioning: true,
        ..PanelState::default()
    };

    assert_eq!(
        resolve_panel_shell_scene_state_for_runtime(&state, None, &PanelSceneBuildInput::default()),
        PanelShellSceneState::default()
    );
}

#[test]
fn runtime_shell_scene_state_uses_built_scene_when_snapshot_exists() {
    let state = PanelState {
        expanded: true,
        transitioning: true,
        status_queue: vec![StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now(),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        }],
        ..PanelState::default()
    };

    assert_eq!(
        resolve_panel_shell_scene_state_for_runtime(
            &state,
            Some(&snapshot(1, 1)),
            &PanelSceneBuildInput::default()
        ),
        PanelShellSceneState {
            headline_emphasized: true,
            edge_actions_visible: true,
        }
    );
}

#[test]
fn runtime_render_state_combines_transitioning_and_shell_scene() {
    let state = PanelState {
        expanded: true,
        transitioning: true,
        status_queue: vec![StatusQueueItem {
            key: "approval:request-1".to_string(),
            session_id: "session-1".to_string(),
            sort_time: Utc::now(),
            expires_at: Instant::now(),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: StatusQueuePayload::Approval(pending_permission("request-1", "session-1")),
        }],
        ..PanelState::default()
    };

    assert_eq!(
        resolve_panel_runtime_render_state(
            &state,
            Some(&snapshot(1, 1)),
            &PanelSceneBuildInput::default()
        ),
        PanelRuntimeRenderState {
            transitioning: true,
            shell_scene: PanelShellSceneState {
                headline_emphasized: true,
                edge_actions_visible: true,
            },
        }
    );
}
