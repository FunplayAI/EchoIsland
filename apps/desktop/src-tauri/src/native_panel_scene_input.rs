use crate::{
    app_settings::AppSettings,
    native_panel_core::PanelSettingsState,
    native_panel_renderer::{NativePanelRuntimeInputContext, NativePanelRuntimeInputDescriptor},
    native_panel_scene::PanelSceneBuildInput,
};

pub(crate) fn panel_scene_build_input_from_app_settings(
    display_count: usize,
    selected_display_index: usize,
    settings: &AppSettings,
) -> PanelSceneBuildInput {
    PanelSceneBuildInput {
        display_count: display_count.max(1),
        settings: panel_settings_state_from_app_settings(selected_display_index, settings),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

pub(crate) fn panel_settings_state_from_app_settings(
    selected_display_index: usize,
    settings: &AppSettings,
) -> PanelSettingsState {
    PanelSettingsState {
        selected_display_index,
        completion_sound_enabled: settings.completion_sound_enabled,
        mascot_enabled: settings.mascot_enabled,
    }
}

pub(crate) fn native_panel_runtime_input_descriptor_from_app_settings(
    display_count: usize,
    selected_display_index: usize,
    settings: &AppSettings,
    screen_frame: Option<crate::native_panel_core::PanelRect>,
) -> NativePanelRuntimeInputDescriptor {
    NativePanelRuntimeInputDescriptor {
        scene_input: panel_scene_build_input_from_app_settings(
            display_count,
            selected_display_index,
            settings,
        ),
        screen_frame,
    }
}

pub(crate) fn native_panel_runtime_input_descriptor_from_context(
    settings: &AppSettings,
    context: NativePanelRuntimeInputContext,
) -> NativePanelRuntimeInputDescriptor {
    native_panel_runtime_input_descriptor_from_app_settings(
        context.display_count,
        context.selected_display_index,
        settings,
        context.screen_frame,
    )
}

#[cfg(test)]
mod tests {
    use super::panel_scene_build_input_from_app_settings;
    use super::{
        native_panel_runtime_input_descriptor_from_app_settings,
        native_panel_runtime_input_descriptor_from_context,
    };
    use crate::app_settings::AppSettings;

    #[test]
    fn scene_build_input_clamps_display_count_and_projects_settings() {
        let input = panel_scene_build_input_from_app_settings(
            0,
            2,
            &AppSettings {
                completion_sound_enabled: false,
                mascot_enabled: false,
                preferred_display_index: 7,
                preferred_display_key: Some("display-key".to_string()),
            },
        );

        assert_eq!(input.display_count, 1);
        assert_eq!(input.settings.selected_display_index, 2);
        assert!(!input.settings.completion_sound_enabled);
        assert!(!input.settings.mascot_enabled);
        assert_eq!(input.app_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn runtime_input_descriptor_wraps_scene_input_and_screen_frame() {
        let descriptor = native_panel_runtime_input_descriptor_from_app_settings(
            2,
            1,
            &AppSettings::default(),
            Some(crate::native_panel_core::PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            }),
        );

        assert_eq!(descriptor.selected_display_index(), 1);
        assert_eq!(descriptor.scene_input.display_count, 2);
        assert_eq!(
            descriptor.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            })
        );
    }

    #[test]
    fn runtime_input_descriptor_can_be_built_from_shared_context() {
        let descriptor = native_panel_runtime_input_descriptor_from_context(
            &AppSettings::default(),
            crate::native_panel_renderer::NativePanelRuntimeInputContext {
                display_count: 3,
                selected_display_index: 2,
                screen_frame: Some(crate::native_panel_core::PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 1280.0,
                    height: 720.0,
                }),
            },
        );

        assert_eq!(descriptor.selected_display_index(), 2);
        assert_eq!(descriptor.scene_input.display_count, 3);
        assert_eq!(
            descriptor.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 10.0,
                y: 20.0,
                width: 1280.0,
                height: 720.0,
            })
        );
    }
}
