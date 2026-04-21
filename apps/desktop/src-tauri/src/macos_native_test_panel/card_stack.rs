use super::*;
use crate::native_panel_scene::{PanelScene, SceneCard};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_expanded_cards(
    cards_container: &NSView,
    snapshot: &RuntimeSnapshot,
    expanded_width: f64,
) {
    clear_card_animation_layouts();
    clear_subviews(cards_container);
    let scene = build_native_panel_scene(snapshot);
    let mut card_hit_targets = Vec::new();
    let cards_width = expanded_cards_width(expanded_width);
    render_scene_cards(cards_container, cards_width, &scene, &mut card_hit_targets);
    replace_native_card_hit_targets(card_hit_targets);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_settings_surface(cards_container: &NSView, cards_width: f64) {
    let body_height = settings_surface_card_height();
    set_cards_container_body_height(cards_container, cards_width, body_height);
    let mut cursor_y = body_height;
    if let Some(frame) = next_expanded_card_frame(&mut cursor_y, false, body_height, cards_width) {
        let card = create_settings_surface_card(frame);
        cards_container.addSubview(&card);
        replace_native_card_hit_targets(settings_surface_hit_targets(frame));
        return;
    }
    replace_native_card_hit_targets(Vec::new());
}

pub(super) fn settings_surface_hit_targets(frame: NSRect) -> Vec<NativeCardHitTarget> {
    vec![
        NativeCardHitTarget {
            action: NativePanelHitAction::CycleDisplay,
            value: String::new(),
            frame: settings_surface_row_frame(frame, 0),
        },
        NativeCardHitTarget {
            action: NativePanelHitAction::ToggleCompletionSound,
            value: String::new(),
            frame: settings_surface_row_frame(frame, 1),
        },
        NativeCardHitTarget {
            action: NativePanelHitAction::ToggleMascot,
            value: String::new(),
            frame: settings_surface_row_frame(frame, 2),
        },
        NativeCardHitTarget {
            action: NativePanelHitAction::OpenReleasePage,
            value: String::new(),
            frame: settings_surface_row_frame(frame, 3),
        },
    ]
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_status_queue_cards(
    cards_container: &NSView,
    scene: &PanelScene,
    cards_width: f64,
    card_hit_targets: &mut Vec<NativeCardHitTarget>,
) {
    set_cards_container_body_height(
        cards_container,
        cards_width,
        estimated_scene_content_height(scene),
    );

    let mut cursor_y = estimated_scene_content_height(scene);
    let mut rendered_count = 0usize;
    for card in &scene.cards {
        let item = match card {
            SceneCard::StatusApproval { item } | SceneCard::StatusCompletion { item } => item,
            _ => continue,
        };
        let card_height = estimated_scene_card_height(card);
        let Some(frame) =
            next_expanded_card_frame(&mut cursor_y, rendered_count > 0, card_height, cards_width)
        else {
            break;
        };
        let card = create_status_queue_card(frame, item);
        apply_status_queue_item_visual_state(&card, item);
        cards_container.addSubview(&card);
        card_hit_targets.push(NativeCardHitTarget {
            action: NativePanelHitAction::FocusSession,
            value: item.session_id.clone(),
            frame,
        });
        rendered_count += 1;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_default_cards(
    cards_container: &NSView,
    scene: &PanelScene,
    cards_width: f64,
    card_hit_targets: &mut Vec<NativeCardHitTarget>,
) {
    let body_height = estimated_scene_content_height(scene);
    set_cards_container_body_height(cards_container, cards_width, body_height);

    let mut cursor_y = body_height;
    let mut rendered_count = 0usize;

    for card in &scene.cards {
        let card_height = estimated_scene_card_height(card);
        let Some(frame) =
            next_expanded_card_frame(&mut cursor_y, rendered_count > 0, card_height, cards_width)
        else {
            break;
        };
        match card {
            SceneCard::PendingPermission { pending, count } => {
                let view = create_pending_permission_card(frame, pending, *count);
                cards_container.addSubview(&view);
                card_hit_targets.push(NativeCardHitTarget {
                    action: NativePanelHitAction::FocusSession,
                    value: pending.session_id.clone(),
                    frame,
                });
            }
            SceneCard::PendingQuestion { pending, count } => {
                let view = create_pending_question_card(frame, pending, *count);
                cards_container.addSubview(&view);
                card_hit_targets.push(NativeCardHitTarget {
                    action: NativePanelHitAction::FocusSession,
                    value: pending.session_id.clone(),
                    frame,
                });
            }
            SceneCard::PromptAssist { session } => {
                let view = create_prompt_assist_card(frame, session);
                cards_container.addSubview(&view);
                card_hit_targets.push(NativeCardHitTarget {
                    action: NativePanelHitAction::FocusSession,
                    value: session.session_id.clone(),
                    frame,
                });
            }
            SceneCard::Session { session, .. } => {
                let view = create_session_card(frame, session, false);
                cards_container.addSubview(&view);
                card_hit_targets.push(NativeCardHitTarget {
                    action: NativePanelHitAction::FocusSession,
                    value: session.session_id.clone(),
                    frame,
                });
            }
            SceneCard::Empty => {
                let empty = create_empty_card(frame);
                cards_container.addSubview(&empty);
            }
            SceneCard::Settings { .. }
            | SceneCard::StatusApproval { .. }
            | SceneCard::StatusCompletion { .. } => continue,
        }
        rendered_count += 1;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_scene_cards(
    cards_container: &NSView,
    cards_width: f64,
    scene: &PanelScene,
    card_hit_targets: &mut Vec<NativeCardHitTarget>,
) {
    if matches!(scene.cards.first(), Some(SceneCard::Settings { .. })) {
        render_settings_surface(cards_container, cards_width);
        return;
    }

    if scene.cards.iter().any(|card| {
        matches!(
            card,
            SceneCard::StatusApproval { .. } | SceneCard::StatusCompletion { .. }
        )
    }) {
        render_status_queue_cards(cards_container, scene, cards_width, card_hit_targets);
        return;
    }

    render_default_cards(cards_container, scene, cards_width, card_hit_targets);
}

fn set_cards_container_body_height(cards_container: &NSView, cards_width: f64, body_height: f64) {
    let current_frame = cards_container.frame();
    cards_container.setFrame(NSRect::new(
        current_frame.origin,
        NSSize::new(cards_width, body_height),
    ));
}

pub(super) fn next_expanded_card_frame(
    cursor_y: &mut f64,
    needs_gap: bool,
    height: f64,
    expanded_width: f64,
) -> Option<NSRect> {
    crate::native_panel_core::resolve_next_stacked_card_frame(
        cursor_y,
        needs_gap,
        height,
        expanded_width,
        EXPANDED_CARD_GAP,
        EXPANDED_CARD_OVERHANG,
    )
    .map(|frame| {
        NSRect::new(
            NSPoint::new(frame.x, frame.y),
            NSSize::new(frame.width, frame.height),
        )
    })
}
