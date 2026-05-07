use objc2_app_kit::{NSFont, NSTextAlignment, NSTextField, NSView};
use objc2_core_graphics::CGAffineTransformMakeScale;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::completion_glow_view::update_completion_glow_layout_from_slices;
use super::macos_visual_plan::{
    MacosMascotCompletionBadgePrimitive, MacosMascotEllipsePrimitive,
    MacosMascotMessageBubblePrimitive, MacosMascotTextPrimitive, apply_macos_mascot_body_primitive,
    completion_glow_primitive, mascot_body_primitive, mascot_completion_badge_primitive,
    mascot_eye_primitive, mascot_message_bubble_primitive, mascot_mouth_primitive,
    mascot_sleep_label_primitive, resolve_macos_completion_glow_visual_plan,
    resolve_macos_mascot_visual_plan,
};
use super::mascot::{NativeMascotFrame, NativeMascotState};
use super::panel_helpers::ns_color;
use super::panel_refs::resolve_native_panel_refs;
use super::panel_types::NativePanelHandles;
use super::panel_view_updates::with_disabled_layer_actions;
use crate::native_panel_core::PanelRect;
use crate::native_panel_renderer::facade::presentation::{
    COMPLETION_GLOW_VISIBLE_THRESHOLD, NativePanelVisualColor,
};
use crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_native_mascot_frame(
    handles: NativePanelHandles,
    frame: NativeMascotFrame,
) {
    let refs = resolve_native_panel_refs(handles);
    if frame.mascot_hidden {
        refs.mascot_shell.setHidden(true);
        refs.mascot_completion_badge.setHidden(true);
        refs.completion_glow.setHidden(true);
        return;
    }
    let mascot_shell = refs.mascot_shell;
    let mascot_body = refs.mascot_body;
    let mascot_left_eye = refs.mascot_left_eye;
    let mascot_right_eye = refs.mascot_right_eye;
    let mascot_mouth = refs.mascot_mouth;
    let mascot_bubble = refs.mascot_bubble;
    let mascot_sleep_label = refs.mascot_sleep_label;
    let mascot_completion_badge = refs.mascot_completion_badge;
    let mascot_completion_badge_label = refs.mascot_completion_badge_label;
    let completion_glow = refs.completion_glow;
    let visual_plan = resolve_macos_mascot_visual_plan(frame);

    mascot_shell.setHidden(false);
    mascot_shell.setAlphaValue(frame.motion.shell_alpha.clamp(0.0, 1.0));
    if let Some(body) = mascot_body_primitive(&visual_plan) {
        apply_macos_mascot_body_primitive(mascot_body, body, mascot_body_stroke_color(frame));
    }

    apply_mascot_ellipse_primitive(mascot_left_eye, mascot_eye_primitive(&visual_plan, true));
    apply_mascot_ellipse_primitive(mascot_right_eye, mascot_eye_primitive(&visual_plan, false));
    if let Some(mouth) = mascot_mouth_primitive(&visual_plan) {
        mascot_mouth.setFrame(ns_rect_from_panel_rect(mouth.frame));
        if let Some(layer) = mascot_mouth.layer() {
            layer.setCornerRadius(mouth.radius.max(0.8));
            layer.setBackgroundColor(Some(&ns_color(visual_color(mouth.color)).CGColor()));
        }
    }
    mascot_mouth.setAlphaValue(native_mascot_mouth_alpha(frame.state));

    apply_mascot_message_bubble_primitive(
        mascot_bubble,
        mascot_message_bubble_primitive(&visual_plan),
    );
    apply_mascot_sleep_label_primitive(
        mascot_sleep_label,
        mascot_sleep_label_primitive(&visual_plan),
    );

    let completion_badge_primitive = mascot_completion_badge_primitive(&visual_plan);
    let completion_visible = completion_badge_primitive.is_some();
    mascot_completion_badge.setHidden(!completion_visible);
    mascot_completion_badge.setAlphaValue(if completion_visible { 1.0 } else { 0.0 });
    if let Some(badge) = completion_badge_primitive {
        apply_completion_badge_primitive(
            mascot_completion_badge,
            mascot_completion_badge_label,
            &badge,
        );
    }
    sync_completion_glow(completion_glow, frame);

    apply_mascot_face_layer(
        mascot_left_eye,
        mascot_eye_primitive(&visual_plan, true).map(|eye| eye.color),
    );
    apply_mascot_face_layer(
        mascot_right_eye,
        mascot_eye_primitive(&visual_plan, false).map(|eye| eye.color),
    );
    apply_mascot_face_layer(
        mascot_mouth,
        mascot_mouth_primitive(&visual_plan).map(|mouth| mouth.color),
    );

    mascot_shell.displayIfNeeded();
    mascot_body.displayIfNeeded();
    mascot_left_eye.displayIfNeeded();
    mascot_right_eye.displayIfNeeded();
    mascot_mouth.displayIfNeeded();
    mascot_bubble.displayIfNeeded();
    mascot_sleep_label.displayIfNeeded();
    mascot_completion_badge.displayIfNeeded();
    mascot_completion_badge_label.displayIfNeeded();
    completion_glow.displayIfNeeded();
}

fn mascot_body_stroke_color(frame: NativeMascotFrame) -> [f64; 4] {
    if frame.debug_mode_enabled {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        frame.color
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_completion_badge_primitive(
    badge: &NSView,
    label: &NSTextField,
    spec: &MacosMascotCompletionBadgePrimitive,
) {
    badge.setFrame(NSRect::new(
        NSPoint::new(spec.fill.frame.x, spec.fill.frame.y),
        NSSize::new(spec.fill.frame.width, spec.fill.frame.height),
    ));
    label.setStringValue(&NSString::from_str(&spec.label.text));
    label.setHidden(false);
    label.setAlphaValue(spec.label.alpha.clamp(0.0, 1.0));
    label.setAlignment(NSTextAlignment::Center);
    label.setTextColor(Some(&ns_color(visual_color(spec.label.color))));
    label.setFont(Some(&font_for_mascot_text_weight(
        spec.label.weight,
        spec.label.size as f64,
    )));
    label.setFrame(ns_rect_from_panel_rect(completion_badge_label_rect(spec)));
    if let Some(layer) = badge.layer() {
        layer.setCornerRadius(spec.fill.radius);
        layer.setBackgroundColor(Some(&ns_color(visual_color(spec.fill.color)).CGColor()));
        layer.setBorderColor(Some(&ns_color(visual_color(spec.outline.color)).CGColor()));
    }
}

fn completion_badge_label_rect(spec: &MacosMascotCompletionBadgePrimitive) -> PanelRect {
    let height = (spec.label.size as f64 + 3.0).max(1.0);
    PanelRect {
        x: 0.0,
        y: ((spec.fill.frame.height - height) / 2.0).round(),
        width: spec.fill.frame.width.max(1.0),
        height,
    }
}

fn font_for_mascot_text_weight(
    weight: NativePanelVisualTextWeight,
    size: f64,
) -> objc2::rc::Retained<NSFont> {
    match weight {
        NativePanelVisualTextWeight::Bold | NativePanelVisualTextWeight::Semibold => {
            NSFont::boldSystemFontOfSize(size)
        }
        NativePanelVisualTextWeight::Normal => NSFont::systemFontOfSize(size),
    }
}

fn visual_color(color: NativePanelVisualColor) -> [f64; 4] {
    [
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        1.0,
    ]
}

fn ns_rect_from_panel_rect(rect: PanelRect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, rect.y),
        NSSize::new(rect.width, rect.height),
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_mascot_message_bubble_primitive(
    bubble: &NSView,
    spec: Option<MacosMascotMessageBubblePrimitive>,
) {
    let Some(spec) = spec else {
        bubble.setHidden(true);
        bubble.setAlphaValue(0.0);
        return;
    };
    bubble.setHidden(false);
    bubble.setAlphaValue(spec.bubble.alpha.clamp(0.0, 1.0));
    bubble.setFrame(ns_rect_from_panel_rect(spec.bubble.frame));
    let fill = ns_color(visual_color(spec.bubble.color));
    if let Some(layer) = bubble.layer() {
        layer.setCornerRadius(spec.bubble.radius);
        layer.setBackgroundColor(Some(&fill.CGColor()));
        layer.setShadowColor(Some(&fill.CGColor()));
        layer.setShadowOpacity(0.24);
        layer.setShadowRadius(7.0);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_mascot_sleep_label_primitive(
    sleep_label: &NSTextField,
    spec: Option<MacosMascotTextPrimitive>,
) {
    let Some(spec) = spec else {
        sleep_label.setHidden(true);
        sleep_label.setAlphaValue(0.0);
        return;
    };
    sleep_label.setHidden(false);
    sleep_label.setAlphaValue(spec.alpha.clamp(0.0, 1.0));
    sleep_label.setStringValue(&NSString::from_str(&spec.text));
    sleep_label.setFrame(NSRect::new(
        NSPoint::new(spec.origin.x, spec.origin.y),
        NSSize::new(spec.max_width, 10.0),
    ));
    sleep_label.setTextColor(Some(&ns_color(visual_color(spec.color))));
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_mascot_ellipse_primitive(view: &NSView, spec: Option<MacosMascotEllipsePrimitive>) {
    let Some(spec) = spec else {
        view.setHidden(true);
        return;
    };
    view.setHidden(false);
    view.setAlphaValue(spec.alpha.clamp(0.0, 1.0));
    view.setFrame(ns_rect_from_panel_rect(spec.frame));
    if let Some(layer) = view.layer() {
        layer.setCornerRadius((spec.frame.height / 2.0).max(0.8));
        layer.setBackgroundColor(Some(&ns_color(visual_color(spec.color)).CGColor()));
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_mascot_face_layer(view: &NSView, color: Option<NativePanelVisualColor>) {
    let Some(color) = color else {
        return;
    };
    let fill = ns_color(visual_color(color));
    if let Some(layer) = view.layer() {
        layer.setBackgroundColor(Some(&fill.CGColor()));
        layer.setShadowColor(Some(&fill.CGColor()));
        layer.setShadowOpacity(0.22);
        layer.setShadowRadius(6.0);
    }
}

fn sync_completion_glow(completion_glow: &NSView, frame: NativeMascotFrame) {
    let visual_plan = resolve_macos_completion_glow_visual_plan(
        panel_rect_from_ns_rect(completion_glow.frame()),
        frame.completion_glow_opacity > 0.0,
        frame.completion_glow_opacity,
        (frame.t * 1000.0).max(0.0).round() as u128,
    );
    let primitive = completion_glow_primitive(&visual_plan);
    let alpha = primitive.map(|primitive| primitive.opacity).unwrap_or(0.0);
    with_disabled_layer_actions(|| {
        completion_glow.setHidden(alpha <= COMPLETION_GLOW_VISIBLE_THRESHOLD);
        completion_glow.setAlphaValue(alpha);
        if let Some(primitive) = primitive {
            completion_glow.setFrame(ns_rect_from_panel_rect(primitive.frame));
            update_completion_glow_layout_from_slices(completion_glow, &primitive.slices);
        }
        if let Some(layer) = completion_glow.layer() {
            layer.setAffineTransform(CGAffineTransformMakeScale(1.0, 1.0));
        }
    });
}

fn panel_rect_from_ns_rect(rect: NSRect) -> PanelRect {
    PanelRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

fn native_mascot_mouth_alpha(state: NativeMascotState) -> f64 {
    match state {
        NativeMascotState::Sleepy => 0.92,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::macos_native_test_panel::macos_visual_plan::{
        mascot_completion_badge_primitive, resolve_macos_mascot_visual_plan,
    };
    use crate::macos_native_test_panel::mascot::{
        NativeMascotFrame, NativeMascotMotion, NativeMascotState,
    };

    fn frame(
        state: NativeMascotState,
        completion_count: usize,
        debug_mode_enabled: bool,
    ) -> NativeMascotFrame {
        frame_at(state, completion_count, debug_mode_enabled, 0.0)
    }

    fn frame_at(
        state: NativeMascotState,
        completion_count: usize,
        debug_mode_enabled: bool,
        t: f64,
    ) -> NativeMascotFrame {
        NativeMascotFrame {
            state,
            t,
            motion: NativeMascotMotion {
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shell_alpha: 1.0,
                shadow_opacity: 0.0,
                shadow_radius: 0.0,
                eye_open: 1.0,
            },
            color: [1.0, 0.48, 0.14, 1.0],
            completion_count,
            mascot_hidden: false,
            debug_mode_enabled,
            completion_glow_opacity: 0.0,
        }
    }

    #[test]
    fn mascot_body_stroke_color_preserves_runtime_wake_palette() {
        let mut wake_frame = frame(NativeMascotState::WakeAngry, 0, false);
        wake_frame.color = [1.0, 0.38, 0.24, 1.0];

        assert_eq!(
            super::mascot_body_stroke_color(wake_frame),
            wake_frame.color
        );

        let debug_frame = frame(NativeMascotState::Idle, 0, true);
        assert_eq!(
            super::mascot_body_stroke_color(debug_frame),
            [1.0, 1.0, 1.0, 1.0]
        );
    }

    #[test]
    fn completion_badge_label_rect_stays_inside_badge_fill() {
        let plan =
            resolve_macos_mascot_visual_plan(frame(NativeMascotState::MessageBubble, 7, false));
        let badge = mascot_completion_badge_primitive(&plan).expect("completion badge");
        let rect = super::completion_badge_label_rect(&badge);

        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 1.0);
        assert_eq!(rect.width, badge.fill.frame.width);
        assert_eq!(rect.height, 11.0);
        assert!(rect.y >= 0.0);
        assert!(rect.y + rect.height <= badge.fill.frame.height);
    }
}
