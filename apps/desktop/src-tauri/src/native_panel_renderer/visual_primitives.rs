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
    },
    MascotDot {
        center: PanelPoint,
        radius: f64,
        pose: SceneMascotPose,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NativePanelVisualPlan {
    pub(crate) hidden: bool,
    pub(crate) primitives: Vec<NativePanelVisualPrimitive>,
}

#[cfg(test)]
mod tests {
    use super::{NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive};
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
                    pose: SceneMascotPose::Complete,
                },
            ],
        };

        assert!(!plan.hidden);
        assert_eq!(plan.primitives.len(), 2);
    }
}
