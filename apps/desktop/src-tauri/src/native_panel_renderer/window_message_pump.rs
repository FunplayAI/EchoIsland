use crate::native_panel_core::PanelRect;

use super::descriptors::{
    NativePanelPlatformEvent, NativePanelPointerInputOutcome,
    NativePanelQueuedRuntimeCommandHandler, NativePanelRuntimeCommandHandler,
    NativePanelRuntimeInputDescriptor, native_panel_runtime_input_descriptor_with_screen_frame,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativePanelPlatformWindowMessage {
    pub(crate) message_id: u32,
    pub(crate) lparam: isize,
}

pub(crate) trait NativePanelPlatformWindowMessagePump {
    fn take_platform_window_messages_for_pump(&mut self) -> Vec<NativePanelPlatformWindowMessage>;

    fn platform_window_message_screen_frame(&self) -> Option<PanelRect>;

    fn record_platform_window_message_processed(&mut self, message_id: u32);

    fn is_platform_paint_message(&self, message_id: u32) -> bool;

    fn dispatch_platform_paint_message(&mut self) -> Result<(), String>;

    fn handle_platform_window_message_with_handler(
        &mut self,
        message_id: u32,
        lparam: isize,
        input: &NativePanelRuntimeInputDescriptor,
        handler: &mut impl NativePanelRuntimeCommandHandler<Error = String>,
    ) -> Result<Option<NativePanelPointerInputOutcome>, String>;

    fn pending_platform_events_mut(&mut self) -> &mut Vec<NativePanelPlatformEvent>;
}

pub(crate) fn pump_native_panel_platform_window_messages<R>(runtime: &mut R) -> Result<(), String>
where
    R: NativePanelPlatformWindowMessagePump,
{
    let queued_messages = runtime.take_platform_window_messages_for_pump();
    if queued_messages.is_empty() {
        return Ok(());
    }

    let input = native_panel_runtime_input_descriptor_with_screen_frame(
        runtime.platform_window_message_screen_frame(),
    );
    let mut handler = NativePanelQueuedRuntimeCommandHandler::default();
    for message in queued_messages {
        runtime.record_platform_window_message_processed(message.message_id);
        if runtime.is_platform_paint_message(message.message_id) {
            runtime.dispatch_platform_paint_message()?;
            continue;
        }
        let _ = runtime.handle_platform_window_message_with_handler(
            message.message_id,
            message.lparam,
            &input,
            &mut handler,
        )?;
    }
    runtime
        .pending_platform_events_mut()
        .extend(handler.take_events());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelPlatformWindowMessage, NativePanelPlatformWindowMessagePump,
        pump_native_panel_platform_window_messages,
    };
    use crate::{
        native_panel_core::PanelRect,
        native_panel_renderer::descriptors::{
            NativePanelPlatformEvent, NativePanelPointerInputOutcome,
            NativePanelRuntimeCommandHandler, NativePanelRuntimeInputDescriptor,
        },
    };

    #[derive(Default)]
    struct MessagePumpRuntime {
        messages: Vec<NativePanelPlatformWindowMessage>,
        processed_message_ids: Vec<u32>,
        paint_count: usize,
        handled_message_ids: Vec<u32>,
        pending_events: Vec<NativePanelPlatformEvent>,
    }

    impl NativePanelPlatformWindowMessagePump for MessagePumpRuntime {
        fn take_platform_window_messages_for_pump(
            &mut self,
        ) -> Vec<NativePanelPlatformWindowMessage> {
            std::mem::take(&mut self.messages)
        }

        fn platform_window_message_screen_frame(&self) -> Option<PanelRect> {
            Some(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            })
        }

        fn record_platform_window_message_processed(&mut self, message_id: u32) {
            self.processed_message_ids.push(message_id);
        }

        fn is_platform_paint_message(&self, message_id: u32) -> bool {
            message_id == 1
        }

        fn dispatch_platform_paint_message(&mut self) -> Result<(), String> {
            self.paint_count += 1;
            Ok(())
        }

        fn handle_platform_window_message_with_handler(
            &mut self,
            message_id: u32,
            _lparam: isize,
            input: &NativePanelRuntimeInputDescriptor,
            handler: &mut impl NativePanelRuntimeCommandHandler<Error = String>,
        ) -> Result<Option<NativePanelPointerInputOutcome>, String> {
            assert_eq!(input.screen_frame.map(|frame| frame.width), Some(1440.0));
            self.handled_message_ids.push(message_id);
            handler.focus_session(format!("message-{message_id}"))?;
            Ok(None)
        }

        fn pending_platform_events_mut(&mut self) -> &mut Vec<NativePanelPlatformEvent> {
            &mut self.pending_events
        }
    }

    #[test]
    fn platform_window_message_pump_processes_paint_and_queues_events() {
        let mut runtime = MessagePumpRuntime {
            messages: vec![
                NativePanelPlatformWindowMessage {
                    message_id: 1,
                    lparam: 0,
                },
                NativePanelPlatformWindowMessage {
                    message_id: 2,
                    lparam: 99,
                },
            ],
            ..MessagePumpRuntime::default()
        };

        pump_native_panel_platform_window_messages(&mut runtime).expect("pump messages");

        assert_eq!(runtime.processed_message_ids, vec![1, 2]);
        assert_eq!(runtime.paint_count, 1);
        assert_eq!(runtime.handled_message_ids, vec![2]);
        assert_eq!(
            runtime.pending_events,
            vec![NativePanelPlatformEvent::FocusSession(
                "message-2".to_string()
            )]
        );
    }
}
