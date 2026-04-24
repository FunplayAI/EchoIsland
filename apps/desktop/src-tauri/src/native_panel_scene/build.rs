use crate::native_panel_core::{
    ExpandedSurface, PanelHitAction, PanelMascotBaseState, PanelSettingsState, PanelState,
    StatusQueuePayload, compact_active_session_count, displayed_default_pending_permissions,
    displayed_default_pending_questions, displayed_prompt_assist_sessions, displayed_sessions,
    format_status, normalize_status, resolve_mascot_base_state, session_title,
    sync_panel_snapshot_state,
};
use chrono::{DateTime, Utc};
use echoisland_runtime::RuntimeSnapshot;
use std::time::Instant;

use super::PanelRuntimeRenderState;

use super::{
    CompactBarScene, PanelScene, SceneBadge, SceneCard, SceneGlow, SceneGlowStyle, SceneHitTarget,
    SceneMascotPose, SceneNode, SceneText, SessionSurfaceScene, SettingsRowScene,
    SettingsSurfaceScene, StatusSurfaceDefaultState, StatusSurfaceDisplayMode,
    StatusSurfaceQueueState, StatusSurfaceScene, SurfaceScene,
    build_pending_permission_status_card_scene, build_pending_question_status_card_scene,
    build_prompt_assist_status_card_scene, build_session_card_scene, build_settings_surface_scene,
    build_status_queue_status_card_scene, settings_surface_row_action, surface_scene_mode,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PanelSceneBuildInput {
    pub(crate) display_count: usize,
    pub(crate) settings: PanelSettingsState,
    pub(crate) app_version: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PanelRuntimeSceneBundle {
    pub(crate) scene: PanelScene,
    pub(crate) runtime_render_state: PanelRuntimeRenderState,
    pub(crate) displayed_snapshot: RuntimeSnapshot,
}

impl Default for PanelSceneBuildInput {
    fn default() -> Self {
        Self {
            display_count: 1,
            settings: PanelSettingsState::default(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

pub(crate) fn build_panel_scene(
    state: &PanelState,
    snapshot: &RuntimeSnapshot,
    input: &PanelSceneBuildInput,
) -> PanelScene {
    let compact_bar = build_compact_bar_scene(state, snapshot);
    let surface_scene = build_surface_scene(state, &compact_bar);
    let status_surface = build_status_surface_scene(state, snapshot);
    let session_surface = build_session_surface_scene(state, snapshot);
    let settings_surface =
        build_settings_surface_scene(input.display_count, input.settings, &input.app_version);
    let glow = build_completion_glow(state);
    let mascot_pose = build_mascot_pose(state, snapshot, input.settings.mascot_enabled);
    let mut cards = Vec::new();
    let mut hit_targets = Vec::new();

    match state.surface_mode {
        ExpandedSurface::Settings if state.expanded => {
            let settings = build_settings_card(&settings_surface);
            for row in settings_rows(&settings) {
                hit_targets.push(SceneHitTarget {
                    action: row.action,
                    value: String::new(),
                });
            }
            cards.push(settings);
        }
        ExpandedSurface::Status if !state.status_queue.is_empty() => {
            for item in &state.status_queue {
                match &item.payload {
                    StatusQueuePayload::Approval(_) => {
                        cards.push(SceneCard::StatusApproval { item: item.clone() });
                        hit_targets.push(SceneHitTarget {
                            action: PanelHitAction::FocusSession,
                            value: item.session_id.clone(),
                        });
                    }
                    StatusQueuePayload::Completion(_) => {
                        cards.push(SceneCard::StatusCompletion { item: item.clone() });
                        hit_targets.push(SceneHitTarget {
                            action: PanelHitAction::FocusSession,
                            value: item.session_id.clone(),
                        });
                    }
                }
            }
        }
        _ => {
            let pending_permissions = displayed_default_pending_permissions(snapshot);
            let pending_questions = displayed_default_pending_questions(snapshot);
            let prompt_assist_sessions = displayed_prompt_assist_sessions(snapshot);
            let sessions = displayed_sessions(snapshot, &prompt_assist_sessions);

            for pending in pending_permissions.iter().take(1) {
                cards.push(SceneCard::PendingPermission {
                    pending: pending.clone(),
                    count: snapshot.pending_permission_count.max(1),
                });
                hit_targets.push(SceneHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: pending.session_id.clone(),
                });
            }

            for pending in pending_questions.iter().take(1) {
                cards.push(SceneCard::PendingQuestion {
                    pending: pending.clone(),
                    count: snapshot.pending_question_count.max(1),
                });
                hit_targets.push(SceneHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: pending.session_id.clone(),
                });
            }

            for session in prompt_assist_sessions {
                cards.push(SceneCard::PromptAssist {
                    session: session.clone(),
                });
                hit_targets.push(SceneHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: session.session_id.clone(),
                });
            }

            for session in sessions {
                cards.push(SceneCard::Session {
                    session: session.clone(),
                    title: session_title(&session),
                    status: SceneBadge {
                        text: format_status(&session.status),
                        emphasized: normalize_status(&session.status) == "running",
                    },
                    snippet: session
                        .last_assistant_message
                        .clone()
                        .or(session.tool_description.clone()),
                });
                hit_targets.push(SceneHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: session.session_id.clone(),
                });
            }

            if cards.is_empty() {
                cards.push(SceneCard::Empty);
            }
        }
    }

    let mut nodes = vec![
        SceneNode::Text(compact_bar.headline.clone()),
        SceneNode::Text(SceneText {
            text: compact_bar.active_count.clone(),
            emphasized: false,
        }),
        SceneNode::Text(SceneText {
            text: compact_bar.total_count.clone(),
            emphasized: false,
        }),
        SceneNode::Mascot(mascot_pose),
    ];
    if let Some(glow) = glow.clone() {
        nodes.push(SceneNode::Glow(glow));
    }
    for card in &cards {
        nodes.push(SceneNode::Card(card.clone()));
    }

    PanelScene {
        surface: state.surface_mode,
        compact_bar,
        surface_scene,
        status_surface,
        session_surface,
        settings_surface,
        cards,
        glow,
        mascot_pose,
        hit_targets,
        nodes,
    }
}

pub(crate) fn build_panel_runtime_scene_bundle(
    panel_state: &PanelState,
    displayed_snapshot: &RuntimeSnapshot,
    input: &PanelSceneBuildInput,
) -> PanelRuntimeSceneBundle {
    let scene = build_panel_scene(panel_state, displayed_snapshot, input);
    let runtime_render_state =
        resolve_panel_runtime_render_state(panel_state, Some(displayed_snapshot), input);

    PanelRuntimeSceneBundle {
        scene,
        runtime_render_state,
        displayed_snapshot: displayed_snapshot.clone(),
    }
}

pub(crate) fn sync_panel_runtime_scene_bundle(
    panel_state: &mut PanelState,
    raw_snapshot: &RuntimeSnapshot,
    input: &PanelSceneBuildInput,
    now: DateTime<Utc>,
) -> PanelRuntimeSceneBundle {
    let sync_result = sync_panel_snapshot_state(panel_state, raw_snapshot, now);
    build_panel_runtime_scene_bundle(panel_state, &sync_result.displayed_snapshot, input)
}

fn build_surface_scene(state: &PanelState, compact_bar: &CompactBarScene) -> SurfaceScene {
    SurfaceScene {
        mode: surface_scene_mode(state.surface_mode),
        headline_text: compact_bar.headline.text.clone(),
        headline_emphasized: compact_bar.headline.emphasized,
        edge_actions_visible: compact_bar.actions_visible,
    }
}

fn build_session_surface_scene(
    state: &PanelState,
    snapshot: &RuntimeSnapshot,
) -> SessionSurfaceScene {
    let prompt_assist_sessions = displayed_prompt_assist_sessions(snapshot);
    let completion_session_ids = state
        .status_queue
        .iter()
        .filter_map(|item| match &item.payload {
            StatusQueuePayload::Completion(_) => Some(item.session_id.as_str()),
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>();

    SessionSurfaceScene {
        cards: displayed_sessions(snapshot, &prompt_assist_sessions)
            .into_iter()
            .map(|session| {
                let completion = completion_session_ids.contains(session.session_id.as_str());
                build_session_card_scene(&session, completion)
            })
            .collect(),
    }
}

fn build_status_surface_scene(
    state: &PanelState,
    snapshot: &RuntimeSnapshot,
) -> StatusSurfaceScene {
    if !state.status_queue.is_empty() {
        let now = Instant::now();
        return StatusSurfaceScene {
            cards: state
                .status_queue
                .iter()
                .map(build_status_queue_status_card_scene)
                .collect(),
            display_mode: StatusSurfaceDisplayMode::Queue,
            default_state: StatusSurfaceDefaultState::default(),
            queue_state: StatusSurfaceQueueState {
                total_count: state.status_queue.len(),
                live_count: state
                    .status_queue
                    .iter()
                    .filter(|item| item.is_live)
                    .count(),
                removing_count: state
                    .status_queue
                    .iter()
                    .filter(|item| item.is_removing)
                    .count(),
                next_transition_in_ms: state
                    .status_queue
                    .iter()
                    .filter_map(|item| {
                        if item.is_removing {
                            item.remove_after
                        } else {
                            Some(item.expires_at)
                        }
                    })
                    .filter(|transition_at| *transition_at > now)
                    .map(|transition_at| {
                        transition_at
                            .saturating_duration_since(now)
                            .as_millis()
                            .min(u64::MAX as u128) as u64
                    })
                    .min(),
            },
            completion_badge_count: state.completion_badge_items.len(),
            show_completion_glow: !state.completion_badge_items.is_empty() && !state.expanded,
        };
    }

    let mut cards = Vec::new();
    cards.extend(
        displayed_default_pending_permissions(snapshot)
            .into_iter()
            .take(1)
            .map(|pending| build_pending_permission_status_card_scene(&pending)),
    );
    cards.extend(
        displayed_default_pending_questions(snapshot)
            .into_iter()
            .take(1)
            .map(|pending| build_pending_question_status_card_scene(&pending)),
    );
    cards.extend(
        displayed_prompt_assist_sessions(snapshot)
            .into_iter()
            .map(|session| build_prompt_assist_status_card_scene(&session)),
    );

    StatusSurfaceScene {
        display_mode: if cards.is_empty() {
            StatusSurfaceDisplayMode::Hidden
        } else {
            StatusSurfaceDisplayMode::DefaultStack
        },
        cards,
        default_state: StatusSurfaceDefaultState {
            approval_count: snapshot.pending_permission_count,
            question_count: snapshot.pending_question_count,
            prompt_assist_count: displayed_prompt_assist_sessions(snapshot).len(),
        },
        queue_state: StatusSurfaceQueueState::default(),
        completion_badge_count: state.completion_badge_items.len(),
        show_completion_glow: !state.completion_badge_items.is_empty() && !state.expanded,
    }
}

pub(crate) fn resolve_panel_shell_scene_state_for_runtime(
    state: &PanelState,
    snapshot: Option<&RuntimeSnapshot>,
    input: &PanelSceneBuildInput,
) -> super::PanelShellSceneState {
    snapshot
        .map(|snapshot| resolve_panel_shell_scene_state(&build_panel_scene(state, snapshot, input)))
        .unwrap_or_default()
}

pub(crate) fn resolve_panel_runtime_render_state(
    state: &PanelState,
    snapshot: Option<&RuntimeSnapshot>,
    input: &PanelSceneBuildInput,
) -> super::PanelRuntimeRenderState {
    super::PanelRuntimeRenderState {
        transitioning: state.transitioning,
        shell_scene: resolve_panel_shell_scene_state_for_runtime(state, snapshot, input),
    }
}

pub(crate) fn resolve_panel_shell_scene_state(scene: &PanelScene) -> super::PanelShellSceneState {
    super::PanelShellSceneState {
        headline_emphasized: scene.compact_bar.headline.emphasized,
        edge_actions_visible: scene.compact_bar.actions_visible,
    }
}

fn build_compact_bar_scene(state: &PanelState, snapshot: &RuntimeSnapshot) -> CompactBarScene {
    let active_count = compact_active_session_count(snapshot);
    CompactBarScene {
        headline: SceneText {
            text: compact_headline(state, snapshot),
            emphasized: !state.status_queue.is_empty(),
        },
        active_count: if active_count == 0 {
            "0".to_string()
        } else {
            active_count.to_string()
        },
        total_count: snapshot.total_session_count.to_string(),
        completion_count: state.completion_badge_items.len(),
        actions_visible: state.expanded || state.transitioning,
    }
}

fn build_completion_glow(state: &PanelState) -> Option<SceneGlow> {
    if state.completion_badge_items.is_empty() || state.expanded {
        return None;
    }
    Some(SceneGlow {
        style: SceneGlowStyle::Completion,
        opacity: 0.78,
    })
}

fn build_mascot_pose(
    state: &PanelState,
    snapshot: &RuntimeSnapshot,
    mascot_enabled: bool,
) -> SceneMascotPose {
    if !mascot_enabled {
        return SceneMascotPose::Hidden;
    }
    let has_status_completion = state.expanded
        && state.surface_mode == ExpandedSurface::Status
        && state
            .status_queue
            .iter()
            .any(|item| matches!(item.payload, StatusQueuePayload::Completion(_)));
    match resolve_mascot_base_state(
        Some(snapshot),
        has_status_completion,
        !state.completion_badge_items.is_empty(),
    ) {
        PanelMascotBaseState::Idle => SceneMascotPose::Idle,
        PanelMascotBaseState::Running => SceneMascotPose::Running,
        PanelMascotBaseState::Approval => SceneMascotPose::Approval,
        PanelMascotBaseState::Question => SceneMascotPose::Question,
        PanelMascotBaseState::MessageBubble => SceneMascotPose::MessageBubble,
        PanelMascotBaseState::Complete => SceneMascotPose::Complete,
    }
}

fn build_settings_card(settings_surface: &SettingsSurfaceScene) -> SceneCard {
    SceneCard::Settings {
        title: settings_surface.title.clone(),
        version: SceneBadge {
            text: settings_surface
                .version_text
                .strip_prefix("EchoIsland ")
                .unwrap_or(&settings_surface.version_text)
                .to_string(),
            emphasized: false,
        },
        rows: settings_surface
            .rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                Some(SettingsRowScene {
                    title: row.label.clone(),
                    value: SceneBadge {
                        text: row.value_text.clone(),
                        emphasized: row.checked.unwrap_or(false),
                    },
                    action: settings_surface_row_action(index)?,
                })
            })
            .collect(),
    }
}

fn settings_rows(card: &SceneCard) -> &[SettingsRowScene] {
    match card {
        SceneCard::Settings { rows, .. } => rows.as_slice(),
        _ => &[],
    }
}

fn compact_headline(state: &PanelState, snapshot: &RuntimeSnapshot) -> String {
    let approval_count = state
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, StatusQueuePayload::Approval(_)))
        .count();
    if approval_count > 0 {
        return if approval_count > 1 {
            "Approvals waiting".to_string()
        } else {
            "Approval waiting".to_string()
        };
    }

    let completion_count = state
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, StatusQueuePayload::Completion(_)))
        .count();
    if completion_count > 1 {
        return format!("{completion_count} tasks complete");
    }
    if completion_count == 1 {
        if let Some(StatusQueuePayload::Completion(session)) =
            state.status_queue.first().map(|item| &item.payload)
        {
            return session
                .last_assistant_message
                .clone()
                .filter(|message| !message.trim().is_empty())
                .unwrap_or_else(|| "Task complete".to_string());
        }
    }

    let active_count = compact_active_session_count(snapshot);
    if active_count > 0 {
        format!(
            "{} active task{}",
            active_count,
            if active_count > 1 { "s" } else { "" }
        )
    } else {
        "No active tasks".to_string()
    }
}
