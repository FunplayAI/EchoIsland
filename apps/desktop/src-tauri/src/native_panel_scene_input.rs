use crate::{
    app_settings::AppSettings,
    display_settings::DisplayOption,
    native_panel_core::resolve_preferred_panel_display_index,
    native_panel_core::{PanelRect, PanelSettingsState},
    native_panel_renderer::facade::descriptor::{
        NativePanelRuntimeInputContext, NativePanelRuntimeInputDescriptor,
    },
    native_panel_scene::{
        PanelDisplayOptionState, PanelSceneBuildInput, fallback_panel_display_option,
        panel_display_option_state,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativePanelDisplaySelectionUpdate {
    pub(crate) selected_display_index: usize,
    pub(crate) selected_display_key: String,
}

pub(crate) fn panel_scene_build_input_from_app_settings(
    display_options: Vec<PanelDisplayOptionState>,
    selected_display_index: usize,
    settings: &AppSettings,
) -> PanelSceneBuildInput {
    PanelSceneBuildInput {
        display_options: sanitize_panel_display_options(display_options),
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
    display_options: Vec<PanelDisplayOptionState>,
    selected_display_index: usize,
    settings: &AppSettings,
    screen_frame: Option<crate::native_panel_core::PanelRect>,
) -> NativePanelRuntimeInputDescriptor {
    NativePanelRuntimeInputDescriptor {
        scene_input: panel_scene_build_input_from_app_settings(
            display_options,
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
        context.display_options,
        context.selected_display_index,
        settings,
        context.screen_frame,
    )
}

pub(crate) fn panel_scene_build_input_from_display_options(
    displays: &[DisplayOption],
    settings: &AppSettings,
    fallback_index: Option<usize>,
) -> PanelSceneBuildInput {
    let selected_display_index =
        resolve_selected_display_index_from_display_options(displays, settings, fallback_index);
    panel_scene_build_input_from_app_settings(
        panel_display_options_from_display_options(displays),
        selected_display_index,
        settings,
    )
}

pub(crate) fn native_panel_runtime_input_descriptor_from_display_options_with_screen_frame(
    displays: &[DisplayOption],
    settings: &AppSettings,
    fallback_index: Option<usize>,
    screen_frame_for_selected_index: impl FnOnce(usize) -> Option<PanelRect>,
) -> NativePanelRuntimeInputDescriptor {
    let context = native_panel_runtime_input_context_from_display_options_with_screen_frame(
        panel_display_options_from_display_options(displays),
        settings,
        fallback_index,
        screen_frame_for_selected_index,
    );
    native_panel_runtime_input_descriptor_from_context(settings, context)
}

pub(crate) fn native_panel_runtime_input_context_from_display_options(
    display_options: Vec<PanelDisplayOptionState>,
    settings: &AppSettings,
    fallback_index: Option<usize>,
    screen_frame: Option<PanelRect>,
) -> NativePanelRuntimeInputContext {
    let selected_display_index = resolve_panel_selected_display_index(
        &display_options
            .iter()
            .map(|display| display.key.clone())
            .collect::<Vec<_>>(),
        settings,
        fallback_index,
    );

    NativePanelRuntimeInputContext {
        display_options,
        selected_display_index,
        screen_frame,
    }
}

pub(crate) fn native_panel_runtime_input_context_from_display_options_with_screen_frame(
    display_options: Vec<PanelDisplayOptionState>,
    settings: &AppSettings,
    fallback_index: Option<usize>,
    screen_frame_for_selected_index: impl FnOnce(usize) -> Option<PanelRect>,
) -> NativePanelRuntimeInputContext {
    let mut context = native_panel_runtime_input_context_from_display_options(
        display_options,
        settings,
        fallback_index,
        None,
    );
    context.screen_frame = screen_frame_for_selected_index(context.selected_display_index);
    context
}

pub(crate) fn panel_display_options_from_display_options(
    displays: &[DisplayOption],
) -> Vec<PanelDisplayOptionState> {
    displays
        .iter()
        .map(|display| {
            panel_display_option_state(
                display.index,
                display.key.clone(),
                &display.name,
                display.width,
                display.height,
            )
        })
        .collect()
}

pub(crate) fn resolve_selected_display_index_from_display_options(
    displays: &[DisplayOption],
    settings: &AppSettings,
    fallback_index: Option<usize>,
) -> usize {
    if displays.is_empty() {
        return fallback_index.unwrap_or(settings.preferred_display_index);
    }
    resolve_panel_selected_display_index(
        &displays
            .iter()
            .map(|display| display.key.clone())
            .collect::<Vec<_>>(),
        settings,
        fallback_index,
    )
}

pub(crate) fn resolve_next_display_selection_update_from_display_options(
    displays: &[DisplayOption],
    settings: &AppSettings,
) -> Option<NativePanelDisplaySelectionUpdate> {
    if displays.is_empty() {
        return None;
    }
    let current = resolve_selected_display_index_from_display_options(displays, settings, Some(0));
    let selected = displays.get((current + 1) % displays.len())?;
    Some(NativePanelDisplaySelectionUpdate {
        selected_display_index: selected.index,
        selected_display_key: selected.key.clone(),
    })
}

pub(crate) fn resolve_panel_selected_display_index(
    display_keys: &[String],
    settings: &AppSettings,
    fallback_index: Option<usize>,
) -> usize {
    resolve_preferred_panel_display_index(
        display_keys,
        settings.preferred_display_key.as_deref(),
        settings.preferred_display_index,
        fallback_index,
    )
    .unwrap_or(0)
}

fn sanitize_panel_display_options(
    display_options: Vec<PanelDisplayOptionState>,
) -> Vec<PanelDisplayOptionState> {
    if display_options.is_empty() {
        vec![fallback_panel_display_option()]
    } else {
        display_options
    }
}

#[cfg(test)]
mod tests {
    use super::{
        native_panel_runtime_input_context_from_display_options,
        native_panel_runtime_input_context_from_display_options_with_screen_frame,
        native_panel_runtime_input_descriptor_from_app_settings,
        native_panel_runtime_input_descriptor_from_context,
        native_panel_runtime_input_descriptor_from_display_options_with_screen_frame,
    };
    use super::{
        panel_display_options_from_display_options, panel_scene_build_input_from_app_settings,
        panel_scene_build_input_from_display_options,
        resolve_next_display_selection_update_from_display_options,
        resolve_panel_selected_display_index, resolve_selected_display_index_from_display_options,
    };
    use crate::{
        app_settings::AppSettings, display_settings::DisplayOption,
        native_panel_renderer::facade::descriptor::NativePanelRuntimeInputContext,
    };

    #[test]
    fn scene_build_input_clamps_display_count_and_projects_settings() {
        let input = panel_scene_build_input_from_app_settings(
            Vec::new(),
            2,
            &AppSettings {
                completion_sound_enabled: false,
                mascot_enabled: false,
                preferred_display_index: 7,
                preferred_display_key: Some("display-key".to_string()),
            },
        );

        assert_eq!(input.display_options.len(), 1);
        assert_eq!(input.display_options[0].label, "Display 1");
        assert_eq!(input.settings.selected_display_index, 2);
        assert!(!input.settings.completion_sound_enabled);
        assert!(!input.settings.mascot_enabled);
        assert_eq!(input.app_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn runtime_input_descriptor_wraps_scene_input_and_screen_frame() {
        let descriptor = native_panel_runtime_input_descriptor_from_app_settings(
            vec![
                crate::native_panel_scene::panel_display_option_state(
                    0,
                    "display-1",
                    "Studio Display",
                    2560,
                    1440,
                ),
                crate::native_panel_scene::panel_display_option_state(
                    1,
                    "display-2",
                    "LG UltraFine",
                    1512,
                    982,
                ),
            ],
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
        assert_eq!(descriptor.scene_input.display_options.len(), 2);
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
                        "LG",
                        1920,
                        1080,
                    ),
                ],
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
        assert_eq!(descriptor.scene_input.display_options.len(), 3);
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

    #[test]
    fn runtime_input_context_selects_display_from_shared_display_options() {
        let context = native_panel_runtime_input_context_from_display_options(
            vec![
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
            &AppSettings {
                preferred_display_index: 9,
                preferred_display_key: Some("display-2".to_string()),
                ..AppSettings::default()
            },
            Some(0),
            Some(crate::native_panel_core::PanelRect {
                x: 5.0,
                y: 10.0,
                width: 1280.0,
                height: 720.0,
            }),
        );

        assert_eq!(context.selected_display_index, 1);
        assert_eq!(context.display_options.len(), 2);
        assert_eq!(
            context.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 5.0,
                y: 10.0,
                width: 1280.0,
                height: 720.0,
            })
        );
    }

    #[test]
    fn runtime_input_context_with_screen_frame_uses_selected_display_index() {
        let context = native_panel_runtime_input_context_from_display_options_with_screen_frame(
            vec![
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
            &AppSettings {
                preferred_display_index: 9,
                preferred_display_key: Some("display-2".to_string()),
                ..AppSettings::default()
            },
            Some(0),
            |selected_display_index| {
                Some(crate::native_panel_core::PanelRect {
                    x: selected_display_index as f64,
                    y: 10.0,
                    width: 1280.0,
                    height: 720.0,
                })
            },
        );

        assert_eq!(context.selected_display_index, 1);
        assert_eq!(
            context.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 1.0,
                y: 10.0,
                width: 1280.0,
                height: 720.0,
            })
        );
    }

    #[test]
    fn scene_build_input_from_display_options_resolves_selected_display() {
        let displays = vec![
            DisplayOption {
                index: 0,
                key: "display-1".to_string(),
                name: "Built-in".to_string(),
                width: 3024,
                height: 1964,
            },
            DisplayOption {
                index: 1,
                key: "display-2".to_string(),
                name: "Studio Display".to_string(),
                width: 2560,
                height: 1440,
            },
        ];

        let input = panel_scene_build_input_from_display_options(
            &displays,
            &AppSettings {
                preferred_display_key: Some("display-2".to_string()),
                ..AppSettings::default()
            },
            Some(0),
        );

        assert_eq!(input.settings.selected_display_index, 1);
        assert_eq!(input.display_options.len(), 2);
    }

    #[test]
    fn runtime_input_descriptor_from_display_options_resolves_screen_frame() {
        let displays = vec![DisplayOption {
            index: 0,
            key: "display-1".to_string(),
            name: "Built-in".to_string(),
            width: 3024,
            height: 1964,
        }];

        let descriptor =
            native_panel_runtime_input_descriptor_from_display_options_with_screen_frame(
                &displays,
                &AppSettings::default(),
                Some(0),
                |selected_display_index| {
                    assert_eq!(selected_display_index, 0);
                    Some(crate::native_panel_core::PanelRect {
                        x: 1.0,
                        y: 2.0,
                        width: 300.0,
                        height: 200.0,
                    })
                },
            );

        assert_eq!(descriptor.selected_display_index(), 0);
        assert_eq!(
            descriptor.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 1.0,
                y: 2.0,
                width: 300.0,
                height: 200.0,
            })
        );
    }

    #[test]
    fn display_options_preserve_name_and_resolution_format() {
        let options = panel_display_options_from_display_options(&[DisplayOption {
            index: 1,
            key: "display-2".to_string(),
            name: "Studio Display".to_string(),
            width: 2560,
            height: 1440,
        }]);

        assert_eq!(options[0].index, 1);
        assert_eq!(options[0].key, "display-2");
        assert_eq!(options[0].label, "Studio Display · 2560×1440");
    }

    #[test]
    fn selected_display_index_prefers_key_then_index_then_fallback() {
        let settings = AppSettings {
            preferred_display_index: 7,
            preferred_display_key: Some("display-2".to_string()),
            ..AppSettings::default()
        };
        let display_keys = vec!["display-1".to_string(), "display-2".to_string()];

        assert_eq!(
            resolve_panel_selected_display_index(&display_keys, &settings, Some(0)),
            1
        );
    }

    #[test]
    fn selected_display_index_from_display_options_uses_key_and_fallback() {
        let settings = AppSettings {
            preferred_display_index: 7,
            preferred_display_key: Some("display-2".to_string()),
            ..AppSettings::default()
        };
        let displays = vec![
            DisplayOption {
                index: 0,
                key: "display-1".to_string(),
                name: "Built-in".to_string(),
                width: 3024,
                height: 1964,
            },
            DisplayOption {
                index: 3,
                key: "display-2".to_string(),
                name: "Studio Display".to_string(),
                width: 2560,
                height: 1440,
            },
        ];

        assert_eq!(
            resolve_selected_display_index_from_display_options(&displays, &settings, Some(1)),
            1
        );
        assert_eq!(
            resolve_selected_display_index_from_display_options(&[], &settings, Some(4)),
            4
        );
    }

    #[test]
    fn next_display_selection_update_cycles_and_skips_empty_display_lists() {
        let settings = AppSettings {
            preferred_display_index: 0,
            preferred_display_key: Some("display-1".to_string()),
            ..AppSettings::default()
        };
        let displays = vec![
            DisplayOption {
                index: 0,
                key: "display-1".to_string(),
                name: "Built-in".to_string(),
                width: 3024,
                height: 1964,
            },
            DisplayOption {
                index: 1,
                key: "display-2".to_string(),
                name: "Studio Display".to_string(),
                width: 2560,
                height: 1440,
            },
        ];

        assert_eq!(
            resolve_next_display_selection_update_from_display_options(&displays, &settings),
            Some(super::NativePanelDisplaySelectionUpdate {
                selected_display_index: 1,
                selected_display_key: "display-2".to_string(),
            })
        );
        assert_eq!(
            resolve_next_display_selection_update_from_display_options(&[], &settings),
            None
        );
    }
}
