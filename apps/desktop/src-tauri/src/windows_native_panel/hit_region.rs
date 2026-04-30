use crate::{
    native_panel_core::PanelPoint,
    native_panel_renderer::facade::descriptor::{
        NativePanelInteractionPlan, NativePanelPointerRegion,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowsNativePanelHitTest {
    Client,
    Transparent,
}

pub(super) fn resolve_windows_native_panel_hit_test(
    regions: &[NativePanelPointerRegion],
    point: PanelPoint,
) -> WindowsNativePanelHitTest {
    if NativePanelInteractionPlan::from_pointer_regions(regions).inside_regions(point) {
        WindowsNativePanelHitTest::Client
    } else {
        WindowsNativePanelHitTest::Transparent
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        native_panel_core::PanelRect,
        native_panel_renderer::facade::descriptor::{
            NativePanelPointerRegion, NativePanelPointerRegionKind,
        },
    };

    use super::{WindowsNativePanelHitTest, resolve_windows_native_panel_hit_test};

    #[test]
    fn hit_test_accepts_points_inside_shared_pointer_regions() {
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 10.0,
                width: 120.0,
                height: 36.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        assert_eq!(
            resolve_windows_native_panel_hit_test(
                &regions,
                crate::native_panel_core::PanelPoint { x: 40.0, y: 20.0 }
            ),
            WindowsNativePanelHitTest::Client
        );
    }

    #[test]
    fn hit_test_passes_through_transparent_margins() {
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 10.0,
                width: 120.0,
                height: 36.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        assert_eq!(
            resolve_windows_native_panel_hit_test(
                &regions,
                crate::native_panel_core::PanelPoint { x: 4.0, y: 4.0 }
            ),
            WindowsNativePanelHitTest::Transparent
        );
    }
}
