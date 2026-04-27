use crate::native_panel_renderer::facade::{
    descriptor::{NativePanelHostWindowState, NativePanelPointerRegion},
    presentation::NativePanelPresentationModel,
};

use super::{
    draw_presenter::WindowsNativePanelDrawPresenter,
    host_window::{WindowsNativePanelDrawFrame, WindowsNativePanelHostWindow},
    window_shell::{WindowsNativePanelShellPresentResult, WindowsNativePanelWindowShell},
};

pub(super) fn present_window_into_presenter(
    window: &mut WindowsNativePanelHostWindow,
    presenter: &mut WindowsNativePanelDrawPresenter,
    window_state: NativePanelHostWindowState,
    pointer_regions: &[NativePanelPointerRegion],
    presentation_model: Option<NativePanelPresentationModel>,
) {
    window.present(window_state, pointer_regions, presentation_model);
    sync_draw_presenter(window, presenter);
}

pub(super) fn sync_draw_presenter(
    window: &mut WindowsNativePanelHostWindow,
    presenter: &mut WindowsNativePanelDrawPresenter,
) {
    if let Some(frame) = window.take_pending_draw_frame() {
        presenter.present(frame);
    }
}

pub(super) fn take_pending_draw_frame(
    presenter: &mut WindowsNativePanelDrawPresenter,
) -> Option<WindowsNativePanelDrawFrame> {
    presenter.take_redraw_frame()
}

pub(super) fn consume_presenter_into_shell(
    presenter: &mut WindowsNativePanelDrawPresenter,
    shell: &mut WindowsNativePanelWindowShell,
) -> bool {
    consume_presenter_into_shell_result(presenter, shell).redraw_requested
}

pub(super) fn consume_presenter_into_shell_result(
    presenter: &mut WindowsNativePanelDrawPresenter,
    shell: &mut WindowsNativePanelWindowShell,
) -> WindowsNativePanelShellPresentResult {
    shell.consume_presenter(presenter)
}
