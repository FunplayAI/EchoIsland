use super::PanelTransitionFrame;

const SHARED_CONTENT_REVEAL_PROGRESS: f64 = 0.94;
const SHARED_CONTENT_INTERACTIVE_PROGRESS: f64 = 0.985;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRenderProgress {
    pub(crate) bar: f64,
    pub(crate) height: f64,
    pub(crate) shoulder: f64,
    pub(crate) drop: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SharedExpandedContentInput {
    pub(crate) enabled: bool,
    pub(crate) shell_visible: bool,
    pub(crate) height_progress: f64,
    pub(crate) bar_progress: f64,
    pub(crate) cards_height: f64,
    pub(crate) status_surface_active: bool,
    pub(crate) content_visibility: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SharedExpandedContentState {
    pub(crate) visible: bool,
    pub(crate) interactive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SharedExpandedRenderInput {
    pub(crate) enabled: bool,
    pub(crate) shell_visible: bool,
    pub(crate) height_progress: f64,
    pub(crate) bar_progress: f64,
    pub(crate) cards_height: f64,
    pub(crate) status_surface_active: bool,
    pub(crate) content_visibility: f64,
    pub(crate) transitioning: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SharedExpandedRenderState {
    pub(crate) enabled: bool,
    pub(crate) visible: bool,
    pub(crate) interactive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRenderLayerStyleInput {
    pub(crate) shell_visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRenderLayerStyleState {
    pub(crate) shell_visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRenderStateInput {
    pub(crate) shared_expanded_enabled: bool,
    pub(crate) shell_visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) cards_height: f64,
    pub(crate) status_surface_active: bool,
    pub(crate) content_visibility: f64,
    pub(crate) transitioning: bool,
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRenderState {
    pub(crate) shared: SharedExpandedRenderState,
    pub(crate) layer_style: PanelRenderLayerStyleState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SharedBodyHeightDecisionInput {
    pub(crate) current_height: Option<f64>,
    pub(crate) requested_height: f64,
    pub(crate) has_snapshot: bool,
    pub(crate) update_threshold: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SharedBodyHeightDecision {
    pub(crate) next_height: f64,
    pub(crate) should_update: bool,
    pub(crate) should_rerender: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRect {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelSize {
    pub(crate) width: f64,
    pub(crate) height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelGeometryMetrics {
    pub(crate) compact_height: f64,
    pub(crate) compact_width: f64,
    pub(crate) expanded_width: f64,
    pub(crate) panel_width: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelScreenTopArea {
    pub(crate) screen_width: f64,
    pub(crate) auxiliary_left_width: f64,
    pub(crate) auxiliary_right_width: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelScreenWidthInput {
    pub(crate) top_area: PanelScreenTopArea,
    pub(crate) compact_height: f64,
    pub(crate) default_compact_width: f64,
    pub(crate) expanded_width_delta: f64,
    pub(crate) default_expanded_width: f64,
    pub(crate) default_canvas_width: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelLayoutInput {
    pub(crate) screen_frame: PanelRect,
    pub(crate) metrics: PanelGeometryMetrics,
    pub(crate) canvas_height: f64,
    pub(crate) visible_height: f64,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) drop_progress: f64,
    pub(crate) content_visibility: f64,
    pub(crate) collapsed_height: f64,
    pub(crate) drop_distance: f64,
    pub(crate) content_top_gap: f64,
    pub(crate) content_bottom_inset: f64,
    pub(crate) cards_side_inset: f64,
    pub(crate) shoulder_size: f64,
    pub(crate) separator_side_inset: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelLayout {
    pub(crate) panel_frame: PanelRect,
    pub(crate) content_frame: PanelRect,
    pub(crate) pill_frame: PanelRect,
    pub(crate) left_shoulder_frame: PanelRect,
    pub(crate) right_shoulder_frame: PanelRect,
    pub(crate) expanded_frame: PanelRect,
    pub(crate) cards_frame: PanelRect,
    pub(crate) separator_frame: PanelRect,
    pub(crate) shared_content_frame: PanelRect,
    pub(crate) shell_visible: bool,
    pub(crate) separator_visibility: f64,
}

pub(crate) fn resolve_panel_render_progress(frame: PanelTransitionFrame) -> PanelRenderProgress {
    PanelRenderProgress {
        bar: frame.bar_progress.clamp(0.0, 1.0),
        height: frame.height_progress.clamp(0.0, 1.0),
        shoulder: frame.shoulder_progress.clamp(0.0, 1.0),
        drop: frame.drop_progress.clamp(0.0, 1.0),
    }
}

pub(crate) fn resolve_shared_expanded_content_state(
    input: SharedExpandedContentInput,
) -> SharedExpandedContentState {
    let visible = input.enabled
        && input.shell_visible
        && input.height_progress > SHARED_CONTENT_REVEAL_PROGRESS
        && input.content_visibility > SHARED_CONTENT_REVEAL_PROGRESS
        && input.cards_height > 4.0
        && !input.status_surface_active;
    let interactive = visible
        && input.height_progress > SHARED_CONTENT_INTERACTIVE_PROGRESS
        && input.bar_progress > SHARED_CONTENT_INTERACTIVE_PROGRESS
        && input.content_visibility > SHARED_CONTENT_INTERACTIVE_PROGRESS;

    SharedExpandedContentState {
        visible,
        interactive,
    }
}

pub(crate) fn resolve_shared_expanded_render_state(
    input: SharedExpandedRenderInput,
) -> SharedExpandedRenderState {
    let state = resolve_shared_expanded_content_state(SharedExpandedContentInput {
        enabled: input.enabled,
        shell_visible: input.shell_visible,
        height_progress: input.height_progress,
        bar_progress: input.bar_progress,
        cards_height: input.cards_height,
        status_surface_active: input.status_surface_active,
        content_visibility: input.content_visibility,
    });

    SharedExpandedRenderState {
        enabled: input.enabled,
        visible: state.visible && !input.transitioning,
        interactive: state.interactive && !input.transitioning,
    }
}

pub(crate) fn resolve_panel_render_layer_style_state(
    input: PanelRenderLayerStyleInput,
) -> PanelRenderLayerStyleState {
    PanelRenderLayerStyleState {
        shell_visible: input.shell_visible,
        separator_visibility: input.separator_visibility,
        shared_visible: input.shared_visible,
        bar_progress: input.bar_progress,
        height_progress: input.height_progress,
        headline_emphasized: input.headline_emphasized,
        edge_actions_visible: input.edge_actions_visible,
    }
}

pub(crate) fn resolve_panel_render_state(input: PanelRenderStateInput) -> PanelRenderState {
    let shared = resolve_shared_expanded_render_state(SharedExpandedRenderInput {
        enabled: input.shared_expanded_enabled,
        shell_visible: input.shell_visible,
        height_progress: input.height_progress,
        bar_progress: input.bar_progress,
        cards_height: input.cards_height,
        status_surface_active: input.status_surface_active,
        content_visibility: input.content_visibility,
        transitioning: input.transitioning,
    });
    let layer_style = resolve_panel_render_layer_style_state(PanelRenderLayerStyleInput {
        shell_visible: input.shell_visible,
        separator_visibility: input.separator_visibility,
        shared_visible: shared.visible,
        bar_progress: input.bar_progress,
        height_progress: input.height_progress,
        headline_emphasized: input.headline_emphasized,
        edge_actions_visible: input.edge_actions_visible,
    });

    PanelRenderState {
        shared,
        layer_style,
    }
}

pub(crate) fn resolve_shared_body_height_decision(
    input: SharedBodyHeightDecisionInput,
) -> SharedBodyHeightDecision {
    let next_height = input.requested_height.max(0.0);
    let threshold = input.update_threshold.max(0.0);
    let should_update = !input
        .current_height
        .is_some_and(|current| (current - next_height).abs() < threshold);

    SharedBodyHeightDecision {
        next_height,
        should_update,
        should_rerender: should_update && input.has_snapshot,
    }
}

pub(crate) fn resolve_centered_top_frame(screen_frame: PanelRect, size: PanelSize) -> PanelRect {
    let snapped_width = size.width.max(1.0).round();
    let snapped_height = size.height.max(1.0).round();
    let top_edge = screen_frame.y + screen_frame.height;

    PanelRect {
        x: (screen_frame.x + ((screen_frame.width - snapped_width) / 2.0).max(0.0)).round(),
        y: (top_edge - snapped_height).round(),
        width: snapped_width,
        height: snapped_height,
    }
}

pub(crate) fn rects_nearly_equal(a: PanelRect, b: PanelRect, tolerance: f64) -> bool {
    let tolerance = tolerance.max(0.0);
    (a.x - b.x).abs() < tolerance
        && (a.y - b.y).abs() < tolerance
        && (a.width - b.width).abs() < tolerance
        && (a.height - b.height).abs() < tolerance
}

pub(crate) fn absolute_rect(parent_frame: PanelRect, local_frame: PanelRect) -> PanelRect {
    compose_local_rect(parent_frame, local_frame)
}

pub(crate) fn compose_local_rect(parent_frame: PanelRect, child_frame: PanelRect) -> PanelRect {
    PanelRect {
        x: parent_frame.x + child_frame.x,
        y: parent_frame.y + child_frame.y,
        width: child_frame.width,
        height: child_frame.height,
    }
}

pub(crate) fn point_in_rect(point: PanelPoint, rect: PanelRect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

pub(crate) fn resolve_panel_screen_has_camera_housing(top_area: PanelScreenTopArea) -> bool {
    let center_gap = resolve_panel_screen_center_gap(top_area);
    (top_area.auxiliary_left_width > 0.0 || top_area.auxiliary_right_width > 0.0)
        && center_gap > 40.0
}

pub(crate) fn resolve_panel_notch_width(top_area: PanelScreenTopArea) -> f64 {
    if top_area.auxiliary_left_width > 0.0 || top_area.auxiliary_right_width > 0.0 {
        return resolve_panel_screen_center_gap(top_area);
    }

    (top_area.screen_width * 0.18).clamp(160.0, 240.0)
}

pub(crate) fn resolve_panel_shell_width(input: PanelScreenWidthInput) -> f64 {
    if !resolve_panel_screen_has_camera_housing(input.top_area) {
        return resolve_panel_shell_width_for_non_camera_housing(
            input.compact_height,
            input.default_compact_width,
        );
    }

    let mascot_size = resolve_compact_mascot_size(input.compact_height);
    let compact_wing = mascot_size + 14.0;
    let notch_width = resolve_panel_notch_width(input.top_area);
    let screen_extra = (input.top_area.screen_width * 0.012).clamp(10.0, 22.0);
    let max_width = (input.top_area.screen_width - 24.0)
        .min(input.default_canvas_width)
        .max(input.default_compact_width);
    (notch_width + compact_wing * 2.0 + 10.0 + screen_extra)
        .clamp(input.default_compact_width, max_width)
}

pub(crate) fn resolve_panel_shell_width_for_non_camera_housing(
    compact_height: f64,
    default_compact_width: f64,
) -> f64 {
    let mascot_size = resolve_compact_mascot_size(compact_height);
    let minimum_content_width = mascot_size + 14.0 + 138.0;
    default_compact_width.max(minimum_content_width)
}

pub(crate) fn resolve_panel_expanded_width(input: PanelScreenWidthInput) -> f64 {
    if !resolve_panel_screen_has_camera_housing(input.top_area) {
        return input.default_expanded_width;
    }

    resolve_panel_expanded_width_for_camera_housing(
        resolve_panel_shell_width(input),
        input.expanded_width_delta,
        input.default_canvas_width,
    )
}

pub(crate) fn resolve_panel_expanded_width_for_camera_housing(
    compact_width: f64,
    expanded_width_delta: f64,
    default_canvas_width: f64,
) -> f64 {
    (compact_width + expanded_width_delta).clamp(compact_width, default_canvas_width)
}

pub(crate) fn resolve_panel_canvas_width(input: PanelScreenWidthInput) -> f64 {
    let compact_width = resolve_panel_shell_width(input);
    resolve_panel_expanded_width(input)
        .max(compact_width + 24.0)
        .max(input.default_canvas_width)
}

pub(crate) fn resolve_fallback_panel_expanded_width(
    fallback_width: f64,
    default_compact_width: f64,
) -> f64 {
    default_compact_width.min(fallback_width.max(1.0))
}

pub(crate) fn resolve_fallback_panel_canvas_width(
    fallback_width: f64,
    default_canvas_width: f64,
) -> f64 {
    fallback_width.max(default_canvas_width)
}

pub(crate) fn resolve_panel_layout(input: PanelLayoutInput) -> PanelLayout {
    let canvas_height = input.canvas_height.max(input.collapsed_height);
    let visible_height = input
        .visible_height
        .clamp(input.collapsed_height, canvas_height);
    let content_frame = PanelRect {
        x: 0.0,
        y: 0.0,
        width: input.metrics.panel_width,
        height: canvas_height,
    };
    let drop_offset = input.drop_distance * input.drop_progress;
    let panel_frame = resolve_centered_top_frame(
        input.screen_frame,
        PanelSize {
            width: content_frame.width,
            height: content_frame.height,
        },
    );
    let pill_frame = resolve_island_bar_frame(
        PanelSize {
            width: content_frame.width,
            height: content_frame.height,
        },
        input.bar_progress,
        input.metrics.compact_width,
        input.metrics.expanded_width,
        input.metrics.compact_height,
        drop_offset,
    );
    let expanded_frame = resolve_expanded_background_frame(
        PanelSize {
            width: content_frame.width,
            height: content_frame.height,
        },
        visible_height,
        input.bar_progress,
        input.height_progress,
        input.metrics.compact_width,
        input.metrics.expanded_width,
        input.metrics.compact_height,
        drop_offset,
        input.collapsed_height,
    );
    let cards_frame = resolve_expanded_cards_frame(
        expanded_frame,
        input.metrics.compact_height,
        input.content_top_gap,
        input.content_bottom_inset,
        input.cards_side_inset,
    );
    PanelLayout {
        panel_frame,
        content_frame,
        pill_frame,
        left_shoulder_frame: resolve_left_shoulder_frame(pill_frame, input.shoulder_size),
        right_shoulder_frame: resolve_right_shoulder_frame(pill_frame, input.shoulder_size),
        expanded_frame,
        cards_frame,
        separator_frame: resolve_expanded_separator_frame(
            expanded_frame,
            input.metrics.compact_height,
            input.separator_side_inset,
        ),
        shared_content_frame: absolute_rect(
            panel_frame,
            compose_local_rect(expanded_frame, cards_frame),
        ),
        shell_visible: input.bar_progress > 0.01 || input.height_progress > 0.01,
        separator_visibility: (input.height_progress.min(input.content_visibility) * 0.88)
            .clamp(0.0, 0.88),
    }
}

pub(crate) fn resolve_island_bar_frame(
    content_size: PanelSize,
    progress: f64,
    compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> PanelRect {
    let width = lerp(compact_width, expanded_width, progress);
    PanelRect {
        x: (content_size.width - width) / 2.0,
        y: content_size.height - compact_height - drop_offset,
        width,
        height: compact_height,
    }
}

pub(crate) fn resolve_left_shoulder_frame(pill_frame: PanelRect, shoulder_size: f64) -> PanelRect {
    PanelRect {
        x: pill_frame.x - shoulder_size,
        y: pill_frame.y + pill_frame.height - shoulder_size,
        width: shoulder_size,
        height: shoulder_size,
    }
}

pub(crate) fn resolve_right_shoulder_frame(pill_frame: PanelRect, shoulder_size: f64) -> PanelRect {
    PanelRect {
        x: pill_frame.x + pill_frame.width,
        y: pill_frame.y + pill_frame.height - shoulder_size,
        width: shoulder_size,
        height: shoulder_size,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_expanded_background_frame(
    content_size: PanelSize,
    visible_height: f64,
    bar_progress: f64,
    height_progress: f64,
    compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
    collapsed_height: f64,
) -> PanelRect {
    let height_progress = height_progress.clamp(0.0, 1.0);
    let width = lerp(compact_width, expanded_width, bar_progress);
    let visible_height = visible_height
        .max(collapsed_height)
        .min(content_size.height.max(collapsed_height));
    let height = lerp(
        compact_height,
        (visible_height - drop_offset).max(compact_height),
        height_progress,
    );
    PanelRect {
        x: (content_size.width - width) / 2.0,
        y: content_size.height - drop_offset - height,
        width,
        height,
    }
}

pub(crate) fn resolve_expanded_cards_frame(
    container_frame: PanelRect,
    compact_height: f64,
    top_gap: f64,
    bottom_inset: f64,
    side_inset: f64,
) -> PanelRect {
    let body_height = (container_frame.height - compact_height - top_gap - bottom_inset).max(0.0);
    PanelRect {
        x: side_inset,
        y: bottom_inset,
        width: resolve_expanded_cards_width(container_frame.width, side_inset),
        height: body_height,
    }
}

pub(crate) fn resolve_expanded_separator_frame(
    container_frame: PanelRect,
    compact_height: f64,
    side_inset: f64,
) -> PanelRect {
    PanelRect {
        x: side_inset,
        y: (container_frame.height - compact_height - 0.5).max(0.0),
        width: (container_frame.width - (side_inset * 2.0)).max(0.0),
        height: 1.0,
    }
}

pub(crate) fn resolve_expanded_cards_width(container_width: f64, side_inset: f64) -> f64 {
    (container_width - (side_inset * 2.0)).max(0.0)
}

pub(crate) fn resolve_expanded_total_height(
    estimated_body_height: f64,
    shared_body_height: Option<f64>,
    compact_height: f64,
    top_gap: f64,
    bottom_inset: f64,
    max_body_height: f64,
) -> f64 {
    let body_height = shared_body_height
        .map(|shared_height| shared_height.max(estimated_body_height))
        .unwrap_or(estimated_body_height)
        .min(max_body_height);
    compact_height + top_gap + bottom_inset + body_height
}

pub(crate) fn resolve_panel_transition_canvas_height(
    start_height: f64,
    target_height: f64,
    collapsed_height: f64,
) -> f64 {
    start_height.max(target_height).max(collapsed_height)
}

pub(crate) fn resolve_next_stacked_card_frame(
    cursor_y: &mut f64,
    needs_gap: bool,
    height: f64,
    expanded_width: f64,
    card_gap: f64,
    card_overhang: f64,
) -> Option<PanelRect> {
    if needs_gap {
        *cursor_y -= card_gap;
    }
    if *cursor_y < height {
        return None;
    }

    *cursor_y -= height;
    Some(PanelRect {
        x: -card_overhang,
        y: *cursor_y,
        width: expanded_width + (card_overhang * 2.0),
        height,
    })
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}

fn resolve_panel_screen_center_gap(top_area: PanelScreenTopArea) -> f64 {
    (top_area.screen_width - top_area.auxiliary_left_width - top_area.auxiliary_right_width)
        .max(0.0)
}

fn resolve_compact_mascot_size(compact_height: f64) -> f64 {
    (compact_height - 6.0).min(27.0).max(20.0)
}
