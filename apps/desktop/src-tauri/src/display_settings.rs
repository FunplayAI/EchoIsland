use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{
    constants::MAIN_WINDOW_LABEL,
    native_panel_core::{PanelDisplayGeometry, PanelRect, panel_display_key},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOption {
    pub index: usize,
    pub key: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

pub fn display_key_for_panel_geometry(geometry: PanelDisplayGeometry) -> String {
    panel_display_key(geometry)
}

pub fn panel_geometry_for_monitor(monitor: &tauri::Monitor) -> PanelDisplayGeometry {
    let position = monitor.position();
    let size = monitor.size();
    PanelDisplayGeometry {
        x: position.x as i64,
        y: position.y as i64,
        width: size.width as i64,
        height: size.height as i64,
    }
}

pub fn panel_rect_from_monitor(monitor: &tauri::Monitor) -> PanelRect {
    let position = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();
    PanelRect {
        x: position.x as f64 / scale,
        y: position.y as f64 / scale,
        width: size.width as f64 / scale,
        height: size.height as f64 / scale,
    }
}

#[cfg_attr(target_os = "windows", allow(dead_code))]
pub fn panel_rect_from_panel_geometry(geometry: PanelDisplayGeometry) -> PanelRect {
    PanelRect {
        x: geometry.x as f64,
        y: geometry.y as f64,
        width: geometry.width as f64,
        height: geometry.height as f64,
    }
}

pub fn display_option_from_panel_geometry(
    index: usize,
    geometry: PanelDisplayGeometry,
    name: Option<String>,
) -> DisplayOption {
    DisplayOption {
        index,
        key: display_key_for_panel_geometry(geometry),
        name: name.unwrap_or_else(|| format!("Display {}", index + 1)),
        width: geometry.width.max(0) as u32,
        height: geometry.height.max(0) as u32,
    }
}

pub fn display_option_from_monitor(index: usize, monitor: &tauri::Monitor) -> DisplayOption {
    display_option_from_panel_geometry(
        index,
        panel_geometry_for_monitor(monitor),
        monitor.name().cloned(),
    )
}

pub fn display_options_from_monitors(monitors: &[tauri::Monitor]) -> Vec<DisplayOption> {
    monitors
        .iter()
        .enumerate()
        .map(|(index, monitor)| display_option_from_monitor(index, monitor))
        .collect()
}

pub fn list_available_displays<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<Vec<DisplayOption>, String> {
    let window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "main window not found".to_string())?;
    let monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?;
    Ok(display_options_from_monitors(&monitors))
}

pub fn resolve_preferred_display_index(
    displays: &[DisplayOption],
    preferred_key: Option<&str>,
) -> usize {
    preferred_key
        .and_then(|key| displays.iter().position(|display| display.key == key))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        display_key_for_panel_geometry, display_option_from_panel_geometry,
        panel_rect_from_panel_geometry,
    };
    use crate::native_panel_core::PanelDisplayGeometry;

    #[test]
    fn display_option_from_geometry_uses_shared_key_and_default_name() {
        let geometry = PanelDisplayGeometry {
            x: 10,
            y: 20,
            width: 1440,
            height: 900,
        };

        let option = display_option_from_panel_geometry(1, geometry, None);

        assert_eq!(option.index, 1);
        assert_eq!(option.key, display_key_for_panel_geometry(geometry));
        assert_eq!(option.name, "Display 2");
        assert_eq!(option.width, 1440);
        assert_eq!(option.height, 900);
    }

    #[test]
    fn panel_rect_from_geometry_preserves_position_and_size() {
        let rect = panel_rect_from_panel_geometry(PanelDisplayGeometry {
            x: -100,
            y: 40,
            width: 3024,
            height: 1964,
        });

        assert_eq!(rect.x, -100.0);
        assert_eq!(rect.y, 40.0);
        assert_eq!(rect.width, 3024.0);
        assert_eq!(rect.height, 1964.0);
    }
}
