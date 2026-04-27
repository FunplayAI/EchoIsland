use std::time::Instant;

use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use super::{
    WindowsNativePanelRenderer,
    draw_presenter::WindowsNativePanelDrawPresenter,
    host_window::{WindowsNativePanelDrawFrame, WindowsNativePanelHostWindow},
    message_dispatch::pump_window_messages as pump_dispatched_window_messages,
    paint_bridge::{
        consume_presenter_into_shell, consume_presenter_into_shell_result,
        present_window_into_presenter, take_pending_draw_frame,
    },
    platform_loop::WindowsNativePanelPlatformLoopState,
    runtime_input::windows_runtime_input_descriptor,
    window_shell::{
        WindowsNativePanelShellCommand, WindowsNativePanelShellPresentResult,
        WindowsNativePanelWindowShell,
    },
};
use crate::{
    native_panel_core::{HoverTransition, PanelAnimationDescriptor, PanelPoint, PanelState},
    native_panel_renderer::facade::{
        command::{
            NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
            NativePanelRuntimeCommandHandler,
        },
        descriptor::{
            NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPointerRegion,
            NativePanelRuntimeInputDescriptor,
        },
        host::{
            NativePanelHost, NativePanelHostDisplayReposition, NativePanelRuntimeHostController,
            NativePanelSceneHost, create_native_panel_via_host_controller,
            hide_native_panel_via_host_controller, native_panel_presentation_cards_visible,
            reposition_native_panel_host_from_input_descriptor_via_controller,
            set_native_panel_host_shared_body_height_via_controller,
        },
        interaction::{
            NativePanelHostPollingInteractionResult, NativePanelHoverSyncResult,
            NativePanelPointerRegionInteractionBridge, NativePanelPollingHostFacts,
            NativePanelQueuedPlatformEventBridge, NativePanelSettingsSurfaceToggleResult,
            dispatch_native_panel_click_command_at_point_with_handler,
            handle_native_panel_pointer_input_with_handler,
            handle_optional_native_panel_pointer_input_with_handler,
            native_panel_click_state_slots, native_panel_interactive_inside_from_host_facts,
            native_panel_polling_interaction_input_from_host_facts,
            sync_native_panel_host_polling_interaction_for_state,
            sync_native_panel_hover_and_refresh_for_runtime,
            sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor,
            sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor,
            sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor,
            sync_native_panel_hover_interaction_at_point_for_state,
            sync_native_panel_hover_interaction_for_pointer_input_for_state,
            sync_native_panel_hover_interaction_for_state,
            sync_native_panel_mouse_passthrough_for_interactive_inside,
        },
        renderer::{NativePanelRuntimeSceneCache, NativePanelRuntimeSceneStateBridge},
        runtime::{
            NativePanelRuntimeSceneSyncResult, apply_native_panel_hover_sync_result_for_runtime,
            apply_native_panel_runtime_scene_sync_result_for_runtime,
            apply_native_panel_settings_surface_toggle_result_for_runtime,
            rerender_runtime_scene_sync_result_to_host_for_runtime_with_input_descriptor,
            sync_runtime_scene_bundle_for_runtime_with_input,
            toggle_native_panel_settings_surface_and_rerender_for_runtime_with_input_descriptor,
        },
        shell::pump_native_panel_host_shell_runtime,
        transition::NativePanelTransitionRequest,
    },
};

#[derive(Default)]
pub(crate) struct WindowsNativePanelHost {
    pub(super) renderer: WindowsNativePanelRenderer,
    pub(super) window: WindowsNativePanelHostWindow,
    pub(super) presenter: WindowsNativePanelDrawPresenter,
    pub(super) shell: WindowsNativePanelWindowShell,
    pub(super) pending_events: Vec<NativePanelPlatformEvent>,
}

impl NativePanelHost for WindowsNativePanelHost {
    type Error = String;
    type Renderer = WindowsNativePanelRenderer;

    fn renderer(&mut self) -> &mut Self::Renderer {
        &mut self.renderer
    }

    fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
        self.window.descriptor
    }

    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
        &mut self.window.descriptor
    }

    fn window_state(&self) -> NativePanelHostWindowState {
        self.window.window_state()
    }

    fn create(&mut self) -> Result<(), Self::Error> {
        self.window.create();
        self.shell.create();
        self.shell.sync_window_state(self.window.window_state());
        self.sync_renderer_host_window_descriptor()
    }

    fn after_host_window_descriptor_updated(&mut self) -> Result<(), Self::Error> {
        self.window.refresh_frame_from_descriptor();
        self.shell.sync_window_state(self.window.window_state());
        Ok(())
    }

    fn show(&mut self) -> Result<(), Self::Error> {
        NativePanelHost::create(self)?;
        self.window.show();
        self.shell.show();
        self.shell.sync_window_state(self.window.window_state());
        self.sync_renderer_host_window_descriptor()
    }

    fn hide(&mut self) -> Result<(), Self::Error> {
        self.window.hide();
        self.shell.hide();
        self.shell.sync_window_state(self.window.window_state());
        self.sync_renderer_host_window_descriptor()
    }

    fn take_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn present_renderer_state(&mut self) -> Result<(), Self::Error> {
        let window_state = self
            .renderer
            .last_window_state
            .unwrap_or_else(|| self.window.window_state());
        present_window_into_presenter(
            &mut self.window,
            &mut self.presenter,
            window_state,
            &self.renderer.last_pointer_regions,
            self.renderer.current_presentation_model(),
        );
        Ok(())
    }
}

impl WindowsNativePanelHost {
    pub(super) fn record_platform_loop_spawn(&mut self) {
        self.shell.record_platform_loop_spawn();
    }

    pub(super) fn take_pending_draw_frame(&mut self) -> Option<WindowsNativePanelDrawFrame> {
        take_pending_draw_frame(&mut self.presenter)
    }

    pub(super) fn take_pending_shell_commands(&mut self) -> Vec<WindowsNativePanelShellCommand> {
        self.shell.take_pending_commands()
    }

    pub(super) fn sync_shell_mouse_event_passthrough(&mut self, ignores_mouse_events: bool) {
        self.shell
            .sync_mouse_event_passthrough(ignores_mouse_events);
    }

    pub(super) fn consume_presenter_into_shell(&mut self) -> bool {
        consume_presenter_into_shell(&mut self.presenter, &mut self.shell)
    }

    pub(super) fn consume_presenter_into_shell_result(
        &mut self,
    ) -> WindowsNativePanelShellPresentResult {
        consume_presenter_into_shell_result(&mut self.presenter, &mut self.shell)
    }

    pub(super) fn resolved_pointer_regions(&self) -> &[NativePanelPointerRegion] {
        self.window
            .pointer_regions(&self.renderer.last_pointer_regions)
    }

    pub(super) fn cards_visible(&self) -> bool {
        let current = self.renderer.current_presentation_model();
        native_panel_presentation_cards_visible(
            self.window.presented_presentation_model.as_ref(),
            current.as_ref(),
        )
    }
}

impl NativePanelSceneHost for WindowsNativePanelHost {}

impl NativePanelPointerRegionInteractionBridge for WindowsNativePanelHost {
    fn interaction_pointer_regions(&self) -> &[NativePanelPointerRegion] {
        self.resolved_pointer_regions()
    }

    fn interaction_cards_visible(&self) -> bool {
        self.cards_visible()
    }
}

impl NativePanelQueuedPlatformEventBridge for WindowsNativePanelHost {
    fn queued_platform_events_mut(&mut self) -> &mut Vec<NativePanelPlatformEvent> {
        &mut self.pending_events
    }

    fn queued_pointer_regions(&self) -> &[NativePanelPointerRegion] {
        self.resolved_pointer_regions()
    }
}

impl NativePanelRuntimeHostController for WindowsNativePanelHost {
    type Error = String;

    fn runtime_host_create_panel(&mut self) -> Result<(), Self::Error> {
        self.create()
    }

    fn runtime_host_hide_panel(&mut self) -> Result<(), Self::Error> {
        self.hide()
    }

    fn runtime_host_reposition(
        &mut self,
        reposition: NativePanelHostDisplayReposition,
    ) -> Result<(), Self::Error> {
        self.reposition_to_display_with_payload(reposition)
    }

    fn runtime_host_set_shared_body_height(&mut self, body_height: f64) -> Result<(), Self::Error> {
        self.set_shared_body_height(body_height)
    }
}

#[derive(Default)]
pub(crate) struct WindowsNativePanelRuntime {
    pub(super) panel_state: PanelState,
    pub(super) primary_pointer_down: bool,
    pub(super) ignores_mouse_events: bool,
    pub(super) host: WindowsNativePanelHost,
    pub(super) platform_loop: WindowsNativePanelPlatformLoopState,
    pub(super) scene_cache: NativePanelRuntimeSceneCache,
    pub(super) last_animation_descriptor: Option<PanelAnimationDescriptor>,
    pub(super) last_transition_request: Option<NativePanelTransitionRequest>,
    pub(super) last_focus_click: Option<(String, Instant)>,
}

impl WindowsNativePanelRuntime {
    pub(super) fn pump_platform_loop(&mut self) -> Result<(), String> {
        pump_native_panel_host_shell_runtime(self)
    }

    pub(super) fn pump_window_messages(&mut self) -> Result<(), String> {
        pump_dispatched_window_messages(self)
    }

    pub(super) fn apply_runtime_scene_sync_result(
        &mut self,
        sync_result: NativePanelRuntimeSceneSyncResult,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<(), String> {
        apply_native_panel_runtime_scene_sync_result_for_runtime(self, sync_result, input)
    }

    pub(super) fn apply_settings_surface_toggle_result(
        &mut self,
        result: NativePanelSettingsSurfaceToggleResult,
    ) -> bool {
        apply_native_panel_settings_surface_toggle_result_for_runtime(self, result)
    }

    pub(super) fn apply_hover_sync_result(
        &mut self,
        hover_sync: NativePanelHoverSyncResult,
    ) -> Option<HoverTransition> {
        apply_native_panel_hover_sync_result_for_runtime(self, hover_sync)
    }

    pub(super) fn sync_hover_and_refresh_with_input(
        &mut self,
        resolve: impl FnOnce(
            &mut WindowsNativePanelHost,
            &mut NativePanelRuntimeSceneCache,
            &mut PanelState,
        ) -> Result<Option<NativePanelHoverSyncResult>, String>,
    ) -> Result<Option<HoverTransition>, String> {
        sync_native_panel_hover_and_refresh_for_runtime(self, resolve)
    }

    pub(super) fn create_panel(&mut self) -> Result<(), String> {
        create_native_panel_via_host_controller(&mut self.host)
    }

    pub(super) fn hide_panel(&mut self) -> Result<(), String> {
        hide_native_panel_via_host_controller(&mut self.host)
    }

    pub(super) fn reposition_to_selected_display_with_input(
        &mut self,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<(), String> {
        reposition_native_panel_host_from_input_descriptor_via_controller(&mut self.host, input)
    }

    pub(super) fn set_shared_expanded_body_height(
        &mut self,
        body_height: f64,
    ) -> Result<(), String> {
        set_native_panel_host_shared_body_height_via_controller(&mut self.host, body_height)
    }

    pub(super) fn sync_hover_inside(
        &mut self,
        inside: bool,
        now: Instant,
    ) -> Option<HoverTransition> {
        sync_native_panel_hover_interaction_for_state(
            &mut self.panel_state,
            inside,
            now,
            crate::native_panel_core::HOVER_DELAY_MS,
        )
        .transition
    }

    pub(super) fn sync_hover_at_point(
        &mut self,
        point: PanelPoint,
        now: Instant,
    ) -> Option<HoverTransition> {
        sync_native_panel_hover_interaction_at_point_for_state(
            &mut self.panel_state,
            &self.host,
            point,
            now,
            crate::native_panel_core::HOVER_DELAY_MS,
        )
    }

    pub(super) fn sync_hover_for_pointer_input(
        &mut self,
        input: NativePanelPointerInput,
        now: Instant,
    ) -> Option<HoverTransition> {
        sync_native_panel_hover_interaction_for_pointer_input_for_state(
            &mut self.panel_state,
            &self.host,
            input,
            now,
            crate::native_panel_core::HOVER_DELAY_MS,
        )
    }

    pub(super) fn polling_host_facts(
        &self,
        pointer: PanelPoint,
        primary_mouse_down: bool,
    ) -> Option<NativePanelPollingHostFacts<'_>> {
        self.host.shell.polling_host_facts(
            pointer,
            primary_mouse_down,
            self.runtime_scene_current_snapshot().cloned(),
        )
    }

    pub(super) fn interactive_inside_for_pointer_input(
        &self,
        input: NativePanelPointerInput,
    ) -> Option<bool> {
        match input {
            NativePanelPointerInput::Move(point) => self
                .polling_host_facts(point, false)
                .map(native_panel_interactive_inside_from_host_facts),
            NativePanelPointerInput::Leave => Some(false),
            NativePanelPointerInput::Click(_) => None,
        }
    }

    pub(super) fn sync_mouse_passthrough_for_pointer_input(
        &mut self,
        input: NativePanelPointerInput,
    ) {
        let Some(interactive_inside) = self.interactive_inside_for_pointer_input(input) else {
            return;
        };
        let Some(next_ignores_mouse_events) =
            sync_native_panel_mouse_passthrough_for_interactive_inside(self, interactive_inside)
        else {
            return;
        };
        self.host
            .sync_shell_mouse_event_passthrough(next_ignores_mouse_events);
    }

    pub(super) fn sync_host_polling_interaction(
        &mut self,
        pointer: PanelPoint,
        primary_mouse_down: bool,
        now: Instant,
    ) -> Option<NativePanelHostPollingInteractionResult> {
        let facts = self.polling_host_facts(pointer, primary_mouse_down)?;
        let input = native_panel_polling_interaction_input_from_host_facts(facts);
        let interaction = sync_native_panel_host_polling_interaction_for_state(
            self,
            input,
            now,
            crate::native_panel_core::HOVER_DELAY_MS,
            crate::native_panel_core::CARD_FOCUS_CLICK_DEBOUNCE_MS,
        );
        if interaction.sync_mouse_event_passthrough {
            self.host
                .sync_shell_mouse_event_passthrough(interaction.next_ignores_mouse_events);
        }
        Some(interaction)
    }

    pub(super) fn sync_snapshot_bundle(
        &mut self,
        snapshot: &RuntimeSnapshot,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<(), String> {
        sync_runtime_scene_bundle_for_runtime_with_input(self, snapshot, input)
    }

    pub(super) fn rerender_from_last_snapshot_with_input(
        &mut self,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<bool, String> {
        rerender_runtime_scene_sync_result_to_host_for_runtime_with_input_descriptor(self, input)
    }

    pub(super) fn rerender_from_last_snapshot<R: tauri::Runtime>(
        &mut self,
        app: &AppHandle<R>,
    ) -> Result<bool, String> {
        let input = windows_runtime_input_descriptor(app);
        self.rerender_from_last_snapshot_with_input(&input)
    }

    pub(super) fn toggle_settings_surface_with_input(
        &mut self,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<bool, String> {
        toggle_native_panel_settings_surface_and_rerender_for_runtime_with_input_descriptor(
            self, input,
        )
    }

    pub(super) fn sync_hover_and_refresh_at_point_with_input(
        &mut self,
        point: PanelPoint,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, String> {
        self.sync_hover_and_refresh_with_input(|host, cache, state| {
            sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor(
                host,
                cache,
                state,
                point,
                now,
                crate::native_panel_core::HOVER_DELAY_MS,
                input,
            )
            .map(Some)
        })
    }

    pub(super) fn sync_hover_and_refresh_inside_with_input(
        &mut self,
        inside: bool,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, String> {
        self.sync_hover_and_refresh_with_input(|host, cache, state| {
            sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor(
                host,
                cache,
                state,
                inside,
                now,
                crate::native_panel_core::HOVER_DELAY_MS,
                input,
            )
            .map(Some)
        })
    }

    pub(super) fn sync_hover_and_refresh_for_pointer_input_with_input(
        &mut self,
        input_event: NativePanelPointerInput,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, String> {
        self.sync_hover_and_refresh_with_input(|host, cache, state| {
            sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor(
                host,
                cache,
                state,
                input_event,
                now,
                crate::native_panel_core::HOVER_DELAY_MS,
                input,
            )
        })
    }

    pub(super) fn dispatch_click_command_at_point_with_handler<H>(
        &mut self,
        point: PanelPoint,
        now: Instant,
        handler: &mut H,
    ) -> Result<Option<NativePanelPlatformEvent>, H::Error>
    where
        H: NativePanelRuntimeCommandHandler,
    {
        let mut click_state =
            native_panel_click_state_slots(&self.panel_state, &mut self.last_focus_click);
        dispatch_native_panel_click_command_at_point_with_handler(
            &mut click_state,
            &self.host,
            point,
            now,
            crate::native_panel_core::CARD_FOCUS_CLICK_DEBOUNCE_MS,
            handler,
        )
    }

    pub(super) fn take_queued_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        self.host.take_platform_events()
    }

    pub(super) fn handle_window_message_with_handler(
        &mut self,
        message_id: u32,
        lparam: isize,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
        handler: &mut impl NativePanelRuntimeCommandHandler<Error = String>,
    ) -> Result<Option<NativePanelPointerInputOutcome>, String> {
        let message = self.host.shell.decode_window_message(message_id, lparam);
        handle_optional_native_panel_pointer_input_with_handler(self, message, now, input, handler)
    }

    pub(super) fn handle_pointer_input_with_handler(
        &mut self,
        input_event: NativePanelPointerInput,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
        handler: &mut impl NativePanelRuntimeCommandHandler<Error = String>,
    ) -> Result<NativePanelPointerInputOutcome, String> {
        handle_native_panel_pointer_input_with_handler(self, input_event, now, input, handler)
    }
}
