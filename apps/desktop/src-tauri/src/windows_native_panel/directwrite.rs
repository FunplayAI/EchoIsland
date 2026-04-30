#[cfg(windows)]
use std::sync::OnceLock;
#[cfg(windows)]
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_NORMAL,
    DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_WORD_WRAPPING_NO_WRAP,
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat,
};
#[cfg(windows)]
use windows::core::HSTRING;

#[derive(Clone, Debug, Default)]
pub(super) struct WindowsDirectWriteFactory {
    #[cfg(windows)]
    factory: Option<IDWriteFactory>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsDirectWriteFontFallback {
    pub(super) primary: &'static str,
    pub(super) fallback: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WindowsDirectWriteTextLayoutRequest {
    pub(super) text: String,
    pub(super) max_width: f64,
    pub(super) size: i32,
    pub(super) weight: crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight,
    pub(super) alignment:
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment,
    pub(super) fonts: WindowsDirectWriteFontFallback,
}

impl WindowsDirectWriteFontFallback {
    pub(super) fn native_ui_default() -> Self {
        Self {
            primary: "Noto Sans SC",
            fallback: "Segoe UI Variable",
        }
    }

    pub(super) fn windows_settings_icon() -> Self {
        Self {
            primary: "Segoe MDL2 Assets",
            fallback: "Segoe Fluent Icons",
        }
    }

    pub(super) fn for_text(text: &str) -> Self {
        if text.chars().any(|character| character == '\u{E713}') {
            Self::windows_settings_icon()
        } else {
            Self::native_ui_default()
        }
    }
}

impl WindowsDirectWriteTextLayoutRequest {
    pub(super) fn new(
        text: String,
        max_width: f64,
        size: i32,
        weight: crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight,
        alignment: crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment,
    ) -> Self {
        Self {
            fonts: WindowsDirectWriteFontFallback::for_text(&text),
            text,
            max_width,
            size,
            weight,
            alignment,
        }
    }
}

impl WindowsDirectWriteFactory {
    pub(super) fn empty() -> Self {
        Self {
            #[cfg(windows)]
            factory: None,
        }
    }

    #[cfg(windows)]
    pub(super) fn create() -> Result<Self, String> {
        let factory = unsafe { DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) }
            .map_err(|error| error.to_string())?;
        Ok(Self {
            factory: Some(factory),
        })
    }

    #[cfg(not(windows))]
    pub(super) fn create() -> Result<Self, String> {
        Ok(Self::empty())
    }

    #[cfg(windows)]
    pub(super) fn shared() -> Result<Self, String> {
        static FACTORY: OnceLock<Result<WindowsDirectWriteFactory, String>> = OnceLock::new();
        FACTORY.get_or_init(Self::create).clone()
    }

    #[cfg(not(windows))]
    pub(super) fn shared() -> Result<Self, String> {
        Self::create()
    }

    pub(super) fn is_initialized(&self) -> bool {
        #[cfg(windows)]
        {
            self.factory.is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    #[cfg(windows)]
    pub(super) fn create_text_format(
        &self,
        fonts: WindowsDirectWriteFontFallback,
        size: i32,
        weight: crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight,
        alignment: crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment,
    ) -> Result<IDWriteTextFormat, String> {
        let Some(factory) = self.factory.as_ref() else {
            return Err("DirectWrite factory is not initialized".to_string());
        };
        let format = unsafe {
            factory.CreateTextFormat(
                &HSTRING::from(fonts.primary),
                None,
                dwrite_font_weight(weight),
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size.max(1) as f32,
                &HSTRING::from("en-us"),
            )
        }
        .map_err(|error| error.to_string())?;
        unsafe {
            format
                .SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)
                .map_err(|error| error.to_string())?;
            format
                .SetTextAlignment(dwrite_text_alignment(alignment))
                .map_err(|error| error.to_string())?;
            format
                .SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)
                .map_err(|error| error.to_string())?;
        }
        Ok(format)
    }
}

#[cfg(windows)]
fn dwrite_font_weight(
    weight: crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight,
) -> DWRITE_FONT_WEIGHT {
    match weight {
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Normal => {
            DWRITE_FONT_WEIGHT_NORMAL
        }
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Semibold => {
            DWRITE_FONT_WEIGHT_SEMI_BOLD
        }
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Bold => {
            DWRITE_FONT_WEIGHT_BOLD
        }
    }
}

#[cfg(windows)]
fn dwrite_text_alignment(
    alignment: crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment,
) -> windows::Win32::Graphics::DirectWrite::DWRITE_TEXT_ALIGNMENT {
    match alignment {
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Left => {
            DWRITE_TEXT_ALIGNMENT_LEADING
        }
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Center => {
            DWRITE_TEXT_ALIGNMENT_CENTER
        }
        crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Right => {
            DWRITE_TEXT_ALIGNMENT_TRAILING
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WindowsDirectWriteFactory, WindowsDirectWriteFontFallback,
        WindowsDirectWriteTextLayoutRequest,
    };

    #[test]
    fn directwrite_factory_wrapper_can_represent_uninitialized_factory() {
        let factory = WindowsDirectWriteFactory::empty();

        assert!(!factory.is_initialized());
    }

    #[cfg(windows)]
    #[test]
    fn directwrite_factory_can_initialize_on_windows() {
        let factory = WindowsDirectWriteFactory::create().expect("create DirectWrite factory");

        assert!(factory.is_initialized());
    }

    #[test]
    fn directwrite_text_requests_use_windows_native_font_fallback() {
        let request = WindowsDirectWriteTextLayoutRequest::new(
            "Codex ready".to_string(),
            160.0,
            14,
            crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Semibold,
            crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Left,
        );

        assert_eq!(
            request.fonts,
            WindowsDirectWriteFontFallback {
                primary: "Noto Sans SC",
                fallback: "Segoe UI Variable",
            }
        );
        assert_eq!(request.max_width, 160.0);
        assert_eq!(request.size, 14);
        assert_eq!(
            request.weight,
            crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Semibold
        );
    }

    #[test]
    fn directwrite_text_requests_use_windows_icon_font_for_settings_glyph() {
        let request = WindowsDirectWriteTextLayoutRequest::new(
            "\u{E713}".to_string(),
            26.0,
            16,
            crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Normal,
            crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Center,
        );

        assert_eq!(
            request.fonts,
            WindowsDirectWriteFontFallback {
                primary: "Segoe MDL2 Assets",
                fallback: "Segoe Fluent Icons",
            }
        );
    }
}
