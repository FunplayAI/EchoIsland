use echoisland_runtime::{PendingPermissionView, PendingQuestionView, SessionSnapshotView};

use super::super::panel_types::NativeStatusQueueItem;
use crate::native_panel_scene::{
    StatusCardScene, StatusCardSceneKind, build_pending_permission_status_card_scene,
    build_pending_question_status_card_scene, build_prompt_assist_status_card_scene,
    build_status_queue_status_card_scene,
};

pub(super) struct StatusCardBadgeSpec {
    pub(super) text: String,
    pub(super) background: [f64; 4],
    pub(super) foreground: [f64; 4],
}

pub(super) struct StatusCardHeaderSpec {
    pub(super) title: String,
    pub(super) meta: String,
    pub(super) status_badge: StatusCardBadgeSpec,
    pub(super) source_badge: StatusCardBadgeSpec,
}

pub(super) struct PendingCardSpec {
    pub(super) header: StatusCardHeaderSpec,
    pub(super) body: String,
    pub(super) action_hint: String,
    pub(super) body_prefix: &'static str,
    pub(super) body_prefix_color: [f64; 4],
    pub(super) background: [f64; 4],
    pub(super) border: [f64; 4],
    pub(super) collapsed_height: f64,
}

pub(super) struct CompletionCardSpec {
    pub(super) header: StatusCardHeaderSpec,
    pub(super) preview: String,
    pub(super) preview_prefix: &'static str,
    pub(super) preview_prefix_color: [f64; 4],
    pub(super) background: [f64; 4],
    pub(super) border: [f64; 4],
    pub(super) collapsed_height: f64,
}

pub(super) fn build_pending_permission_card_spec(
    pending: &PendingPermissionView,
) -> PendingCardSpec {
    pending_card_spec_from_scene(build_pending_permission_status_card_scene(pending))
}

pub(super) fn build_pending_question_card_spec(pending: &PendingQuestionView) -> PendingCardSpec {
    pending_card_spec_from_scene(build_pending_question_status_card_scene(pending))
}

pub(super) fn build_prompt_assist_card_spec(session: &SessionSnapshotView) -> PendingCardSpec {
    pending_card_spec_from_scene(build_prompt_assist_status_card_scene(session))
}

pub(super) fn build_status_queue_pending_card_spec(
    item: &NativeStatusQueueItem,
) -> PendingCardSpec {
    pending_card_spec_from_scene(build_status_queue_status_card_scene(item))
}

pub(super) fn build_status_queue_completion_card_spec(
    item: &NativeStatusQueueItem,
) -> CompletionCardSpec {
    completion_card_spec_from_scene(build_status_queue_status_card_scene(item))
}

fn pending_card_spec_from_scene(scene: StatusCardScene) -> PendingCardSpec {
    let (background, border, foreground, body_prefix) = match scene.kind {
        StatusCardSceneKind::Approval => (
            [1.0, 0.61, 0.26, 0.13],
            [1.0, 0.61, 0.26, 0.24],
            [1.0, 0.68, 0.40, 1.0],
            "!",
        ),
        StatusCardSceneKind::Question => (
            [0.69, 0.55, 1.0, 0.13],
            [0.69, 0.55, 1.0, 0.24],
            [0.79, 0.69, 1.0, 1.0],
            "?",
        ),
        StatusCardSceneKind::PromptAssist => (
            [1.0, 0.61, 0.26, 0.08],
            [1.0, 0.61, 0.26, 0.32],
            [1.0, 0.70, 0.40, 1.0],
            "!",
        ),
        StatusCardSceneKind::Completion => unreachable!("completion scene must not map to pending"),
    };

    PendingCardSpec {
        header: header_spec_from_scene(&scene, [1.0, 1.0, 1.0, 0.08], foreground),
        body: scene.body,
        action_hint: scene.action_hint.unwrap_or_default(),
        body_prefix,
        body_prefix_color: foreground,
        background,
        border,
        collapsed_height: match scene.kind {
            StatusCardSceneKind::PromptAssist => 52.0,
            _ => 46.0,
        },
    }
}

fn completion_card_spec_from_scene(scene: StatusCardScene) -> CompletionCardSpec {
    CompletionCardSpec {
        header: header_spec_from_scene(&scene, [0.40, 0.87, 0.57, 0.14], [0.40, 0.87, 0.57, 1.0]),
        preview: scene.body,
        preview_prefix: "$",
        preview_prefix_color: [0.40, 0.87, 0.57, 0.96],
        background: [0.40, 0.87, 0.57, 0.08],
        border: [0.40, 0.87, 0.57, 0.28],
        collapsed_height: 52.0,
    }
}

fn header_spec_from_scene(
    scene: &StatusCardScene,
    status_background: [f64; 4],
    status_foreground: [f64; 4],
) -> StatusCardHeaderSpec {
    StatusCardHeaderSpec {
        title: scene.title.clone(),
        meta: scene.meta.clone(),
        status_badge: StatusCardBadgeSpec {
            text: scene.status_text.clone(),
            background: status_background,
            foreground: status_foreground,
        },
        source_badge: StatusCardBadgeSpec {
            text: scene.source_text.clone(),
            background: [0.47, 0.65, 1.0, 0.12],
            foreground: [0.47, 0.65, 1.0, 1.0],
        },
    }
}
