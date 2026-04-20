use super::*;

pub(super) fn native_panel_geometry_metrics(
    screen: Option<&NSScreen>,
    screen_frame: NSRect,
) -> NativePanelGeometryMetrics {
    let compact_height = compact_pill_height_for_screen_rect(screen, screen_frame);
    NativePanelGeometryMetrics {
        compact_height,
        compact_width: compact_pill_width_for_screen_rect(screen, compact_height),
        expanded_width: expanded_panel_width_for_screen_rect(screen, screen_frame),
        panel_width: panel_canvas_width_for_screen_rect(screen, compact_height, screen_frame),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_native_panel_layout(
    screen_frame: NSRect,
    metrics: NativePanelGeometryMetrics,
    canvas_height: f64,
    visible_height: f64,
    bar_progress: f64,
    height_progress: f64,
    drop_progress: f64,
    content_visibility: f64,
) -> NativePanelLayout {
    let canvas_height = canvas_height.max(COLLAPSED_PANEL_HEIGHT);
    let visible_height = visible_height.clamp(COLLAPSED_PANEL_HEIGHT, canvas_height);
    let content_frame = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(metrics.panel_width, canvas_height),
    );
    let drop_offset = PANEL_DROP_DISTANCE * drop_progress;
    let panel_frame = centered_top_frame(screen_frame, content_frame.size);
    let pill_frame = island_bar_frame(
        content_frame.size,
        bar_progress,
        metrics.compact_width,
        metrics.expanded_width,
        metrics.compact_height,
        drop_offset,
    );
    let expanded_frame = expanded_background_frame(
        content_frame.size,
        visible_height,
        bar_progress,
        height_progress,
        metrics.compact_width,
        metrics.expanded_width,
        metrics.compact_height,
        drop_offset,
    );
    let cards_frame = expanded_cards_frame(expanded_frame, metrics.compact_height);
    NativePanelLayout {
        panel_frame,
        content_frame,
        pill_frame,
        left_shoulder_frame: left_shoulder_frame(pill_frame),
        right_shoulder_frame: right_shoulder_frame(pill_frame),
        expanded_frame,
        cards_frame,
        separator_frame: expanded_separator_frame(expanded_frame, metrics.compact_height),
        shared_content_frame: absolute_rect(
            panel_frame,
            compose_local_rect(expanded_frame, cards_frame),
        ),
        shell_visible: bar_progress > 0.01 || height_progress > 0.01,
        separator_visibility: (height_progress.min(content_visibility) * 0.88).clamp(0.0, 0.88),
    }
}

pub(super) fn shared_expanded_content_state(
    shared_expanded_enabled: bool,
    shell_visible: bool,
    height_progress: f64,
    bar_progress: f64,
    cards_height: f64,
    status_surface_active: bool,
    content_visibility: f64,
) -> (bool, bool) {
    let visible = shared_expanded_enabled
        && shell_visible
        && height_progress > SHARED_CONTENT_REVEAL_PROGRESS
        && content_visibility > SHARED_CONTENT_REVEAL_PROGRESS
        && cards_height > 4.0
        && !status_surface_active;
    let interactive = visible
        && height_progress > SHARED_CONTENT_INTERACTIVE_PROGRESS
        && bar_progress > SHARED_CONTENT_INTERACTIVE_PROGRESS
        && content_visibility > SHARED_CONTENT_INTERACTIVE_PROGRESS;
    (visible, interactive)
}

pub(super) fn rects_nearly_equal(a: NSRect, b: NSRect) -> bool {
    (a.origin.x - b.origin.x).abs() < 0.5
        && (a.origin.y - b.origin.y).abs() < 0.5
        && (a.size.width - b.size.width).abs() < 0.5
        && (a.size.height - b.size.height).abs() < 0.5
}

pub(super) fn apply_panel_frame(panel: &NSPanel, frame: NSRect) {
    let current = panel.frame();
    if rects_nearly_equal(current, frame) {
        return;
    }
    let top_left = NSPoint::new(frame.origin.x, frame.origin.y + frame.size.height);
    panel.setContentSize(frame.size);
    panel.setFrameTopLeftPoint(top_left);
}

pub(super) fn centered_top_frame(screen_frame: NSRect, size: NSSize) -> NSRect {
    let snapped_width = size.width.max(1.0).round();
    let snapped_height = size.height.max(1.0).round();
    let top_edge = screen_frame.origin.y + screen_frame.size.height;
    NSRect::new(
        NSPoint::new(
            (screen_frame.origin.x + ((screen_frame.size.width - snapped_width) / 2.0).max(0.0))
                .round(),
            (top_edge - snapped_height).round(),
        ),
        NSSize::new(snapped_width, snapped_height),
    )
}

pub(super) fn compact_pill_height_for_screen(screen: &NSScreen) -> f64 {
    let safe_top = screen.safeAreaInsets().top;
    if safe_top > 0.0 {
        return safe_top;
    }

    let frame = screen.frame();
    let visible = screen.visibleFrame();
    let menu_bar_height =
        (frame.origin.y + frame.size.height) - (visible.origin.y + visible.size.height);
    if menu_bar_height > 5.0 {
        return menu_bar_height;
    }

    if let Some(mtm) = MainThreadMarker::new() {
        if let Some(main_screen) = NSScreen::mainScreen(mtm) {
            let main_frame = main_screen.frame();
            let main_visible = main_screen.visibleFrame();
            let main_menu = (main_frame.origin.y + main_frame.size.height)
                - (main_visible.origin.y + main_visible.size.height);
            if main_menu > 5.0 {
                return main_menu;
            }
        }
    }

    DEFAULT_COMPACT_PILL_HEIGHT
}

pub(super) fn notch_width_for_screen(screen: &NSScreen) -> f64 {
    let left_width = screen.auxiliaryTopLeftArea().size.width;
    let right_width = screen.auxiliaryTopRightArea().size.width;
    if left_width > 0.0 || right_width > 0.0 {
        return (screen.frame().size.width - left_width - right_width).max(0.0);
    }

    (screen.frame().size.width * 0.18).clamp(160.0, 240.0)
}

pub(super) fn screen_has_camera_housing(screen: &NSScreen) -> bool {
    let left_width = screen.auxiliaryTopLeftArea().size.width;
    let right_width = screen.auxiliaryTopRightArea().size.width;
    let center_gap = (screen.frame().size.width - left_width - right_width).max(0.0);
    (left_width > 0.0 || right_width > 0.0) && center_gap > 40.0
}

pub(super) fn shell_width_for_screen(screen: &NSScreen, compact_height: f64) -> f64 {
    let mascot_size = (compact_height - 6.0).min(27.0).max(20.0);
    let compact_wing = mascot_size + 14.0;
    let notch_width = notch_width_for_screen(screen);
    let screen_extra = (screen.frame().size.width * 0.012).clamp(10.0, 22.0);
    let max_width = (screen.frame().size.width - 24.0)
        .min(DEFAULT_PANEL_CANVAS_WIDTH)
        .max(DEFAULT_COMPACT_PILL_WIDTH);
    (notch_width + compact_wing * 2.0 + 10.0 + screen_extra)
        .clamp(DEFAULT_COMPACT_PILL_WIDTH, max_width)
}

pub(super) fn compact_pill_width_for_screen(screen: &NSScreen, compact_height: f64) -> f64 {
    shell_width_for_screen(screen, compact_height)
}

pub(super) fn compact_pill_height_for_screen_rect(
    screen: Option<&NSScreen>,
    fallback_rect: NSRect,
) -> f64 {
    screen
        .map(compact_pill_height_for_screen)
        .unwrap_or_else(|| {
            if fallback_rect.size.height > 0.0 {
                DEFAULT_COMPACT_PILL_HEIGHT
            } else {
                25.0
            }
        })
}

pub(super) fn compact_pill_width_for_screen_rect(
    screen: Option<&NSScreen>,
    compact_height: f64,
) -> f64 {
    screen
        .map(|screen| compact_pill_width_for_screen(screen, compact_height))
        .unwrap_or(DEFAULT_COMPACT_PILL_WIDTH)
}

pub(super) fn expanded_panel_width_for_screen(screen: &NSScreen) -> f64 {
    let compact_height = compact_pill_height_for_screen(screen);
    shell_width_for_screen(screen, compact_height)
}

pub(super) fn expanded_panel_width_for_screen_rect(
    screen: Option<&NSScreen>,
    fallback_rect: NSRect,
) -> f64 {
    screen
        .map(expanded_panel_width_for_screen)
        .unwrap_or(DEFAULT_COMPACT_PILL_WIDTH.min(fallback_rect.size.width.max(1.0)))
}

pub(super) fn panel_canvas_width_for_screen(screen: &NSScreen, compact_height: f64) -> f64 {
    let compact_width = compact_pill_width_for_screen(screen, compact_height);
    expanded_panel_width_for_screen(screen)
        .max(compact_width + 24.0)
        .max(DEFAULT_PANEL_CANVAS_WIDTH)
}

pub(super) fn panel_canvas_width_for_screen_rect(
    screen: Option<&NSScreen>,
    compact_height: f64,
    fallback_rect: NSRect,
) -> f64 {
    screen
        .map(|screen| panel_canvas_width_for_screen(screen, compact_height))
        .unwrap_or_else(|| fallback_rect.size.width.max(DEFAULT_PANEL_CANVAS_WIDTH))
}

pub(super) fn compact_pill_frame(panel: &NSPanel, content_size: NSSize) -> NSRect {
    let compact_height = compact_pill_height_for_screen_rect(
        panel.screen().as_deref(),
        resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame()),
    );
    let compact_width =
        compact_pill_width_for_screen_rect(panel.screen().as_deref(), compact_height);
    let expanded_width = expanded_panel_width_for_screen_rect(
        panel.screen().as_deref(),
        resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame()),
    );
    island_bar_frame(
        content_size,
        0.0,
        compact_width,
        expanded_width,
        compact_height,
        0.0,
    )
}

pub(super) fn island_bar_frame(
    content_size: NSSize,
    _progress: f64,
    compact_width: f64,
    _expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> NSRect {
    NSRect::new(
        NSPoint::new(
            (content_size.width - compact_width) / 2.0,
            content_size.height - compact_height - drop_offset,
        ),
        NSSize::new(compact_width, compact_height),
    )
}

pub(super) fn left_shoulder_frame(pill_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            pill_frame.origin.x - COMPACT_SHOULDER_SIZE,
            pill_frame.origin.y + pill_frame.size.height - COMPACT_SHOULDER_SIZE,
        ),
        NSSize::new(COMPACT_SHOULDER_SIZE, COMPACT_SHOULDER_SIZE),
    )
}

pub(super) fn right_shoulder_frame(pill_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            pill_frame.origin.x + pill_frame.size.width,
            pill_frame.origin.y + pill_frame.size.height - COMPACT_SHOULDER_SIZE,
        ),
        NSSize::new(COMPACT_SHOULDER_SIZE, COMPACT_SHOULDER_SIZE),
    )
}

pub(super) fn expanded_background_frame(
    content_size: NSSize,
    visible_height: f64,
    _bar_progress: f64,
    height_progress: f64,
    _compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> NSRect {
    let height_progress = height_progress.clamp(0.0, 1.0);
    let visible_height = visible_height
        .max(COLLAPSED_PANEL_HEIGHT)
        .min(content_size.height.max(COLLAPSED_PANEL_HEIGHT));
    let height = lerp(
        compact_height,
        (visible_height - drop_offset).max(compact_height),
        height_progress,
    );
    NSRect::new(
        NSPoint::new(
            (content_size.width - expanded_width) / 2.0,
            content_size.height - drop_offset - height,
        ),
        NSSize::new(expanded_width, height),
    )
}

pub(super) fn expanded_cards_frame(container_frame: NSRect, compact_height: f64) -> NSRect {
    let body_height = (container_frame.size.height
        - compact_height
        - EXPANDED_CONTENT_TOP_GAP
        - EXPANDED_CONTENT_BOTTOM_INSET)
        .max(0.0);
    NSRect::new(
        NSPoint::new(EXPANDED_CARDS_SIDE_INSET, EXPANDED_CONTENT_BOTTOM_INSET),
        NSSize::new(
            expanded_cards_width(container_frame.size.width),
            body_height,
        ),
    )
}

pub(super) fn expanded_separator_frame(container_frame: NSRect, compact_height: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(
            14.0,
            (container_frame.size.height - compact_height - 0.5).max(0.0),
        ),
        NSSize::new((container_frame.size.width - 28.0).max(0.0), 1.0),
    )
}

pub(super) fn expanded_total_height(
    snapshot: &RuntimeSnapshot,
    compact_height: f64,
    shared_body_height: Option<f64>,
) -> f64 {
    let estimated_height = estimated_expanded_body_height(snapshot);
    let body_height = shared_body_height
        .map(|shared_height| shared_height.max(estimated_height))
        .unwrap_or(estimated_height)
        .min(EXPANDED_MAX_BODY_HEIGHT);
    compact_height + EXPANDED_CONTENT_TOP_GAP + EXPANDED_CONTENT_BOTTOM_INSET + body_height
}

pub(super) fn panel_transition_canvas_height(start_height: f64, target_height: f64) -> f64 {
    start_height.max(target_height).max(COLLAPSED_PANEL_HEIGHT)
}

pub(super) fn expanded_cards_width(container_width: f64) -> f64 {
    (container_width - (EXPANDED_CARDS_SIDE_INSET * 2.0)).max(0.0)
}

pub(super) fn estimated_expanded_body_height(snapshot: &RuntimeSnapshot) -> f64 {
    estimated_expanded_content_height(snapshot).min(EXPANDED_MAX_BODY_HEIGHT)
}

pub(super) fn estimated_expanded_content_height(snapshot: &RuntimeSnapshot) -> f64 {
    let status_queue = native_status_queue_surface_items();
    if !status_queue.is_empty() {
        let cards = status_queue
            .iter()
            .map(native_status_queue_card_height)
            .sum::<f64>();
        let gaps = EXPANDED_CARD_GAP * (status_queue.len().saturating_sub(1) as f64);
        return cards + gaps;
    }

    let pending_permissions = displayed_default_pending_permissions(snapshot);
    let pending_questions = displayed_default_pending_questions(snapshot);
    let prompt_assist_sessions = displayed_prompt_assist_sessions(snapshot);
    let sessions = displayed_sessions(snapshot, &prompt_assist_sessions);
    let mut heights = Vec::new();

    for pending in pending_permissions.iter() {
        heights.push(pending_permission_card_height(pending));
    }
    for pending in pending_questions.iter() {
        heights.push(pending_question_card_height(pending));
    }
    heights.extend(prompt_assist_sessions.iter().map(prompt_assist_card_height));
    heights.extend(sessions.iter().map(estimated_card_height));

    if heights.is_empty() {
        return 84.0;
    }

    let cards = heights.iter().sum::<f64>();
    let gaps = EXPANDED_CARD_GAP * (heights.len().saturating_sub(1) as f64);
    cards + gaps
}

pub(super) fn absolute_rect(panel_frame: NSRect, local_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            panel_frame.origin.x + local_frame.origin.x,
            panel_frame.origin.y + local_frame.origin.y,
        ),
        local_frame.size,
    )
}

pub(super) fn compose_local_rect(parent_frame: NSRect, child_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            parent_frame.origin.x + child_frame.origin.x,
            parent_frame.origin.y + child_frame.origin.y,
        ),
        child_frame.size,
    )
}

pub(super) fn point_in_rect(point: NSPoint, rect: NSRect) -> bool {
    point.x >= rect.origin.x
        && point.x <= rect.origin.x + rect.size.width
        && point.y >= rect.origin.y
        && point.y <= rect.origin.y + rect.size.height
}

pub(super) fn resolve_screen_frame_for_panel(panel: &NSPanel) -> Option<NSRect> {
    if let Some(screen) = panel.screen() {
        return Some(screen.frame());
    }
    let mtm = MainThreadMarker::new()?;
    NSScreen::mainScreen(mtm)
        .or_else(|| {
            let screens = NSScreen::screens(mtm);
            if screens.is_empty() {
                None
            } else {
                Some(screens.objectAtIndex(0))
            }
        })
        .map(|screen| screen.frame())
}
