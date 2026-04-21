#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelStyleResolverInput {
    pub(crate) shell_visible: bool,
    pub(crate) separator_visibility: f64,
    pub(crate) shared_visible: bool,
    pub(crate) bar_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
    pub(crate) compact_pill_radius: f64,
    pub(crate) panel_morph_pill_radius: f64,
    pub(crate) expanded_panel_radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelStyleResolved {
    pub(crate) expanded_hidden: bool,
    pub(crate) expanded_alpha: f64,
    pub(crate) separator_hidden: bool,
    pub(crate) separator_alpha: f64,
    pub(crate) cards_hidden: bool,
    pub(crate) highlight_hidden: bool,
    pub(crate) highlight_alpha: f64,
    pub(crate) actions_hidden: bool,
    pub(crate) action_alpha: f64,
    pub(crate) action_scale: f64,
    pub(crate) pill_corner_radius: f64,
    pub(crate) use_compact_corner_mask: bool,
    pub(crate) pill_border_width: f64,
    pub(crate) expanded_corner_radius: f64,
}

pub(crate) fn resolve_panel_style(input: PanelStyleResolverInput) -> PanelStyleResolved {
    let bar_progress = input.bar_progress.clamp(0.0, 1.0);
    let height_progress = input.height_progress.clamp(0.0, 1.0);
    let highlight_alpha = if input.headline_emphasized {
        lerp(0.0, 0.12, bar_progress)
    } else {
        0.0
    };
    let action_progress = if input.edge_actions_visible {
        edge_action_progress(bar_progress)
    } else {
        0.0
    };
    let action_alpha = action_progress;
    let action_scale = lerp(0.82, 1.0, action_progress);

    PanelStyleResolved {
        expanded_hidden: !input.shell_visible,
        expanded_alpha: if input.shell_visible { 1.0 } else { 0.0 },
        separator_hidden: input.separator_visibility <= 0.02,
        separator_alpha: input.separator_visibility,
        cards_hidden: input.shared_visible,
        highlight_hidden: highlight_alpha <= 0.02,
        highlight_alpha,
        actions_hidden: action_alpha <= 0.02,
        action_alpha,
        action_scale,
        pill_corner_radius: lerp(
            input.compact_pill_radius,
            input.panel_morph_pill_radius,
            bar_progress,
        ),
        use_compact_corner_mask: bar_progress <= 0.01,
        pill_border_width: lerp(1.0, 0.0, bar_progress),
        expanded_corner_radius: lerp(
            input.compact_pill_radius,
            input.expanded_panel_radius,
            bar_progress.max(height_progress),
        ),
    }
}

fn edge_action_progress(bar_progress: f64) -> f64 {
    ((bar_progress - 0.48) / 0.52).clamp(0.0, 1.0)
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}
