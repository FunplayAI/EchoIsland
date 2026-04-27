use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::NSScreen;
use objc2_foundation::NSArray;

use crate::{
    app_settings::AppSettings,
    display_settings::{
        DisplayOption, display_key_for_panel_geometry, display_option_from_panel_geometry,
        panel_rect_from_panel_geometry,
    },
    native_panel_core::{PanelDisplayGeometry, PanelRect},
    native_panel_scene_input::resolve_selected_display_index_from_display_options,
};

pub(super) struct NativePanelScreenCatalog {
    pub(super) screens: Retained<NSArray<NSScreen>>,
    pub(super) displays: Vec<DisplayOption>,
    pub(super) fallback_index: Option<usize>,
}

pub(super) struct NativePanelSelectedScreenTarget {
    pub(super) screen: Retained<NSScreen>,
}

pub(super) fn native_panel_screen_catalog(mtm: MainThreadMarker) -> NativePanelScreenCatalog {
    let screens = NSScreen::screens(mtm);
    let displays = (0..screens.len())
        .map(|index| native_panel_display_option_for_screen(index, &screens.objectAtIndex(index)))
        .collect::<Vec<_>>();
    let fallback_index = NSScreen::mainScreen(mtm).and_then(|main_screen| {
        let key = native_panel_display_key_for_screen(&main_screen);
        displays.iter().position(|display| display.key == key)
    });

    NativePanelScreenCatalog {
        screens,
        displays,
        fallback_index,
    }
}

impl NativePanelScreenCatalog {
    pub(super) fn selected_screen_target(
        &self,
        settings: &AppSettings,
        mtm: MainThreadMarker,
    ) -> Option<NativePanelSelectedScreenTarget> {
        let selected_display_index = resolve_preferred_native_screen_index(self, settings)?;
        let screen = native_panel_screen_for_selected_index(self, selected_display_index, mtm)?;
        Some(NativePanelSelectedScreenTarget { screen })
    }
}

pub(super) fn resolve_preferred_native_screen_target(
    mtm: MainThreadMarker,
    settings: &AppSettings,
) -> Option<NativePanelSelectedScreenTarget> {
    let catalog = native_panel_screen_catalog(mtm);
    catalog.selected_screen_target(settings, mtm)
}

pub(super) fn native_panel_display_geometry_for_screen(screen: &NSScreen) -> PanelDisplayGeometry {
    let frame = screen.frame();
    PanelDisplayGeometry {
        x: frame.origin.x as i64,
        y: frame.origin.y as i64,
        width: frame.size.width as i64,
        height: frame.size.height as i64,
    }
}

pub(super) fn native_panel_display_key_for_screen(screen: &NSScreen) -> String {
    display_key_for_panel_geometry(native_panel_display_geometry_for_screen(screen))
}

pub(super) fn native_panel_display_option_for_screen(
    index: usize,
    screen: &NSScreen,
) -> DisplayOption {
    display_option_from_panel_geometry(
        index,
        native_panel_display_geometry_for_screen(screen),
        Some(format!("Display {}", index + 1)),
    )
}

pub(super) fn native_panel_screen_frame(screen: &NSScreen) -> PanelRect {
    panel_rect_from_panel_geometry(native_panel_display_geometry_for_screen(screen))
}

pub(super) fn resolve_preferred_native_screen_index(
    catalog: &NativePanelScreenCatalog,
    settings: &AppSettings,
) -> Option<usize> {
    if catalog.screens.is_empty() {
        None
    } else {
        Some(resolve_selected_display_index_from_display_options(
            &catalog.displays,
            settings,
            catalog.fallback_index,
        ))
    }
}

pub(super) fn native_panel_screen_for_selected_index(
    catalog: &NativePanelScreenCatalog,
    selected_display_index: usize,
    mtm: MainThreadMarker,
) -> Option<Retained<NSScreen>> {
    if catalog.screens.is_empty() {
        NSScreen::mainScreen(mtm)
    } else {
        Some(catalog.screens.objectAtIndex(selected_display_index))
    }
}
