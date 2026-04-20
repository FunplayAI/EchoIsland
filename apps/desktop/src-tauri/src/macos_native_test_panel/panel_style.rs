use super::*;

#[derive(Clone, Copy)]
pub(super) struct PanelLayerStyleState {
    pub(super) shell_visible: bool,
    pub(super) separator_visibility: f64,
    pub(super) shared_visible: bool,
    pub(super) bar_progress: f64,
    pub(super) height_progress: f64,
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_panel_layer_styles(refs: &NativePanelRefs, state: PanelLayerStyleState) {
    let expanded_container = refs.expanded_container;
    let body_separator = refs.body_separator;
    let cards_container = refs.cards_container;
    let settings_button = refs.settings_button;
    let quit_button = refs.quit_button;

    expanded_container.setHidden(!state.shell_visible);
    expanded_container.setAlphaValue(if state.shell_visible { 1.0 } else { 0.0 });
    body_separator.setHidden(state.separator_visibility <= 0.02);
    body_separator.setAlphaValue(state.separator_visibility);
    cards_container.setHidden(state.shared_visible);
    let action_progress = edge_action_progress(state.bar_progress);
    let action_alpha = action_progress;
    let action_scale = lerp(0.82, 1.0, action_progress);
    let actions_hidden = action_alpha <= 0.02;
    settings_button.setHidden(actions_hidden);
    settings_button.setAlphaValue(action_alpha);
    quit_button.setHidden(actions_hidden);
    quit_button.setAlphaValue(action_alpha);
    if let Some(layer) = settings_button.layer() {
        layer.setAffineTransform(CGAffineTransformMakeScale(action_scale, action_scale));
    }
    if let Some(layer) = quit_button.layer() {
        layer.setAffineTransform(CGAffineTransformMakeScale(action_scale, action_scale));
    }

    if let Some(layer) = refs.pill_view.layer() {
        layer.setCornerRadius(lerp(
            COMPACT_PILL_RADIUS,
            PANEL_MORPH_PILL_RADIUS,
            state.bar_progress,
        ));
        if state.bar_progress <= 0.01 {
            layer.setMaskedCorners(compact_pill_corner_mask());
        } else {
            layer.setMaskedCorners(all_corner_mask());
        }
        layer.setBorderWidth(lerp(1.0, 0.0, state.bar_progress));
    }
    if let Some(layer) = expanded_container.layer() {
        layer.setCornerRadius(lerp(
            COMPACT_PILL_RADIUS,
            EXPANDED_PANEL_RADIUS,
            state.bar_progress.max(state.height_progress),
        ));
        layer.setBorderWidth(0.0);
        layer.setOpacity(if state.shell_visible { 1.0 } else { 0.0 });
    }
}

pub(super) fn compact_pill_corner_mask() -> CACornerMask {
    CACornerMask::LayerMinXMinYCorner | CACornerMask::LayerMaxXMinYCorner
}

fn all_corner_mask() -> CACornerMask {
    CACornerMask::LayerMinXMinYCorner
        | CACornerMask::LayerMaxXMinYCorner
        | CACornerMask::LayerMinXMaxYCorner
        | CACornerMask::LayerMaxXMaxYCorner
}

fn edge_action_progress(bar_progress: f64) -> f64 {
    ((bar_progress - 0.48) / 0.52).clamp(0.0, 1.0)
}
