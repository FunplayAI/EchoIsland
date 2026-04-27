use serde::Serialize;

use crate::{
    native_panel_core::{PanelHitAction, PanelSettingsState, settings_row_action},
    native_panel_scene::PanelDisplayOptionState,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsSurfaceScene {
    pub(crate) title: String,
    pub(crate) version_text: String,
    pub(crate) rows: Vec<SettingsSurfaceRowScene>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsSurfaceRowScene {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) control_kind: SettingsSurfaceControlKind,
    pub(crate) value_text: String,
    pub(crate) checked: Option<bool>,
    pub(crate) enabled: bool,
    pub(crate) action_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SettingsSurfaceControlKind {
    Select,
    Toggle,
    Action,
}

pub(crate) fn build_settings_surface_scene(
    display_options: &[PanelDisplayOptionState],
    settings: PanelSettingsState,
    app_version: &str,
) -> SettingsSurfaceScene {
    let selected_display_label = display_options
        .iter()
        .find(|display| display.index == settings.selected_display_index)
        .or_else(|| display_options.get(settings.selected_display_index))
        .or_else(|| display_options.first())
        .map(|display| display.label.clone())
        .unwrap_or_else(|| format!("Display {}", settings.selected_display_index + 1));
    SettingsSurfaceScene {
        title: "Settings".to_string(),
        version_text: format!("EchoIsland v{app_version}"),
        rows: vec![
            SettingsSurfaceRowScene {
                id: "island_display".to_string(),
                label: "Island Display".to_string(),
                control_kind: SettingsSurfaceControlKind::Select,
                value_text: selected_display_label,
                checked: None,
                enabled: !display_options.is_empty(),
                action_key: settings_action_key(0),
            },
            SettingsSurfaceRowScene {
                id: "completion_sound".to_string(),
                label: "Mute Sound".to_string(),
                control_kind: SettingsSurfaceControlKind::Toggle,
                value_text: if settings.completion_sound_enabled {
                    "On".to_string()
                } else {
                    "Off".to_string()
                },
                checked: Some(!settings.completion_sound_enabled),
                enabled: true,
                action_key: settings_action_key(1),
            },
            SettingsSurfaceRowScene {
                id: "mascot".to_string(),
                label: "Hide Mascot".to_string(),
                control_kind: SettingsSurfaceControlKind::Toggle,
                value_text: if settings.mascot_enabled {
                    "On".to_string()
                } else {
                    "Off".to_string()
                },
                checked: Some(!settings.mascot_enabled),
                enabled: true,
                action_key: settings_action_key(2),
            },
            SettingsSurfaceRowScene {
                id: "update".to_string(),
                label: "Update & Upgrade".to_string(),
                control_kind: SettingsSurfaceControlKind::Action,
                value_text: "Open".to_string(),
                checked: None,
                enabled: true,
                action_key: settings_action_key(3),
            },
        ],
    }
}

pub(crate) fn settings_surface_row_action(index: usize) -> Option<PanelHitAction> {
    settings_row_action(index)
}

fn settings_action_key(index: usize) -> String {
    match settings_row_action(index) {
        Some(PanelHitAction::CycleDisplay) => "cycle_display",
        Some(PanelHitAction::ToggleCompletionSound) => "toggle_completion_sound",
        Some(PanelHitAction::ToggleMascot) => "toggle_mascot",
        Some(PanelHitAction::OpenSettingsLocation) => "open_settings_location",
        Some(PanelHitAction::OpenReleasePage) => "open_release_page",
        Some(PanelHitAction::FocusSession) => "focus_session",
        None => "unknown",
    }
    .to_string()
}
