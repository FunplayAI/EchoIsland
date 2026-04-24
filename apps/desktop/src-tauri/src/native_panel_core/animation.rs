use super::PanelTransitionFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PanelAnimationKind {
    Open,
    Close,
    SurfaceSwitch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelAnimationDescriptor {
    pub(crate) kind: PanelAnimationKind,
    pub(crate) canvas_height: f64,
    pub(crate) visible_height: f64,
    pub(crate) width_progress: f64,
    pub(crate) height_progress: f64,
    pub(crate) shoulder_progress: f64,
    pub(crate) drop_progress: f64,
    pub(crate) cards_progress: f64,
}

pub(crate) fn resolve_panel_animation_descriptor(
    kind: PanelAnimationKind,
    frame: PanelTransitionFrame,
) -> PanelAnimationDescriptor {
    PanelAnimationDescriptor {
        kind,
        canvas_height: frame.canvas_height,
        visible_height: frame.visible_height,
        width_progress: frame.bar_progress.clamp(0.0, 1.0),
        height_progress: frame.height_progress.clamp(0.0, 1.0),
        shoulder_progress: frame.shoulder_progress.clamp(0.0, 1.0),
        drop_progress: frame.drop_progress.clamp(0.0, 1.0),
        cards_progress: frame.cards_progress.clamp(0.0, 1.0),
    }
}
