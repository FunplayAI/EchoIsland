use std::time::Instant;

use echoisland_runtime::RuntimeSnapshot;

use super::{
    host_runtime::{WindowsNativePanelHost, WindowsNativePanelRuntime},
    paint_backend::paint_windows_native_panel_job,
    platform_loop::take_windows_native_panel_window_messages,
    window_shell::WINDOWS_WM_PAINT,
};
use crate::{
    native_panel_core::{HoverTransition, PanelPoint, PanelRect, PanelState},
    native_panel_renderer::facade::{
        command::{
            NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
            NativePanelRuntimeCommandHandler,
        },
        descriptor::{NativePanelPointerRegion, NativePanelRuntimeInputDescriptor},
        interaction::{
            NativePanelClickStateBridge, NativePanelCoreStateBridge,
            NativePanelHostInteractionStateBridge, NativePanelPointerInputRuntimeBridge,
            NativePanelPrimaryPointerStateBridge, record_native_panel_focus_click_session,
            resolve_native_panel_last_focus_click,
        },
        renderer::{
            NativePanelRuntimeSceneCache, NativePanelRuntimeSceneMutableStateBridge,
            NativePanelRuntimeSceneStateBridge, NativePanelSceneRuntimeBridge,
        },
        shell::{
            NativePanelHostShellCommand, NativePanelHostShellRuntimePump,
            NativePanelPlatformWindowMessage, NativePanelPlatformWindowMessagePump,
        },
        transition::NativePanelTransitionRequest,
    },
};

impl NativePanelClickStateBridge for WindowsNativePanelRuntime {
    fn click_expanded(&self) -> bool {
        self.panel_state.expanded
    }

    fn click_transitioning(&self) -> bool {
        self.panel_state.transitioning
    }

    fn click_last_focus_click(&self) -> Option<crate::native_panel_core::LastFocusClick<'_>> {
        resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
    }

    fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
        record_native_panel_focus_click_session(&mut self.last_focus_click, session_id, now);
    }
}

impl NativePanelCoreStateBridge for WindowsNativePanelRuntime {
    fn snapshot_core_panel_state(&self) -> PanelState {
        self.panel_state.clone()
    }

    fn apply_core_panel_state(&mut self, core: PanelState) {
        self.panel_state = core;
    }
}

impl NativePanelPrimaryPointerStateBridge for WindowsNativePanelRuntime {
    fn primary_pointer_down(&self) -> bool {
        self.primary_pointer_down
    }

    fn set_primary_pointer_down(&mut self, down: bool) {
        self.primary_pointer_down = down;
    }
}

impl NativePanelHostInteractionStateBridge for WindowsNativePanelRuntime {
    fn host_ignores_mouse_events(&self) -> bool {
        self.ignores_mouse_events
    }

    fn set_host_ignores_mouse_events(&mut self, ignores_mouse_events: bool) {
        self.ignores_mouse_events = ignores_mouse_events;
    }
}

impl NativePanelRuntimeSceneStateBridge for WindowsNativePanelRuntime {
    fn runtime_scene_cache(&self) -> &NativePanelRuntimeSceneCache {
        &self.scene_cache
    }

    fn runtime_scene_current_snapshot(&self) -> Option<&RuntimeSnapshot> {
        self.scene_cache.last_snapshot.as_ref()
    }
}

impl NativePanelRuntimeSceneMutableStateBridge for WindowsNativePanelRuntime {
    fn runtime_scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache {
        &mut self.scene_cache
    }

    fn runtime_pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion> {
        &mut self.host.renderer.last_pointer_regions
    }
}

impl NativePanelSceneRuntimeBridge for WindowsNativePanelRuntime {
    type Host = WindowsNativePanelHost;
    type State = PanelState;

    fn with_runtime_scene_slots<T>(
        &mut self,
        f: impl FnOnce(
            &mut Option<NativePanelTransitionRequest>,
            &mut Self::Host,
            &mut NativePanelRuntimeSceneCache,
            &mut Self::State,
        ) -> T,
    ) -> T {
        f(
            &mut self.last_transition_request,
            &mut self.host,
            &mut self.scene_cache,
            &mut self.panel_state,
        )
    }
}

impl NativePanelHostShellRuntimePump for WindowsNativePanelRuntime {
    type RawWindowHandle = isize;
    type Error = String;

    fn has_pending_shell_destroy_command(&self) -> bool {
        self.host.shell.has_pending_destroy_command()
    }

    fn consume_presenter_into_shell_for_pump(&mut self) {
        let _ = self.host.consume_presenter_into_shell_result();
    }

    fn raw_shell_window_handle(&self) -> Option<Self::RawWindowHandle> {
        self.host.shell.raw_window_handle()
    }

    fn take_pending_shell_commands_for_pump(&mut self) -> Vec<NativePanelHostShellCommand> {
        self.host.take_pending_shell_commands()
    }

    fn apply_shell_command_for_pump(
        &mut self,
        raw_window_handle: &mut Option<Self::RawWindowHandle>,
        command: NativePanelHostShellCommand,
    ) -> Result<(), Self::Error> {
        self.platform_loop
            .consume_shell_command(raw_window_handle, command)
    }

    fn sync_raw_shell_window_handle(&mut self, raw_window_handle: Option<Self::RawWindowHandle>) {
        self.host.shell.set_raw_window_handle(raw_window_handle);
    }

    fn pump_platform_window_messages(&mut self) -> Result<(), Self::Error> {
        self.pump_window_messages()
    }
}

impl NativePanelPointerInputRuntimeBridge for WindowsNativePanelRuntime {
    type Error = String;

    fn sync_mouse_passthrough_for_pointer_input(&mut self, input: NativePanelPointerInput) {
        WindowsNativePanelRuntime::sync_mouse_passthrough_for_pointer_input(self, input);
    }

    fn record_pointer_input(&mut self, input: NativePanelPointerInput) {
        self.host.shell.record_pointer_input(input);
    }

    fn sync_hover_and_refresh_for_pointer_input(
        &mut self,
        input: NativePanelPointerInput,
        now: Instant,
        runtime_input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, Self::Error> {
        self.sync_hover_and_refresh_for_pointer_input_with_input(input, now, runtime_input)
    }

    fn dispatch_click_command_for_pointer_point<H>(
        &mut self,
        point: PanelPoint,
        now: Instant,
        handler: &mut H,
    ) -> Result<Option<NativePanelPlatformEvent>, Self::Error>
    where
        H: NativePanelRuntimeCommandHandler<Error = Self::Error>,
    {
        self.dispatch_click_command_at_point_with_handler(point, now, handler)
    }
}

impl NativePanelPlatformWindowMessagePump for WindowsNativePanelRuntime {
    fn take_platform_window_messages_for_pump(&mut self) -> Vec<NativePanelPlatformWindowMessage> {
        take_windows_native_panel_window_messages(self.host.shell.raw_window_handle())
            .into_iter()
            .map(|message| NativePanelPlatformWindowMessage {
                message_id: message.message_id,
                lparam: message.lparam,
            })
            .collect()
    }

    fn platform_window_message_screen_frame(&self) -> Option<PanelRect> {
        self.host.window.descriptor.screen_frame
    }

    fn record_platform_window_message_processed(&mut self, message_id: u32) {
        self.platform_loop.processed_window_message_count += 1;
        self.platform_loop.last_window_message_id = Some(message_id);
    }

    fn is_platform_paint_message(&self, message_id: u32) -> bool {
        message_id == WINDOWS_WM_PAINT
    }

    fn dispatch_platform_paint_message(&mut self) -> Result<(), String> {
        if let Some(job) = self.host.shell.paint_next_frame() {
            let paint_plan =
                paint_windows_native_panel_job(self.host.shell.raw_window_handle(), &job)?;
            self.platform_loop.paint_dispatch_count += 1;
            self.platform_loop.last_paint_plan = Some(paint_plan);
            self.platform_loop.last_painted_job = Some(job);
        }
        Ok(())
    }

    fn handle_platform_window_message_with_handler(
        &mut self,
        message_id: u32,
        lparam: isize,
        input: &NativePanelRuntimeInputDescriptor,
        handler: &mut impl NativePanelRuntimeCommandHandler<Error = String>,
    ) -> Result<Option<NativePanelPointerInputOutcome>, String> {
        self.handle_window_message_with_handler(message_id, lparam, Instant::now(), input, handler)
    }

    fn pending_platform_events_mut(&mut self) -> &mut Vec<NativePanelPlatformEvent> {
        &mut self.host.pending_events
    }
}
