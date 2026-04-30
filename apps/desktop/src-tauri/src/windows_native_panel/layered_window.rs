use crate::native_panel_core::PanelRect;

use super::dpi::WindowsDpiScale;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowsLayeredWindowCompositionMode {
    PerPixelAlpha,
    GdiColorKeyFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsLayeredBitmapSize {
    pub(super) width: i32,
    pub(super) height: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WindowsLayeredAlphaBitmap {
    size: WindowsLayeredBitmapSize,
    pixels: Vec<u8>,
}

impl WindowsLayeredBitmapSize {
    pub(super) fn from_logical_frame(frame: PanelRect, dpi_scale: WindowsDpiScale) -> Self {
        let physical = dpi_scale.rect_to_physical(frame);
        Self {
            width: physical.width.max(1),
            height: physical.height.max(1),
        }
    }

    pub(super) fn stride(self) -> usize {
        self.width.max(1) as usize * 4
    }

    pub(super) fn byte_len(self) -> usize {
        self.stride() * self.height.max(1) as usize
    }
}

impl WindowsLayeredAlphaBitmap {
    pub(super) fn new(size: WindowsLayeredBitmapSize) -> Self {
        Self {
            size,
            pixels: vec![0; size.byte_len()],
        }
    }

    pub(super) fn resize_for_frame(
        &mut self,
        frame: PanelRect,
        dpi_scale: WindowsDpiScale,
    ) -> bool {
        let next_size = WindowsLayeredBitmapSize::from_logical_frame(frame, dpi_scale);
        if next_size == self.size {
            return false;
        }
        self.size = next_size;
        self.pixels.resize(next_size.byte_len(), 0);
        true
    }

    pub(super) fn clear_transparent(&mut self) {
        self.pixels.fill(0);
    }

    pub(super) fn size(&self) -> WindowsLayeredBitmapSize {
        self.size
    }

    pub(super) fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

pub(super) fn windows_layered_window_composition_mode_for_painter(
    per_pixel_alpha_ready: bool,
) -> WindowsLayeredWindowCompositionMode {
    if per_pixel_alpha_ready {
        WindowsLayeredWindowCompositionMode::PerPixelAlpha
    } else {
        WindowsLayeredWindowCompositionMode::GdiColorKeyFallback
    }
}

#[cfg(all(windows, not(test)))]
pub(super) fn apply_windows_layered_window_initial_attributes(
    hwnd: windows_sys::Win32::Foundation::HWND,
    mode: WindowsLayeredWindowCompositionMode,
    transparent_color_key: u32,
) -> Result<(), String> {
    match mode {
        WindowsLayeredWindowCompositionMode::PerPixelAlpha => Ok(()),
        WindowsLayeredWindowCompositionMode::GdiColorKeyFallback => {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                LWA_ALPHA, LWA_COLORKEY, SetLayeredWindowAttributes,
            };

            unsafe {
                let _ = SetLayeredWindowAttributes(
                    hwnd,
                    transparent_color_key,
                    255,
                    LWA_ALPHA | LWA_COLORKEY,
                );
            }
            Ok(())
        }
    }
}

#[cfg(any(not(windows), test))]
pub(super) fn apply_windows_layered_window_initial_attributes(
    _hwnd: isize,
    _mode: WindowsLayeredWindowCompositionMode,
    _transparent_color_key: u32,
) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        WindowsLayeredAlphaBitmap, WindowsLayeredBitmapSize, WindowsLayeredWindowCompositionMode,
        windows_layered_window_composition_mode_for_painter,
    };
    use crate::{native_panel_core::PanelRect, windows_native_panel::dpi::WindowsDpiScale};

    #[test]
    fn layered_bitmap_size_uses_dpi_physical_pixels() {
        let size = WindowsLayeredBitmapSize::from_logical_frame(
            PanelRect {
                x: 100.0,
                y: 20.0,
                width: 253.0,
                height: 80.0,
            },
            WindowsDpiScale::from_scale(1.25),
        );

        assert_eq!(
            size,
            WindowsLayeredBitmapSize {
                width: 316,
                height: 100,
            }
        );
        assert_eq!(size.stride(), 1264);
        assert_eq!(size.byte_len(), 126_400);
    }

    #[test]
    fn layered_bitmap_clamps_empty_sizes_to_one_pixel() {
        let size = WindowsLayeredBitmapSize::from_logical_frame(
            PanelRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            WindowsDpiScale::from_scale(1.0),
        );

        assert_eq!(
            size,
            WindowsLayeredBitmapSize {
                width: 1,
                height: 1,
            }
        );
        assert_eq!(size.byte_len(), 4);
    }

    #[test]
    fn layered_bitmap_resizes_only_when_physical_size_changes() {
        let mut bitmap = WindowsLayeredAlphaBitmap::new(WindowsLayeredBitmapSize {
            width: 253,
            height: 80,
        });

        assert!(!bitmap.resize_for_frame(
            PanelRect {
                x: 10.0,
                y: 20.0,
                width: 253.0,
                height: 80.0,
            },
            WindowsDpiScale::from_scale(1.0),
        ));
        assert!(bitmap.resize_for_frame(
            PanelRect {
                x: 10.0,
                y: 20.0,
                width: 253.0,
                height: 80.0,
            },
            WindowsDpiScale::from_scale(1.5),
        ));
        assert_eq!(
            bitmap.size(),
            WindowsLayeredBitmapSize {
                width: 380,
                height: 120,
            }
        );
        assert_eq!(bitmap.pixels().len(), 182_400);
    }

    #[test]
    fn layered_bitmap_clear_transparent_zeros_bgra_pixels() {
        let mut bitmap = WindowsLayeredAlphaBitmap::new(WindowsLayeredBitmapSize {
            width: 2,
            height: 2,
        });
        bitmap.pixels.fill(255);

        bitmap.clear_transparent();

        assert!(bitmap.pixels().iter().all(|value| *value == 0));
    }

    #[test]
    fn layered_window_uses_color_key_fallback_until_alpha_painter_is_ready() {
        assert_eq!(
            windows_layered_window_composition_mode_for_painter(false),
            WindowsLayeredWindowCompositionMode::GdiColorKeyFallback
        );
        assert_eq!(
            windows_layered_window_composition_mode_for_painter(true),
            WindowsLayeredWindowCompositionMode::PerPixelAlpha
        );
    }
}
