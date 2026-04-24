use objc2_app_kit::NSView;
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::card_animation::clear_card_animation_layouts;
use super::card_metrics::{estimated_scene_card_height, estimated_scene_cards_content_height};
use super::card_views::{
    apply_status_queue_item_visual_state, clear_subviews, create_empty_card,
    create_pending_permission_card, create_pending_question_card, create_prompt_assist_card,
    create_session_card, create_settings_surface_card, create_status_queue_card,
    settings_surface_card_height,
};
use super::panel_constants::{EXPANDED_CARD_GAP, EXPANDED_CARD_OVERHANG};
use super::panel_geometry::expanded_cards_width;
use super::panel_scene_adapter::NativePanelSnapshotRenderPlan;
use crate::native_panel_core::PanelRect;
use crate::native_panel_renderer::NativePanelCardStackCommand;
use crate::native_panel_scene::SceneCard;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_expanded_cards_with_plan(
    cards_container: &NSView,
    plan: &NativePanelSnapshotRenderPlan,
    expanded_width: f64,
) {
    clear_card_animation_layouts();
    clear_subviews(cards_container);
    let cards_width = expanded_cards_width(expanded_width);
    let command = plan.card_stack_command(
        PanelRect {
            x: cards_container.frame().origin.x,
            y: cards_container.frame().origin.y,
            width: cards_width,
            height: cards_container.frame().size.height,
        },
        true,
    );
    render_card_stack_command(
        cards_container,
        cards_width,
        &command,
        Some(plan.expanded_content_height),
    );
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_settings_surface(cards_container: &NSView, cards_width: f64) {
    let body_height = settings_surface_card_height();
    set_cards_container_body_height(cards_container, cards_width, body_height);
    let mut cursor_y = body_height;
    if let Some(frame) = next_expanded_card_frame(&mut cursor_y, false, body_height, cards_width) {
        let card = create_settings_surface_card(frame);
        cards_container.addSubview(&card);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_status_queue_cards(
    cards_container: &NSView,
    command: &NativePanelCardStackCommand,
    cards_width: f64,
    body_height: f64,
) {
    set_cards_container_body_height(cards_container, cards_width, body_height);

    let mut cursor_y = body_height;
    let mut rendered_count = 0usize;
    for card in &command.cards {
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
        rendered_count += 1;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_default_cards(
    cards_container: &NSView,
    command: &NativePanelCardStackCommand,
    cards_width: f64,
    body_height: f64,
) {
    set_cards_container_body_height(cards_container, cards_width, body_height);

    let mut cursor_y = body_height;
    let mut rendered_count = 0usize;

    for card in &command.cards {
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
            }
            SceneCard::PendingQuestion { pending, count } => {
                let view = create_pending_question_card(frame, pending, *count);
                cards_container.addSubview(&view);
            }
            SceneCard::PromptAssist { session } => {
                let view = create_prompt_assist_card(frame, session);
                cards_container.addSubview(&view);
            }
            SceneCard::Session { session, .. } => {
                let view = create_session_card(frame, session, false);
                cards_container.addSubview(&view);
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
unsafe fn render_card_stack_command(
    cards_container: &NSView,
    cards_width: f64,
    command: &NativePanelCardStackCommand,
    body_height_override: Option<f64>,
) {
    if matches!(command.cards.first(), Some(SceneCard::Settings { .. })) {
        render_settings_surface(cards_container, cards_width);
        return;
    }

    let body_height = body_height_override
        .unwrap_or_else(|| estimated_scene_cards_content_height(&command.cards));

    if command.cards.iter().any(|card| {
        matches!(
            card,
            SceneCard::StatusApproval { .. } | SceneCard::StatusCompletion { .. }
        )
    }) {
        render_status_queue_cards(cards_container, command, cards_width, body_height);
        return;
    }

    render_default_cards(cards_container, command, cards_width, body_height);
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
