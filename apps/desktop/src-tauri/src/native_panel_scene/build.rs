use crate::native_panel_core::{
    ExpandedSurface, PanelHitAction, PanelMascotBaseState, PanelSettingsState, PanelState,
    StatusQueuePayload, compact_active_session_count, displayed_default_pending_permissions,
    displayed_default_pending_questions, displayed_prompt_assist_sessions, displayed_sessions,
    format_status, normalize_status, resolve_mascot_base_state, session_title, settings_row_action,
};
use echoisland_runtime::RuntimeSnapshot;

use super::{
    CompactBarScene, PanelScene, SceneBadge, SceneCard, SceneGlow, SceneGlowStyle, SceneHitTarget,
    SceneMascotPose, SceneNode, SceneText, SettingsRowScene,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PanelSceneBuildInput {
    pub(crate) display_count: usize,
    pub(crate) settings: PanelSettingsState,
    pub(crate) app_version: String,
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
    let glow = build_completion_glow(state);
    let mascot_pose = build_mascot_pose(state, snapshot, input.settings.mascot_enabled);
    let mut cards = Vec::new();
    let mut hit_targets = Vec::new();

    match state.surface_mode {
        ExpandedSurface::Settings if state.expanded => {
            let settings = build_settings_card(input);
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
        cards,
        glow,
        mascot_pose,
        hit_targets,
        nodes,
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

fn build_settings_card(input: &PanelSceneBuildInput) -> SceneCard {
    SceneCard::Settings {
        title: "Settings".to_string(),
        version: SceneBadge {
            text: format!("v{}", input.app_version),
            emphasized: false,
        },
        rows: vec![
            SettingsRowScene {
                title: "Island display".to_string(),
                value: SceneBadge {
                    text: format!(
                        "Screen {}/{}",
                        input.settings.selected_display_index + 1,
                        input.display_count.max(1)
                    ),
                    emphasized: false,
                },
                action: settings_row_action(0).unwrap(),
            },
            SettingsRowScene {
                title: "Completion sound".to_string(),
                value: SceneBadge {
                    text: if input.settings.completion_sound_enabled {
                        "On".to_string()
                    } else {
                        "Off".to_string()
                    },
                    emphasized: input.settings.completion_sound_enabled,
                },
                action: settings_row_action(1).unwrap(),
            },
            SettingsRowScene {
                title: "Mascot".to_string(),
                value: SceneBadge {
                    text: if input.settings.mascot_enabled {
                        "On".to_string()
                    } else {
                        "Off".to_string()
                    },
                    emphasized: input.settings.mascot_enabled,
                },
                action: settings_row_action(2).unwrap(),
            },
            SettingsRowScene {
                title: "Update & upgrade".to_string(),
                value: SceneBadge {
                    text: "Open".to_string(),
                    emphasized: false,
                },
                action: settings_row_action(3).unwrap(),
            },
        ],
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
