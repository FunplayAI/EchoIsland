use super::host_window::WindowsNativePanelDrawFrame;

#[derive(Clone, Debug, Default)]
pub(super) struct WindowsNativePanelDrawPresenter {
    last_frame: Option<WindowsNativePanelDrawFrame>,
    redraw_requested: bool,
}

impl WindowsNativePanelDrawPresenter {
    pub(super) fn present(&mut self, frame: WindowsNativePanelDrawFrame) {
        self.last_frame = Some(frame);
        self.redraw_requested = true;
    }

    pub(super) fn redraw_requested(&self) -> bool {
        self.redraw_requested
    }

    pub(super) fn last_frame(&self) -> Option<&WindowsNativePanelDrawFrame> {
        self.last_frame.as_ref()
    }

    pub(super) fn take_redraw_frame(&mut self) -> Option<WindowsNativePanelDrawFrame> {
        if !self.redraw_requested {
            return None;
        }
        self.redraw_requested = false;
        self.last_frame.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsNativePanelDrawPresenter;
    use crate::{
        native_panel_core::PanelRect,
        native_panel_renderer::facade::descriptor::{
            NativePanelHostWindowState, NativePanelPointerRegion,
        },
        windows_native_panel::host_window::WindowsNativePanelDrawFrame,
    };

    #[test]
    fn presenter_caches_frame_and_invalidates_once() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let frame = WindowsNativePanelDrawFrame {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 300.0,
                    height: 120.0,
                }),
                visible: true,
                preferred_display_index: 1,
            },
            pointer_regions: Vec::<NativePanelPointerRegion>::new(),
            presentation_model: None,
        };

        presenter.present(frame.clone());

        assert!(presenter.redraw_requested());
        assert_eq!(
            presenter
                .last_frame()
                .and_then(|frame| frame.window_state.frame),
            frame.window_state.frame
        );
        assert_eq!(
            presenter
                .take_redraw_frame()
                .and_then(|frame| frame.window_state.frame),
            frame.window_state.frame
        );
        assert!(!presenter.redraw_requested());
        assert!(presenter.take_redraw_frame().is_none());
    }
}
