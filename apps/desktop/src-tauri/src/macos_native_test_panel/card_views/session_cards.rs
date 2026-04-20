use super::*;

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_session_card(
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
