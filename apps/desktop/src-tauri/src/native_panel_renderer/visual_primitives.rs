use crate::{
    native_panel_core::{PanelPoint, PanelRect},
    native_panel_scene::SceneMascotPose,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativePanelVisualColor {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl NativePanelVisualColor {
    pub(crate) const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NativePanelVisualPrimitive {
    RoundRect {
        frame: PanelRect,
        radius: f64,
        color: NativePanelVisualColor,
    },
    Rect {
        frame: PanelRect,
        color: NativePanelVisualColor,
    },
    Ellipse {
        frame: PanelRect,
        color: NativePanelVisualColor,
    },
    StrokeLine {
        from: PanelPoint,
        to: PanelPoint,
        color: NativePanelVisualColor,
        width: i32,
    },
    Text {
        origin: PanelPoint,
        max_width: f64,
        text: String,
        color: NativePanelVisualColor,
        size: i32,
        weight: NativePanelVisualTextWeight,
        alignment: NativePanelVisualTextAlignment,
    },
    MascotDot {
        center: PanelPoint,
        radius: f64,
        scale_x: f64,
        scale_y: f64,
        pose: SceneMascotPose,
    },
    CompactShoulder {
        frame: PanelRect,
        side: NativePanelVisualShoulderSide,
        progress: f64,
        fill: NativePanelVisualColor,
        border: NativePanelVisualColor,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelVisualTextWeight {
    Normal,
    Semibold,
    Bold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelVisualTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelVisualShoulderSide {
    Left,
    Right,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NativePanelVisualPlan {
    pub(crate) hidden: bool,
    pub(crate) primitives: Vec<NativePanelVisualPrimitive>,
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
        NativePanelVisualShoulderSide,
    };
    use crate::{
        native_panel_core::{PanelPoint, PanelRect},
        native_panel_scene::SceneMascotPose,
    };

    #[test]
    fn visual_plan_carries_platform_neutral_primitives() {
        let plan = NativePanelVisualPlan {
            hidden: false,
            primitives: vec![
                NativePanelVisualPrimitive::RoundRect {
                    frame: PanelRect {
                        x: 0.0,
                        y: 0.0,
                        width: 120.0,
                        height: 48.0,
                    },
                    radius: 24.0,
                    color: NativePanelVisualColor::rgb(18, 18, 22),
                },
                NativePanelVisualPrimitive::MascotDot {
                    center: PanelPoint { x: 24.0, y: 24.0 },
                    radius: 10.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    pose: SceneMascotPose::Complete,
                },
                NativePanelVisualPrimitive::CompactShoulder {
                    frame: PanelRect {
                        x: -6.0,
                        y: 31.0,
                        width: 6.0,
                        height: 6.0,
                    },
                    side: NativePanelVisualShoulderSide::Left,
                    progress: 0.0,
                    fill: NativePanelVisualColor::rgb(12, 12, 15),
                    border: NativePanelVisualColor::rgb(44, 44, 50),
                },
            ],
        };

        assert!(!plan.hidden);
        assert_eq!(plan.primitives.len(), 3);
    }
}
