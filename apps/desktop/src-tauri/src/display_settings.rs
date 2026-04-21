use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::constants::MAIN_WINDOW_LABEL;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOption {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

pub fn list_available_displays<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<Vec<DisplayOption>, String> {
    let window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "main window not found".to_string())?;
    let monitors = window.available_monitors().map_err(|error| error.to_string())?;
    Ok(monitors
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| DisplayOption {
            index,
            name: monitor
                .name()
                .cloned()
                .unwrap_or_else(|| format!("Display {}", index + 1)),
            width: monitor.size().width,
            height: monitor.size().height,
        })
        .collect())
}
