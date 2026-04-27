use serde::Serialize;

use echoisland_runtime::{PendingPermissionView, PendingQuestionView, SessionSnapshotView};

use crate::native_panel_core::{ExpandedSurface, PanelHitAction, StatusQueueItem};
use crate::native_panel_scene::{
    SessionSurfaceScene, SettingsSurfaceScene, StatusCardScene, SurfaceScene,
};

#[derive(Clone, Debug)]
pub(crate) struct PanelScene {
    pub(crate) surface: ExpandedSurface,
    pub(crate) compact_bar: CompactBarScene,
    pub(crate) surface_scene: SurfaceScene,
    pub(crate) status_surface: StatusSurfaceScene,
    pub(crate) session_surface: SessionSurfaceScene,
    pub(crate) settings_surface: SettingsSurfaceScene,
    pub(crate) cards: Vec<SceneCard>,
    pub(crate) glow: Option<SceneGlow>,
    pub(crate) mascot_pose: SceneMascotPose,
    pub(crate) hit_targets: Vec<SceneHitTarget>,
    pub(crate) nodes: Vec<SceneNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompactBarScene {
    pub(crate) headline: SceneText,
    pub(crate) active_count: String,
    pub(crate) total_count: String,
    pub(crate) completion_count: usize,
    pub(crate) actions_visible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatusSurfaceScene {
    pub(crate) cards: Vec<StatusCardScene>,
    pub(crate) display_mode: StatusSurfaceDisplayMode,
    pub(crate) default_state: StatusSurfaceDefaultState,
    pub(crate) queue_state: StatusSurfaceQueueState,
    pub(crate) completion_badge_count: usize,
    pub(crate) show_completion_glow: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusSurfaceDisplayMode {
    Hidden,
    DefaultStack,
    Queue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatusSurfaceDefaultState {
    pub(crate) approval_count: usize,
    pub(crate) question_count: usize,
    pub(crate) prompt_assist_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatusSurfaceQueueState {
    pub(crate) total_count: usize,
    pub(crate) live_count: usize,
    pub(crate) removing_count: usize,
    pub(crate) next_transition_in_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PanelShellSceneState {
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PanelRuntimeRenderState {
    pub(crate) transitioning: bool,
    pub(crate) shell_scene: PanelShellSceneState,
}

#[derive(Clone, Debug)]
pub(crate) enum SceneNode {
    Text(SceneText),
    Badge(SceneBadge),
    Card(SceneCard),
    Glow(SceneGlow),
    Mascot(SceneMascotPose),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneText {
    pub(crate) text: String,
    pub(crate) emphasized: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneBadge {
    pub(crate) text: String,
    pub(crate) emphasized: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum SceneCard {
    Settings {
        title: String,
        version: SceneBadge,
        rows: Vec<SettingsRowScene>,
    },
    PendingPermission {
        pending: PendingPermissionView,
        count: usize,
    },
    PendingQuestion {
        pending: PendingQuestionView,
        count: usize,
    },
    PromptAssist {
        session: SessionSnapshotView,
    },
    Session {
        session: SessionSnapshotView,
        title: String,
        status: SceneBadge,
        snippet: Option<String>,
    },
    StatusApproval {
        item: StatusQueueItem,
    },
    StatusCompletion {
        item: StatusQueueItem,
    },
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SceneCardHeightInput<'a> {
    Settings { row_count: usize },
    PendingPermission(&'a PendingPermissionView),
    PendingQuestion(&'a PendingQuestionView),
    PromptAssist(&'a SessionSnapshotView),
    Session(&'a SessionSnapshotView),
    StatusItem(&'a StatusQueueItem),
    Empty,
}

pub(crate) fn resolve_scene_card_height_input(card: &SceneCard) -> SceneCardHeightInput<'_> {
    match card {
        SceneCard::Settings { rows, .. } => SceneCardHeightInput::Settings {
            row_count: rows.len(),
        },
        SceneCard::PendingPermission { pending, .. } => {
            SceneCardHeightInput::PendingPermission(pending)
        }
        SceneCard::PendingQuestion { pending, .. } => {
            SceneCardHeightInput::PendingQuestion(pending)
        }
        SceneCard::PromptAssist { session } => SceneCardHeightInput::PromptAssist(session),
        SceneCard::Session { session, .. } => SceneCardHeightInput::Session(session),
        SceneCard::StatusApproval { item } | SceneCard::StatusCompletion { item } => {
            SceneCardHeightInput::StatusItem(item)
        }
        SceneCard::Empty => SceneCardHeightInput::Empty,
    }
}

pub(crate) fn resolve_scene_cards_total_height(
    scene: &PanelScene,
    resolve_card_height: impl FnMut(&SceneCard) -> f64,
    card_gap: f64,
    empty_height: f64,
) -> f64 {
    let card_heights = scene
        .cards
        .iter()
        .map(resolve_card_height)
        .collect::<Vec<_>>();
    crate::native_panel_core::resolve_stacked_cards_total_height(
        &card_heights,
        card_gap,
        empty_height,
    )
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SettingsRowScene {
    pub(crate) title: String,
    pub(crate) value: SceneBadge,
    pub(crate) action: PanelHitAction,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneGlow {
    pub(crate) style: SceneGlowStyle,
    pub(crate) opacity: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SceneGlowStyle {
    Completion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SceneMascotPose {
    Hidden,
    Idle,
    Running,
    Approval,
    Question,
    MessageBubble,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SceneHitTarget {
    pub(crate) action: PanelHitAction,
    pub(crate) value: String,
}
