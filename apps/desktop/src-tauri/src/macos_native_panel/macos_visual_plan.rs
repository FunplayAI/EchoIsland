mod compact_adapter;
mod glow_adapter;
mod mascot_adapter;

pub(super) use compact_adapter::apply_macos_visual_plan_compact_primitives;
pub(super) use glow_adapter::{
    completion_glow_primitive, resolve_macos_completion_glow_visual_plan,
};
pub(super) use mascot_adapter::{
    MacosMascotCompletionBadgePrimitive, MacosMascotEllipsePrimitive,
    MacosMascotMessageBubblePrimitive, MacosMascotTextPrimitive, apply_macos_mascot_body_primitive,
    mascot_body_primitive, mascot_completion_badge_primitive, mascot_eye_primitive,
    mascot_message_bubble_primitive, mascot_mouth_primitive, mascot_sleep_label_primitive,
    resolve_macos_mascot_visual_plan,
};

use objc2_foundation::NSRect;

use super::panel_types::NativePanelLayout;
use crate::native_panel_core::PanelRect;
use crate::native_panel_renderer::facade::{
    descriptor::NativePanelHostWindowState,
    presentation::{
        NativePanelPresentationModel, native_panel_visual_display_mode_from_presentation,
        native_panel_visual_plan_input_from_presentation,
    },
    visual::{NativePanelVisualPlan, resolve_native_panel_visual_plan},
};

pub(super) fn resolve_macos_native_panel_visual_plan(
    layout: &NativePanelLayout,
    presentation: &NativePanelPresentationModel,
) -> NativePanelVisualPlan {
    let window_state = NativePanelHostWindowState {
        frame: Some(panel_rect_from_ns_rect(layout.panel_frame)),
        visible: true,
        preferred_display_index: 0,
    };
    let display_mode =
        native_panel_visual_display_mode_from_presentation(window_state, Some(presentation));
    let input = native_panel_visual_plan_input_from_presentation(
        window_state,
        display_mode,
        Some(presentation),
    );

    resolve_native_panel_visual_plan(&input)
}

fn panel_rect_from_ns_rect(rect: NSRect) -> PanelRect {
    PanelRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}
