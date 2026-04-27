use objc2_foundation::NSRect;

use crate::native_panel_renderer::facade::descriptor::{
    NativePanelHostWindowDescriptor, NativePanelTimelineDescriptor,
    native_panel_host_window_descriptor as shared_native_panel_host_window_descriptor,
};

pub(super) fn native_panel_host_window_descriptor(
    visible: bool,
    preferred_display_index: usize,
    screen_frame: Option<NSRect>,
    shared_body_height: Option<f64>,
    timeline: Option<NativePanelTimelineDescriptor>,
) -> NativePanelHostWindowDescriptor {
    shared_native_panel_host_window_descriptor(
        visible,
        preferred_display_index,
        screen_frame.map(ns_rect_to_panel_rect),
        shared_body_height,
        timeline,
    )
}

fn ns_rect_to_panel_rect(rect: NSRect) -> crate::native_panel_core::PanelRect {
    crate::native_panel_core::PanelRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

#[cfg(test)]
mod tests {
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::native_panel_host_window_descriptor;
    use crate::native_panel_renderer::facade::descriptor::NativePanelTimelineDescriptor;

    #[test]
    fn host_window_descriptor_maps_ns_screen_frame() {
        let descriptor = native_panel_host_window_descriptor(
            true,
            1,
            Some(NSRect::new(
                NSPoint::new(10.0, 20.0),
                NSSize::new(300.0, 200.0),
            )),
            Some(180.0),
            Some(NativePanelTimelineDescriptor {
                animation: crate::native_panel_core::PanelAnimationDescriptor {
                    kind: crate::native_panel_core::PanelAnimationKind::Open,
                    canvas_height: 160.0,
                    visible_height: 120.0,
                    width_progress: 0.5,
                    height_progress: 0.75,
                    shoulder_progress: 1.0,
                    drop_progress: 0.25,
                    cards_progress: 0.8,
                },
                cards_entering: true,
            }),
        );

        assert!(descriptor.visible);
        assert_eq!(descriptor.preferred_display_index, 1);
        assert_eq!(descriptor.shared_body_height, Some(180.0));
        assert_eq!(
            descriptor.screen_frame,
            Some(crate::native_panel_core::PanelRect {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            })
        );
        assert!(descriptor.timeline.is_some());
    }
}
