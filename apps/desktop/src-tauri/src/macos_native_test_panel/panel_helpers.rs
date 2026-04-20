use super::*;

pub(super) fn native_panel_content_visibility() -> f64 {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                if guard.transitioning {
                    card_content_visibility_phase(
                        guard.transition_cards_progress,
                        guard.transition_cards_entering,
                    )
                } else if guard.expanded {
                    1.0
                } else {
                    0.0
                }
            })
        })
        .unwrap_or(0.0)
}

pub(super) fn card_chat_body_width(card_width: f64) -> f64 {
    (card_width - (CARD_INSET_X * 2.0) - CARD_CHAT_PREFIX_WIDTH).max(1.0)
}

pub(super) fn estimated_chat_body_height(body: &str, width: f64, max_lines: isize) -> f64 {
    estimated_chat_line_count(body, width, max_lines) as f64 * CARD_CHAT_LINE_HEIGHT
}

pub(super) fn estimated_chat_line_count(body: &str, width: f64, max_lines: isize) -> isize {
    let max_lines = max_lines.max(1);
    let line_count = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                1
            } else {
                (estimated_text_width(trimmed, 10.0) / width.max(1.0)).ceil() as isize
            }
        })
        .sum::<isize>()
        .max(1);
    line_count.min(max_lines)
}

pub(super) fn estimated_text_width(text: &str, font_size: f64) -> f64 {
    text.chars()
        .map(|ch| {
            let factor = if ch.is_ascii_whitespace() {
                0.34
            } else if ch.is_ascii_uppercase() {
                0.66
            } else if ch.is_ascii_punctuation() {
                0.42
            } else if ch.is_ascii() {
                0.60
            } else {
                1.0
            };
            factor * font_size
        })
        .sum::<f64>()
        .max(font_size)
}

pub(super) fn estimated_default_chat_body_width() -> f64 {
    card_chat_body_width(expanded_cards_width(DEFAULT_PANEL_CANVAS_WIDTH))
}

pub(super) fn summarize_headline(snapshot: &RuntimeSnapshot) -> String {
    let status_queue = native_status_queue_surface_items();
    if !status_queue.is_empty() {
        let approval_count = status_queue
            .iter()
            .filter(|item| matches!(&item.payload, NativeStatusQueuePayload::Approval(_)))
            .count();
        if approval_count > 0 {
            return if approval_count > 1 {
                "Approvals waiting".to_string()
            } else {
                "Approval waiting".to_string()
            };
        }

        let completion_count = status_queue
            .iter()
            .filter(|item| matches!(&item.payload, NativeStatusQueuePayload::Completion(_)))
            .count();
        if completion_count > 1 {
            return format!("{completion_count} tasks complete");
        }
        if let Some(NativeStatusQueueItem {
            payload: NativeStatusQueuePayload::Completion(session),
            ..
        }) = status_queue.first()
        {
            return display_snippet(
                session
                    .last_assistant_message
                    .as_deref()
                    .or(session.tool_description.as_deref()),
                30,
            )
            .unwrap_or_else(|| "Task complete".to_string());
        }
    }

    let active_count = compact_active_count_value(snapshot);
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

pub(super) fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}

pub(super) fn animation_phase(elapsed_ms: u64, delay_ms: u64, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        return 1.0;
    }

    elapsed_ms.saturating_sub(delay_ms) as f64 / duration_ms as f64
}

pub(super) fn ease_in_cubic(progress: f64) -> f64 {
    progress.clamp(0.0, 1.0).powi(3)
}

pub(super) fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

pub(super) fn status_queue_exit_duration() -> Duration {
    Duration::from_millis(PANEL_CARD_EXIT_MS.max(220) + STATUS_QUEUE_EXIT_EXTRA_MS)
}

pub(super) fn status_pill_colors(status: &str, emphasize: bool) -> ([f64; 4], [f64; 4]) {
    if emphasize {
        return ([0.40, 0.87, 0.57, 0.14], [0.40, 0.87, 0.57, 1.0]);
    }

    match status {
        "running" => ([0.30, 0.87, 0.46, 0.14], [0.49, 0.95, 0.64, 1.0]),
        "processing" => ([1.0, 0.74, 0.41, 0.14], [1.0, 0.81, 0.46, 1.0]),
        "waitingapproval" | "waitingquestion" => ([1.0, 0.61, 0.26, 0.16], [1.0, 0.68, 0.40, 1.0]),
        _ => ([1.0, 1.0, 1.0, 0.08], [0.96, 0.97, 0.99, 1.0]),
    }
}

pub(super) fn ns_color(rgba: [f64; 4]) -> objc2::rc::Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(rgba[0], rgba[1], rgba[2], rgba[3])
}
