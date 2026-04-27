use tauri::{AppHandle, Manager};

use crate::{
    app_settings::current_app_settings,
    constants::MAIN_WINDOW_LABEL,
    display_settings::{display_options_from_monitors, panel_rect_from_monitor},
    native_panel_renderer::facade::descriptor::NativePanelRuntimeInputDescriptor,
    native_panel_scene_input::native_panel_runtime_input_descriptor_from_display_options_with_screen_frame,
};

pub(super) fn windows_runtime_input_descriptor<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> NativePanelRuntimeInputDescriptor {
    let settings = current_app_settings();
    let monitors = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .and_then(|window| window.available_monitors().ok())
        .unwrap_or_default();
    let displays = display_options_from_monitors(&monitors);
    native_panel_runtime_input_descriptor_from_display_options_with_screen_frame(
        &displays,
        &settings,
        Some(0),
        |selected_display_index| {
            monitors
                .get(selected_display_index)
                .or_else(|| monitors.first())
                .map(panel_rect_from_monitor)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        app_settings::AppSettings, native_panel_core::PanelRect,
        native_panel_renderer::facade::descriptor::NativePanelRuntimeInputContext,
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
                display_options: vec![
                    crate::native_panel_scene::panel_display_option_state(
                        0,
                        "display-1",
                        "Built-in",
                        3024,
                        1964,
                    ),
                    crate::native_panel_scene::panel_display_option_state(
                        1,
                        "display-2",
                        "Studio Display",
                        2560,
                        1440,
                    ),
                    crate::native_panel_scene::panel_display_option_state(
                        2,
                        "display-3",
                        "Projector",
                        1920,
                        1080,
                    ),
                ],
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
        assert_eq!(descriptor.scene_input.display_options.len(), 3);
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
    fn runtime_input_descriptor_clamps_empty_display_list() {
        let descriptor = native_panel_runtime_input_descriptor_from_context(
            &AppSettings::default(),
            NativePanelRuntimeInputContext::default(),
        );

        assert_eq!(descriptor.scene_input.display_options.len(), 1);
        assert_eq!(descriptor.selected_display_index(), 0);
        assert_eq!(descriptor.screen_frame, None);
    }
}
