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

    expanded_container.setHidden(!state.shell_visible);
    expanded_container.setAlphaValue(if state.shell_visible { 1.0 } else { 0.0 });
    body_separator.setHidden(state.separator_visibility <= 0.02);
    body_separator.setAlphaValue(state.separator_visibility);
    cards_container.setHidden(state.shared_visible);

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
