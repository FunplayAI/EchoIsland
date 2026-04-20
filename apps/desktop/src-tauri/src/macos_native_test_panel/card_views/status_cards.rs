use super::*;

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_pending_permission_card(
    frame: NSRect,
    pending: &PendingPermissionView,
    _waiting_count: usize,
) -> objc2::rc::Retained<NSView> {
    let title = pending
        .tool_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Tool permission");
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());
    create_pending_card(
        frame,
        "Approval",
        &compact_title(title, 34),
        &body,
        "Allow / Deny in terminal",
        &pending.source,
        &pending.session_id,
        [1.0, 0.61, 0.26, 0.13],
        [1.0, 0.61, 0.26, 0.24],
        [1.0, 0.68, 0.40, 1.0],
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_pending_question_card(
    frame: NSRect,
    pending: &PendingQuestionView,
    _waiting_count: usize,
) -> objc2::rc::Retained<NSView> {
    let title = pending
        .header
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Need your input");
    let body = display_snippet(Some(&pending.text), 82)
        .unwrap_or_else(|| "Waiting for your answer".to_string());
    let action_hint = pending_question_action_hint(pending);
    create_pending_card(
        frame,
        "Question",
        &compact_title(title, 34),
        &body,
        &action_hint,
        &pending.source,
        &pending.session_id,
        [0.69, 0.55, 1.0, 0.13],
        [0.69, 0.55, 1.0, 0.24],
        [0.79, 0.69, 1.0, 1.0],
    )
}

pub(crate) fn pending_question_action_hint(pending: &PendingQuestionView) -> String {
    if pending.options.is_empty() {
        return "Answer in terminal".to_string();
    }

    let options = pending
        .options
        .iter()
        .take(3)
        .map(|option| compact_title(option, 12))
        .collect::<Vec<_>>()
        .join(" / ");
    if pending.options.len() > 3 {
        format!("{options} / …")
    } else {
        options
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn create_pending_card(
    frame: NSRect,
    label: &str,
    title: &str,
    body: &str,
    action_hint: &str,
    source: &str,
    session_id: &str,
    background: [f64; 4],
    border: [f64; 4],
    accent: [f64; 4],
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, background, border);
    register_card_animation_layout(&view, frame, 46.0);

    let status_badge = make_badge_view(
        mtm,
        label,
        badge_width(label, 10.0, 16.0),
        [1.0, 1.0, 1.0, 0.08],
        accent,
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let formatted_source = format_source(source);
    let source_badge = make_badge_view(
        mtm,
        &formatted_source,
        badge_width(&formatted_source, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let title_label = make_label(
        mtm,
        title,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let meta_label = make_label(
        mtm,
        &format!("#{} · {}", short_session_id(session_id), label),
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    let prefix = if label.eq_ignore_ascii_case("Approval") {
        "!"
    } else {
        "?"
    };
    let prefix_color = if label.eq_ignore_ascii_case("Approval") {
        [1.0, 0.68, 0.40, 1.0]
    } else {
        [0.79, 0.69, 1.0, 1.0]
    };
    let body_width = card_chat_body_width(frame.size.width);
    let body_height = estimated_chat_body_height(body, body_width, 2);
    let header_bottom = frame.size.height - 40.0;
    let body_gap_from_header = 6.0;
    let body_origin_y = (header_bottom - body_gap_from_header - body_height)
        .max(CARD_PENDING_ACTION_Y + CARD_PENDING_ACTION_HEIGHT + CARD_PENDING_ACTION_GAP);
    let prefix_y = body_origin_y + body_height - 12.0;
    let prefix_label = make_label(
        mtm,
        prefix,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, prefix_y),
            NSSize::new(10.0, 12.0),
        ),
        10.0,
        prefix_color,
        true,
        true,
    );
    prefix_label.setFont(Some(&NSFont::boldSystemFontOfSize(10.0)));
    view.addSubview(&prefix_label);

    let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(body), mtm);
    body_label.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X + CARD_CHAT_PREFIX_WIDTH, body_origin_y),
        NSSize::new(body_width, body_height),
    ));
    body_label.setTextColor(Some(&ns_color([0.86, 0.88, 0.92, 0.78])));
    body_label.setFont(Some(&NSFont::systemFontOfSize(10.0)));
    body_label.setDrawsBackground(false);
    body_label.setBezeled(false);
    body_label.setBordered(false);
    body_label.setEditable(false);
    body_label.setSelectable(false);
    body_label.setMaximumNumberOfLines(2);
    view.addSubview(&body_label);

    let action_badge = make_badge_view(
        mtm,
        action_hint,
        badge_width(action_hint, 10.0, 18.0).min(frame.size.width - (CARD_INSET_X * 2.0)),
        [1.0, 1.0, 1.0, 0.07],
        [0.90, 0.92, 0.96, 0.86],
    );
    action_badge.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X, CARD_PENDING_ACTION_Y),
        action_badge.frame().size,
    ));
    view.addSubview(&action_badge);

    view
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_status_queue_card(
    frame: NSRect,
    item: &NativeStatusQueueItem,
) -> objc2::rc::Retained<NSView> {
    match &item.payload {
        NativeStatusQueuePayload::Approval(pending) => {
            create_pending_permission_card(frame, pending, 1)
        }
        NativeStatusQueuePayload::Completion(session) => create_completion_card(frame, session),
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn apply_status_queue_item_visual_state(
    card: &NSView,
    item: &NativeStatusQueueItem,
) {
    if !item.is_removing {
        return;
    }

    let Some(remove_after) = item.remove_after else {
        return;
    };
    let exit_duration = status_queue_exit_duration();
    let elapsed =
        exit_duration.saturating_sub(remove_after.saturating_duration_since(Instant::now()));
    let progress = (elapsed.as_secs_f64() / exit_duration.as_secs_f64()).clamp(0.0, 1.0);
    apply_card_exit_phase(card, progress);
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_completion_card(
    frame: NSRect,
    session: &SessionSnapshotView,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [0.40, 0.87, 0.57, 0.08], [0.40, 0.87, 0.57, 0.28]);
    register_card_animation_layout(&view, frame, 52.0);

    let status_text = "Complete";
    let status_badge = make_badge_view(
        mtm,
        status_text,
        badge_width(status_text, 10.0, 16.0),
        [0.40, 0.87, 0.57, 0.14],
        [0.40, 0.87, 0.57, 1.0],
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let source_text = format_source(&session.source);
    let source_badge = make_badge_view(
        mtm,
        &source_text,
        badge_width(&source_text, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let project_name = session_title(session);
    let title = compact_title(&project_name, 30);
    let title_label = make_label(
        mtm,
        &title,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let meta_label = make_label(
        mtm,
        &format!(
            "#{} · {}",
            short_session_id(&session.session_id),
            time_ago(session.last_activity)
        ),
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    let preview = completion_preview_text(session);
    add_chat_line_with_max_lines_from_bottom(
        &view,
        mtm,
        CARD_CONTENT_BOTTOM_INSET,
        "$",
        &preview,
        [0.40, 0.87, 0.57, 0.96],
        [0.86, 0.88, 0.92, 0.78],
        frame.size.width,
        2,
        0.0,
    );

    view
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_prompt_assist_card(
    frame: NSRect,
    session: &SessionSnapshotView,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 0.61, 0.26, 0.08], [1.0, 0.61, 0.26, 0.32]);
    register_card_animation_layout(&view, frame, 52.0);

    let status_text = "Check";
    let status_badge = make_badge_view(
        mtm,
        status_text,
        badge_width(status_text, 10.0, 16.0),
        [1.0, 0.61, 0.26, 0.16],
        [1.0, 0.70, 0.40, 1.0],
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let source_badge = make_badge_view(
        mtm,
        "Codex",
        badge_width("Codex", 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let title_label = make_label(
        mtm,
        "Codex needs attention",
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let project_name = compact_title(&session_title(session), 24);
    let meta_label = make_label(
        mtm,
        &format!(
            "#{} · {} · {}",
            short_session_id(&session.session_id),
            project_name,
            time_ago(session.last_activity)
        ),
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    add_chat_line_with_max_lines_from_bottom(
        &view,
        mtm,
        CARD_PENDING_ACTION_Y + CARD_PENDING_ACTION_HEIGHT + CARD_PENDING_ACTION_GAP,
        "!",
        "Approval may be required in the Codex terminal.",
        [1.0, 0.68, 0.40, 1.0],
        [0.86, 0.88, 0.92, 0.78],
        frame.size.width,
        2,
        0.0,
    );

    let action_badge = make_badge_view(
        mtm,
        "Open terminal to check",
        badge_width("Open terminal to check", 10.0, 18.0)
            .min(frame.size.width - (CARD_INSET_X * 2.0)),
        [1.0, 1.0, 1.0, 0.07],
        [0.90, 0.92, 0.96, 0.86],
    );
    action_badge.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X, CARD_PENDING_ACTION_Y),
        action_badge.frame().size,
    ));
    view.addSubview(&action_badge);

    view
}
