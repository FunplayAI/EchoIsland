use crate::{
    native_panel_core::{
        PanelAnimationDescriptor, PanelPoint, PanelRect, resolve_native_panel_host_frame,
    },
    native_panel_renderer::{
        NativePanelHostWindowDescriptor, NativePanelHostWindowDescriptorPatch,
        NativePanelHostWindowState, NativePanelPointerInput, NativePanelTimelineDescriptor,
        native_panel_host_window_frame, patch_native_panel_host_window_descriptor,
    },
};

const FALLBACK_SCREEN_FRAME: PanelRect = PanelRect {
    x: 0.0,
    y: 0.0,
    width: 1440.0,
    height: 900.0,
};

pub(super) const WINDOWS_WM_MOUSEMOVE: u32 = 0x0200;
pub(super) const WINDOWS_WM_LBUTTONUP: u32 = 0x0202;
pub(super) const WINDOWS_WM_MOUSELEAVE: u32 = 0x02A3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum WindowsNativePanelWindowLifecycle {
    #[default]
    NotCreated,
    Created,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WindowsNativePanelWindowHandle {
    pub(super) hwnd: Option<isize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct WindowsNativePanelHostWindow {
    pub(super) lifecycle: WindowsNativePanelWindowLifecycle,
    pub(super) handle: WindowsNativePanelWindowHandle,
    pub(super) descriptor: NativePanelHostWindowDescriptor,
    pub(super) last_frame: Option<PanelRect>,
}

impl WindowsNativePanelHostWindow {
    pub(super) fn create(&mut self) {
        self.lifecycle = WindowsNativePanelWindowLifecycle::Created;
    }

    pub(super) fn show(&mut self) {
        self.create();
        patch_native_panel_host_window_descriptor(
            &mut self.descriptor,
            NativePanelHostWindowDescriptorPatch {
                visible: Some(true),
                ..NativePanelHostWindowDescriptorPatch::default()
            },
        );
    }

    pub(super) fn hide(&mut self) {
        patch_native_panel_host_window_descriptor(
            &mut self.descriptor,
            NativePanelHostWindowDescriptorPatch {
                visible: Some(false),
                ..NativePanelHostWindowDescriptorPatch::default()
            },
        );
    }

    pub(super) fn reposition_to_display(
        &mut self,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) {
        self.create();
        patch_native_panel_host_window_descriptor(
            &mut self.descriptor,
            NativePanelHostWindowDescriptorPatch {
                preferred_display_index: Some(preferred_display_index),
                screen_frame: Some(screen_frame),
                ..NativePanelHostWindowDescriptorPatch::default()
            },
        );
        self.refresh_frame();
    }

    pub(super) fn set_shared_body_height(&mut self, body_height: f64) {
        patch_native_panel_host_window_descriptor(
            &mut self.descriptor,
            NativePanelHostWindowDescriptorPatch {
                shared_body_height: Some(Some(body_height)),
                ..NativePanelHostWindowDescriptorPatch::default()
            },
        );
    }

    pub(super) fn apply_timeline_descriptor(&mut self, descriptor: NativePanelTimelineDescriptor) {
        self.create();
        patch_native_panel_host_window_descriptor(
            &mut self.descriptor,
            NativePanelHostWindowDescriptorPatch {
                timeline: Some(Some(descriptor)),
                ..NativePanelHostWindowDescriptorPatch::default()
            },
        );
        self.refresh_frame();
    }

    pub(super) fn refresh_frame_from_descriptor(&mut self) {
        self.refresh_frame();
    }

    fn refresh_frame(&mut self) {
        if self.descriptor.animation_descriptor().is_none() {
            return;
        }
        self.last_frame = native_panel_host_window_frame(
            self.descriptor,
            self.descriptor
                .screen_frame
                .unwrap_or(FALLBACK_SCREEN_FRAME),
            crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH,
            crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
        );
    }

    pub(super) fn window_state(&self) -> NativePanelHostWindowState {
        self.descriptor.window_state(self.last_frame)
    }
}

pub(super) fn decode_windows_native_panel_window_message(
    message_id: u32,
    lparam: isize,
) -> Option<NativePanelPointerInput> {
    match message_id {
        WINDOWS_WM_MOUSEMOVE => Some(NativePanelPointerInput::Move(
            panel_point_from_window_lparam(lparam),
        )),
        WINDOWS_WM_LBUTTONUP => Some(NativePanelPointerInput::Click(
            panel_point_from_window_lparam(lparam),
        )),
        WINDOWS_WM_MOUSELEAVE => Some(NativePanelPointerInput::Leave),
        _ => None,
    }
}

fn panel_point_from_window_lparam(lparam: isize) -> PanelPoint {
    let x = (lparam as u32 & 0xFFFF) as u16 as i16 as f64;
    let y = ((lparam as u32 >> 16) & 0xFFFF) as u16 as i16 as f64;
    PanelPoint { x, y }
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
    use super::{
        WINDOWS_WM_LBUTTONUP, WINDOWS_WM_MOUSELEAVE, WINDOWS_WM_MOUSEMOVE,
        decode_windows_native_panel_window_message,
    };
    use crate::{
        native_panel_core::{PanelAnimationKind, PanelPoint, PanelRect},
        native_panel_renderer::{
            NativePanelHostWindowDescriptor, NativePanelPointerInput, NativePanelTimelineDescriptor,
        },
    };

    #[test]
    fn decodes_pointer_move_message_from_lparam() {
        let message =
            decode_windows_native_panel_window_message(WINDOWS_WM_MOUSEMOVE, 0x001E_000Aisize);

        assert_eq!(
            message,
            Some(NativePanelPointerInput::Move(PanelPoint {
                x: 10.0,
                y: 30.0,
            }))
        );
    }

    #[test]
    fn decodes_pointer_click_message_from_signed_lparam() {
        let message = decode_windows_native_panel_window_message(
            WINDOWS_WM_LBUTTONUP,
            0xFFEC_FFF6u32 as isize,
        );

        assert_eq!(
            message,
            Some(NativePanelPointerInput::Click(PanelPoint {
                x: -10.0,
                y: -20.0,
            }))
        );
    }

    #[test]
    fn decodes_pointer_leave_message() {
        let message =
            decode_windows_native_panel_window_message(WINDOWS_WM_MOUSELEAVE, 0x0000_0000isize);

        assert_eq!(message, Some(NativePanelPointerInput::Leave));
    }

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
}
