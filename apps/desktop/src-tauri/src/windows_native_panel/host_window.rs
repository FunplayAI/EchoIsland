use crate::{
    native_panel_core::{PanelAnimationDescriptor, PanelRect, resolve_native_panel_host_frame},
    native_panel_renderer::facade::{
        descriptor::{
            NativePanelComputedHostWindow, NativePanelHostWindowDescriptor,
            NativePanelHostWindowState, NativePanelPointerRegion, NativePanelTimelineDescriptor,
        },
        presentation::NativePanelPresentationModel,
    },
};

const FALLBACK_SCREEN_FRAME: PanelRect = PanelRect {
    x: 0.0,
    y: 0.0,
    width: 1440.0,
    height: 900.0,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum WindowsNativePanelWindowLifecycle {
    #[default]
    NotCreated,
    Created,
}

#[derive(Clone, Debug, Default)]
pub(super) struct WindowsNativePanelHostWindow {
    pub(super) lifecycle: WindowsNativePanelWindowLifecycle,
    pub(super) descriptor: NativePanelHostWindowDescriptor,
    pub(super) last_frame: Option<PanelRect>,
    pub(super) presented_window_state: Option<NativePanelHostWindowState>,
    pub(super) presented_pointer_regions: Vec<NativePanelPointerRegion>,
    pub(super) presented_presentation_model: Option<NativePanelPresentationModel>,
    pub(super) pending_redraw: bool,
}

#[derive(Clone, Debug)]
pub(super) struct WindowsNativePanelDrawFrame {
    pub(super) window_state: NativePanelHostWindowState,
    pub(super) pointer_regions: Vec<NativePanelPointerRegion>,
    pub(super) presentation_model: Option<NativePanelPresentationModel>,
}

impl WindowsNativePanelHostWindow {
    pub(super) fn create(&mut self) {
        self.host_window_create();
    }

    pub(super) fn show(&mut self) {
        self.host_window_show();
    }

    pub(super) fn hide(&mut self) {
        self.host_window_hide();
    }

    pub(super) fn reposition_to_display(
        &mut self,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) {
        self.host_window_reposition_to_display(preferred_display_index, screen_frame);
    }

    pub(super) fn set_shared_body_height(&mut self, body_height: f64) {
        self.host_window_set_shared_body_height(body_height);
    }

    pub(super) fn apply_timeline_descriptor(&mut self, descriptor: NativePanelTimelineDescriptor) {
        self.host_window_apply_timeline_descriptor(descriptor);
    }

    pub(super) fn refresh_frame_from_descriptor(&mut self) {
        self.refresh_host_window_frame_from_descriptor();
    }

    pub(super) fn window_state(&self) -> NativePanelHostWindowState {
        self.computed_host_window_state()
    }

    pub(super) fn present(
        &mut self,
        window_state: NativePanelHostWindowState,
        pointer_regions: &[NativePanelPointerRegion],
        presentation_model: Option<NativePanelPresentationModel>,
    ) {
        self.presented_window_state = Some(window_state);
        self.presented_pointer_regions = pointer_regions.to_vec();
        self.presented_presentation_model = presentation_model;
        self.pending_redraw = true;
    }

    pub(super) fn take_pending_draw_frame(&mut self) -> Option<WindowsNativePanelDrawFrame> {
        if !self.pending_redraw {
            return None;
        }
        self.pending_redraw = false;
        self.presented_window_state
            .map(|window_state| WindowsNativePanelDrawFrame {
                window_state,
                pointer_regions: self.presented_pointer_regions.clone(),
                presentation_model: self.presented_presentation_model.clone(),
            })
    }

    pub(super) fn pointer_regions<'a>(
        &'a self,
        fallback: &'a [NativePanelPointerRegion],
    ) -> &'a [NativePanelPointerRegion] {
        if self.presented_pointer_regions.is_empty() {
            fallback
        } else {
            &self.presented_pointer_regions
        }
    }
}

impl NativePanelComputedHostWindow for WindowsNativePanelHostWindow {
    fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
        self.descriptor
    }

    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
        &mut self.descriptor
    }

    fn host_window_last_frame_mut(&mut self) -> &mut Option<PanelRect> {
        &mut self.last_frame
    }

    fn host_window_frame(&self) -> Option<PanelRect> {
        self.last_frame
    }

    fn set_host_window_created(&mut self) {
        self.lifecycle = WindowsNativePanelWindowLifecycle::Created;
    }

    fn fallback_screen_frame(&self) -> PanelRect {
        FALLBACK_SCREEN_FRAME
    }

    fn compact_width(&self) -> f64 {
        crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH
    }

    fn expanded_width(&self) -> f64 {
        crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH
    }
}

pub(super) fn resolve_windows_panel_window_frame(
    descriptor: PanelAnimationDescriptor,
    screen_frame: PanelRect,
    compact_width: f64,
    expanded_width: f64,
) -> PanelRect {
    resolve_native_panel_host_frame(descriptor, screen_frame, compact_width, expanded_width)
}

#[cfg(test)]
mod tests {
    use super::WindowsNativePanelHostWindow;
    use crate::{
        native_panel_core::{PanelAnimationKind, PanelRect},
        native_panel_renderer::facade::descriptor::{
            NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPointerRegion,
            NativePanelPointerRegionKind, NativePanelTimelineDescriptor,
        },
    };

    #[test]
    fn refresh_frame_uses_shared_host_window_descriptor_helper() {
        let mut host = WindowsNativePanelHostWindow {
            descriptor: NativePanelHostWindowDescriptor {
                visible: true,
                preferred_display_index: 0,
                screen_frame: Some(PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1440.0,
                    height: 900.0,
                }),
                shared_body_height: None,
                timeline: Some(NativePanelTimelineDescriptor {
                    animation: crate::native_panel_core::PanelAnimationDescriptor {
                        kind: PanelAnimationKind::Open,
                        canvas_height: 180.2,
                        visible_height: 140.0,
                        width_progress: 0.5,
                        height_progress: 0.0,
                        shoulder_progress: 0.0,
                        drop_progress: 0.0,
                        cards_progress: 0.0,
                    },
                    cards_entering: true,
                }),
            },
            ..Default::default()
        };

        host.refresh_frame_from_descriptor();

        assert_eq!(
            host.last_frame,
            Some(PanelRect {
                x: 586.0,
                y: 720.0,
                width: 268.0,
                height: 180.0,
            })
        );
    }

    #[test]
    fn host_window_presents_pointer_regions_and_draw_invalidates_once() {
        let mut host = WindowsNativePanelHostWindow::default();
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 12.0,
                y: 16.0,
                width: 48.0,
                height: 24.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }];

        host.present(Default::default(), &regions, None);

        assert_eq!(host.pointer_regions(&[]), regions.as_slice());
        let frame = host.take_pending_draw_frame().expect("pending draw frame");
        assert_eq!(frame.window_state, NativePanelHostWindowState::default());
        assert_eq!(frame.pointer_regions, regions);
        assert!(frame.presentation_model.is_none());
        assert!(host.take_pending_draw_frame().is_none());
    }
}
