use objc2::rc::Retained;
use objc2_app_kit::NSFont;

use crate::native_panel_renderer::facade::visual::{
    NativePanelVisualTextRole, NativePanelVisualTextWeight,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::macos_native_panel) struct MacosFontVerticalMetrics {
    pub(in crate::macos_native_panel) ascender: f64,
    pub(in crate::macos_native_panel) descender: f64,
    pub(in crate::macos_native_panel) cap_height: f64,
    pub(in crate::macos_native_panel) leading: f64,
}

pub(in crate::macos_native_panel) fn centered_child_origin(
    container_origin: f64,
    container_size: f64,
    child_size: f64,
) -> f64 {
    container_origin + ((container_size - child_size) / 2.0).max(0.0)
}

pub(in crate::macos_native_panel) fn macos_text_frame_origin_for_visual_center(
    container_origin: f64,
    container_height: f64,
    font: &NSFont,
) -> f64 {
    macos_text_frame_origin_for_visual_center_from_metrics(
        container_origin,
        container_height,
        macos_font_vertical_metrics(font),
    )
}

fn macos_text_frame_origin_for_visual_center_from_metrics(
    container_origin: f64,
    container_height: f64,
    metrics: MacosFontVerticalMetrics,
) -> f64 {
    container_origin + container_height / 2.0 - macos_glyph_center_from_frame_origin(metrics)
}

pub(in crate::macos_native_panel) fn macos_text_frame_height(font: &NSFont) -> f64 {
    macos_text_frame_height_from_metrics(macos_font_vertical_metrics(font))
}

fn macos_text_frame_height_from_metrics(metrics: MacosFontVerticalMetrics) -> f64 {
    (metrics.ascender - metrics.descender + metrics.leading)
        .ceil()
        .max(1.0)
}

pub(in crate::macos_native_panel) fn macos_glyph_center_from_frame_origin(
    metrics: MacosFontVerticalMetrics,
) -> f64 {
    (-metrics.descender + metrics.cap_height / 2.0).max(0.0)
}

pub(in crate::macos_native_panel) fn macos_font_vertical_metrics(
    font: &NSFont,
) -> MacosFontVerticalMetrics {
    MacosFontVerticalMetrics {
        ascender: font.ascender(),
        descender: font.descender(),
        cap_height: font.capHeight(),
        leading: font.leading(),
    }
}

pub(in crate::macos_native_panel) fn macos_action_icon_font_size(
    role: NativePanelVisualTextRole,
    shared_size: i32,
) -> f64 {
    match role {
        NativePanelVisualTextRole::ActionButtonSettings => 20.0,
        _ => shared_size as f64,
    }
}

pub(in crate::macos_native_panel) fn macos_action_icon_glyph_offset_y(
    role: NativePanelVisualTextRole,
) -> f64 {
    match role {
        NativePanelVisualTextRole::ActionButtonQuit => -2.0,
        _ => 0.0,
    }
}

pub(in crate::macos_native_panel) fn font_for_visual_weight(
    weight: NativePanelVisualTextWeight,
    size: f64,
) -> Retained<NSFont> {
    match weight {
        NativePanelVisualTextWeight::Bold | NativePanelVisualTextWeight::Semibold => {
            NSFont::boldSystemFontOfSize(size)
        }
        NativePanelVisualTextWeight::Normal => NSFont::systemFontOfSize(size),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MacosFontVerticalMetrics, centered_child_origin, macos_action_icon_font_size,
        macos_action_icon_glyph_offset_y, macos_glyph_center_from_frame_origin,
        macos_text_frame_height_from_metrics,
        macos_text_frame_origin_for_visual_center_from_metrics,
    };
    use crate::native_panel_renderer::facade::visual::NativePanelVisualTextRole;

    #[test]
    fn centered_child_origin_uses_container_center() {
        assert_eq!(centered_child_origin(5.5, 24.0, 16.0), 9.5);
        assert_eq!(centered_child_origin(0.0, 26.0, 24.0), 1.0);
        assert_eq!(centered_child_origin(0.0, 26.0, 26.0), 0.0);
    }

    #[test]
    fn action_icon_font_size_keeps_settings_larger_than_shared_visual_size() {
        assert_eq!(
            macos_action_icon_font_size(NativePanelVisualTextRole::ActionButtonSettings, 16),
            20.0
        );
        assert_eq!(
            macos_action_icon_font_size(NativePanelVisualTextRole::ActionButtonQuit, 16),
            16.0
        );
    }

    #[test]
    fn action_icon_glyph_offset_is_separate_from_centering_layout() {
        assert_eq!(
            macos_action_icon_glyph_offset_y(NativePanelVisualTextRole::ActionButtonSettings),
            0.0
        );
        assert_eq!(
            macos_action_icon_glyph_offset_y(NativePanelVisualTextRole::ActionButtonQuit),
            -2.0
        );
    }

    #[test]
    fn text_frame_origin_centers_cap_height_using_font_metrics() {
        let metrics = MacosFontVerticalMetrics {
            ascender: 12.0,
            descender: -4.0,
            cap_height: 8.0,
            leading: 0.0,
        };

        assert_eq!(macos_text_frame_height_from_metrics(metrics), 16.0);
        assert_eq!(macos_glyph_center_from_frame_origin(metrics), 8.0);
        assert_eq!(
            macos_text_frame_origin_for_visual_center_from_metrics(5.5, 24.0, metrics),
            9.5
        );
    }
}
