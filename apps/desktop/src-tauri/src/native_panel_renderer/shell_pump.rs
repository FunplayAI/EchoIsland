use super::shell_command::NativePanelHostShellCommand;

pub(crate) trait NativePanelHostShellRuntimePump {
    type RawWindowHandle: Copy + PartialEq;
    type Error;

    fn has_pending_shell_destroy_command(&self) -> bool;

    fn consume_presenter_into_shell_for_pump(&mut self);

    fn raw_shell_window_handle(&self) -> Option<Self::RawWindowHandle>;

    fn take_pending_shell_commands_for_pump(&mut self) -> Vec<NativePanelHostShellCommand>;

    fn apply_shell_command_for_pump(
        &mut self,
        raw_window_handle: &mut Option<Self::RawWindowHandle>,
        command: NativePanelHostShellCommand,
    ) -> Result<(), Self::Error>;

    fn sync_raw_shell_window_handle(&mut self, raw_window_handle: Option<Self::RawWindowHandle>);

    fn pump_platform_window_messages(&mut self) -> Result<(), Self::Error>;
}

pub(crate) fn pump_native_panel_host_shell_runtime<R>(runtime: &mut R) -> Result<(), R::Error>
where
    R: NativePanelHostShellRuntimePump,
{
    if !runtime.has_pending_shell_destroy_command() {
        runtime.consume_presenter_into_shell_for_pump();
    }

    let mut raw_window_handle = runtime.raw_shell_window_handle();
    for command in runtime.take_pending_shell_commands_for_pump() {
        runtime.apply_shell_command_for_pump(&mut raw_window_handle, command)?;
    }

    if runtime.raw_shell_window_handle() != raw_window_handle {
        runtime.sync_raw_shell_window_handle(raw_window_handle);
    }

    runtime.pump_platform_window_messages()
}

#[cfg(test)]
mod tests {
    use super::{NativePanelHostShellRuntimePump, pump_native_panel_host_shell_runtime};
    use crate::native_panel_renderer::shell_command::NativePanelHostShellCommand;

    #[derive(Default)]
    struct PumpRuntime {
        pending_destroy: bool,
        presenter_consumed: bool,
        raw_window_handle: Option<i32>,
        pending_commands: Vec<NativePanelHostShellCommand>,
        applied_commands: Vec<NativePanelHostShellCommand>,
        sync_count: usize,
        message_pump_count: usize,
    }

    impl NativePanelHostShellRuntimePump for PumpRuntime {
        type RawWindowHandle = i32;
        type Error = String;

        fn has_pending_shell_destroy_command(&self) -> bool {
            self.pending_destroy
        }

        fn consume_presenter_into_shell_for_pump(&mut self) {
            self.presenter_consumed = true;
        }

        fn raw_shell_window_handle(&self) -> Option<Self::RawWindowHandle> {
            self.raw_window_handle
        }

        fn take_pending_shell_commands_for_pump(&mut self) -> Vec<NativePanelHostShellCommand> {
            std::mem::take(&mut self.pending_commands)
        }

        fn apply_shell_command_for_pump(
            &mut self,
            raw_window_handle: &mut Option<Self::RawWindowHandle>,
            command: NativePanelHostShellCommand,
        ) -> Result<(), Self::Error> {
            if command == NativePanelHostShellCommand::Create {
                *raw_window_handle = Some(99);
            }
            if command == NativePanelHostShellCommand::Destroy {
                *raw_window_handle = None;
            }
            self.applied_commands.push(command);
            Ok(())
        }

        fn sync_raw_shell_window_handle(
            &mut self,
            raw_window_handle: Option<Self::RawWindowHandle>,
        ) {
            self.raw_window_handle = raw_window_handle;
            self.sync_count += 1;
        }

        fn pump_platform_window_messages(&mut self) -> Result<(), Self::Error> {
            self.message_pump_count += 1;
            Ok(())
        }
    }

    #[test]
    fn shell_runtime_pump_consumes_presenter_applies_commands_and_syncs_handle() {
        let mut runtime = PumpRuntime {
            pending_commands: vec![NativePanelHostShellCommand::Create],
            ..PumpRuntime::default()
        };

        pump_native_panel_host_shell_runtime(&mut runtime).expect("pump shell runtime");

        assert!(runtime.presenter_consumed);
        assert_eq!(
            runtime.applied_commands,
            vec![NativePanelHostShellCommand::Create]
        );
        assert_eq!(runtime.raw_window_handle, Some(99));
        assert_eq!(runtime.sync_count, 1);
        assert_eq!(runtime.message_pump_count, 1);
    }

    #[test]
    fn shell_runtime_pump_skips_presenter_when_destroy_is_pending() {
        let mut runtime = PumpRuntime {
            pending_destroy: true,
            raw_window_handle: Some(99),
            pending_commands: vec![NativePanelHostShellCommand::Destroy],
            ..PumpRuntime::default()
        };

        pump_native_panel_host_shell_runtime(&mut runtime).expect("pump shell runtime");

        assert!(!runtime.presenter_consumed);
        assert_eq!(
            runtime.applied_commands,
            vec![NativePanelHostShellCommand::Destroy]
        );
        assert_eq!(runtime.raw_window_handle, None);
        assert_eq!(runtime.sync_count, 1);
        assert_eq!(runtime.message_pump_count, 1);
    }
}
