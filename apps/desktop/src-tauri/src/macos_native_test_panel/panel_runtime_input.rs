use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;

use crate::{
    native_panel_renderer::{NativePanelRuntimeInputContext, NativePanelRuntimeInputDescriptor},
    native_panel_scene_input::native_panel_runtime_input_descriptor_from_context,
};

use super::panel_host_adapter::ns_rect_to_panel_rect;

pub(super) fn native_panel_runtime_input_descriptor() -> NativePanelRuntimeInputDescriptor {
    let settings = crate::app_settings::current_app_settings();
    let context = native_panel_runtime_display_context().unwrap_or_default();
    native_panel_runtime_input_descriptor_from_context(&settings, context)
}

fn native_panel_runtime_display_context() -> Option<NativePanelRuntimeInputContext> {
    let Some(mtm) = MainThreadMarker::new() else {
        return None;
    };
    let screens = NSScreen::screens(mtm);
    let display_count = screens.len().max(1);
    let settings = crate::app_settings::current_app_settings();
    let index = settings.preferred_display_index.min(display_count - 1);
    let screen = if screens.is_empty() {
        NSScreen::mainScreen(mtm)
    } else {
        Some(screens.objectAtIndex(index))
    };
    Some(NativePanelRuntimeInputContext {
        display_count,
        selected_display_index: settings.preferred_display_index,
        screen_frame: screen.map(|screen| ns_rect_to_panel_rect(screen.frame())),
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
                preferred_display_index: 3,
                preferred_display_key: Some("external".to_string()),
            },
            NativePanelRuntimeInputContext {
                display_count: 2,
                selected_display_index: 3,
                screen_frame: Some(PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 1440.0,
                    height: 900.0,
                }),
            },
        );

        assert_eq!(descriptor.selected_display_index(), 3);
        assert_eq!(descriptor.scene_input.display_count, 2);
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

        assert_eq!(descriptor.scene_input.display_count, 1);
        assert_eq!(descriptor.selected_display_index(), 0);
        assert_eq!(descriptor.screen_frame, None);
    }
}
