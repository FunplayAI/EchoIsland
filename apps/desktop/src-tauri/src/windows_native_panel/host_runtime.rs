use std::time::{Duration, Instant};

use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;
use tracing::info;

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
    platform_loop::current_windows_native_panel_pointer_polling_sample,
    runtime_input::windows_runtime_input_descriptor,
    window_shell::{
        WindowsNativePanelShellCommand, WindowsNativePanelShellPresentResult,
        WindowsNativePanelWindowShell,
    },
};
use crate::{
    native_panel_core::{
        HoverTransition, PanelAnimationDescriptor, PanelPoint, PanelSnapshotSyncResult, PanelState,
        panel_state_needs_status_queue_refresh, take_pending_status_reopen_after_transition,
    },
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
        presentation::NativePanelCardStackPresentation,
        renderer::{
            NativePanelRuntimeSceneCache, NativePanelRuntimeSceneStateBridge,
            NativePanelStatusClosePreservationInput, NativePanelStatusClosePreservationPlan,
            resolve_native_panel_status_close_preservation_plan,
        },
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
            self.renderer.latest_scene_presentation_model(),
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
        let current = self.renderer.latest_scene_presentation_model();
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
        self.show()
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
    pub(super) user_hidden: bool,
    pub(super) host: WindowsNativePanelHost,
    pub(super) platform_loop: WindowsNativePanelPlatformLoopState,
    pub(super) scene_cache: NativePanelRuntimeSceneCache,
    pub(super) animation_scheduler:
        crate::native_panel_renderer::facade::renderer::NativePanelAnimationFrameScheduler,
    pub(super) next_animation_wake_at: Option<Instant>,
    pub(super) last_animation_descriptor: Option<PanelAnimationDescriptor>,
    pub(super) last_transition_request: Option<NativePanelTransitionRequest>,
    pub(super) pending_close_card_stack: Option<NativePanelCardStackPresentation>,
    /// True for the duration of a close animation that was triggered by the user
    /// hovering out of the panel (not by status-queue auto-collapse). Used to keep
    /// the edge action buttons (settings / quit) visible during the close so they
    /// fade out via the normal width morph instead of popping off, and to preserve
    /// cards from any surface — not just Status — across mid-close re-renders.
    pub(super) hover_close_in_progress: bool,
    pub(super) active_count_marquee_started_at: Option<Instant>,
    pub(super) mascot_animation_started_at: Option<Instant>,
    pub(super) last_focus_click: Option<(String, Instant)>,
}

impl WindowsNativePanelRuntime {
    pub(super) fn pump_platform_loop(&mut self) -> Result<(), String> {
        if self.host.shell.has_pending_destroy_command() {
            super::platform_loop::clear_windows_native_panel_hit_regions(
                self.platform_loop.last_raw_window_handle,
            );
        }
        let had_unstarted_hover_open = self.has_unstarted_hover_open_request();
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: Default::default(),
            screen_frame: self.host.window.descriptor.screen_frame,
        };
        let status_queue_refresh =
            self.refresh_status_queue_from_last_raw_snapshot_with_input(&input)?;
        let result = pump_native_panel_host_shell_runtime(self);
        self.cancel_unstarted_hover_open_if_pointer_left(had_unstarted_hover_open);
        let now = Instant::now();
        let poll_result = self.sync_current_pointer_polling_interaction(now, &input)?;
        let animation_frame = self.advance_animation_frame_at(now)?;
        let marquee_frame = self.refresh_active_count_marquee_frame_at(now);
        let mascot_frame = self.refresh_mascot_animation_frame_at(now);
        if status_queue_refresh
            || poll_result.is_some()
            || animation_frame.is_some()
            || marquee_frame
            || mascot_frame
        {
            pump_native_panel_host_shell_runtime(self)?;
        }
        self.schedule_next_status_queue_refresh_wake();
        self.schedule_next_animation_frame_wake(now);
        result
    }

    fn has_unstarted_hover_open_request(&self) -> bool {
        self.last_transition_request == Some(NativePanelTransitionRequest::Open)
            && !self.panel_state.transitioning
    }

    fn cancel_unstarted_hover_open_if_pointer_left(&mut self, had_unstarted_hover_open: bool) {
        if !had_unstarted_hover_open
            || self.panel_state.transitioning
            || self.host.shell.last_pointer_input() != Some(NativePanelPointerInput::Leave)
        {
            return;
        }

        self.last_transition_request = None;
        self.panel_state.expanded = false;
        self.panel_state.status_auto_expanded = false;
        self.panel_state.surface_mode = crate::native_panel_core::ExpandedSurface::Default;
        self.panel_state.pointer_inside_since = None;
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
            .map(|_| ())
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
        self.user_hidden = false;
        create_native_panel_via_host_controller(&mut self.host)
    }

    pub(super) fn hide_panel(&mut self) -> Result<(), String> {
        self.user_hidden = true;
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

    pub(super) fn sync_host_polling_interaction_and_refresh(
        &mut self,
        pointer: PanelPoint,
        primary_mouse_down: bool,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<NativePanelHostPollingInteractionResult>, String> {
        let Some(interaction) =
            self.sync_host_polling_interaction(pointer, primary_mouse_down, now)
        else {
            return Ok(None);
        };
        if interaction.transition_request.is_some() {
            self.last_transition_request = interaction.transition_request;
            let previous_transitioning = self.panel_state.transitioning;
            let is_close =
                interaction.transition_request == Some(NativePanelTransitionRequest::Close);
            if is_close {
                self.panel_state.transitioning = true;
                self.hover_close_in_progress = true;
            }
            // Hover sync already flipped panel_state.expanded = false. The upcoming
            // rerender rebuilds the scene as collapsed and clears the card stack from
            // scene_cache. Capture the soon-to-be-lost cards now so we can restore them
            // onto the rebuilt scene; without this the close animation starts with
            // card_count = 0 and visually skips the card-exit phase.
            let preserved_close_card_stack =
                is_close.then(|| self.capture_card_stack_for_hover_close_transition()).flatten();
            if let Err(error) = self.rerender_from_last_snapshot_with_input(input) {
                self.panel_state.transitioning = previous_transitioning;
                if is_close {
                    self.hover_close_in_progress = false;
                }
                return Err(error);
            }
            if is_close {
                // Stash for refresh_status_queue_from_last_raw_snapshot_with_input,
                // which fires on subsequent ticks while a status_queue / pending card
                // is still present and would otherwise wipe our preserved cards.
                self.pending_close_card_stack = preserved_close_card_stack.clone();
                self.host
                    .renderer
                    .preserve_card_stack_for_hover_close_transition(
                        preserved_close_card_stack.as_ref(),
                    );
                self.host.present_renderer_state()?;
            }
        }
        Ok(Some(interaction))
    }

    fn capture_card_stack_for_hover_close_transition(
        &self,
    ) -> Option<NativePanelCardStackPresentation> {
        self.host
            .window
            .presented_presentation_model
            .as_ref()
            .or(self.host.renderer.last_presentation_model.as_ref())
            .map(|presentation| presentation.card_stack.clone())
            .filter(|card_stack| !card_stack.cards.is_empty())
    }

    fn sync_current_pointer_polling_interaction(
        &mut self,
        now: Instant,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<NativePanelHostPollingInteractionResult>, String> {
        if self.user_hidden {
            return Ok(None);
        }
        let Some(sample) = current_windows_native_panel_pointer_polling_sample(
            self.host.shell.raw_window_handle(),
        ) else {
            log_windows_native_hover_probe(
                self.host.shell.raw_window_handle(),
                None,
                self.host.shell.pointer_regions().len(),
                None,
                None,
                self.panel_state.expanded,
            );
            return Ok(None);
        };
        let interaction =
            self.sync_host_polling_interaction_and_refresh(sample.point, false, now, input)?;
        log_windows_native_hover_probe(
            self.host.shell.raw_window_handle(),
            Some(sample.point),
            self.host.shell.pointer_regions().len(),
            interaction.as_ref().map(|result| result.interactive_inside),
            interaction
                .as_ref()
                .and_then(|result| result.transition_request),
            self.panel_state.expanded,
        );
        Ok(interaction)
    }

    pub(super) fn sync_snapshot_bundle(
        &mut self,
        snapshot: &RuntimeSnapshot,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<PanelSnapshotSyncResult>, String> {
        if self.user_hidden {
            return Ok(None);
        }
        let preserved_close_card_stack = self.capture_close_preservation_card_stack();
        let active_close_before_sync = self.status_close_preservation_plan().active_close;
        let sync = sync_runtime_scene_bundle_for_runtime_with_input(self, snapshot, input)?;
        if active_close_before_sync {
            self.apply_close_preservation_card_stack(preserved_close_card_stack.as_ref());
            self.host.present_renderer_state()?;
        }
        self.store_pending_close_card_stack_if_needed(preserved_close_card_stack);
        Ok(Some(sync))
    }

    pub(super) fn refresh_status_queue_from_last_raw_snapshot_with_input(
        &mut self,
        input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<bool, String> {
        let Some(snapshot) = self.status_queue_refresh_snapshot() else {
            return Ok(false);
        };
        let pending_transition = self.last_transition_request;
        let active_close_before_refresh = self.status_close_preservation_plan().active_close;
        let preserved_close_card_stack = self.capture_close_preservation_card_stack();
        sync_runtime_scene_bundle_for_runtime_with_input(self, &snapshot, input)?;
        if self.last_transition_request.is_none() {
            self.last_transition_request = pending_transition;
        }
        let close_preservation_after_refresh = self.status_close_preservation_plan();
        if active_close_before_refresh || close_preservation_after_refresh.pending_close {
            self.apply_close_preservation_card_stack(preserved_close_card_stack.as_ref());
            self.host.present_renderer_state()?;
        }
        self.store_pending_close_card_stack_if_needed(preserved_close_card_stack);
        Ok(true)
    }

    /// Capture the card stack that should survive the next mid-close re-render.
    /// Hover-driven close preserves cards from any surface (and falls back to
    /// the stash we set when the close was kicked off, since the currently
    /// presented model may already have been mutated by an earlier preserve
    /// pass on this tick). Status-queue auto-collapse keeps the original
    /// Status-only filter.
    fn capture_close_preservation_card_stack(
        &self,
    ) -> Option<NativePanelCardStackPresentation> {
        if self.hover_close_in_progress {
            self.capture_card_stack_for_hover_close_transition()
                .or_else(|| self.pending_close_card_stack.clone())
        } else {
            self.capture_status_card_stack_for_close_transition()
        }
    }

    /// Re-apply the captured card stack onto the just-rebuilt scene. Hover-
    /// driven close uses the slim variant that does not suppress edge action
    /// buttons, so settings / quit fade out via the natural width morph.
    fn apply_close_preservation_card_stack(
        &mut self,
        preserved: Option<&NativePanelCardStackPresentation>,
    ) {
        if self.hover_close_in_progress {
            self.host
                .renderer
                .preserve_card_stack_for_hover_close_transition(preserved);
        } else {
            self.host
                .renderer
                .preserve_card_stack_for_close_transition(preserved);
        }
    }

    fn capture_status_card_stack_for_close_transition(
        &self,
    ) -> Option<NativePanelCardStackPresentation> {
        self.host
            .window
            .presented_presentation_model
            .as_ref()
            .or(self.host.renderer.last_presentation_model.as_ref())
            .map(|presentation| presentation.card_stack.clone())
            .filter(|card_stack| {
                card_stack.surface == crate::native_panel_core::ExpandedSurface::Status
                    && !card_stack.cards.is_empty()
            })
    }

    fn store_pending_close_card_stack_if_needed(
        &mut self,
        card_stack: Option<NativePanelCardStackPresentation>,
    ) {
        if self
            .status_close_preservation_plan()
            .should_store_pending_stack
        {
            self.pending_close_card_stack = card_stack;
        }
    }

    fn status_queue_refresh_snapshot(&self) -> Option<RuntimeSnapshot> {
        if self.user_hidden || !panel_state_needs_status_queue_refresh(&self.panel_state) {
            return None;
        }
        self.panel_state.last_raw_snapshot.clone()
    }

    fn status_close_transition_active(&self) -> bool {
        self.status_close_preservation_plan().active_close
    }

    fn should_preserve_pending_status_close_frame(&self) -> bool {
        self.status_close_preservation_plan().pending_close
    }

    fn status_close_preservation_plan(&self) -> NativePanelStatusClosePreservationPlan {
        self.status_close_preservation_plan_for_request(self.last_transition_request)
    }

    fn status_close_preservation_plan_for_request(
        &self,
        last_transition_request: Option<NativePanelTransitionRequest>,
    ) -> NativePanelStatusClosePreservationPlan {
        resolve_native_panel_status_close_preservation_plan(
            NativePanelStatusClosePreservationInput {
                last_transition_request,
                skip_next_close_card_exit: self.panel_state.skip_next_close_card_exit,
                transitioning: self.panel_state.transitioning,
                last_animation: self.last_animation_descriptor,
            },
        )
    }

    pub(super) fn refresh_active_count_marquee_frame_at(&mut self, now: Instant) -> bool {
        if self.panel_state.transitioning || self.animation_scheduler.is_active() {
            self.active_count_marquee_started_at = None;
            return false;
        }
        if !self.host.shell.active_count_marquee_needs_refresh() {
            self.active_count_marquee_started_at = None;
            return false;
        }
        let started_at = *self.active_count_marquee_started_at.get_or_insert(now);
        self.host
            .shell
            .refresh_active_count_marquee(now.duration_since(started_at).as_millis())
    }

    pub(super) fn refresh_mascot_animation_frame_at(&mut self, now: Instant) -> bool {
        if self.panel_state.transitioning || self.animation_scheduler.is_active() {
            self.mascot_animation_started_at = None;
            return false;
        }
        if !self.host.shell.mascot_animation_needs_refresh() {
            self.mascot_animation_started_at = None;
            return false;
        }
        let Some(started_at) = self.mascot_animation_started_at else {
            self.mascot_animation_started_at = Some(now);
            return false;
        };
        self.host
            .shell
            .refresh_mascot_animation(now.duration_since(started_at).as_millis())
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

    pub(super) fn advance_animation_frame_at(
        &mut self,
        now: Instant,
    ) -> Result<
        Option<crate::native_panel_renderer::facade::renderer::NativePanelAnimationFrame>,
        String,
    > {
        if let Some(request) = self.last_transition_request.take() {
            // If this transition is anything other than Close, any in-flight
            // hover-close state from an earlier (now-superseded) close request
            // is stale and would route subsequent status_queue / snapshot syncs
            // through the hover-close preservation path even though we're no
            // longer closing. Clear it before starting the new animation.
            if request != NativePanelTransitionRequest::Close {
                self.hover_close_in_progress = false;
                self.pending_close_card_stack = None;
            }
            let close_preservation = self.status_close_preservation_plan_for_request(Some(request));
            if request == NativePanelTransitionRequest::Close
                && close_preservation.should_prepare_close_animation_stack
            {
                let preserved_card_stack = self
                    .pending_close_card_stack
                    .take()
                    .or_else(|| self.capture_status_card_stack_for_close_transition());
                self.panel_state.skip_next_close_card_exit = false;
                self.host
                    .renderer
                    .preserve_card_stack_for_close_transition(preserved_card_stack.as_ref());
            }
            if request == NativePanelTransitionRequest::Open
                && self.panel_state.status_auto_expanded
                && self.panel_state.surface_mode
                    == crate::native_panel_core::ExpandedSurface::Status
            {
                let input = NativePanelRuntimeInputDescriptor {
                    scene_input: Default::default(),
                    screen_frame: self.host.window.descriptor.screen_frame,
                };
                self.rerender_from_last_snapshot_with_input(&input)?;
            }
            let target = self.resolve_animation_target(request);
            self.panel_state.transitioning = true;
            let frame = self.animation_scheduler.start(target, now);
            self.apply_animation_frame(frame)?;
            return Ok(Some(frame));
        }

        let Some(frame) = self.animation_scheduler.sample(now) else {
            self.next_animation_wake_at = None;
            return Ok(None);
        };
        self.apply_animation_frame(frame)?;
        Ok(Some(frame))
    }

    fn apply_animation_frame(
        &mut self,
        frame: crate::native_panel_renderer::facade::renderer::NativePanelAnimationFrame,
    ) -> Result<(), String> {
        self.host.apply_timeline_descriptor(frame.plan.timeline)?;
        self.last_animation_descriptor = Some(frame.plan.timeline.animation);
        if !frame.continue_animating {
            self.panel_state.transitioning = false;
            self.next_animation_wake_at = None;
            if frame.descriptor.animation.kind
                == crate::native_panel_core::PanelAnimationKind::Close
            {
                self.hover_close_in_progress = false;
                self.pending_close_card_stack = None;
                if take_pending_status_reopen_after_transition(&mut self.panel_state) {
                    self.last_transition_request = Some(NativePanelTransitionRequest::Open);
                }
            }
        }
        Ok(())
    }

    fn resolve_animation_target(
        &self,
        request: NativePanelTransitionRequest,
    ) -> crate::native_panel_renderer::facade::renderer::NativePanelAnimationTarget {
        let start_height = self
            .host
            .renderer
            .last_animation_descriptor
            .map(|descriptor| descriptor.visible_height)
            .unwrap_or(crate::native_panel_core::COLLAPSED_PANEL_HEIGHT);
        let target_height = match request {
            NativePanelTransitionRequest::Close => crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            NativePanelTransitionRequest::Open | NativePanelTransitionRequest::SurfaceSwitch => {
                self.resolved_expanded_target_height()
            }
        };
        crate::native_panel_renderer::facade::renderer::NativePanelAnimationTarget {
            request,
            start_height,
            target_height,
            card_count: self
                .host
                .renderer
                .scene_cache
                .last_scene
                .as_ref()
                .map(|scene| scene.cards.len())
                .unwrap_or_default(),
        }
    }

    pub(super) fn resolved_expanded_target_height(&self) -> f64 {
        let body_height = self
            .host
            .renderer
            .latest_scene_presentation_model()
            .map(|presentation| presentation.metrics.expanded_body_height)
            .or(self.host.window.descriptor.shared_body_height)
            .unwrap_or(0.0);
        (crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT
            + crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP
            + body_height
            + crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET)
            .max(crate::native_panel_core::COLLAPSED_PANEL_HEIGHT)
    }

    fn schedule_next_animation_frame_wake(&mut self, now: Instant) {
        let Some(delay_ms) = self.animation_scheduler.next_frame_delay_ms() else {
            self.next_animation_wake_at = None;
            return;
        };
        let next_wake = now + Duration::from_millis(delay_ms);
        if self
            .next_animation_wake_at
            .is_some_and(|scheduled| scheduled > now)
        {
            return;
        }
        self.next_animation_wake_at = Some(next_wake);
        super::platform_loop::schedule_windows_native_platform_loop_wake(delay_ms);
    }

    fn schedule_next_status_queue_refresh_wake(&self) {
        if panel_state_needs_status_queue_refresh(&self.panel_state) {
            super::platform_loop::schedule_windows_native_platform_loop_wake(
                crate::native_panel_core::STATUS_QUEUE_REFRESH_MS,
            );
            return;
        }
        if self.host.shell.active_count_marquee_needs_refresh() {
            super::platform_loop::schedule_windows_native_platform_loop_wake(
                crate::native_panel_core::ACTIVE_COUNT_SCROLL_REFRESH_MS,
            );
            return;
        }
        if self.host.shell.mascot_animation_needs_refresh() {
            super::platform_loop::schedule_windows_native_platform_loop_wake(
                crate::native_panel_core::MASCOT_ANIMATION_REFRESH_MS,
            );
            return;
        }
        super::platform_loop::schedule_windows_native_platform_loop_wake(
            crate::native_panel_core::HOVER_POLL_MS,
        );
    }
}

fn windows_native_hover_probe_enabled() -> bool {
    std::env::var("ECHOISLAND_WINDOWS_HOVER_PROBE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn log_windows_native_hover_probe(
    raw_window_handle: Option<isize>,
    pointer: Option<PanelPoint>,
    pointer_region_count: usize,
    interactive_inside: Option<bool>,
    transition_request: Option<NativePanelTransitionRequest>,
    expanded: bool,
) {
    if !windows_native_hover_probe_enabled() {
        return;
    }
    info!(
        raw_window_handle = ?raw_window_handle,
        pointer_x = pointer.map(|point| point.x),
        pointer_y = pointer.map(|point| point.y),
        pointer_region_count,
        interactive_inside,
        transition_request = ?transition_request,
        expanded,
        "windows native hover probe"
    );
}
