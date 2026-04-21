use super::*;

const SETTINGS_CARD_HEIGHT: f64 = 206.0;
const SETTINGS_HEADER_TOP: f64 = 20.0;
const SETTINGS_HEADER_HEIGHT: f64 = 18.0;
const SETTINGS_ROWS_START_Y: f64 = 46.0;
const SETTINGS_ROW_HEIGHT: f64 = 30.0;
const SETTINGS_ROW_GAP: f64 = 8.0;
const SETTINGS_ROW_TITLE_HEIGHT: f64 = 16.0;

pub(crate) fn settings_surface_card_height() -> f64 {
    SETTINGS_CARD_HEIGHT
}

pub(crate) fn settings_surface_row_frame(card_frame: NSRect, index: usize) -> NSRect {
    let y = card_frame.size.height
        - SETTINGS_ROWS_START_Y
        - SETTINGS_ROW_HEIGHT
        - ((SETTINGS_ROW_HEIGHT + SETTINGS_ROW_GAP) * index as f64);
    NSRect::new(
        NSPoint::new(CARD_INSET_X, y),
        NSSize::new(card_frame.size.width - (CARD_INSET_X * 2.0), SETTINGS_ROW_HEIGHT),
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_settings_surface_card(frame: NSRect) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 1.0, 1.0, 0.055], [1.0, 1.0, 1.0, 0.08]);
    register_card_animation_layout(&view, frame, 64.0);

    let title = make_label(
        mtm,
        "Settings",
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - SETTINGS_HEADER_TOP - SETTINGS_HEADER_HEIGHT),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0) - 80.0, SETTINGS_HEADER_HEIGHT),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title);

    let version_text = format!("v{}", env!("CARGO_PKG_VERSION"));
    let version_badge = make_badge_view(
        mtm,
        &version_text,
        badge_width(&version_text, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let version_width = version_badge.frame().size.width;
    version_badge.setFrame(NSRect::new(
        NSPoint::new(
            frame.size.width - version_width - CARD_INSET_X,
            frame.size.height - SETTINGS_HEADER_TOP - version_badge.frame().size.height,
        ),
        version_badge.frame().size,
    ));
    view.addSubview(&version_badge);

    let settings = crate::app_settings::current_app_settings();
    let display_count = current_display_count();
    let display_value = format!("Screen {}/{}", settings.preferred_display_index + 1, display_count.max(1));
    let sound_value = if settings.completion_sound_enabled { "On" } else { "Off" };
    let mascot_value = if settings.mascot_enabled { "On" } else { "Off" };

    let rows = [
        ("Island display", display_value.as_str(), false),
        ("Completion sound", sound_value, settings.completion_sound_enabled),
        ("Mascot", mascot_value, settings.mascot_enabled),
        ("Update & upgrade", "Open", false),
    ];

    for (index, (title, value, active)) in rows.into_iter().enumerate() {
        let row = make_settings_action_row(
            mtm,
            settings_surface_row_frame(frame, index),
            title,
            value,
            active,
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
            NSPoint::new(12.0, ((frame.size.height - SETTINGS_ROW_TITLE_HEIGHT) / 2.0).round()),
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

fn current_display_count() -> usize {
    let Some(mtm) = MainThreadMarker::new() else {
        return 1;
    };
    NSScreen::screens(mtm).len()
}
