use super::*;
use objc2::rc::Retained;

pub(super) struct NativePanelSetup {
    pub(super) screen: Retained<NSScreen>,
    pub(super) compact_height: f64,
    pub(super) compact_width: f64,
    pub(super) expanded_width: f64,
    pub(super) size: NSSize,
    pub(super) pill_size: NSSize,
    pub(super) screen_frame: NSRect,
    pub(super) frame: NSRect,
    pub(super) pill_frame: NSRect,
}

pub(super) struct NativePanelColors {
    pub(super) pill_background: Retained<NSColor>,
    pub(super) pill_border: Retained<NSColor>,
    pub(super) pill_highlight: Retained<NSColor>,
    pub(super) mascot_shell_border: Retained<NSColor>,
    pub(super) mascot_body_fill: Retained<NSColor>,
    pub(super) mascot_stroke: Retained<NSColor>,
    pub(super) mascot_face: Retained<NSColor>,
    pub(super) expanded_background: Retained<NSColor>,
    pub(super) expanded_border: Retained<NSColor>,
    pub(super) text_primary: Retained<NSColor>,
    pub(super) accent_active: Retained<NSColor>,
    pub(super) separator_color: Retained<NSColor>,
}

pub(super) fn resolve_native_panel_setup(
    mtm: MainThreadMarker,
) -> Result<NativePanelSetup, String> {
    let screen = NSScreen::mainScreen(mtm)
        .or_else(|| {
            let screens = NSScreen::screens(mtm);
            if screens.is_empty() {
                None
            } else {
                Some(screens.objectAtIndex(0))
            }
        })
        .ok_or_else(|| "failed to resolve a macOS screen".to_string())?;

    let compact_height = compact_pill_height_for_screen(&screen);
    let compact_width = compact_pill_width_for_screen(&screen, compact_height);
    let expanded_width = expanded_panel_width_for_screen(&screen);
    let panel_width = panel_canvas_width_for_screen(&screen, compact_height);
    let size = NSSize::new(panel_width, COLLAPSED_PANEL_HEIGHT);
    let pill_size = NSSize::new(compact_width, compact_height);
    let screen_frame = screen.frame();
    let frame = centered_top_frame(screen_frame, size);
    let pill_frame = island_bar_frame(
        size,
        0.0,
        compact_width,
        expanded_width,
        compact_height,
        0.0,
    );

    Ok(NativePanelSetup {
        screen,
        compact_height,
        compact_width,
        expanded_width,
        size,
        pill_size,
        screen_frame,
        frame,
        pill_frame,
    })
}

pub(super) fn native_panel_colors() -> NativePanelColors {
    NativePanelColors {
        pill_background: NSColor::colorWithSRGBRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0),
        pill_border: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.055),
        pill_highlight: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.16),
        mascot_shell_border: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.08),
        mascot_body_fill: NSColor::colorWithSRGBRed_green_blue_alpha(0.02, 0.02, 0.02, 1.0),
        mascot_stroke: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.48, 0.14, 1.0),
        mascot_face: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0),
        expanded_background: NSColor::colorWithSRGBRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0),
        expanded_border: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.0),
        text_primary: NSColor::colorWithSRGBRed_green_blue_alpha(0.96, 0.97, 0.99, 1.0),
        accent_active: NSColor::colorWithSRGBRed_green_blue_alpha(0.40, 0.87, 0.57, 1.0),
        separator_color: NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.11),
    }
}
