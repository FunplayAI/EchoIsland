use std::sync::Mutex;

use chrono::Utc;
use echoisland_runtime::RuntimeSnapshot;

use crate::{
    native_panel_core::PanelState,
    native_panel_scene::{
        PanelSceneBuildInput, SessionSurfaceScene, SettingsSurfaceScene, StatusSurfaceScene,
        SurfaceScene, build_panel_scene,
    },
};

#[derive(Default)]
pub(crate) struct WebPanelSceneState {
    panel_state: Mutex<PanelState>,
}

impl WebPanelSceneState {
    pub(crate) fn build_surface_scenes(
        &self,
        raw_snapshot: &RuntimeSnapshot,
        input: &PanelSceneBuildInput,
    ) -> Result<
        (
            SurfaceScene,
            StatusSurfaceScene,
            SessionSurfaceScene,
            SettingsSurfaceScene,
        ),
        String,
    > {
        let mut panel_state = self
            .panel_state
            .lock()
            .map_err(|_| "web panel scene state poisoned".to_string())?;
        let sync_result = crate::native_panel_core::sync_panel_snapshot_state(
            &mut panel_state,
            raw_snapshot,
            Utc::now(),
        );
        let scene = build_panel_scene(&panel_state, &sync_result.displayed_snapshot, input);
        Ok((
            scene.surface_scene,
            scene.status_surface,
            scene.session_surface,
            scene.settings_surface,
        ))
    }

    pub(crate) fn build_status_surface_scene(
        &self,
        raw_snapshot: &RuntimeSnapshot,
    ) -> Result<StatusSurfaceScene, String> {
        self.build_surface_scenes(raw_snapshot, &PanelSceneBuildInput::default())
            .map(|(_, status_surface, _, _)| status_surface)
    }
}
