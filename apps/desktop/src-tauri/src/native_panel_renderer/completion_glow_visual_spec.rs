use crate::native_panel_core::{COMPACT_PILL_RADIUS, PanelRect};

pub(crate) const COMPLETION_GLOW_IMAGE_WIDTH: f64 = 480.0;
pub(crate) const COMPLETION_GLOW_IMAGE_HEIGHT: f64 = 160.0;
pub(crate) const COMPLETION_GLOW_IMAGE_RADIUS: f64 = 64.0;
pub(crate) const COMPLETION_GLOW_SLICE_LEFT: f64 = 160.0;
pub(crate) const COMPLETION_GLOW_SLICE_RIGHT: f64 = 160.0;
pub(crate) const COMPLETION_GLOW_VISIBLE_THRESHOLD: f64 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompletionGlowVisualSpecInput {
    pub(crate) visible: bool,
    pub(crate) frame: PanelRect,
    pub(crate) base_opacity: f64,
    pub(crate) elapsed_ms: u128,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompletionGlowVisualSpec {
    pub(crate) frame: PanelRect,
    pub(crate) opacity: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompletionGlowImageSliceSpec {
    pub(crate) dest: PanelRect,
    pub(crate) source: PanelRect,
}

pub(crate) fn resolve_completion_glow_visual_spec(
    input: CompletionGlowVisualSpecInput,
) -> Option<CompletionGlowVisualSpec> {
    if !input.visible {
        return None;
    }
    let breath = (((input.elapsed_ms as f64 / 1000.0) * 3.2).sin() + 1.0) * 0.5;
    let opacity = input.base_opacity.clamp(0.0, 1.0) * (0.42 + breath * 0.46);
    (opacity > COMPLETION_GLOW_VISIBLE_THRESHOLD).then_some(CompletionGlowVisualSpec {
        frame: input.frame,
        opacity: opacity.clamp(0.0, 1.0),
    })
}

pub(crate) fn resolve_completion_glow_image_slices(
    frame: PanelRect,
) -> [CompletionGlowImageSliceSpec; 3] {
    let display_scale = (COMPACT_PILL_RADIUS / COMPLETION_GLOW_IMAGE_RADIUS).max(0.0);
    let display_height = (COMPLETION_GLOW_IMAGE_HEIGHT * display_scale)
        .min(frame.height)
        .max(0.0);
    let mut left_width = COMPLETION_GLOW_SLICE_LEFT * display_scale;
    let mut right_width = COMPLETION_GLOW_SLICE_RIGHT * display_scale;

    let total_cap_width = left_width + right_width;
    if total_cap_width > frame.width && total_cap_width > 0.0 {
        let shrink = frame.width / total_cap_width;
        left_width *= shrink;
        right_width *= shrink;
    }

    let center_width = (frame.width - left_width - right_width).max(0.0);
    [
        CompletionGlowImageSliceSpec {
            dest: PanelRect {
                x: frame.x,
                y: frame.y,
                width: left_width,
                height: display_height,
            },
            source: PanelRect {
                x: 0.0,
                y: 0.0,
                width: COMPLETION_GLOW_SLICE_LEFT,
                height: COMPLETION_GLOW_IMAGE_HEIGHT,
            },
        },
        CompletionGlowImageSliceSpec {
            dest: PanelRect {
                x: frame.x + left_width,
                y: frame.y,
                width: center_width,
                height: display_height,
            },
            source: PanelRect {
                x: COMPLETION_GLOW_SLICE_LEFT,
                y: 0.0,
                width: COMPLETION_GLOW_IMAGE_WIDTH
                    - COMPLETION_GLOW_SLICE_LEFT
                    - COMPLETION_GLOW_SLICE_RIGHT,
                height: COMPLETION_GLOW_IMAGE_HEIGHT,
            },
        },
        CompletionGlowImageSliceSpec {
            dest: PanelRect {
                x: frame.x + frame.width - right_width,
                y: frame.y,
                width: right_width,
                height: display_height,
            },
            source: PanelRect {
                x: COMPLETION_GLOW_IMAGE_WIDTH - COMPLETION_GLOW_SLICE_RIGHT,
                y: 0.0,
                width: COMPLETION_GLOW_SLICE_RIGHT,
                height: COMPLETION_GLOW_IMAGE_HEIGHT,
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::native_panel_core::PanelRect;

    use super::{
        COMPLETION_GLOW_VISIBLE_THRESHOLD, CompletionGlowVisualSpecInput,
        resolve_completion_glow_image_slices, resolve_completion_glow_visual_spec,
    };

    #[test]
    fn completion_glow_visual_spec_resolves_shared_breathing_opacity() {
        let spec = resolve_completion_glow_visual_spec(CompletionGlowVisualSpecInput {
            visible: true,
            frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 170.0,
                height: 25.0,
            },
            base_opacity: 0.78,
            elapsed_ms: 0,
        })
        .expect("visible glow");

        assert!((spec.opacity - 0.507).abs() < 0.001);
    }

    #[test]
    fn completion_glow_visual_spec_hides_below_threshold() {
        assert!(
            resolve_completion_glow_visual_spec(CompletionGlowVisualSpecInput {
                visible: true,
                frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 170.0,
                    height: 25.0,
                },
                base_opacity: COMPLETION_GLOW_VISIBLE_THRESHOLD * 0.1,
                elapsed_ms: 0,
            })
            .is_none()
        );
    }

    #[test]
    fn completion_glow_image_slices_use_shared_radius_scale() {
        let slices = resolve_completion_glow_image_slices(PanelRect {
            x: 10.0,
            y: 4.0,
            width: 170.0,
            height: 25.0,
        });

        assert_eq!(slices[0].dest.x, 10.0);
        assert!((slices[0].dest.width - 31.25).abs() < 0.001);
        assert_eq!(slices[1].dest.y, 4.0);
        assert!((slices[2].dest.x - 148.75).abs() < 0.001);
        assert_eq!(slices[2].source.x, 320.0);
    }
}
