use objc2::MainThreadMarker;

use crate::{
    native_panel_renderer::facade::descriptor::{
        NativePanelRuntimeInputContext, NativePanelRuntimeInputDescriptor,
    },
    native_panel_scene_input::{
        native_panel_runtime_input_descriptor_from_context,
        native_panel_runtime_input_descriptor_from_display_options_with_screen_frame,
    },
};

use super::panel_display_source::{
    native_panel_screen_catalog, native_panel_screen_for_selected_index, native_panel_screen_frame,
};

pub(super) fn native_panel_runtime_input_descriptor() -> NativePanelRuntimeInputDescriptor {
    let settings = crate::app_settings::current_app_settings();
    native_panel_runtime_display_descriptor(&settings).unwrap_or_else(|| {
        native_panel_runtime_input_descriptor_from_context(
            &settings,
            NativePanelRuntimeInputContext::default(),
        )
    })
}

fn native_panel_runtime_display_descriptor(
    settings: &crate::app_settings::AppSettings,
) -> Option<NativePanelRuntimeInputDescriptor> {
    let Some(mtm) = MainThreadMarker::new() else {
        return None;
    };
    let catalog = native_panel_screen_catalog(mtm);

    Some(
        native_panel_runtime_input_descriptor_from_display_options_with_screen_frame(
            &catalog.displays,
            settings,
            catalog.fallback_index,
            |selected_display_index| {
                native_panel_screen_for_selected_index(&catalog, selected_display_index, mtm)
                    .map(|screen| native_panel_screen_frame(&screen))
            },
        ),
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
                debug_mode_enabled: false,
                preferred_display_index: 3,
                preferred_display_key: Some("display-2".to_string()),
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
                ],
                selected_display_index: 1,
                screen_frame: Some(PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 1440.0,
                    height: 900.0,
                }),
            },
        );

        assert_eq!(descriptor.selected_display_index(), 1);
        assert_eq!(descriptor.scene_input.display_options.len(), 2);
        assert!(!descriptor.scene_input.settings.completion_sound_enabled);
        assert!(!descriptor.scene_input.settings.mascot_enabled);
        assert_eq!(
            descriptor.screen_frame,
            Some(PanelRect {
                x: 10.0,
                y: 20.0,
                width: 1440.0,
                height: 900.0,
            })
        );
    }

    #[test]
    fn runtime_input_descriptor_uses_default_context_when_appkit_unavailable() {
        let descriptor = native_panel_runtime_input_descriptor_from_context(
            &AppSettings::default(),
            NativePanelRuntimeInputContext::default(),
        );

        assert_eq!(descriptor.scene_input.display_options.len(), 1);
        assert_eq!(descriptor.selected_display_index(), 0);
        assert_eq!(descriptor.screen_frame, None);
    }
}
