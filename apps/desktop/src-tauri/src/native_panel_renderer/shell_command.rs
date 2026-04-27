use super::descriptors::NativePanelHostWindowState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativePanelHostShellCommand {
    Create,
    Show,
    Hide,
    Destroy,
    SyncWindowState(NativePanelHostWindowState),
    SyncMouseEventPassthrough(bool),
    RequestRedraw,
}

pub(crate) trait NativePanelHostShellCommandBackend {
    type RawWindowHandle: Copy;
    type Error;

    fn create_shell_window(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
    ) -> Result<Option<Self::RawWindowHandle>, Self::Error>;

    fn destroy_shell_window(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
    ) -> Result<Option<Self::RawWindowHandle>, Self::Error>;

    fn set_shell_window_visible(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
        visible: bool,
    ) -> Result<(), Self::Error>;

    fn sync_shell_window_state(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
        window_state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error>;

    fn sync_shell_mouse_event_passthrough(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
        ignores_mouse_events: bool,
    ) -> Result<(), Self::Error>;

    fn request_shell_redraw(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
    ) -> Result<(), Self::Error>;

    fn record_shell_command_applied(&mut self, raw_window_handle: Option<Self::RawWindowHandle>);
}

pub(crate) fn apply_native_panel_host_shell_command<B>(
    backend: &mut B,
    raw_window_handle: &mut Option<B::RawWindowHandle>,
    command: NativePanelHostShellCommand,
) -> Result<(), B::Error>
where
    B: NativePanelHostShellCommandBackend,
{
    match command {
        NativePanelHostShellCommand::Create => {
            *raw_window_handle = backend.create_shell_window(*raw_window_handle)?;
        }
        NativePanelHostShellCommand::Show => {
            backend.set_shell_window_visible(*raw_window_handle, true)?;
        }
        NativePanelHostShellCommand::Hide => {
            backend.set_shell_window_visible(*raw_window_handle, false)?;
        }
        NativePanelHostShellCommand::Destroy => {
            *raw_window_handle = backend.destroy_shell_window(*raw_window_handle)?;
        }
        NativePanelHostShellCommand::SyncWindowState(window_state) => {
            backend.sync_shell_window_state(*raw_window_handle, window_state)?;
        }
        NativePanelHostShellCommand::SyncMouseEventPassthrough(ignores_mouse_events) => {
            backend.sync_shell_mouse_event_passthrough(*raw_window_handle, ignores_mouse_events)?;
        }
        NativePanelHostShellCommand::RequestRedraw => {
            backend.request_shell_redraw(*raw_window_handle)?;
        }
    }
    backend.record_shell_command_applied(*raw_window_handle);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelHostShellCommand, NativePanelHostShellCommandBackend,
        apply_native_panel_host_shell_command,
    };
    use crate::{
        native_panel_core::PanelRect,
        native_panel_renderer::descriptors::NativePanelHostWindowState,
    };

    fn window_state() -> NativePanelHostWindowState {
        NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 4.0,
                y: 6.0,
                width: 220.0,
                height: 72.0,
            }),
            visible: true,
            preferred_display_index: 1,
        }
    }

    #[derive(Default)]
    struct RecordingBackend {
        applied: usize,
        created: usize,
        destroyed: usize,
        visible: Option<bool>,
        last_raw: Option<i32>,
        last_window_state: Option<NativePanelHostWindowState>,
        last_ignores_mouse_events: Option<bool>,
        redraws: usize,
    }

    impl NativePanelHostShellCommandBackend for RecordingBackend {
        type RawWindowHandle = i32;
        type Error = String;

        fn create_shell_window(
            &mut self,
            raw_window_handle: Option<Self::RawWindowHandle>,
        ) -> Result<Option<Self::RawWindowHandle>, Self::Error> {
            self.created += 1;
            Ok(Some(raw_window_handle.unwrap_or(42)))
        }

        fn destroy_shell_window(
            &mut self,
            _raw_window_handle: Option<Self::RawWindowHandle>,
        ) -> Result<Option<Self::RawWindowHandle>, Self::Error> {
            self.destroyed += 1;
            self.visible = Some(false);
            Ok(None)
        }

        fn set_shell_window_visible(
            &mut self,
            _raw_window_handle: Option<Self::RawWindowHandle>,
            visible: bool,
        ) -> Result<(), Self::Error> {
            self.visible = Some(visible);
            Ok(())
        }

        fn sync_shell_window_state(
            &mut self,
            _raw_window_handle: Option<Self::RawWindowHandle>,
            window_state: NativePanelHostWindowState,
        ) -> Result<(), Self::Error> {
            self.last_window_state = Some(window_state);
            Ok(())
        }

        fn sync_shell_mouse_event_passthrough(
            &mut self,
            _raw_window_handle: Option<Self::RawWindowHandle>,
            ignores_mouse_events: bool,
        ) -> Result<(), Self::Error> {
            self.last_ignores_mouse_events = Some(ignores_mouse_events);
            Ok(())
        }

        fn request_shell_redraw(
            &mut self,
            _raw_window_handle: Option<Self::RawWindowHandle>,
        ) -> Result<(), Self::Error> {
            self.redraws += 1;
            Ok(())
        }

        fn record_shell_command_applied(
            &mut self,
            raw_window_handle: Option<Self::RawWindowHandle>,
        ) {
            self.applied += 1;
            self.last_raw = raw_window_handle;
        }
    }

    #[test]
    fn shell_command_backend_helper_applies_commands_and_tracks_handle() {
        let mut backend = RecordingBackend::default();
        let mut raw_window_handle = None;
        let window_state = window_state();

        for command in [
            NativePanelHostShellCommand::Create,
            NativePanelHostShellCommand::Show,
            NativePanelHostShellCommand::SyncWindowState(window_state),
            NativePanelHostShellCommand::SyncMouseEventPassthrough(true),
            NativePanelHostShellCommand::RequestRedraw,
            NativePanelHostShellCommand::Hide,
            NativePanelHostShellCommand::Destroy,
        ] {
            apply_native_panel_host_shell_command(&mut backend, &mut raw_window_handle, command)
                .expect("apply shell command");
        }

        assert_eq!(backend.applied, 7);
        assert_eq!(backend.created, 1);
        assert_eq!(backend.destroyed, 1);
        assert_eq!(backend.visible, Some(false));
        assert_eq!(backend.last_window_state, Some(window_state));
        assert_eq!(backend.last_ignores_mouse_events, Some(true));
        assert_eq!(backend.redraws, 1);
        assert_eq!(backend.last_raw, None);
        assert_eq!(raw_window_handle, None);
    }
}
