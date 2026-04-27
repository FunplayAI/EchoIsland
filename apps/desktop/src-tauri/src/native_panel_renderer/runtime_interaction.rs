use std::time::Instant;

use echoisland_runtime::RuntimeSnapshot;

use crate::native_panel_core::{
    HoverTransition, LastFocusClick, PanelInteractionCommand, PanelPoint, PanelRect, PanelState,
};

use super::descriptors::{
    NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerPointState,
    NativePanelPointerRegion, NativePanelRuntimeCommandHandler, NativePanelRuntimeInputDescriptor,
    native_panel_platform_event_at_point, native_panel_pointer_inside_for_input,
    native_panel_pointer_state_at_point, queue_native_panel_platform_event,
    queue_native_panel_platform_event_for_pointer_region,
};
use super::runtime_scene_cache::NativePanelRuntimeSceneCache;
use super::traits::{NativePanelHost, NativePanelSceneHost};
use super::transition_controller::NativePanelTransitionRequest;

pub(crate) trait NativePanelCoreStateBridge {
    fn snapshot_core_panel_state(&self) -> PanelState;

    fn apply_core_panel_state(&mut self, core: PanelState);
}

impl NativePanelCoreStateBridge for PanelState {
    fn snapshot_core_panel_state(&self) -> PanelState {
        self.clone()
    }

    fn apply_core_panel_state(&mut self, core: PanelState) {
        *self = core;
    }
}

pub(crate) trait NativePanelClickStateBridge {
    fn click_expanded(&self) -> bool;

    fn click_transitioning(&self) -> bool;

    fn click_last_focus_click(&self) -> Option<LastFocusClick<'_>>;

    fn record_click_focus_session(&mut self, session_id: String, now: Instant);
}

pub(crate) fn resolve_native_panel_last_focus_click(
    last_focus_click: Option<&(String, Instant)>,
) -> Option<LastFocusClick<'_>> {
    last_focus_click.map(|(session_id, clicked_at)| LastFocusClick {
        session_id,
        clicked_at: *clicked_at,
    })
}

pub(crate) fn record_native_panel_focus_click_session(
    last_focus_click: &mut Option<(String, Instant)>,
    session_id: String,
    now: Instant,
) {
    *last_focus_click = Some((session_id, now));
}

pub(crate) struct NativePanelClickStateSlots<'a> {
    panel_state: &'a PanelState,
    last_focus_click: &'a mut Option<(String, Instant)>,
}

pub(crate) fn native_panel_click_state_slots<'a>(
    panel_state: &'a PanelState,
    last_focus_click: &'a mut Option<(String, Instant)>,
) -> NativePanelClickStateSlots<'a> {
    NativePanelClickStateSlots {
        panel_state,
        last_focus_click,
    }
}

impl NativePanelClickStateBridge for NativePanelClickStateSlots<'_> {
    fn click_expanded(&self) -> bool {
        self.panel_state.expanded
    }

    fn click_transitioning(&self) -> bool {
        self.panel_state.transitioning
    }

    fn click_last_focus_click(&self) -> Option<LastFocusClick<'_>> {
        resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
    }

    fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
        record_native_panel_focus_click_session(self.last_focus_click, session_id, now);
    }
}

pub(crate) trait NativePanelPrimaryPointerStateBridge {
    fn primary_pointer_down(&self) -> bool;

    fn set_primary_pointer_down(&mut self, down: bool);
}

pub(crate) trait NativePanelHostInteractionStateBridge {
    fn host_ignores_mouse_events(&self) -> bool;

    fn set_host_ignores_mouse_events(&mut self, ignores_mouse_events: bool);
}

pub(crate) trait NativePanelPointerInputRuntimeBridge {
    type Error;

    fn sync_mouse_passthrough_for_pointer_input(&mut self, input: NativePanelPointerInput);

    fn record_pointer_input(&mut self, input: NativePanelPointerInput);

    fn sync_hover_and_refresh_for_pointer_input(
        &mut self,
        input: NativePanelPointerInput,
        now: Instant,
        runtime_input: &NativePanelRuntimeInputDescriptor,
    ) -> Result<Option<HoverTransition>, Self::Error>;

    fn dispatch_click_command_for_pointer_point<H>(
        &mut self,
        point: PanelPoint,
        now: Instant,
        handler: &mut H,
    ) -> Result<Option<NativePanelPlatformEvent>, Self::Error>
    where
        H: NativePanelRuntimeCommandHandler<Error = Self::Error>;
}

pub(crate) trait NativePanelSceneRuntimeBridge {
    type Host: NativePanelSceneHost;
    type State: NativePanelCoreStateBridge;

    fn with_runtime_scene_slots<T>(
        &mut self,
        f: impl FnOnce(
            &mut Option<NativePanelTransitionRequest>,
            &mut Self::Host,
            &mut NativePanelRuntimeSceneCache,
            &mut Self::State,
        ) -> T,
    ) -> T;
}

pub(crate) trait NativePanelClickInteractionHost {
    fn click_pointer_state_at_point(&self, point: PanelPoint) -> NativePanelPointerPointState;

    fn click_cards_visible(&self) -> bool;
}

pub(crate) trait NativePanelHoverInteractionHost {
    fn hover_inside_at_point(&self, point: PanelPoint) -> bool;

    fn hover_inside_for_input(&self, input: NativePanelPointerInput) -> Option<bool>;
}

pub(crate) trait NativePanelPointerRegionInteractionBridge {
    fn interaction_pointer_regions(&self) -> &[NativePanelPointerRegion];

    fn interaction_cards_visible(&self) -> bool;
}

pub(crate) trait NativePanelQueuedPlatformEventBridge {
    fn queued_platform_events_mut(&mut self) -> &mut Vec<NativePanelPlatformEvent>;

    fn queued_pointer_regions(&self) -> &[NativePanelPointerRegion];

    fn queue_platform_event_for_pointer_region(
        &mut self,
        region: &NativePanelPointerRegion,
    ) -> Option<NativePanelPlatformEvent> {
        queue_native_panel_platform_event_for_pointer_region(
            self.queued_platform_events_mut(),
            region,
        )
    }

    fn queue_platform_event_at_point(
        &mut self,
        point: PanelPoint,
    ) -> Option<NativePanelPlatformEvent> {
        let event = native_panel_platform_event_at_point(self.queued_pointer_regions(), point);
        queue_native_panel_platform_event(self.queued_platform_events_mut(), event)
    }
}

impl<T> NativePanelClickInteractionHost for T
where
    T: NativePanelPointerRegionInteractionBridge,
{
    fn click_pointer_state_at_point(&self, point: PanelPoint) -> NativePanelPointerPointState {
        native_panel_pointer_state_at_point(self.interaction_pointer_regions(), point)
    }

    fn click_cards_visible(&self) -> bool {
        self.interaction_cards_visible()
    }
}

impl<T> NativePanelHoverInteractionHost for T
where
    T: NativePanelPointerRegionInteractionBridge,
{
    fn hover_inside_at_point(&self, point: PanelPoint) -> bool {
        native_panel_pointer_state_at_point(self.interaction_pointer_regions(), point).inside
    }

    fn hover_inside_for_input(&self, input: NativePanelPointerInput) -> Option<bool> {
        native_panel_pointer_inside_for_input(self.interaction_pointer_regions(), input)
    }
}

pub(crate) trait NativePanelQueuedPlatformEventSource {
    fn take_queued_platform_events(&mut self) -> Vec<NativePanelPlatformEvent>;
}

impl<T> NativePanelQueuedPlatformEventSource for T
where
    T: NativePanelHost,
{
    fn take_queued_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        self.take_platform_events()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativePanelHoverSyncResult {
    pub(crate) transition: Option<HoverTransition>,
    pub(crate) request: Option<NativePanelTransitionRequest>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativePanelSettingsSurfaceToggleResult {
    pub(crate) changed: bool,
    pub(crate) transition_request: Option<NativePanelTransitionRequest>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelSettingsSurfaceSnapshotUpdate {
    pub(crate) snapshot: Option<RuntimeSnapshot>,
    pub(crate) transition_request: Option<NativePanelTransitionRequest>,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelPollingInteractionInput {
    pub(crate) pointer_state: NativePanelPointerPointState,
    pub(crate) pointer_regions_available: bool,
    pub(crate) fallback_hover: NativePanelHoverFallbackState,
    pub(crate) primary_mouse_down: bool,
    pub(crate) cards_visible: bool,
    pub(crate) snapshot: Option<RuntimeSnapshot>,
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelPollingHostFacts<'a> {
    pub(crate) pointer: PanelPoint,
    pub(crate) pointer_regions: &'a [NativePanelPointerRegion],
    pub(crate) hover_frames: NativePanelHoverFallbackFrames,
    pub(crate) primary_mouse_down: bool,
    pub(crate) cards_visible: bool,
    pub(crate) snapshot: Option<RuntimeSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelHoverFallbackFrames {
    pub(crate) interactive_pill_frame: PanelRect,
    pub(crate) hover_pill_frame: PanelRect,
    pub(crate) interactive_expanded_frame: Option<PanelRect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativePanelHoverFallbackState {
    pub(crate) interactive_inside: bool,
    pub(crate) hover_inside: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativePanelHostInteractionState {
    pub(crate) interactive_inside: bool,
    pub(crate) ignores_mouse_events: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelPollingInteractionResult {
    pub(crate) interactive_inside: bool,
    pub(crate) click_command: PanelInteractionCommand,
    pub(crate) transition_request: Option<NativePanelTransitionRequest>,
    pub(crate) transition_snapshot: Option<RuntimeSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelHostPollingInteractionResult {
    pub(crate) interactive_inside: bool,
    pub(crate) click_command: PanelInteractionCommand,
    pub(crate) transition_request: Option<NativePanelTransitionRequest>,
    pub(crate) transition_snapshot: Option<RuntimeSnapshot>,
    pub(crate) next_ignores_mouse_events: bool,
    pub(crate) sync_mouse_event_passthrough: bool,
}
