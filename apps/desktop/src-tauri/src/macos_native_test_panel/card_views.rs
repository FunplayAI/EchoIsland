use super::*;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn create_empty_card(frame: NSRect) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 1.0, 1.0, 0.055], [1.0, 1.0, 1.0, 0.08]);
    register_card_animation_layout(&view, frame, 34.0);

    let label = make_label(
        mtm,
        "No sessions yet.",
        NSRect::new(NSPoint::new(0.0, 31.0), NSSize::new(frame.size.width, 20.0)),
        12.0,
        [0.67, 0.70, 0.76, 1.0],
        true,
        false,
    );
    view.addSubview(&label);
    view
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn create_pending_permission_card(
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
pub(super) unsafe fn create_pending_question_card(
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

pub(super) fn pending_question_action_hint(pending: &PendingQuestionView) -> String {
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
pub(super) unsafe fn create_pending_card(
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
pub(super) unsafe fn create_status_queue_card(
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
pub(super) unsafe fn apply_status_queue_item_visual_state(
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
pub(super) unsafe fn create_completion_card(
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
pub(super) unsafe fn create_prompt_assist_card(
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

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn create_session_card(
    frame: NSRect,
    session: &SessionSnapshotView,
    emphasize: bool,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    let status = normalize_status(&session.status);
    let prompt = session_prompt_preview(session);
    let reply = session_reply_preview(session);
    let tool_preview = session_tool_preview(session);
    let has_body_content = prompt.is_some()
        || reply.is_some()
        || tool_preview
            .as_ref()
            .map(|(name, _)| !name.is_empty())
            .unwrap_or(false);
    let is_compact = is_long_idle_session(session) || !has_body_content;
    let background = if emphasize {
        [0.40, 0.87, 0.57, 0.08]
    } else {
        [1.0, 1.0, 1.0, 0.055]
    };
    let border = if emphasize {
        [0.40, 0.87, 0.57, 0.20]
    } else {
        [1.0, 1.0, 1.0, 0.08]
    };
    apply_card_layer(&view, background, border);
    register_card_animation_layout(
        &view,
        frame,
        session_card_collapsed_height(frame.size.height, is_compact),
    );

    let (status_bg, status_fg) = status_pill_colors(&status, emphasize);
    let status_text = format_status(&session.status);
    let status_badge = make_badge_view(
        mtm,
        &status_text,
        badge_width(&status_text, 10.0, 16.0),
        status_bg,
        status_fg,
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

    let title = compact_title(&session_title(session), 30);
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

    let meta_text = session_meta_line(session);
    let meta_label = make_label(
        mtm,
        &meta_text,
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

    if !is_compact {
        let mut content_top = CARD_CONTENT_BOTTOM_INSET;
        let has_prompt = prompt.is_some();
        let has_reply = reply.is_some();

        if let Some((tool_name, tool_description)) = tool_preview.as_ref() {
            let tool_text = tool_description
                .as_ref()
                .map(|description| format!("{tool_name} · {description}"))
                .unwrap_or_else(|| tool_name.to_string());
            let tool_view = make_live_tool_view(
                mtm,
                tool_name,
                tool_description.as_deref(),
                (frame.size.width - (CARD_INSET_X * 2.0)).min(badge_width(&tool_text, 9.0, 20.0)),
            );
            let tool_size = tool_view.frame().size;
            tool_view.setFrame(NSRect::new(
                NSPoint::new(CARD_INSET_X, content_top),
                NSSize::new(
                    (frame.size.width - (CARD_INSET_X * 2.0)).min(tool_size.width),
                    tool_size.height,
                ),
            ));
            view.addSubview(&tool_view);
            content_top += tool_size.height;
            if has_reply || has_prompt {
                content_top += CARD_TOOL_GAP;
            }
        }

        if let Some(reply) = reply.as_deref() {
            content_top = add_chat_line_with_max_lines_from_bottom(
                &view,
                mtm,
                content_top,
                "$",
                reply,
                [0.85, 0.47, 0.34, 0.96],
                [0.86, 0.88, 0.92, 0.74],
                frame.size.width,
                2,
                if has_prompt { CARD_CHAT_GAP } else { 0.0 },
            );
        }

        if let Some(prompt) = prompt.as_deref() {
            add_chat_line_with_max_lines_from_bottom(
                &view,
                mtm,
                content_top,
                ">",
                prompt,
                [0.40, 0.87, 0.57, 0.96],
                [0.96, 0.97, 0.99, 0.86],
                frame.size.width,
                1,
                0.0,
            );
        }
    }

    view
}

#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn add_chat_line_with_max_lines_from_bottom(
    parent: &NSView,
    mtm: MainThreadMarker,
    bottom_y: f64,
    prefix: &str,
    body: &str,
    prefix_color: [f64; 4],
    body_color: [f64; 4],
    width: f64,
    max_lines: isize,
    gap_after: f64,
) -> f64 {
    let body_width = card_chat_body_width(width);
    let body_height = estimated_chat_body_height(body, body_width, max_lines);
    let prefix_y = bottom_y + body_height - 12.0;
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
    parent.addSubview(&prefix_label);

    let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(body), mtm);
    body_label.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X + CARD_CHAT_PREFIX_WIDTH, bottom_y),
        NSSize::new(body_width, body_height),
    ));
    body_label.setTextColor(Some(&ns_color(body_color)));
    body_label.setFont(Some(&NSFont::systemFontOfSize(10.0)));
    body_label.setDrawsBackground(false);
    body_label.setBezeled(false);
    body_label.setBordered(false);
    body_label.setEditable(false);
    body_label.setSelectable(false);
    body_label.setMaximumNumberOfLines(max_lines);
    parent.addSubview(&body_label);

    bottom_y + body_height + gap_after.max(0.0)
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_card_layer(view: &NSView, background: [f64; 4], border: [f64; 4]) {
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        let background = ns_color(background);
        let border = ns_color(border);
        layer.setCornerRadius(CARD_RADIUS);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&background.CGColor()));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&border.CGColor()));
        layer.setShadowColor(Some(&NSColor::blackColor().CGColor()));
        layer.setShadowOpacity(0.0);
        layer.setShadowRadius(0.0);
        layer.setShadowOffset(NSSize::new(0.0, 0.0));
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn make_label(
    mtm: MainThreadMarker,
    text: &str,
    frame: NSRect,
    font_size: f64,
    color: [f64; 4],
    centered: bool,
    single_line: bool,
) -> objc2::rc::Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    label.setFrame(frame);
    if centered {
        label.setAlignment(NSTextAlignment::Center);
    }
    label.setTextColor(Some(&ns_color(color)));
    label.setFont(Some(&NSFont::systemFontOfSize(font_size)));
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
    if single_line {
        label.setUsesSingleLineMode(true);
    }
    label
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn make_badge_view(
    mtm: MainThreadMarker,
    text: &str,
    width: f64,
    background: [f64; 4],
    foreground: [f64; 4],
) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width.max(24.0), 22.0)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        let background = ns_color(background);
        layer.setCornerRadius(11.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&background.CGColor()));
    }

    let label = make_label(
        mtm,
        text,
        NSRect::new(
            NSPoint::new(7.0, 4.0),
            NSSize::new(width.max(24.0) - 14.0, 13.0),
        ),
        10.0,
        foreground,
        true,
        true,
    );
    label.setFont(Some(&NSFont::systemFontOfSize_weight(10.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    view.addSubview(&label);
    view
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn make_live_tool_view(
    mtm: MainThreadMarker,
    tool_name: &str,
    description: Option<&str>,
    width: f64,
) -> objc2::rc::Retained<NSView> {
    let width = width.max(36.0);
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, 22.0)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(5.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&ns_color([1.0, 1.0, 1.0, 0.04]).CGColor()));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&ns_color([1.0, 1.0, 1.0, 0.06]).CGColor()));
    }

    let name_width = badge_width(tool_name, 9.0, 0.0).min((width - 14.0).max(0.0));
    let name_label = make_label(
        mtm,
        tool_name,
        NSRect::new(NSPoint::new(7.0, 5.0), NSSize::new(name_width, 11.0)),
        9.0,
        tool_tone_color(tool_name),
        false,
        true,
    );
    name_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightBold },
    )));
    view.addSubview(&name_label);

    if let Some(description) = description.filter(|value| !value.trim().is_empty()) {
        let desc_x = 7.0 + name_width + 6.0;
        let desc_width = (width - desc_x - 7.0).max(0.0);
        let desc_label = make_label(
            mtm,
            description,
            NSRect::new(NSPoint::new(desc_x, 5.0), NSSize::new(desc_width, 11.0)),
            9.0,
            [1.0, 1.0, 1.0, 0.70],
            false,
            true,
        );
        desc_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            9.0,
            unsafe { objc2_app_kit::NSFontWeightRegular },
        )));
        view.addSubview(&desc_label);
    }

    view
}

pub(super) fn tool_tone_color(tool: &str) -> [f64; 4] {
    match tool.to_ascii_lowercase().as_str() {
        "bash" => [0.49, 0.95, 0.64, 1.0],
        "edit" | "write" => [0.53, 0.67, 1.0, 1.0],
        "read" => [0.94, 0.82, 0.49, 1.0],
        "grep" | "glob" => [0.76, 0.63, 1.0, 1.0],
        "agent" => [1.0, 0.61, 0.40, 1.0],
        _ => [0.96, 0.97, 0.99, 0.86],
    }
}

pub(super) fn badge_width(text: &str, font_size: f64, horizontal_padding: f64) -> f64 {
    (text.chars().count() as f64 * font_size * 0.58) + horizontal_padding
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn clear_subviews(view: &NSView) {
    let subviews = view.subviews();
    for index in 0..subviews.len() {
        subviews.objectAtIndex(index).removeFromSuperview();
    }
}
