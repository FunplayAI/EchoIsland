use tauri::{AppHandle, Manager};
use tracing::debug;

use crate::{
    constants::MAIN_WINDOW_LABEL,
    island_window::{
        apply_island_bar_stage, apply_island_panel_stage, apply_island_window_mode,
        show_main_window,
    },
    platform::current_platform_capabilities,
};

pub struct WindowSurfaceService<'a, R: tauri::Runtime> {
    app: &'a AppHandle<R>,
}

impl<'a, R: tauri::Runtime> WindowSurfaceService<'a, R> {
    pub fn new(app: &'a AppHandle<R>) -> Self {
        Self { app }
    }

    pub fn initialize_compact(&self) -> Result<(), String> {
        if !supports_window_surface_shaping() {
            debug!("window surface shaping unsupported; skipping compact initialization");
            return Ok(());
        }
        let window = self.main_window()?;
        apply_island_window_mode(&window, false).map_err(|error| error.to_string())
    }

    pub fn set_expanded_passive(&self, expanded: bool) -> Result<(), String> {
        if !supports_window_surface_shaping() {
            debug!(
                expanded,
                "window surface shaping unsupported; skipping mode update"
            );
            return Ok(());
        }
        let window = self.main_window()?;
        apply_island_window_mode(&window, expanded).map_err(|error| error.to_string())
    }

    pub fn set_bar_stage_passive(&self) -> Result<(), String> {
        if !supports_window_surface_shaping() {
            debug!("window surface shaping unsupported; skipping bar stage");
            return Ok(());
        }
        let window = self.main_window()?;
        apply_island_bar_stage(&window).map_err(|error| error.to_string())
    }

    pub fn set_panel_stage_passive(&self, height: f64) -> Result<(), String> {
        if !supports_window_surface_shaping() {
            debug!(
                height,
                "window surface shaping unsupported; skipping panel stage"
            );
            return Ok(());
        }
        let window = self.main_window()?;
        apply_island_panel_stage(&window, height).map_err(|error| error.to_string())
    }

    pub fn show_main_window_interactive(&self) -> Result<(), String> {
        show_main_window(self.app, MAIN_WINDOW_LABEL).map_err(|error| error.to_string())
    }

    pub fn hide_main_window(&self) -> Result<(), String> {
        let window = self.main_window()?;
        window.hide().map_err(|error| error.to_string())
    }

    fn main_window(&self) -> Result<tauri::WebviewWindow<R>, String> {
        self.app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .ok_or_else(|| "main window not found".to_string())
    }
}

fn supports_window_surface_shaping() -> bool {
    current_platform_capabilities().can_shape_window_region
}

#[cfg(test)]
mod tests {
    use super::supports_window_surface_shaping;

    #[test]
    fn window_surface_support_matches_platform_capabilities() {
        #[cfg(target_os = "windows")]
        {
            assert!(supports_window_surface_shaping());
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(!supports_window_surface_shaping());
        }
    }
}
