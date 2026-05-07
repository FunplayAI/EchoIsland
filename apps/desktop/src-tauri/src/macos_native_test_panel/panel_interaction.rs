use crate::native_panel_core::PanelInteractionCommand;
#[cfg(test)]
use crate::native_panel_renderer::facade::interaction::{
    sync_native_panel_hover_expansion_state_for_state,
    toggle_native_panel_settings_surface_for_state,
};
use crate::native_panel_renderer::facade::{
    command::NativePanelPlatformEvent,
    descriptor::NativePanelPointerRegion,
    interaction::{
        NativePanelHostBehaviorCommand, NativePanelHoverFallbackFrames,
        NativePanelPollingHostFacts,
        sync_native_panel_host_polling_interaction_from_host_facts_for_state,
    },
    transition::NativePanelTransitionRequest,
};
use echoisland_runtime::RuntimeSnapshot;
use objc2_app_kit::NSEvent;
use std::time::Instant;
use tauri::AppHandle;

use super::compact_bar_layout::sync_active_count_marquee;
use super::panel_constants::{
    CARD_FOCUS_CLICK_DEBOUNCE_MS, COLLAPSED_PANEL_HEIGHT, HOVER_DELAY_MS,
};
use super::panel_geometry::absolute_rect;
use super::panel_hit_testing::native_hover_pill_rect;
use super::panel_host_adapter::ns_rect_to_panel_rect;
use super::panel_interaction_effects::{handle_native_click_command, handle_native_platform_event};
use super::panel_refs::{
    native_panel_handles, native_panel_state, panel_from_ptr, resolve_native_panel_refs,
    view_from_ptr,
};
use super::panel_runtime_dispatch::{
    clear_pending_native_panel_close_transition_in_state,
    dispatch_native_panel_transition_request_immediate_with_snapshot,
};
use super::panel_types::NativeExpandedSurface;
use super::panel_types::NativePanelHandles;
#[cfg(test)]
use super::panel_types::{NativeHoverTransition, NativePanelState};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn sync_hover_state_on_main_thread<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
) {
    let Some(handles) = native_panel_handles() else {
        return;
    };
    let Some(state_mutex) = native_panel_state() else {
        return;
    };

    let refs = resolve_native_panel_refs(handles);
    sync_active_count_marquee(&refs);

    let polling_sample = collect_native_panel_polling_host_sample(handles, &refs);
    let now = Instant::now();
    let transition_request: Option<NativePanelTransitionRequest>;
    let transition_snapshot;
    let click_platform_event: Option<NativePanelPlatformEvent>;
    let click_command: PanelInteractionCommand;
    let host_behavior_commands;

    {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let pointer_regions = state.pointer_regions.clone();
        let snapshot = state.last_snapshot.clone();
        let polling_facts = polling_sample.with_state(pointer_regions, snapshot);
        let interaction = sync_native_panel_host_polling_interaction_from_host_facts_for_state(
            &mut *state,
            polling_facts.as_shared_facts(),
            now,
            HOVER_DELAY_MS,
            CARD_FOCUS_CLICK_DEBOUNCE_MS,
        );
        click_platform_event = interaction.click_platform_event;
        click_command = interaction.click_command;
        transition_request = interaction.transition_request;
        transition_snapshot = interaction.transition_snapshot;
        host_behavior_commands = interaction.host_behavior.commands;
        if interaction.interactive_inside {
            clear_pending_native_panel_close_transition_in_state(&mut state);
        }
    }

    for command in host_behavior_commands {
        match command {
            NativePanelHostBehaviorCommand::SetMouseEventPassthrough {
                ignores_mouse_events,
            } => panel_from_ptr(handles.panel).setIgnoresMouseEvents(ignores_mouse_events),
        }
    }

    let _ = dispatch_native_panel_transition_request_immediate_with_snapshot(
        app.clone(),
        transition_request,
        transition_snapshot,
    );

    if matches!(
        click_platform_event,
        Some(NativePanelPlatformEvent::MascotDebugClick)
    ) {
        let _ =
            handle_native_platform_event(app.clone(), NativePanelPlatformEvent::MascotDebugClick);
    }

    let _ = handle_native_click_command(app, click_command);
}

struct NativePanelPollingHostSample {
    pointer: crate::native_panel_core::PanelPoint,
    hover_frames: NativePanelHoverFallbackFrames,
    primary_mouse_down: bool,
    cards_visible: bool,
}

impl NativePanelPollingHostSample {
    fn with_state(
        self,
        pointer_regions: Vec<NativePanelPointerRegion>,
        snapshot: Option<RuntimeSnapshot>,
    ) -> NativePanelPollingHostFactsSnapshot {
        NativePanelPollingHostFactsSnapshot {
            pointer: self.pointer,
            pointer_regions,
            hover_frames: self.hover_frames,
            primary_mouse_down: self.primary_mouse_down,
            cards_visible: self.cards_visible,
            snapshot,
        }
    }
}

struct NativePanelPollingHostFactsSnapshot {
    pointer: crate::native_panel_core::PanelPoint,
    pointer_regions: Vec<NativePanelPointerRegion>,
    hover_frames: NativePanelHoverFallbackFrames,
    primary_mouse_down: bool,
    cards_visible: bool,
    snapshot: Option<RuntimeSnapshot>,
}

impl NativePanelPollingHostFactsSnapshot {
    fn as_shared_facts(&self) -> NativePanelPollingHostFacts<'_> {
        NativePanelPollingHostFacts {
            pointer: self.pointer,
            pointer_regions: &self.pointer_regions,
            hover_frames: self.hover_frames,
            primary_mouse_down: self.primary_mouse_down,
            cards_visible: self.cards_visible,
            snapshot: self.snapshot.clone(),
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn collect_native_panel_polling_host_sample(
    handles: NativePanelHandles,
    refs: &super::panel_refs::NativePanelRefs,
) -> NativePanelPollingHostSample {
    let panel = panel_from_ptr(handles.panel);
    let mouse = NSEvent::mouseLocation();
    let primary_mouse_down = (NSEvent::pressedMouseButtons() & 1) != 0;
    let panel_frame = panel.frame();
    let pill_frame = refs.pill_view.frame();
    let interactive_pill_frame = absolute_rect(panel_frame, pill_frame);
    let hover_pill_frame = native_hover_pill_rect(panel_frame, pill_frame);
    let expanded_container = view_from_ptr(handles.expanded_container);
    let cards_container = view_from_ptr(handles.cards_container);
    let interactive_expanded_frame = absolute_rect(panel_frame, expanded_container.frame());
    let pointer = crate::native_panel_core::PanelPoint {
        x: mouse.x,
        y: mouse.y,
    };
    let hover_frames = NativePanelHoverFallbackFrames {
        interactive_pill_frame: ns_rect_to_panel_rect(interactive_pill_frame),
        hover_pill_frame: ns_rect_to_panel_rect(hover_pill_frame),
        interactive_expanded_frame: (panel_frame.size.height > COLLAPSED_PANEL_HEIGHT + 0.5)
            .then(|| ns_rect_to_panel_rect(interactive_expanded_frame)),
    };

    NativePanelPollingHostSample {
        pointer,
        hover_frames,
        primary_mouse_down,
        cards_visible: !cards_container.isHidden(),
    }
}

#[cfg(test)]
pub(super) fn toggle_native_settings_surface(state: &mut NativePanelState) -> bool {
    toggle_native_panel_settings_surface_for_state(state)
}

#[cfg(test)]
pub(super) fn sync_native_hover_expansion_state(
    state: &mut NativePanelState,
    inside: bool,
    now: Instant,
) -> Option<NativeHoverTransition> {
    sync_native_panel_hover_expansion_state_for_state(state, inside, now, HOVER_DELAY_MS)
}

pub(super) fn native_status_surface_active() -> bool {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                guard.surface_mode == NativeExpandedSurface::Status
                    && !guard.status_queue.is_empty()
            })
        })
        .unwrap_or(false)
}
