use crate::{
    native_panel_core::{PanelPoint, PanelRect},
    native_panel_renderer::facade::descriptor::NativePanelPointerRegion,
};

const WINDOWS_BASE_DPI: u32 = 96;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowsDpiScale {
    pub(super) scale: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsPhysicalRect {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) width: i32,
    pub(super) height: i32,
}

impl Default for WindowsDpiScale {
    fn default() -> Self {
        Self::from_scale(1.0)
    }
}

impl WindowsDpiScale {
    pub(super) fn from_scale(scale: f64) -> Self {
        if scale.is_finite() && scale > 0.0 {
            Self { scale }
        } else {
            Self { scale: 1.0 }
        }
    }

    pub(super) fn from_dpi(dpi: u32) -> Self {
        if dpi == 0 {
            return Self::default();
        }
        Self::from_scale(dpi as f64 / WINDOWS_BASE_DPI as f64)
    }

    pub(super) fn logical_to_physical(self, value: f64) -> i32 {
        (value * self.scale).round() as i32
    }

    pub(super) fn physical_to_logical(self, value: i32) -> f64 {
        value as f64 / self.scale
    }

    pub(super) fn point_to_logical(self, x: i32, y: i32) -> PanelPoint {
        PanelPoint {
            x: self.physical_to_logical(x),
            y: self.physical_to_logical(y),
        }
    }

    pub(super) fn rect_to_physical(self, rect: PanelRect) -> WindowsPhysicalRect {
        WindowsPhysicalRect {
            x: self.logical_to_physical(rect.x),
            y: self.logical_to_physical(rect.y),
            width: self.logical_to_physical(rect.width),
            height: self.logical_to_physical(rect.height),
        }
    }

    pub(super) fn pointer_region_to_physical(
        self,
        region: &NativePanelPointerRegion,
    ) -> WindowsPhysicalRect {
        self.rect_to_physical(region.frame)
    }
}

#[cfg(all(windows, not(test)))]
pub(super) fn resolve_windows_dpi_scale_for_window(
    raw_window_handle: Option<isize>,
) -> WindowsDpiScale {
    use windows::Win32::{Foundation::HWND, UI::HiDpi::GetDpiForWindow};

    let Some(hwnd) = raw_window_handle else {
        return WindowsDpiScale::default();
    };
    let dpi = unsafe { GetDpiForWindow(HWND(hwnd as _)) };
    WindowsDpiScale::from_dpi(dpi)
}

#[cfg(any(not(windows), test))]
pub(super) fn resolve_windows_dpi_scale_for_window(
    _raw_window_handle: Option<isize>,
) -> WindowsDpiScale {
    WindowsDpiScale::default()
}

#[cfg(test)]
mod tests {
    use super::{WindowsDpiScale, WindowsPhysicalRect, resolve_windows_dpi_scale_for_window};
    use crate::{
        native_panel_core::PanelRect,
        native_panel_renderer::facade::descriptor::{
            NativePanelPointerRegion, NativePanelPointerRegionKind,
        },
    };

    #[test]
    fn dpi_scale_maps_logical_rect_at_100_percent() {
        let scale = WindowsDpiScale::from_scale(1.0);

        assert_eq!(
            scale.rect_to_physical(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 253.0,
                height: 80.0,
            }),
            WindowsPhysicalRect {
                x: 0,
                y: 0,
                width: 253,
                height: 80,
            }
        );
    }

    #[test]
    fn dpi_scale_rounds_logical_rect_at_125_percent() {
        let scale = WindowsDpiScale::from_scale(1.25);

        assert_eq!(
            scale.rect_to_physical(PanelRect {
                x: 10.0,
                y: 20.0,
                width: 253.0,
                height: 80.0,
            }),
            WindowsPhysicalRect {
                x: 13,
                y: 25,
                width: 316,
                height: 100,
            }
        );
    }

    #[test]
    fn dpi_scale_maps_physical_point_back_to_logical() {
        let scale = WindowsDpiScale::from_scale(1.25);

        assert_eq!(
            scale.point_to_logical(150, 75),
            crate::native_panel_core::PanelPoint { x: 120.0, y: 60.0 }
        );
    }

    #[test]
    fn dpi_scale_uses_same_conversion_for_window_and_hit_regions() {
        let scale = WindowsDpiScale::from_scale(1.5);
        let frame = PanelRect {
            x: 20.0,
            y: 8.0,
            width: 265.0,
            height: 80.0,
        };
        let region = NativePanelPointerRegion {
            frame,
            kind: NativePanelPointerRegionKind::CompactBar,
        };

        assert_eq!(
            scale.pointer_region_to_physical(&region),
            scale.rect_to_physical(frame)
        );
    }

    #[test]
    fn dpi_scale_preserves_negative_monitor_origins() {
        let scale = WindowsDpiScale::from_scale(1.25);

        assert_eq!(
            scale.rect_to_physical(PanelRect {
                x: -1280.0,
                y: -16.0,
                width: 253.0,
                height: 80.0,
            }),
            WindowsPhysicalRect {
                x: -1600,
                y: -20,
                width: 316,
                height: 100,
            }
        );
    }

    #[test]
    fn dpi_scale_from_window_defaults_to_100_percent_in_tests() {
        assert_eq!(
            resolve_windows_dpi_scale_for_window(Some(1)),
            WindowsDpiScale::from_scale(1.0)
        );
    }
}
