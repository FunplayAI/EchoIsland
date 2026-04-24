use tauri::{AppHandle, Manager};

use crate::{
    app_settings::current_app_settings,
    constants::MAIN_WINDOW_LABEL,
    display_settings::{list_available_displays, resolve_preferred_display_index},
    native_panel_core::PanelRect,
    native_panel_renderer::{NativePanelRuntimeInputContext, NativePanelRuntimeInputDescriptor},
    native_panel_scene_input::native_panel_runtime_input_descriptor_from_context,
};

pub(super) fn windows_runtime_input_descriptor<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> NativePanelRuntimeInputDescriptor {
    let settings = current_app_settings();
    let displays = list_available_displays(app).unwrap_or_default();
    let selected_display_index =
        resolve_preferred_display_index(&displays, settings.preferred_display_key.as_deref());
    let screen_frame = windows_display_screen_frame(app, selected_display_index);

    native_panel_runtime_input_descriptor_from_context(
        &settings,
        NativePanelRuntimeInputContext {
            display_count: displays.len(),
            selected_display_index,
            screen_frame,
        },
    )
}

fn windows_display_screen_frame<R: tauri::Runtime>(
    app: &AppHandle<R>,
    preferred_display_index: usize,
) -> Option<PanelRect> {
    let window = app.get_webview_window(MAIN_WINDOW_LABEL)?;
    let monitors = window.available_monitors().ok()?;
    let monitor = monitors
        .get(preferred_display_index)
        .or_else(|| monitors.first())?;
    let position = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();
    Some(PanelRect {
        x: position.x as f64 / scale,
        y: position.y as f64 / scale,
        width: size.width as f64 / scale,
        height: size.height as f64 / scale,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        app_settings::AppSettings, native_panel_core::PanelRect,
        native_panel_renderer::NativePanelRuntimeInputContext,
        native_panel_scene_input::native_panel_runtime_input_descriptor_from_context,
    };

    #[test]
    fn runtime_input_descriptor_projects_settings_and_display_context() {
        let descriptor = native_panel_runtime_input_descriptor_from_context(
            &AppSettings {
                completion_sound_enabled: false,
                mascot_enabled: false,
                preferred_display_index: 8,
                preferred_display_key: Some("display-key".to_string()),
            },
            NativePanelRuntimeInputContext {
                display_count: 3,
                selected_display_index: 1,
                screen_frame: Some(PanelRect {
                    x: 20.0,
                    y: 30.0,
                    width: 1280.0,
                    height: 720.0,
                }),
            },
        );

        assert_eq!(descriptor.selected_display_index(), 1);
        assert_eq!(descriptor.scene_input.display_count, 3);
        assert!(!descriptor.scene_input.settings.completion_sound_enabled);
        assert!(!descriptor.scene_input.settings.mascot_enabled);
        assert_eq!(
            descriptor.screen_frame,
            Some(PanelRect {
                x: 20.0,
                y: 30.0,
                width: 1280.0,
                height: 720.0,
            })
        );
    }

    #[test]
    fn runtime_input_descriptor_clamps_empty_display_count() {
        let descriptor = native_panel_runtime_input_descriptor_from_context(
            &AppSettings::default(),
            NativePanelRuntimeInputContext::default(),
        );

        assert_eq!(descriptor.scene_input.display_count, 1);
        assert_eq!(descriptor.selected_display_index(), 0);
        assert_eq!(descriptor.screen_frame, None);
    }
}
