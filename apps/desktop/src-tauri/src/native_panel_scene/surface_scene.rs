use serde::Serialize;

use crate::native_panel_core::ExpandedSurface;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SurfaceScene {
    pub(crate) mode: SurfaceSceneMode,
    pub(crate) headline_text: String,
    pub(crate) headline_emphasized: bool,
    pub(crate) edge_actions_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SurfaceSceneMode {
    Default,
    Status,
    Settings,
}

pub(crate) fn surface_scene_mode(surface: ExpandedSurface) -> SurfaceSceneMode {
    match surface {
        ExpandedSurface::Default => SurfaceSceneMode::Default,
        ExpandedSurface::Status => SurfaceSceneMode::Status,
        ExpandedSurface::Settings => SurfaceSceneMode::Settings,
    }
}
