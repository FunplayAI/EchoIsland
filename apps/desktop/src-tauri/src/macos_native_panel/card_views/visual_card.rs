use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSFont, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::super::card_animation::register_card_animation_layout;
use super::super::display_helpers::compact_title;
use super::super::panel_constants::{CARD_CHAT_GAP, CARD_TOOL_GAP};
use super::super::panel_helpers::{estimated_chat_body_height, ns_color};
use super::common::{apply_card_layer, make_label};
use crate::native_panel_core::{PanelPoint, PanelRect};
use crate::native_panel_renderer::facade::presentation::{
    CardVisualBodyRole, CardVisualColorSpec, CardVisualSpec, CardVisualStyle,
    card_visual_action_hint_layout, card_visual_badge_layout, card_visual_body_layout,
    card_visual_body_line_paint_spec, card_visual_content_layout,
    card_visual_header_text_paint_spec, card_visual_settings_row_layout,
    card_visual_single_line_text_box_frame, card_visual_spec_from_scene_card_with_height,
    card_visual_tool_pill_layout,
};
use crate::native_panel_scene::SceneCard;

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn create_visual_scene_card(
    frame: NSRect,
    card: &SceneCard,
) -> objc2::rc::Retained<NSView> {
    let spec = card_visual_spec_from_scene_card_with_height(card, frame.size.height);
    create_visual_card(frame, &spec)
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_visual_card(frame: NSRect, spec: &CardVisualSpec) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(
        &view,
        card_color(spec.shell.fill_color),
        card_color(spec.shell.border_color),
    );
    register_card_animation_layout(&view, frame, spec.animation.collapsed_height);

    let local_frame = PanelRect {
        x: 0.0,
        y: 0.0,
        width: frame.size.width,
        height: frame.size.height,
    };
    if spec.style == CardVisualStyle::Empty {
        add_empty_card_content(&view, mtm, local_frame, spec);
        return view;
    }

    add_card_header(&view, mtm, local_frame, spec);
    if !spec.rows.is_empty() {
        add_settings_rows(&view, mtm, local_frame, spec);
        return view;
    }
    add_card_body(&view, mtm, local_frame, spec);
    if let Some(action_hint) = &spec.action_hint {
        add_action_hint(&view, mtm, local_frame, action_hint);
    }

    view
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_empty_card_content(
    view: &NSView,
    mtm: MainThreadMarker,
    frame: PanelRect,
    spec: &CardVisualSpec,
) {
    let layout = card_visual_content_layout(frame);
    let paint = card_visual_header_text_paint_spec(spec.style);
    let label = make_label(
        mtm,
        &spec.title,
        ns_rect(PanelRect {
            x: frame.x,
            y: layout.empty_title_y,
            width: frame.width,
            height: 20.0,
        }),
        paint.title.size as f64,
        card_color(paint.title.color),
        true,
        true,
    );
    label.setFont(Some(&NSFont::boldSystemFontOfSize(paint.title.size as f64)));
    view.addSubview(&label);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_card_header(
    view: &NSView,
    mtm: MainThreadMarker,
    frame: PanelRect,
    spec: &CardVisualSpec,
) {
    let layout = card_visual_content_layout(frame);
    let paint = card_visual_header_text_paint_spec(spec.style);
    let mut badge_right = frame.x + frame.width - 12.0;
    let mut leftmost_badge_x = badge_right;
    for (index, badge) in spec.badges.iter().enumerate() {
        let right = if index == 0 {
            badge_right
        } else {
            badge_right - 6.0
        };
        let badge_layout = card_visual_badge_layout(spec.style, badge, right, layout.title_y);
        let badge_view = make_visual_badge_view(
            mtm,
            &badge.text,
            badge_layout.badge_frame.width,
            badge_layout.paint.height,
            badge_layout.paint.radius,
            badge_layout.paint.text_inset_x,
            badge_layout.paint.text_offset_y,
            badge_layout.paint.text_size as f64,
            card_color(badge_layout.paint.background_color),
            card_color(badge_layout.paint.foreground_color),
        );
        badge_view.setFrame(ns_rect(badge_layout.badge_frame));
        view.addSubview(&badge_view);
        badge_right = badge_layout.badge_frame.x;
        leftmost_badge_x = badge_layout.badge_frame.x;
    }

    let title_width = (leftmost_badge_x - layout.content_x - 8.0).max(92.0);
    let title = make_label(
        mtm,
        &compact_title(&spec.title, paint.title_max_chars),
        ns_rect(PanelRect {
            x: layout.content_x,
            y: layout.title_y,
            width: title_width,
            height: 16.0,
        }),
        paint.title.size as f64,
        card_color(paint.title.color),
        false,
        true,
    );
    title.setFont(Some(&NSFont::boldSystemFontOfSize(paint.title.size as f64)));
    view.addSubview(&title);

    if let Some(subtitle) = &spec.subtitle {
        let subtitle = make_label(
            mtm,
            subtitle,
            ns_rect(PanelRect {
                x: layout.content_x,
                y: layout.subtitle_y,
                width: layout.content_width,
                height: 12.0,
            }),
            paint.subtitle.size as f64,
            card_color(paint.subtitle.color),
            false,
            true,
        );
        subtitle.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            paint.subtitle.size as f64,
            unsafe { objc2_app_kit::NSFontWeightRegular },
        )));
        view.addSubview(&subtitle);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_card_body(
    view: &NSView,
    mtm: MainThreadMarker,
    frame: PanelRect,
    spec: &CardVisualSpec,
) {
    let body_layout = card_visual_body_layout(frame, spec.action_hint.is_some());
    let mut cursor_y = body_layout.initial_y;
    for (index, line) in spec.body.iter().enumerate() {
        if line.role == CardVisualBodyRole::Tool {
            add_tool_pill(view, mtm, frame, cursor_y, &line.text);
            cursor_y += 22.0;
            if index + 1 < spec.body.len() {
                cursor_y += CARD_TOOL_GAP;
            }
            continue;
        }

        let max_lines = line.max_lines as isize;
        let body_height = estimated_chat_body_height(&line.text, body_layout.body_width, max_lines);
        if let Some(prefix) = &line.prefix {
            let line_paint = card_visual_body_line_paint_spec(spec.style, line.role, Some(prefix));
            let prefix_label = make_label(
                mtm,
                prefix,
                ns_rect(PanelRect {
                    x: body_layout.prefix_x,
                    y: cursor_y + body_height - 12.0,
                    width: 10.0,
                    height: 12.0,
                }),
                line_paint.prefix_size as f64,
                card_color(line_paint.prefix_color),
                true,
                true,
            );
            prefix_label.setFont(Some(&NSFont::boldSystemFontOfSize(
                line_paint.prefix_size as f64,
            )));
            view.addSubview(&prefix_label);
        }

        let line_paint =
            card_visual_body_line_paint_spec(spec.style, line.role, line.prefix.as_deref());
        let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(&line.text), mtm);
        body_label.setFrame(ns_rect(PanelRect {
            x: body_layout.text_x,
            y: cursor_y,
            width: body_layout.body_width,
            height: body_height,
        }));
        body_label.setTextColor(Some(&ns_color(card_color(line_paint.text_color))));
        body_label.setFont(Some(&NSFont::systemFontOfSize(line_paint.text_size as f64)));
        body_label.setDrawsBackground(false);
        body_label.setBezeled(false);
        body_label.setBordered(false);
        body_label.setEditable(false);
        body_label.setSelectable(false);
        body_label.setMaximumNumberOfLines(max_lines);
        view.addSubview(&body_label);

        cursor_y += body_height;
        if index + 1 < spec.body.len() {
            cursor_y += CARD_CHAT_GAP;
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_tool_pill(
    view: &NSView,
    mtm: MainThreadMarker,
    frame: PanelRect,
    y: f64,
    text: &str,
) {
    let Some(layout) = card_visual_tool_pill_layout(frame, y, text) else {
        return;
    };
    let pill = NSView::initWithFrame(NSView::alloc(mtm), ns_rect(layout.pill_frame));
    pill.setWantsLayer(true);
    if let Some(layer) = pill.layer() {
        layer.setCornerRadius(layout.paint.radius);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(
            &ns_color(card_color(layout.paint.background_color)).CGColor(),
        ));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&ns_color([1.0, 1.0, 1.0, 0.06]).CGColor()));
    }

    let name = make_label(
        mtm,
        &layout.paint.tool_name,
        ns_rect(relative_rect(
            layout.tool_name_origin,
            layout.tool_name_max_width,
            11.0,
            layout.pill_frame,
        )),
        layout.paint.text_size as f64,
        card_color(layout.paint.tool_name_color),
        false,
        true,
    );
    name.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        layout.paint.text_size as f64,
        unsafe { objc2_app_kit::NSFontWeightBold },
    )));
    pill.addSubview(&name);

    if let Some(description) = &layout.description {
        let desc = make_label(
            mtm,
            &description.text,
            ns_rect(relative_rect(
                description.origin,
                description.max_width,
                11.0,
                layout.pill_frame,
            )),
            layout.paint.text_size as f64,
            card_color(layout.paint.description_color),
            false,
            true,
        );
        desc.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            layout.paint.text_size as f64,
            unsafe { objc2_app_kit::NSFontWeightRegular },
        )));
        pill.addSubview(&desc);
    }

    view.addSubview(&pill);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_action_hint(view: &NSView, mtm: MainThreadMarker, frame: PanelRect, text: &str) {
    let Some(layout) = card_visual_action_hint_layout(frame, text) else {
        return;
    };
    let badge = make_visual_badge_view(
        mtm,
        &layout.paint.text,
        layout.pill_frame.width,
        layout.paint.height,
        layout.paint.radius,
        layout.paint.text_inset_x,
        layout.paint.text_offset_y,
        layout.paint.text_size as f64,
        card_color(layout.paint.background_color),
        card_color(layout.paint.foreground_color),
    );
    badge.setFrame(ns_rect(layout.pill_frame));
    view.addSubview(&badge);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn add_settings_rows(
    view: &NSView,
    mtm: MainThreadMarker,
    frame: PanelRect,
    spec: &CardVisualSpec,
) {
    for (index, row) in spec.rows.iter().enumerate() {
        let Some(layout) = card_visual_settings_row_layout(frame, index, row) else {
            break;
        };
        let row_view = NSView::initWithFrame(NSView::alloc(mtm), ns_rect(layout.row_frame));
        row_view.setWantsLayer(true);
        if let Some(layer) = row_view.layer() {
            layer.setCornerRadius(layout.paint.border_radius);
            layer.setMasksToBounds(true);
            layer.setBackgroundColor(Some(
                &ns_color(card_color(layout.paint.fill_color)).CGColor(),
            ));
            layer.setBorderWidth(1.0);
            layer.setBorderColor(Some(
                &ns_color(card_color(layout.paint.border_color)).CGColor(),
            ));
        }

        let title = make_label(
            mtm,
            &row.title,
            ns_rect(relative_rect(
                layout.title_origin,
                layout.title_max_width,
                16.0,
                layout.row_frame,
            )),
            layout.paint.title_size as f64,
            card_color(layout.paint.title_color),
            false,
            true,
        );
        title.setFont(Some(&NSFont::boldSystemFontOfSize(
            layout.paint.title_size as f64,
        )));
        row_view.addSubview(&title);

        let badge_frame = relative_panel_rect(layout.value_badge_frame, layout.row_frame);
        let badge = make_visual_badge_view(
            mtm,
            &row.value,
            badge_frame.width,
            badge_frame.height,
            layout.paint.value_badge.radius,
            layout.paint.value_badge.text_inset_x,
            layout.paint.value_badge.text_offset_y,
            layout.paint.value_badge.text_size as f64,
            card_color(layout.paint.value_badge.background_color),
            card_color(layout.paint.value_badge.foreground_color),
        );
        badge.setFrame(ns_rect(badge_frame));
        row_view.addSubview(&badge);
        view.addSubview(&row_view);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::too_many_arguments)]
unsafe fn make_visual_badge_view(
    mtm: MainThreadMarker,
    text: &str,
    width: f64,
    height: f64,
    radius: f64,
    text_inset_x: f64,
    text_offset_y: f64,
    text_size: f64,
    background: [f64; 4],
    foreground: [f64; 4],
) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width.max(1.0), height)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(radius);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&ns_color(background).CGColor()));
    }
    let label = make_label(
        mtm,
        text,
        ns_rect(
            card_visual_single_line_text_box_frame(
                width,
                height,
                text_inset_x,
                text_offset_y,
                text_size,
            )
            .frame,
        ),
        text_size,
        foreground,
        true,
        true,
    );
    label.setFont(Some(&NSFont::systemFontOfSize_weight(text_size, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    view.addSubview(&label);
    view
}

fn card_color(color: CardVisualColorSpec) -> [f64; 4] {
    [
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        1.0,
    ]
}

fn ns_rect(rect: PanelRect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, rect.y),
        NSSize::new(rect.width, rect.height),
    )
}

fn relative_rect(origin: PanelPoint, width: f64, height: f64, parent: PanelRect) -> PanelRect {
    PanelRect {
        x: origin.x - parent.x,
        y: origin.y - parent.y,
        width,
        height,
    }
}

fn relative_panel_rect(rect: PanelRect, parent: PanelRect) -> PanelRect {
    PanelRect {
        x: rect.x - parent.x,
        y: rect.y - parent.y,
        width: rect.width,
        height: rect.height,
    }
}
