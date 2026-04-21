use super::*;
use crate::native_panel_core::{
    PanelRenderLayerStyleState, PanelStyleResolverInput, resolve_panel_style,
};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_panel_layer_styles(
    refs: &NativePanelRefs,
    state: PanelRenderLayerStyleState,
) {
    let resolved = resolve_panel_style(PanelStyleResolverInput {
        shell_visible: state.shell_visible,
        separator_visibility: state.separator_visibility,
        shared_visible: state.shared_visible,
        bar_progress: state.bar_progress,
        height_progress: state.height_progress,
        headline_emphasized: state.headline_emphasized,
        edge_actions_visible: state.edge_actions_visible,
        compact_pill_radius: COMPACT_PILL_RADIUS,
        panel_morph_pill_radius: PANEL_MORPH_PILL_RADIUS,
        expanded_panel_radius: EXPANDED_PANEL_RADIUS,
    });
    let expanded_container = refs.expanded_container;
    let body_separator = refs.body_separator;
    let cards_container = refs.cards_container;
    let top_highlight = refs.top_highlight;
    let settings_button = refs.settings_button;
    let quit_button = refs.quit_button;

    expanded_container.setHidden(resolved.expanded_hidden);
    expanded_container.setAlphaValue(resolved.expanded_alpha);
    body_separator.setHidden(resolved.separator_hidden);
    body_separator.setAlphaValue(resolved.separator_alpha);
    cards_container.setHidden(resolved.cards_hidden);
    top_highlight.setHidden(resolved.highlight_hidden);
    top_highlight.setAlphaValue(resolved.highlight_alpha);
    if let Some(layer) = top_highlight.layer() {
        layer.setOpacity(resolved.highlight_alpha as f32);
    }
    settings_button.setHidden(resolved.actions_hidden);
    settings_button.setAlphaValue(resolved.action_alpha);
    quit_button.setHidden(resolved.actions_hidden);
    quit_button.setAlphaValue(resolved.action_alpha);
    if let Some(layer) = settings_button.layer() {
        layer.setAffineTransform(CGAffineTransformMakeScale(
            resolved.action_scale,
            resolved.action_scale,
        ));
    }
    if let Some(layer) = quit_button.layer() {
        layer.setAffineTransform(CGAffineTransformMakeScale(
            resolved.action_scale,
            resolved.action_scale,
        ));
    }

    if let Some(layer) = refs.pill_view.layer() {
        layer.setCornerRadius(resolved.pill_corner_radius);
        if resolved.use_compact_corner_mask {
            layer.setMaskedCorners(compact_pill_corner_mask());
        } else {
            layer.setMaskedCorners(all_corner_mask());
        }
        layer.setBorderWidth(resolved.pill_border_width);
    }
    if let Some(layer) = expanded_container.layer() {
        layer.setCornerRadius(resolved.expanded_corner_radius);
        layer.setBorderWidth(0.0);
        layer.setOpacity(resolved.expanded_alpha as f32);
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
