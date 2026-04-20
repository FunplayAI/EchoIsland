use super::*;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn render_expanded_cards(
    cards_container: &NSView,
    snapshot: &RuntimeSnapshot,
    expanded_width: f64,
) {
    clear_card_animation_layouts();
    clear_subviews(cards_container);
    let mut card_hit_targets = Vec::new();
    let cards_width = expanded_cards_width(expanded_width);

    let status_queue = native_status_queue_surface_items();
    if !status_queue.is_empty() {
        render_status_queue_cards(
            cards_container,
            snapshot,
            &status_queue,
            cards_width,
            &mut card_hit_targets,
        );
        replace_native_card_hit_targets(card_hit_targets);
        return;
    }

    render_default_cards(
        cards_container,
        snapshot,
        cards_width,
        &mut card_hit_targets,
    );
    replace_native_card_hit_targets(card_hit_targets);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_status_queue_cards(
    cards_container: &NSView,
    snapshot: &RuntimeSnapshot,
    status_queue: &[NativeStatusQueueItem],
    cards_width: f64,
    card_hit_targets: &mut Vec<NativeCardHitTarget>,
) {
    set_cards_container_body_height(
        cards_container,
        cards_width,
        estimated_expanded_body_height(snapshot),
    );

    let mut cursor_y = estimated_expanded_body_height(snapshot);
    let mut rendered_count = 0usize;
    for item in status_queue.iter() {
        let card_height = native_status_queue_card_height(item);
        let Some(frame) =
            next_expanded_card_frame(&mut cursor_y, rendered_count > 0, card_height, cards_width)
        else {
            break;
        };
        let card = create_status_queue_card(frame, item);
        apply_status_queue_item_visual_state(&card, item);
        cards_container.addSubview(&card);
        card_hit_targets.push(NativeCardHitTarget {
            session_id: item.session_id.clone(),
            frame,
        });
        rendered_count += 1;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_default_cards(
    cards_container: &NSView,
    snapshot: &RuntimeSnapshot,
    cards_width: f64,
    card_hit_targets: &mut Vec<NativeCardHitTarget>,
) {
    let pending_permissions = displayed_default_pending_permissions(snapshot);
    let pending_questions = displayed_default_pending_questions(snapshot);
    let prompt_assist_sessions = displayed_prompt_assist_sessions(snapshot);
    let sessions = displayed_sessions(snapshot, &prompt_assist_sessions);
    let body_height = estimated_expanded_body_height(snapshot);
    set_cards_container_body_height(cards_container, cards_width, body_height);

    let mut cursor_y = body_height;
    let mut rendered_count = 0usize;

    for pending in pending_permissions.iter() {
        if let Some(frame) = next_expanded_card_frame(
            &mut cursor_y,
            rendered_count > 0,
            pending_permission_card_height(pending),
            cards_width,
        ) {
            let card =
                create_pending_permission_card(frame, pending, snapshot.pending_permission_count);
            cards_container.addSubview(&card);
            card_hit_targets.push(NativeCardHitTarget {
                session_id: pending.session_id.clone(),
                frame,
            });
            rendered_count += 1;
        }
    }

    for pending in pending_questions.iter() {
        if let Some(frame) = next_expanded_card_frame(
            &mut cursor_y,
            rendered_count > 0,
            pending_question_card_height(pending),
            cards_width,
        ) {
            let card =
                create_pending_question_card(frame, pending, snapshot.pending_question_count);
            cards_container.addSubview(&card);
            card_hit_targets.push(NativeCardHitTarget {
                session_id: pending.session_id.clone(),
                frame,
            });
            rendered_count += 1;
        }
    }

    for session in prompt_assist_sessions.iter() {
        let Some(frame) = next_expanded_card_frame(
            &mut cursor_y,
            rendered_count > 0,
            prompt_assist_card_height(session),
            cards_width,
        ) else {
            break;
        };
        let card = create_prompt_assist_card(frame, session);
        cards_container.addSubview(&card);
        card_hit_targets.push(NativeCardHitTarget {
            session_id: session.session_id.clone(),
            frame,
        });
        rendered_count += 1;
    }

    if sessions.is_empty() && rendered_count == 0 {
        if let Some(frame) = next_expanded_card_frame(&mut cursor_y, false, 84.0, cards_width) {
            let empty = create_empty_card(frame);
            cards_container.addSubview(&empty);
        }
        return;
    }

    for session in sessions.iter() {
        let card_height = estimated_card_height(session);
        let Some(frame) =
            next_expanded_card_frame(&mut cursor_y, rendered_count > 0, card_height, cards_width)
        else {
            break;
        };
        let card = create_session_card(frame, session, false);
        cards_container.addSubview(&card);
        card_hit_targets.push(NativeCardHitTarget {
            session_id: session.session_id.clone(),
            frame,
        });
        rendered_count += 1;
    }
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
    if needs_gap {
        *cursor_y -= EXPANDED_CARD_GAP;
    }
    if *cursor_y < height {
        return None;
    }

    *cursor_y -= height;
    Some(NSRect::new(
        NSPoint::new(-EXPANDED_CARD_OVERHANG, *cursor_y),
        NSSize::new(expanded_width + (EXPANDED_CARD_OVERHANG * 2.0), height),
    ))
}
