use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSFont, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::super::card_animation::register_card_animation_layout;
use super::super::panel_helpers::ns_color;
use super::common::{apply_card_layer, badge_width, make_badge_view, make_label};
use crate::native_panel_core::{
    PanelRect, SETTINGS_CARD_SIDE_INSET, resolve_settings_surface_card_height,
    settings_surface_row_frame as shared_settings_row_frame,
};
use crate::native_panel_scene::{SceneCard, SettingsRowScene};

const SETTINGS_HEADER_TOP: f64 = 20.0;
const SETTINGS_HEADER_HEIGHT: f64 = 18.0;
const SETTINGS_ROW_TITLE_HEIGHT: f64 = 16.0;

pub(crate) fn settings_surface_card_height(row_count: usize) -> f64 {
    resolve_settings_surface_card_height(row_count)
}

pub(crate) fn settings_surface_row_frame(card_frame: NSRect, index: usize) -> NSRect {
    let frame = shared_settings_row_frame(
        PanelRect {
            x: 0.0,
            y: 0.0,
            width: card_frame.size.width,
            height: card_frame.size.height,
        },
        index,
    );
    NSRect::new(
        NSPoint::new(frame.x, frame.y),
        NSSize::new(frame.width, frame.height),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SettingsSurfaceCardContent {
    pub(crate) title: String,
    pub(crate) version_text: String,
    pub(crate) rows: Vec<SettingsSurfaceCardRowContent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SettingsSurfaceCardRowContent {
    pub(crate) title: String,
    pub(crate) value: String,
    pub(crate) active: bool,
}

pub(crate) fn settings_surface_card_content_from_scene_card(
    card: &SceneCard,
) -> Option<SettingsSurfaceCardContent> {
    let SceneCard::Settings {
        title,
        version,
        rows,
    } = card
    else {
        return None;
    };

    Some(SettingsSurfaceCardContent {
        title: title.clone(),
        version_text: version.text.clone(),
        rows: rows
            .iter()
            .map(settings_surface_card_row_content_from_scene_row)
            .collect(),
    })
}

fn settings_surface_card_row_content_from_scene_row(
    row: &SettingsRowScene,
) -> SettingsSurfaceCardRowContent {
    SettingsSurfaceCardRowContent {
        title: row.title.clone(),
        value: row.value.text.clone(),
        active: row.value.emphasized,
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_settings_surface_card(
    frame: NSRect,
    content: &SettingsSurfaceCardContent,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 1.0, 1.0, 0.055], [1.0, 1.0, 1.0, 0.08]);
    register_card_animation_layout(&view, frame, 64.0);

    let title = make_label(
        mtm,
        &content.title,
        NSRect::new(
            NSPoint::new(
                SETTINGS_CARD_SIDE_INSET,
                frame.size.height - SETTINGS_HEADER_TOP - SETTINGS_HEADER_HEIGHT,
            ),
            NSSize::new(
                frame.size.width - (SETTINGS_CARD_SIDE_INSET * 2.0) - 80.0,
                SETTINGS_HEADER_HEIGHT,
            ),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title);

    let version_badge = make_badge_view(
        mtm,
        &content.version_text,
        badge_width(&content.version_text, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let version_width = version_badge.frame().size.width;
    version_badge.setFrame(NSRect::new(
        NSPoint::new(
            frame.size.width - version_width - SETTINGS_CARD_SIDE_INSET,
            frame.size.height - SETTINGS_HEADER_TOP - version_badge.frame().size.height,
        ),
        version_badge.frame().size,
    ));
    view.addSubview(&version_badge);

    for (index, row) in content.rows.iter().enumerate() {
        let row = make_settings_action_row(
            mtm,
            settings_surface_row_frame(frame, index),
            &row.title,
            &row.value,
            row.active,
        );
        view.addSubview(&row);
    }

    view
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn make_settings_action_row(
    mtm: MainThreadMarker,
    frame: NSRect,
    title: &str,
    value: &str,
    active: bool,
) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(8.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(
            &ns_color(if active {
                [0.40, 0.87, 0.57, 0.10]
            } else {
                [1.0, 1.0, 1.0, 0.04]
            })
            .CGColor(),
        ));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(
            &ns_color(if active {
                [0.40, 0.87, 0.57, 0.22]
            } else {
                [1.0, 1.0, 1.0, 0.08]
            })
            .CGColor(),
        ));
    }

    let title_label = make_label(
        mtm,
        title,
        NSRect::new(
            NSPoint::new(
                12.0,
                ((frame.size.height - SETTINGS_ROW_TITLE_HEIGHT) / 2.0).round(),
            ),
            NSSize::new(frame.size.width - 110.0, SETTINGS_ROW_TITLE_HEIGHT),
        ),
        11.0,
        [0.96, 0.97, 0.99, 0.96],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(11.0)));
    view.addSubview(&title_label);

    let badge_background = if active {
        [0.40, 0.87, 0.57, 0.16]
    } else {
        [1.0, 1.0, 1.0, 0.08]
    };
    let badge_foreground = if active {
        [0.40, 0.87, 0.57, 1.0]
    } else {
        [0.90, 0.92, 0.96, 0.92]
    };
    let badge = make_badge_view(
        mtm,
        value,
        badge_width(value, 10.0, 18.0).max(44.0),
        badge_background,
        badge_foreground,
    );
    let badge_size = badge.frame().size;
    badge.setFrame(NSRect::new(
        NSPoint::new(
            frame.size.width - badge_size.width - 10.0,
            ((frame.size.height - badge_size.height) / 2.0).round(),
        ),
        badge_size,
    ));
    view.addSubview(&badge);

    view
}

#[cfg(test)]
mod tests {
    use super::settings_surface_card_content_from_scene_card;
    use crate::native_panel_core::PanelHitAction;
    use crate::native_panel_scene::{SceneBadge, SceneCard, SettingsRowScene};

    #[test]
    fn settings_surface_card_content_comes_from_shared_scene_card() {
        let card = SceneCard::Settings {
            title: "Settings".to_string(),
            version: SceneBadge {
                text: "v0.5.0".to_string(),
                emphasized: false,
            },
            rows: vec![
                SettingsRowScene {
                    title: "Mute Sound".to_string(),
                    value: SceneBadge {
                        text: "On".to_string(),
                        emphasized: true,
                    },
                    action: PanelHitAction::ToggleCompletionSound,
                },
                SettingsRowScene {
                    title: "Hide Mascot".to_string(),
                    value: SceneBadge {
                        text: "Off".to_string(),
                        emphasized: false,
                    },
                    action: PanelHitAction::ToggleMascot,
                },
            ],
        };

        let content =
            settings_surface_card_content_from_scene_card(&card).expect("settings content");

        assert_eq!(content.title, "Settings");
        assert_eq!(content.version_text, "v0.5.0");
        assert_eq!(content.rows.len(), 2);
        assert_eq!(content.rows[0].title, "Mute Sound");
        assert_eq!(content.rows[0].value, "On");
        assert!(content.rows[0].active);
        assert_eq!(content.rows[1].title, "Hide Mascot");
        assert_eq!(content.rows[1].value, "Off");
        assert!(!content.rows[1].active);
    }
}
