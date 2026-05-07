use crate::native_panel_core::{ExpandedSurface, PanelRect};
use crate::native_panel_renderer::facade::{
    descriptor::NativePanelHostWindowState,
    presentation::{
        CompletionGlowImageSliceSpec, NativePanelVisualDisplayMode, NativePanelVisualPlanInput,
        resolve_completion_glow_image_slices,
    },
    visual::{NativePanelVisualPlan, NativePanelVisualPrimitive, resolve_native_panel_visual_plan},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosCompletionGlowPrimitive {
    pub(in crate::macos_native_panel) frame: PanelRect,
    pub(in crate::macos_native_panel) opacity: f64,
    pub(in crate::macos_native_panel) slices: [CompletionGlowImageSliceSpec; 3],
}

pub(in crate::macos_native_panel) fn resolve_macos_completion_glow_visual_plan(
    frame: PanelRect,
    visible: bool,
    base_opacity: f64,
    elapsed_ms: u128,
) -> NativePanelVisualPlan {
    let zero = PanelRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    resolve_native_panel_visual_plan(&NativePanelVisualPlanInput {
        window_state: NativePanelHostWindowState {
            frame: Some(frame),
            visible: true,
            preferred_display_index: 0,
        },
        display_mode: NativePanelVisualDisplayMode::Compact,
        surface: ExpandedSurface::Default,
        panel_frame: frame,
        compact_bar_frame: frame,
        left_shoulder_frame: zero,
        right_shoulder_frame: zero,
        shoulder_progress: 0.0,
        content_frame: frame,
        card_stack_frame: zero,
        card_stack_content_height: 0.0,
        shell_frame: frame,
        headline_text: String::new(),
        headline_emphasized: false,
        active_count: String::new(),
        active_count_elapsed_ms: 0,
        total_count: String::new(),
        separator_visibility: 0.0,
        cards_visible: false,
        card_count: 0,
        cards: Vec::new(),
        glow_visible: visible,
        glow_opacity: base_opacity,
        action_buttons_visible: false,
        action_buttons: Vec::new(),
        completion_count: 0,
        mascot_elapsed_ms: elapsed_ms,
        mascot_motion_frame: None,
        mascot_pose: crate::native_panel_scene::SceneMascotPose::Idle,
        mascot_debug_mode_enabled: false,
    })
}

pub(in crate::macos_native_panel) fn completion_glow_primitive(
    plan: &NativePanelVisualPlan,
) -> Option<MacosCompletionGlowPrimitive> {
    plan.primitives.iter().find_map(|primitive| {
        let NativePanelVisualPrimitive::CompletionGlow { frame, opacity } = primitive else {
            return None;
        };
        Some(MacosCompletionGlowPrimitive {
            frame: *frame,
            opacity: *opacity,
            slices: resolve_completion_glow_image_slices(*frame),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{completion_glow_primitive, resolve_macos_completion_glow_visual_plan};
    use crate::native_panel_core::PanelRect;

    #[test]
    fn macos_completion_glow_uses_shared_visual_plan_primitive() {
        let frame = PanelRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 36.0,
        };
        let plan = resolve_macos_completion_glow_visual_plan(frame, true, 0.5, 0);
        let primitive = completion_glow_primitive(&plan).expect("completion glow");

        assert_eq!(primitive.frame, frame);
        assert!((primitive.opacity - 0.325).abs() < 0.001);
        assert_eq!(primitive.slices[0].dest.x, frame.x);
        assert_eq!(primitive.slices[2].source.x, 320.0);
    }
}
