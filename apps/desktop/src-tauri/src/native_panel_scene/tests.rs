use std::time::Instant;

use chrono::Utc;
use echoisland_runtime::{PendingPermissionView, RuntimeSnapshot, SessionSnapshotView};

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
        display_count: 3,
        settings: PanelSettingsState {
            selected_display_index: 1,
            completion_sound_enabled: false,
            mascot_enabled: true,
        },
        app_version: "0.2.0".to_string(),
    };

    let scene = build_panel_scene(&state, &snapshot(0, 0), &input);

    let SceneCard::Settings { version, rows, .. } = &scene.cards[0] else {
        panic!("expected settings card");
    };
    assert!(scene.compact_bar.actions_visible);
    assert_eq!(version.text, "v0.2.0");
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].value.text, "Screen 2/3");
    assert_eq!(rows[1].value.text, "Off");
    assert_eq!(rows[2].value.text, "On");
    assert_eq!(rows[3].action, PanelHitAction::OpenReleasePage);
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
    assert!(shell.edge_actions_visible);
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
                text: "v0.2.0".to_string(),
                emphasized: false,
            },
            rows: Vec::new(),
        }),
        SceneCardHeightInput::Settings
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
        cards: vec![SceneCard::Empty, SceneCard::Empty, SceneCard::Empty],
        glow: None,
        mascot_pose: SceneMascotPose::Idle,
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
