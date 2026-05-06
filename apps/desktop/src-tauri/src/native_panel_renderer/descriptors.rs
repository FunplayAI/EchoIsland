use crate::{
    native_panel_core::{
        CompactBarContentLayoutInput, HoverTransition, PanelAnimationDescriptor, PanelHitTarget,
        PanelInteractionCommand, PanelLayout, PanelPoint, PanelRect, point_in_rect,
        resolve_compact_action_button_layout, resolve_compact_bar_content_layout,
        resolve_native_panel_host_frame, resolve_settings_surface_card_height,
        settings_surface_row_frame,
    },
    native_panel_scene::{
        PanelDisplayOptionState, PanelScene, PanelSceneBuildInput, SceneCard, SceneHitTarget,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelRuntimeInputDescriptor {
    pub(crate) scene_input: PanelSceneBuildInput,
    pub(crate) screen_frame: Option<PanelRect>,
}

impl NativePanelRuntimeInputDescriptor {
    pub(crate) fn selected_display_index(&self) -> usize {
        self.scene_input.settings.selected_display_index
    }
}

pub(crate) fn native_panel_runtime_input_descriptor_with_screen_frame(
    screen_frame: Option<PanelRect>,
) -> NativePanelRuntimeInputDescriptor {
    NativePanelRuntimeInputDescriptor {
        scene_input: PanelSceneBuildInput::default(),
        screen_frame,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NativePanelRuntimeInputContext {
    pub(crate) display_options: Vec<PanelDisplayOptionState>,
    pub(crate) selected_display_index: usize,
    pub(crate) screen_frame: Option<PanelRect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelHostWindowState {
    pub(crate) frame: Option<PanelRect>,
    pub(crate) visible: bool,
    pub(crate) preferred_display_index: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelHostWindowDescriptor {
    pub(crate) visible: bool,
    pub(crate) preferred_display_index: usize,
    pub(crate) screen_frame: Option<PanelRect>,
    pub(crate) shared_body_height: Option<f64>,
    pub(crate) timeline: Option<NativePanelTimelineDescriptor>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelHostWindowDescriptorPatch {
    pub(crate) visible: Option<bool>,
    pub(crate) preferred_display_index: Option<usize>,
    pub(crate) screen_frame: Option<Option<PanelRect>>,
    pub(crate) shared_body_height: Option<Option<f64>>,
    pub(crate) timeline: Option<Option<NativePanelTimelineDescriptor>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelTimelineDescriptor {
    pub(crate) animation: PanelAnimationDescriptor,
    pub(crate) cards_entering: bool,
}

impl NativePanelHostWindowDescriptor {
    pub(crate) fn animation_descriptor(self) -> Option<PanelAnimationDescriptor> {
        self.timeline.map(|timeline| timeline.animation)
    }

    pub(crate) fn window_state(self, frame: Option<PanelRect>) -> NativePanelHostWindowState {
        NativePanelHostWindowState {
            frame,
            visible: self.visible,
            preferred_display_index: self.preferred_display_index,
        }
    }
}

pub(crate) fn native_panel_host_window_descriptor(
    visible: bool,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
    shared_body_height: Option<f64>,
    timeline: Option<NativePanelTimelineDescriptor>,
) -> NativePanelHostWindowDescriptor {
    NativePanelHostWindowDescriptor {
        visible,
        preferred_display_index,
        screen_frame,
        shared_body_height,
        timeline,
    }
}

pub(crate) fn native_panel_timeline_descriptor(
    animation: PanelAnimationDescriptor,
    cards_entering: bool,
) -> NativePanelTimelineDescriptor {
    NativePanelTimelineDescriptor {
        animation,
        cards_entering,
    }
}

pub(crate) fn native_panel_timeline_descriptor_for_animation(
    animation: PanelAnimationDescriptor,
) -> NativePanelTimelineDescriptor {
    native_panel_timeline_descriptor(
        animation,
        native_panel_cards_entering_for_animation(animation),
    )
}

pub(crate) fn native_panel_cards_entering_for_animation(
    animation: PanelAnimationDescriptor,
) -> bool {
    !matches!(
        animation.kind,
        crate::native_panel_core::PanelAnimationKind::Close
    )
}

pub(crate) fn sync_native_panel_host_window_visibility(
    descriptor: &mut NativePanelHostWindowDescriptor,
    visible: bool,
) {
    patch_native_panel_host_window_descriptor(
        descriptor,
        NativePanelHostWindowDescriptorPatch {
            visible: Some(visible),
            ..NativePanelHostWindowDescriptorPatch::default()
        },
    );
}

pub(crate) fn sync_native_panel_host_window_screen_frame(
    descriptor: &mut NativePanelHostWindowDescriptor,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
) {
    patch_native_panel_host_window_descriptor(
        descriptor,
        NativePanelHostWindowDescriptorPatch {
            preferred_display_index: Some(preferred_display_index),
            screen_frame: Some(screen_frame),
            ..NativePanelHostWindowDescriptorPatch::default()
        },
    );
}

pub(crate) fn sync_native_panel_host_window_shared_body_height(
    descriptor: &mut NativePanelHostWindowDescriptor,
    shared_body_height: Option<f64>,
) {
    patch_native_panel_host_window_descriptor(
        descriptor,
        NativePanelHostWindowDescriptorPatch {
            shared_body_height: Some(shared_body_height),
            ..NativePanelHostWindowDescriptorPatch::default()
        },
    );
}

pub(crate) fn sync_native_panel_host_window_timeline(
    descriptor: &mut NativePanelHostWindowDescriptor,
    timeline: Option<NativePanelTimelineDescriptor>,
) {
    patch_native_panel_host_window_descriptor(
        descriptor,
        NativePanelHostWindowDescriptorPatch {
            timeline: Some(timeline),
            ..NativePanelHostWindowDescriptorPatch::default()
        },
    );
}

pub(crate) fn patch_native_panel_host_window_descriptor(
    descriptor: &mut NativePanelHostWindowDescriptor,
    patch: NativePanelHostWindowDescriptorPatch,
) {
    if let Some(visible) = patch.visible {
        descriptor.visible = visible;
    }
    if let Some(preferred_display_index) = patch.preferred_display_index {
        descriptor.preferred_display_index = preferred_display_index;
    }
    if let Some(screen_frame) = patch.screen_frame {
        descriptor.screen_frame = screen_frame;
    }
    if let Some(shared_body_height) = patch.shared_body_height {
        descriptor.shared_body_height = shared_body_height;
    }
    if let Some(timeline) = patch.timeline {
        descriptor.timeline = timeline;
    }
}

pub(crate) fn native_panel_host_window_frame(
    descriptor: NativePanelHostWindowDescriptor,
    fallback_screen_frame: PanelRect,
    compact_width: f64,
    expanded_width: f64,
) -> Option<PanelRect> {
    Some(resolve_native_panel_host_frame(
        descriptor.animation_descriptor()?,
        descriptor.screen_frame.unwrap_or(fallback_screen_frame),
        compact_width,
        expanded_width,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelPointerRegionKind {
    Shell,
    CompactBar,
    CardsContainer,
    MascotDebugTrigger,
    EdgeAction(NativePanelEdgeAction),
    HitTarget(PanelHitTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelEdgeAction {
    Settings,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelPointerRegion {
    pub(crate) frame: PanelRect,
    pub(crate) kind: NativePanelPointerRegionKind,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelInteractionPlan {
    pub(crate) pointer_regions: Vec<NativePanelPointerRegion>,
}

impl NativePanelInteractionPlan {
    pub(crate) fn from_pointer_regions(regions: &[NativePanelPointerRegion]) -> Self {
        Self {
            pointer_regions: regions.to_vec(),
        }
    }

    pub(crate) fn pointer_region_at_point(
        &self,
        point: PanelPoint,
    ) -> Option<&NativePanelPointerRegion> {
        native_panel_pointer_region_at_point(&self.pointer_regions, point)
    }

    pub(crate) fn inside_regions(&self, point: PanelPoint) -> bool {
        self.pointer_region_at_point(point).is_some()
    }

    pub(crate) fn pointer_state_at_point(&self, point: PanelPoint) -> NativePanelPointerPointState {
        native_panel_pointer_state_at_point(&self.pointer_regions, point)
    }

    pub(crate) fn platform_event_at_point(
        &self,
        point: PanelPoint,
    ) -> Option<NativePanelPlatformEvent> {
        native_panel_platform_event_at_point(&self.pointer_regions, point)
    }

    pub(crate) fn input_outcome(
        &self,
        input: NativePanelPointerInput,
    ) -> NativePanelPointerInputOutcome {
        native_panel_pointer_input_outcome(&self.pointer_regions, input)
    }

    pub(crate) fn inside_for_input(&self, input: NativePanelPointerInput) -> Option<bool> {
        native_panel_pointer_inside_for_input(&self.pointer_regions, input)
    }

    pub(crate) fn hit_target_at_point(&self, point: PanelPoint) -> Option<PanelHitTarget> {
        native_panel_hit_target_at_point(&self.pointer_regions, point)
    }

    pub(crate) fn queue_platform_event_at_point(
        &self,
        events: &mut Vec<NativePanelPlatformEvent>,
        point: PanelPoint,
    ) -> Option<NativePanelPlatformEvent> {
        queue_native_panel_platform_event(events, self.platform_event_at_point(point))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativePanelPointerPointState {
    pub(crate) inside: bool,
    pub(crate) platform_event: Option<NativePanelPlatformEvent>,
    pub(crate) hit_target: Option<PanelHitTarget>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelEdgeActionFrames {
    pub(crate) settings_action: Option<PanelRect>,
    pub(crate) quit_action: Option<PanelRect>,
}

impl NativePanelEdgeActionFrames {
    fn edge_action_frame(self, action: NativePanelEdgeAction) -> Option<PanelRect> {
        match action {
            NativePanelEdgeAction::Settings => self.settings_action,
            NativePanelEdgeAction::Quit => self.quit_action,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelPointerRegionInput {
    pub(crate) edge_action_frames: NativePanelEdgeActionFrames,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativePanelPointerInput {
    Move(PanelPoint),
    Click(PanelPoint),
    Leave,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelPlatformEvent {
    FocusSession(String),
    ToggleSettingsSurface,
    QuitApplication,
    CycleDisplay,
    ToggleCompletionSound,
    ToggleMascot,
    MascotDebugClick,
    OpenSettingsLocation,
    OpenReleasePage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelRuntimeCommand {
    FocusSession(String),
    ToggleSettingsSurface,
    QuitApplication,
    CycleDisplay,
    ToggleCompletionSound,
    ToggleMascot,
    MascotDebugClick,
    OpenSettingsLocation,
    OpenReleasePage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelPointerInputOutcome {
    Hover(Option<HoverTransition>),
    Click(Option<NativePanelPlatformEvent>),
}

impl NativePanelPointerInputOutcome {
    pub(crate) fn into_hover_transition(self) -> Option<HoverTransition> {
        match self {
            NativePanelPointerInputOutcome::Hover(transition) => transition,
            NativePanelPointerInputOutcome::Click(_) => None,
        }
    }

    pub(crate) fn into_click_event(self) -> Option<NativePanelPlatformEvent> {
        match self {
            NativePanelPointerInputOutcome::Click(event) => event,
            NativePanelPointerInputOutcome::Hover(_) => None,
        }
    }
}

pub(crate) trait NativePanelRuntimeCommandCapability {
    type Error;

    fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error>;

    fn toggle_settings_surface(&mut self) -> Result<(), Self::Error>;

    fn quit_application(&mut self) -> Result<(), Self::Error>;

    fn cycle_display(&mut self) -> Result<(), Self::Error>;

    fn toggle_completion_sound(&mut self) -> Result<(), Self::Error>;

    fn toggle_mascot(&mut self) -> Result<(), Self::Error>;

    fn mascot_debug_click(&mut self) -> Result<(), Self::Error>;

    fn open_settings_location(&mut self) -> Result<(), Self::Error>;

    fn open_release_page(&mut self) -> Result<(), Self::Error>;
}

pub(crate) trait NativePanelRuntimeCommandHandler:
    NativePanelRuntimeCommandCapability
{
    fn execute_runtime_command(
        &mut self,
        command: NativePanelRuntimeCommand,
    ) -> Result<(), Self::Error> {
        match command {
            NativePanelRuntimeCommand::FocusSession(session_id) => self.focus_session(session_id),
            NativePanelRuntimeCommand::ToggleSettingsSurface => self.toggle_settings_surface(),
            NativePanelRuntimeCommand::QuitApplication => self.quit_application(),
            NativePanelRuntimeCommand::CycleDisplay => self.cycle_display(),
            NativePanelRuntimeCommand::ToggleCompletionSound => self.toggle_completion_sound(),
            NativePanelRuntimeCommand::ToggleMascot => self.toggle_mascot(),
            NativePanelRuntimeCommand::MascotDebugClick => self.mascot_debug_click(),
            NativePanelRuntimeCommand::OpenSettingsLocation => self.open_settings_location(),
            NativePanelRuntimeCommand::OpenReleasePage => self.open_release_page(),
        }
    }
}

impl<T> NativePanelRuntimeCommandHandler for T where T: NativePanelRuntimeCommandCapability {}

#[derive(Default)]
pub(crate) struct NativePanelQueuedRuntimeCommandHandler {
    events: Vec<NativePanelPlatformEvent>,
}

impl NativePanelQueuedRuntimeCommandHandler {
    pub(crate) fn take_events(self) -> Vec<NativePanelPlatformEvent> {
        self.events
    }
}

impl NativePanelRuntimeCommandCapability for NativePanelQueuedRuntimeCommandHandler {
    type Error = String;

    fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error> {
        self.events
            .push(NativePanelPlatformEvent::FocusSession(session_id));
        Ok(())
    }

    fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
        self.events
            .push(NativePanelPlatformEvent::ToggleSettingsSurface);
        Ok(())
    }

    fn quit_application(&mut self) -> Result<(), Self::Error> {
        self.events.push(NativePanelPlatformEvent::QuitApplication);
        Ok(())
    }

    fn cycle_display(&mut self) -> Result<(), Self::Error> {
        self.events.push(NativePanelPlatformEvent::CycleDisplay);
        Ok(())
    }

    fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
        self.events
            .push(NativePanelPlatformEvent::ToggleCompletionSound);
        Ok(())
    }

    fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
        self.events.push(NativePanelPlatformEvent::ToggleMascot);
        Ok(())
    }

    fn mascot_debug_click(&mut self) -> Result<(), Self::Error> {
        self.events.push(NativePanelPlatformEvent::MascotDebugClick);
        Ok(())
    }

    fn open_settings_location(&mut self) -> Result<(), Self::Error> {
        self.events
            .push(NativePanelPlatformEvent::OpenSettingsLocation);
        Ok(())
    }

    fn open_release_page(&mut self) -> Result<(), Self::Error> {
        self.events.push(NativePanelPlatformEvent::OpenReleasePage);
        Ok(())
    }
}

impl From<SceneHitTarget> for PanelHitTarget {
    fn from(value: SceneHitTarget) -> Self {
        Self {
            action: value.action,
            value: value.value,
        }
    }
}

pub(crate) fn native_panel_runtime_command_for_platform_event(
    event: NativePanelPlatformEvent,
) -> NativePanelRuntimeCommand {
    match event {
        NativePanelPlatformEvent::FocusSession(session_id) => {
            NativePanelRuntimeCommand::FocusSession(session_id)
        }
        NativePanelPlatformEvent::ToggleSettingsSurface => {
            NativePanelRuntimeCommand::ToggleSettingsSurface
        }
        NativePanelPlatformEvent::QuitApplication => NativePanelRuntimeCommand::QuitApplication,
        NativePanelPlatformEvent::CycleDisplay => NativePanelRuntimeCommand::CycleDisplay,
        NativePanelPlatformEvent::ToggleCompletionSound => {
            NativePanelRuntimeCommand::ToggleCompletionSound
        }
        NativePanelPlatformEvent::ToggleMascot => NativePanelRuntimeCommand::ToggleMascot,
        NativePanelPlatformEvent::MascotDebugClick => NativePanelRuntimeCommand::MascotDebugClick,
        NativePanelPlatformEvent::OpenSettingsLocation => {
            NativePanelRuntimeCommand::OpenSettingsLocation
        }
        NativePanelPlatformEvent::OpenReleasePage => NativePanelRuntimeCommand::OpenReleasePage,
    }
}

pub(crate) fn dispatch_native_panel_runtime_command<H>(
    handler: &mut H,
    command: NativePanelRuntimeCommand,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeCommandHandler,
{
    handler.execute_runtime_command(command)
}

pub(crate) fn dispatch_native_panel_runtime_commands<H>(
    handler: &mut H,
    commands: impl IntoIterator<Item = NativePanelRuntimeCommand>,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeCommandHandler,
{
    for command in commands {
        dispatch_native_panel_runtime_command(handler, command)?;
    }
    Ok(())
}

pub(crate) fn dispatch_native_panel_platform_event<H>(
    handler: &mut H,
    event: NativePanelPlatformEvent,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeCommandHandler,
{
    dispatch_native_panel_runtime_command(
        handler,
        native_panel_runtime_command_for_platform_event(event),
    )
}

pub(crate) fn dispatch_native_panel_platform_events<H>(
    handler: &mut H,
    events: impl IntoIterator<Item = NativePanelPlatformEvent>,
) -> Result<(), H::Error>
where
    H: NativePanelRuntimeCommandHandler,
{
    dispatch_native_panel_runtime_commands(
        handler,
        events
            .into_iter()
            .map(native_panel_runtime_command_for_platform_event),
    )
}

pub(crate) fn native_panel_platform_event_for_hit_target(
    target: &PanelHitTarget,
) -> NativePanelPlatformEvent {
    match target.action {
        crate::native_panel_core::PanelHitAction::FocusSession => {
            NativePanelPlatformEvent::FocusSession(target.value.clone())
        }
        crate::native_panel_core::PanelHitAction::CycleDisplay => {
            NativePanelPlatformEvent::CycleDisplay
        }
        crate::native_panel_core::PanelHitAction::ToggleCompletionSound => {
            NativePanelPlatformEvent::ToggleCompletionSound
        }
        crate::native_panel_core::PanelHitAction::ToggleMascot => {
            NativePanelPlatformEvent::ToggleMascot
        }
        crate::native_panel_core::PanelHitAction::OpenSettingsLocation => {
            NativePanelPlatformEvent::OpenSettingsLocation
        }
        crate::native_panel_core::PanelHitAction::OpenReleasePage => {
            NativePanelPlatformEvent::OpenReleasePage
        }
    }
}

pub(crate) fn native_panel_platform_event_for_pointer_region(
    region: &NativePanelPointerRegion,
) -> Option<NativePanelPlatformEvent> {
    match &region.kind {
        NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings) => {
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        }
        NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Quit) => {
            Some(NativePanelPlatformEvent::QuitApplication)
        }
        NativePanelPointerRegionKind::MascotDebugTrigger => {
            Some(NativePanelPlatformEvent::MascotDebugClick)
        }
        NativePanelPointerRegionKind::HitTarget(target) => {
            Some(native_panel_platform_event_for_hit_target(target))
        }
        NativePanelPointerRegionKind::Shell
        | NativePanelPointerRegionKind::CompactBar
        | NativePanelPointerRegionKind::CardsContainer => None,
    }
}

pub(crate) fn native_panel_pointer_region_at_point<'a>(
    regions: &'a [NativePanelPointerRegion],
    point: PanelPoint,
) -> Option<&'a NativePanelPointerRegion> {
    regions
        .iter()
        .rev()
        .find(|region| point_in_rect(point, region.frame))
}

pub(crate) fn native_panel_pointer_inside_regions(
    regions: &[NativePanelPointerRegion],
    point: PanelPoint,
) -> bool {
    native_panel_pointer_region_at_point(regions, point).is_some()
}

pub(crate) fn native_panel_platform_event_at_point(
    regions: &[NativePanelPointerRegion],
    point: PanelPoint,
) -> Option<NativePanelPlatformEvent> {
    native_panel_pointer_region_at_point(regions, point)
        .and_then(native_panel_platform_event_for_pointer_region)
}

pub(crate) fn queue_native_panel_platform_event(
    events: &mut Vec<NativePanelPlatformEvent>,
    event: Option<NativePanelPlatformEvent>,
) -> Option<NativePanelPlatformEvent> {
    if let Some(event) = event.clone() {
        events.push(event);
    }
    event
}

pub(crate) fn queue_native_panel_platform_event_for_pointer_region(
    events: &mut Vec<NativePanelPlatformEvent>,
    region: &NativePanelPointerRegion,
) -> Option<NativePanelPlatformEvent> {
    queue_native_panel_platform_event(
        events,
        native_panel_platform_event_for_pointer_region(region),
    )
}

pub(crate) fn queue_native_panel_platform_event_at_point(
    events: &mut Vec<NativePanelPlatformEvent>,
    regions: &[NativePanelPointerRegion],
    point: PanelPoint,
) -> Option<NativePanelPlatformEvent> {
    queue_native_panel_platform_event(events, native_panel_platform_event_at_point(regions, point))
}

pub(crate) fn native_panel_pointer_state_at_point(
    regions: &[NativePanelPointerRegion],
    point: PanelPoint,
) -> NativePanelPointerPointState {
    let region = native_panel_pointer_region_at_point(regions, point);
    NativePanelPointerPointState {
        inside: region.is_some(),
        platform_event: region.and_then(native_panel_platform_event_for_pointer_region),
        hit_target: match region.map(|region| &region.kind) {
            Some(NativePanelPointerRegionKind::HitTarget(target)) => Some(target.clone()),
            _ => None,
        },
    }
}

pub(crate) fn native_panel_pointer_inside_for_input(
    regions: &[NativePanelPointerRegion],
    input: NativePanelPointerInput,
) -> Option<bool> {
    match input {
        NativePanelPointerInput::Move(point) => {
            Some(native_panel_pointer_inside_regions(regions, point))
        }
        NativePanelPointerInput::Leave => Some(false),
        NativePanelPointerInput::Click(_) => None,
    }
}

pub(crate) fn native_panel_platform_event_for_pointer_input(
    regions: &[NativePanelPointerRegion],
    input: NativePanelPointerInput,
) -> Option<NativePanelPlatformEvent> {
    match input {
        NativePanelPointerInput::Click(point) => {
            native_panel_platform_event_at_point(regions, point)
        }
        NativePanelPointerInput::Move(_) | NativePanelPointerInput::Leave => None,
    }
}

pub(crate) fn native_panel_hit_target_at_point(
    regions: &[NativePanelPointerRegion],
    point: PanelPoint,
) -> Option<PanelHitTarget> {
    match &native_panel_pointer_region_at_point(regions, point)?.kind {
        NativePanelPointerRegionKind::HitTarget(target) => Some(target.clone()),
        NativePanelPointerRegionKind::Shell
        | NativePanelPointerRegionKind::CompactBar
        | NativePanelPointerRegionKind::CardsContainer
        | NativePanelPointerRegionKind::MascotDebugTrigger
        | NativePanelPointerRegionKind::EdgeAction(_) => None,
    }
}

pub(crate) fn native_panel_pointer_input_outcome(
    regions: &[NativePanelPointerRegion],
    input: NativePanelPointerInput,
) -> NativePanelPointerInputOutcome {
    match input {
        NativePanelPointerInput::Move(point) => NativePanelPointerInputOutcome::Hover(
            native_panel_pointer_inside_regions(regions, point).then_some(HoverTransition::Expand),
        ),
        NativePanelPointerInput::Leave => {
            NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Collapse))
        }
        NativePanelPointerInput::Click(point) => NativePanelPointerInputOutcome::Click(
            native_panel_platform_event_at_point(regions, point),
        ),
    }
}

pub(crate) fn native_panel_platform_event_for_interaction_command(
    command: &PanelInteractionCommand,
) -> Option<NativePanelPlatformEvent> {
    match command {
        PanelInteractionCommand::HitTarget(target) => {
            Some(native_panel_platform_event_for_hit_target(target))
        }
        PanelInteractionCommand::ToggleSettingsSurface => {
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        }
        PanelInteractionCommand::QuitApplication => Some(NativePanelPlatformEvent::QuitApplication),
        PanelInteractionCommand::None => None,
    }
}

pub(crate) fn resolve_native_panel_pointer_regions(
    layout: PanelLayout,
    scene: &PanelScene,
    input: Option<NativePanelPointerRegionInput>,
) -> Vec<NativePanelPointerRegion> {
    resolve_native_panel_interaction_plan(layout, scene, input).pointer_regions
}

pub(crate) fn resolve_native_panel_interaction_plan(
    layout: PanelLayout,
    scene: &PanelScene,
    input: Option<NativePanelPointerRegionInput>,
) -> NativePanelInteractionPlan {
    let mut regions = Vec::new();

    push_region(
        &mut regions,
        absolute_panel_rect(layout, layout.pill_frame),
        NativePanelPointerRegionKind::CompactBar,
    );
    push_mascot_bubble_hover_region(&mut regions, layout, scene);

    if layout.shell_visible {
        push_region(
            &mut regions,
            absolute_panel_rect(layout, layout.expanded_frame),
            NativePanelPointerRegionKind::Shell,
        );
        push_expanded_mascot_debug_region(&mut regions, layout, scene);
        push_expanded_top_gap_region(&mut regions, layout);
        push_region(
            &mut regions,
            absolute_panel_rect(layout, layout.cards_frame),
            NativePanelPointerRegionKind::CardsContainer,
        );
        if scene.compact_bar.actions_visible {
            push_edge_action_regions(
                &mut regions,
                layout,
                input.unwrap_or_default().edge_action_frames,
            );
        }
        push_scene_hit_target_regions(&mut regions, layout, scene);
    }

    NativePanelInteractionPlan {
        pointer_regions: regions,
    }
}

fn push_expanded_mascot_debug_region(
    regions: &mut Vec<NativePanelPointerRegion>,
    layout: PanelLayout,
    scene: &PanelScene,
) {
    if scene.mascot_pose == crate::native_panel_scene::SceneMascotPose::Hidden {
        return;
    }

    let pill = absolute_panel_rect(layout, layout.pill_frame);
    let compact_content = resolve_compact_bar_content_layout(CompactBarContentLayoutInput {
        bar_width: layout.pill_frame.width,
        bar_height: layout.pill_frame.height,
    });
    let mascot_center = PanelPoint {
        x: pill.x + compact_content.mascot_center_x,
        y: pill.y + pill.height / 2.0,
    };
    push_region(
        regions,
        PanelRect {
            x: mascot_center.x - 18.0,
            y: mascot_center.y - 18.0,
            width: 36.0,
            height: 36.0,
        },
        NativePanelPointerRegionKind::MascotDebugTrigger,
    );
}

fn push_expanded_top_gap_region(regions: &mut Vec<NativePanelPointerRegion>, layout: PanelLayout) {
    let gap_y = layout.expanded_frame.y + layout.expanded_frame.height;
    let gap_height = (layout.content_frame.height - gap_y).max(0.0);
    if gap_height <= 0.0 {
        return;
    }
    push_region(
        regions,
        absolute_panel_rect(
            layout,
            PanelRect {
                x: layout.expanded_frame.x,
                y: gap_y,
                width: layout.expanded_frame.width,
                height: gap_height,
            },
        ),
        NativePanelPointerRegionKind::Shell,
    );
}

fn push_mascot_bubble_hover_region(
    regions: &mut Vec<NativePanelPointerRegion>,
    layout: PanelLayout,
    scene: &PanelScene,
) {
    let has_bubble = scene.compact_bar.completion_count > 0
        || scene.mascot_pose == crate::native_panel_scene::SceneMascotPose::MessageBubble;
    if !has_bubble {
        return;
    }

    let pill = absolute_panel_rect(layout, layout.pill_frame);
    push_region(
        regions,
        PanelRect {
            x: pill.x + 20.0,
            y: pill.y + pill.height - 3.0,
            width: 30.0,
            height: 18.0,
        },
        NativePanelPointerRegionKind::CompactBar,
    );
}

fn push_edge_action_regions(
    regions: &mut Vec<NativePanelPointerRegion>,
    layout: PanelLayout,
    edge_action_frames: NativePanelEdgeActionFrames,
) {
    let pill = absolute_panel_rect(layout, layout.pill_frame);
    let action_layout = resolve_compact_action_button_layout(pill);
    let settings_frame = edge_action_frames
        .edge_action_frame(NativePanelEdgeAction::Settings)
        .unwrap_or_else(|| edge_action_hit_frame(action_layout.settings, pill));
    let quit_frame = edge_action_frames
        .edge_action_frame(NativePanelEdgeAction::Quit)
        .unwrap_or_else(|| edge_action_hit_frame(action_layout.quit, pill));
    push_region(
        regions,
        settings_frame,
        NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings),
    );
    push_region(
        regions,
        quit_frame,
        NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Quit),
    );
}

fn edge_action_hit_frame(icon_frame: PanelRect, pill: PanelRect) -> PanelRect {
    let horizontal_padding = 5.0;
    PanelRect {
        x: icon_frame.x - horizontal_padding,
        y: pill.y,
        width: icon_frame.width + horizontal_padding * 2.0,
        height: pill.height,
    }
}

fn push_scene_hit_target_regions(
    regions: &mut Vec<NativePanelPointerRegion>,
    layout: PanelLayout,
    scene: &PanelScene,
) {
    if scene.hit_targets.is_empty() {
        return;
    }

    let cards = absolute_panel_rect(layout, layout.cards_frame);
    if push_settings_hit_target_regions(regions, cards, scene) {
        return;
    }

    let target_count = scene.hit_targets.len();
    let row_height = cards.height / target_count as f64;
    for (index, target) in scene.hit_targets.iter().cloned().enumerate() {
        push_region(
            regions,
            PanelRect {
                x: cards.x,
                y: cards.y + cards.height - row_height * (index + 1) as f64,
                width: cards.width,
                height: row_height,
            },
            NativePanelPointerRegionKind::HitTarget(target.into()),
        );
    }
}

fn push_settings_hit_target_regions(
    regions: &mut Vec<NativePanelPointerRegion>,
    cards: PanelRect,
    scene: &PanelScene,
) -> bool {
    let Some(SceneCard::Settings { rows, .. }) = scene.cards.first() else {
        return false;
    };
    let card_height = resolve_settings_surface_card_height(rows.len());
    let card_frame = PanelRect {
        x: cards.x,
        y: cards.y + (cards.height - card_height).max(0.0),
        width: cards.width,
        height: card_height,
    };
    for (index, target) in scene.hit_targets.iter().cloned().enumerate() {
        push_region(
            regions,
            settings_surface_row_frame(card_frame, index),
            NativePanelPointerRegionKind::HitTarget(target.into()),
        );
    }
    true
}

fn absolute_panel_rect(layout: PanelLayout, local_frame: PanelRect) -> PanelRect {
    crate::native_panel_core::absolute_rect(layout.panel_frame, local_frame)
}

fn push_region(
    regions: &mut Vec<NativePanelPointerRegion>,
    frame: PanelRect,
    kind: NativePanelPointerRegionKind,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }
    regions.push(NativePanelPointerRegion { frame, kind });
}

#[cfg(test)]
mod tests {
    use echoisland_runtime::RuntimeSnapshot;

    use crate::native_panel_core::{
        ExpandedSurface, HoverTransition, PanelGeometryMetrics, PanelHitAction, PanelHitTarget,
        PanelInteractionCommand, PanelLayout, PanelLayoutInput, PanelPoint, PanelRect, PanelState,
        resolve_panel_layout,
    };
    use crate::native_panel_scene::{PanelSceneBuildInput, build_panel_scene};

    use super::{
        NativePanelEdgeAction, NativePanelEdgeActionFrames, NativePanelPointerRegion,
        NativePanelPointerRegionInput, NativePanelPointerRegionKind,
    };
    use super::{
        NativePanelHostWindowDescriptor, NativePanelHostWindowDescriptorPatch,
        NativePanelHostWindowState, NativePanelPlatformEvent, NativePanelPointerInput,
        NativePanelPointerInputOutcome, NativePanelRuntimeCommand, NativePanelTimelineDescriptor,
        absolute_panel_rect, native_panel_hit_target_at_point, native_panel_host_window_descriptor,
        native_panel_host_window_frame, native_panel_platform_event_at_point,
        native_panel_platform_event_for_interaction_command,
        native_panel_platform_event_for_pointer_input,
        native_panel_platform_event_for_pointer_region, native_panel_pointer_input_outcome,
        native_panel_pointer_inside_for_input, native_panel_pointer_inside_regions,
        native_panel_pointer_region_at_point, native_panel_pointer_state_at_point,
        native_panel_runtime_command_for_platform_event, native_panel_timeline_descriptor,
        native_panel_timeline_descriptor_for_animation, patch_native_panel_host_window_descriptor,
        queue_native_panel_platform_event_at_point,
        queue_native_panel_platform_event_for_pointer_region,
        resolve_native_panel_interaction_plan, resolve_native_panel_pointer_regions,
        sync_native_panel_host_window_screen_frame,
        sync_native_panel_host_window_shared_body_height, sync_native_panel_host_window_timeline,
        sync_native_panel_host_window_visibility,
    };

    fn pointer_test_layout() -> PanelLayout {
        resolve_panel_layout(PanelLayoutInput {
            screen_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            },
            metrics: PanelGeometryMetrics {
                compact_height: 38.0,
                compact_width: 253.0,
                expanded_width: 283.0,
                panel_width: 420.0,
            },
            canvas_height: 180.0,
            visible_height: 180.0,
            bar_progress: 1.0,
            height_progress: 1.0,
            drop_progress: 1.0,
            content_visibility: 1.0,
            collapsed_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            drop_distance: crate::native_panel_core::PANEL_DROP_DISTANCE,
            content_top_gap: crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP,
            content_bottom_inset: crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET,
            cards_side_inset: crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
            shoulder_size: crate::native_panel_core::COMPACT_SHOULDER_SIZE,
            separator_side_inset: crate::native_panel_core::EXPANDED_SEPARATOR_SIDE_INSET,
        })
    }

    fn pointer_test_scene() -> crate::native_panel_scene::PanelScene {
        let mut state = PanelState {
            expanded: true,
            surface_mode: ExpandedSurface::Default,
            ..PanelState::default()
        };
        state.transitioning = false;
        build_panel_scene(
            &state,
            &RuntimeSnapshot {
                status: "Idle".to_string(),
                primary_source: "claude".to_string(),
                active_session_count: 0,
                total_session_count: 0,
                pending_permission_count: 0,
                pending_question_count: 0,
                pending_permission: None,
                pending_question: None,
                pending_permissions: Vec::new(),
                pending_questions: Vec::new(),
                sessions: Vec::new(),
            },
            &PanelSceneBuildInput::default(),
        )
    }

    #[test]
    fn interaction_command_maps_to_platform_event() {
        assert_eq!(
            native_panel_platform_event_for_interaction_command(
                &PanelInteractionCommand::HitTarget(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                })
            ),
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            native_panel_platform_event_for_interaction_command(
                &PanelInteractionCommand::ToggleSettingsSurface
            ),
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        );
        assert_eq!(
            native_panel_platform_event_for_interaction_command(
                &PanelInteractionCommand::QuitApplication
            ),
            Some(NativePanelPlatformEvent::QuitApplication)
        );
        assert_eq!(
            native_panel_platform_event_for_interaction_command(&PanelInteractionCommand::None),
            None
        );
    }

    #[test]
    fn platform_event_maps_to_runtime_command() {
        assert_eq!(
            native_panel_runtime_command_for_platform_event(
                NativePanelPlatformEvent::FocusSession("session-1".to_string())
            ),
            NativePanelRuntimeCommand::FocusSession("session-1".to_string())
        );
        assert_eq!(
            native_panel_runtime_command_for_platform_event(
                NativePanelPlatformEvent::ToggleCompletionSound
            ),
            NativePanelRuntimeCommand::ToggleCompletionSound
        );
        assert_eq!(
            native_panel_runtime_command_for_platform_event(
                NativePanelPlatformEvent::OpenReleasePage
            ),
            NativePanelRuntimeCommand::OpenReleasePage
        );
    }

    #[test]
    fn pointer_input_outcome_projects_expected_variant_payload() {
        assert_eq!(
            NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Expand))
                .into_hover_transition(),
            Some(HoverTransition::Expand)
        );
        assert_eq!(
            NativePanelPointerInputOutcome::Click(Some(NativePanelPlatformEvent::QuitApplication))
                .into_click_event(),
            Some(NativePanelPlatformEvent::QuitApplication)
        );
        assert_eq!(
            NativePanelPointerInputOutcome::Click(Some(NativePanelPlatformEvent::QuitApplication))
                .into_hover_transition(),
            None
        );
        assert_eq!(
            NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Collapse))
                .into_click_event(),
            None
        );
    }

    #[test]
    fn pointer_region_maps_to_platform_event() {
        let frame = PanelRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        assert_eq!(
            native_panel_platform_event_for_pointer_region(&NativePanelPointerRegion {
                frame,
                kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings),
            }),
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        );
        assert_eq!(
            native_panel_platform_event_for_pointer_region(&NativePanelPointerRegion {
                frame,
                kind: NativePanelPointerRegionKind::CompactBar,
            }),
            None
        );
    }

    #[test]
    fn point_hit_testing_prefers_topmost_pointer_region() {
        let regions = vec![
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
                kind: NativePanelPointerRegionKind::CardsContainer,
            },
            NativePanelPointerRegion {
                frame: PanelRect {
                    x: 10.0,
                    y: 10.0,
                    width: 40.0,
                    height: 40.0,
                },
                kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                }),
            },
        ];
        let point = PanelPoint { x: 20.0, y: 20.0 };

        assert!(matches!(
            native_panel_pointer_region_at_point(&regions, point).map(|region| &region.kind),
            Some(NativePanelPointerRegionKind::HitTarget(target))
                if target.value == "session-1"
        ));
        assert_eq!(
            native_panel_platform_event_at_point(&regions, point),
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            native_panel_platform_event_at_point(&regions, PanelPoint { x: 80.0, y: 80.0 }),
            None
        );
        assert!(native_panel_pointer_inside_regions(
            &regions,
            PanelPoint { x: 80.0, y: 80.0 }
        ));
        assert!(!native_panel_pointer_inside_regions(
            &regions,
            PanelPoint { x: 180.0, y: 180.0 }
        ));
    }

    #[test]
    fn queue_pointer_region_platform_event_pushes_focus_event() {
        let mut events = Vec::new();
        let region = NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        };

        let event = queue_native_panel_platform_event_for_pointer_region(&mut events, &region);

        assert_eq!(
            event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            events,
            vec![NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )]
        );
    }

    #[test]
    fn queue_platform_event_at_point_pushes_hit_target_event() {
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        }];
        let mut events = Vec::new();

        let event = queue_native_panel_platform_event_at_point(
            &mut events,
            &regions,
            PanelPoint { x: 20.0, y: 20.0 },
        );

        assert_eq!(
            event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            events,
            vec![NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )]
        );
    }

    #[test]
    fn pointer_input_resolves_hover_and_click_semantics() {
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        }];

        assert_eq!(
            native_panel_pointer_inside_for_input(
                &regions,
                NativePanelPointerInput::Move(PanelPoint { x: 20.0, y: 20.0 })
            ),
            Some(true)
        );
        assert_eq!(
            native_panel_pointer_inside_for_input(&regions, NativePanelPointerInput::Leave),
            Some(false)
        );
        assert_eq!(
            native_panel_pointer_inside_for_input(
                &regions,
                NativePanelPointerInput::Click(PanelPoint { x: 20.0, y: 20.0 })
            ),
            None
        );
        assert_eq!(
            native_panel_platform_event_for_pointer_input(
                &regions,
                NativePanelPointerInput::Click(PanelPoint { x: 20.0, y: 20.0 })
            ),
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            native_panel_platform_event_for_pointer_input(
                &regions,
                NativePanelPointerInput::Move(PanelPoint { x: 20.0, y: 20.0 })
            ),
            None
        );
        assert_eq!(
            native_panel_hit_target_at_point(&regions, PanelPoint { x: 20.0, y: 20.0 }),
            Some(PanelHitTarget {
                action: crate::native_panel_core::PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            })
        );
        assert_eq!(
            native_panel_pointer_input_outcome(
                &regions,
                NativePanelPointerInput::Move(PanelPoint { x: 20.0, y: 20.0 })
            ),
            NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Expand))
        );
        assert_eq!(
            native_panel_pointer_input_outcome(&regions, NativePanelPointerInput::Leave),
            NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Collapse))
        );
        assert_eq!(
            native_panel_pointer_input_outcome(
                &regions,
                NativePanelPointerInput::Click(PanelPoint { x: 20.0, y: 20.0 })
            ),
            NativePanelPointerInputOutcome::Click(Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )))
        );
    }

    #[test]
    fn pointer_state_at_point_collects_inside_event_and_hit_target() {
        let regions = vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        }];

        let state = native_panel_pointer_state_at_point(&regions, PanelPoint { x: 20.0, y: 20.0 });

        assert!(state.inside);
        assert_eq!(
            state.platform_event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            state.hit_target,
            Some(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            })
        );
    }

    #[test]
    fn pointer_regions_use_default_edge_action_frames_without_platform_input() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();

        let regions = resolve_native_panel_pointer_regions(layout, &scene, None);
        let settings_frame = regions
            .iter()
            .find_map(|region| match region.kind {
                NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings) => {
                    Some(region.frame)
                }
                _ => None,
            })
            .expect("settings pointer region");

        let pill = crate::native_panel_core::absolute_rect(layout.panel_frame, layout.pill_frame);
        let action_layout = crate::native_panel_core::resolve_compact_action_button_layout(pill);
        assert_eq!(
            settings_frame,
            PanelRect {
                x: action_layout.settings.x - 5.0,
                y: pill.y,
                width: action_layout.settings.width + 10.0,
                height: pill.height,
            }
        );
    }

    #[test]
    fn interaction_plan_carries_shared_pointer_regions() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();

        let plan = resolve_native_panel_interaction_plan(layout, &scene, None);
        let regions = resolve_native_panel_pointer_regions(layout, &scene, None);

        assert_eq!(plan.pointer_regions, regions);
        assert!(
            plan.pointer_regions
                .iter()
                .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CompactBar))
        );
    }

    #[test]
    fn interaction_plan_resolves_pointer_semantics() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();

        let plan = resolve_native_panel_interaction_plan(layout, &scene, None);
        let settings = plan
            .pointer_regions
            .iter()
            .find(|region| {
                matches!(
                    region.kind,
                    NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings)
                )
            })
            .expect("settings region")
            .frame;
        let point = PanelPoint {
            x: settings.x + settings.width / 2.0,
            y: settings.y + settings.height / 2.0,
        };

        assert_eq!(
            plan.platform_event_at_point(point),
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        );
        assert_eq!(
            plan.input_outcome(NativePanelPointerInput::Click(point)),
            NativePanelPointerInputOutcome::Click(Some(
                NativePanelPlatformEvent::ToggleSettingsSurface
            ))
        );
        assert!(plan.pointer_state_at_point(point).inside);
        assert_eq!(
            plan.inside_for_input(NativePanelPointerInput::Move(point)),
            Some(true)
        );
        assert_eq!(
            plan.inside_for_input(NativePanelPointerInput::Leave),
            Some(false)
        );
        assert!(plan.hit_target_at_point(point).is_none());

        let mut events = Vec::new();
        assert_eq!(
            plan.queue_platform_event_at_point(&mut events, point),
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        );
        assert_eq!(
            events,
            vec![NativePanelPlatformEvent::ToggleSettingsSurface]
        );
    }

    #[test]
    fn pointer_regions_accept_platform_edge_action_frame_input() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();
        let input = NativePanelPointerRegionInput {
            edge_action_frames: NativePanelEdgeActionFrames {
                settings_action: Some(PanelRect {
                    x: 640.5,
                    y: 824.0,
                    width: 26.0,
                    height: 26.0,
                }),
                quit_action: Some(PanelRect {
                    x: 774.0,
                    y: 824.0,
                    width: 22.0,
                    height: 22.0,
                }),
            },
        };

        let regions = resolve_native_panel_pointer_regions(layout, &scene, Some(input));

        assert_eq!(
            native_panel_platform_event_at_point(&regions, PanelPoint { x: 645.0, y: 830.0 }),
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        );
        assert_eq!(
            native_panel_platform_event_at_point(&regions, PanelPoint { x: 780.0, y: 830.0 }),
            Some(NativePanelPlatformEvent::QuitApplication)
        );
    }

    #[test]
    fn settings_pointer_regions_match_visible_rows_without_settings_folder_action() {
        let layout = resolve_panel_layout(PanelLayoutInput {
            screen_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            },
            metrics: PanelGeometryMetrics {
                compact_height: 38.0,
                compact_width: 253.0,
                expanded_width: 283.0,
                panel_width: 420.0,
            },
            canvas_height: 260.0,
            visible_height: 260.0,
            bar_progress: 1.0,
            height_progress: 1.0,
            drop_progress: 1.0,
            content_visibility: 1.0,
            collapsed_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            drop_distance: crate::native_panel_core::PANEL_DROP_DISTANCE,
            content_top_gap: crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP,
            content_bottom_inset: crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET,
            cards_side_inset: crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
            shoulder_size: crate::native_panel_core::COMPACT_SHOULDER_SIZE,
            separator_side_inset: crate::native_panel_core::EXPANDED_SEPARATOR_SIDE_INSET,
        });
        let state = PanelState {
            expanded: true,
            surface_mode: ExpandedSurface::Settings,
            ..PanelState::default()
        };
        let scene = build_panel_scene(
            &state,
            &RuntimeSnapshot {
                status: "Idle".to_string(),
                primary_source: "claude".to_string(),
                active_session_count: 0,
                total_session_count: 0,
                pending_permission_count: 0,
                pending_question_count: 0,
                pending_permission: None,
                pending_question: None,
                pending_permissions: Vec::new(),
                pending_questions: Vec::new(),
                sessions: Vec::new(),
            },
            &PanelSceneBuildInput::default(),
        );

        let regions = resolve_native_panel_pointer_regions(layout, &scene, None);
        let hit_regions = regions
            .iter()
            .filter(|region| matches!(region.kind, NativePanelPointerRegionKind::HitTarget(_)))
            .collect::<Vec<_>>();

        assert_eq!(hit_regions.len(), 4);
        assert!(!hit_regions.iter().any(|region| matches!(
            region.kind,
            NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                action: PanelHitAction::OpenSettingsLocation,
                ..
            })
        )));

        let mute_region = hit_regions
            .iter()
            .find(|region| {
                matches!(
                    region.kind,
                    NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                        action: PanelHitAction::ToggleCompletionSound,
                        ..
                    })
                )
            })
            .expect("mute sound region");
        assert_eq!(
            native_panel_platform_event_at_point(
                &regions,
                PanelPoint {
                    x: mute_region.frame.x + 8.0,
                    y: mute_region.frame.y + 2.0,
                },
            ),
            Some(NativePanelPlatformEvent::ToggleCompletionSound)
        );
    }

    #[test]
    fn pointer_regions_do_not_claim_transparent_canvas_margins() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();
        let regions = resolve_native_panel_pointer_regions(layout, &scene, None);

        let content_frame = absolute_panel_rect(layout, layout.content_frame);
        assert!(!regions.iter().any(|region| {
            matches!(region.kind, NativePanelPointerRegionKind::Shell)
                && region.frame == content_frame
        }));
        assert!(!native_panel_pointer_inside_regions(
            &regions,
            PanelPoint {
                x: layout.panel_frame.x + 10.0,
                y: layout.panel_frame.y + layout.panel_frame.height - 2.0,
            }
        ));
        assert!(native_panel_pointer_inside_regions(
            &regions,
            PanelPoint {
                x: layout.panel_frame.x + layout.pill_frame.x + 20.0,
                y: layout.panel_frame.y + layout.pill_frame.y + 20.0,
            }
        ));
    }

    #[test]
    fn pointer_regions_claim_expanded_top_gap_without_claiming_side_margins() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();
        let regions = resolve_native_panel_pointer_regions(layout, &scene, None);
        let shell = absolute_panel_rect(layout, layout.expanded_frame);

        assert!(native_panel_pointer_inside_regions(
            &regions,
            PanelPoint {
                x: shell.x + shell.width / 2.0,
                y: shell.y + shell.height + 1.0,
            }
        ));
        assert!(!native_panel_pointer_inside_regions(
            &regions,
            PanelPoint {
                x: layout.panel_frame.x + 10.0,
                y: shell.y + shell.height + 1.0,
            }
        ));
    }

    #[test]
    fn pointer_regions_include_mascot_bubble_hover_overhang() {
        let layout = pointer_test_layout();
        let mut scene = pointer_test_scene();
        scene.compact_bar.completion_count = 2;
        scene.mascot_pose = crate::native_panel_scene::SceneMascotPose::Complete;

        let regions = resolve_native_panel_pointer_regions(layout, &scene, None);
        let pill = absolute_panel_rect(layout, layout.pill_frame);

        assert!(native_panel_pointer_inside_regions(
            &regions,
            PanelPoint {
                x: pill.x + 42.0,
                y: pill.y + pill.height + 4.0,
            }
        ));
    }

    #[test]
    fn host_window_descriptor_projects_animation_and_window_state() {
        let descriptor = NativePanelHostWindowDescriptor {
            visible: true,
            preferred_display_index: 2,
            screen_frame: Some(PanelRect {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            }),
            shared_body_height: Some(180.0),
            timeline: Some(NativePanelTimelineDescriptor {
                animation: crate::native_panel_core::PanelAnimationDescriptor {
                    kind: crate::native_panel_core::PanelAnimationKind::Open,
                    canvas_height: 140.0,
                    visible_height: 120.0,
                    width_progress: 0.5,
                    height_progress: 0.75,
                    shoulder_progress: 1.0,
                    drop_progress: 0.25,
                    cards_progress: 0.8,
                },
                cards_entering: true,
            }),
        };

        assert_eq!(
            descriptor.animation_descriptor(),
            descriptor.timeline.map(|timeline| timeline.animation)
        );
        assert_eq!(
            descriptor.window_state(Some(PanelRect {
                x: 30.0,
                y: 40.0,
                width: 160.0,
                height: 100.0,
            })),
            NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 30.0,
                    y: 40.0,
                    width: 160.0,
                    height: 100.0,
                }),
                visible: true,
                preferred_display_index: 2,
            }
        );
        assert_eq!(
            native_panel_host_window_frame(
                descriptor,
                PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1440.0,
                    height: 900.0,
                },
                400.0,
                700.0,
            ),
            Some(PanelRect {
                x: 10.0,
                y: 80.0,
                width: 550.0,
                height: 140.0,
            })
        );
    }

    #[test]
    fn host_window_descriptor_helpers_update_shared_fields() {
        let animation = crate::native_panel_core::PanelAnimationDescriptor {
            kind: crate::native_panel_core::PanelAnimationKind::Open,
            canvas_height: 140.0,
            visible_height: 120.0,
            width_progress: 0.5,
            height_progress: 0.75,
            shoulder_progress: 1.0,
            drop_progress: 0.25,
            cards_progress: 0.8,
        };
        let timeline = native_panel_timeline_descriptor(animation, true);
        let mut descriptor = native_panel_host_window_descriptor(false, 0, None, None, None);

        sync_native_panel_host_window_visibility(&mut descriptor, true);
        sync_native_panel_host_window_screen_frame(
            &mut descriptor,
            2,
            Some(PanelRect {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 200.0,
            }),
        );
        sync_native_panel_host_window_shared_body_height(&mut descriptor, Some(180.0));
        sync_native_panel_host_window_timeline(&mut descriptor, Some(timeline));

        assert!(descriptor.visible);
        assert_eq!(descriptor.preferred_display_index, 2);
        assert_eq!(descriptor.shared_body_height, Some(180.0));
        assert_eq!(descriptor.timeline, Some(timeline));
    }

    #[test]
    fn host_window_descriptor_patch_updates_multiple_fields_together() {
        let animation = crate::native_panel_core::PanelAnimationDescriptor {
            kind: crate::native_panel_core::PanelAnimationKind::Open,
            canvas_height: 140.0,
            visible_height: 120.0,
            width_progress: 0.5,
            height_progress: 0.75,
            shoulder_progress: 1.0,
            drop_progress: 0.25,
            cards_progress: 0.8,
        };
        let timeline = native_panel_timeline_descriptor(animation, true);
        let mut descriptor = native_panel_host_window_descriptor(false, 0, None, None, None);

        patch_native_panel_host_window_descriptor(
            &mut descriptor,
            NativePanelHostWindowDescriptorPatch {
                visible: Some(true),
                preferred_display_index: Some(3),
                screen_frame: Some(Some(PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 300.0,
                    height: 200.0,
                })),
                shared_body_height: Some(Some(180.0)),
                timeline: Some(Some(timeline)),
            },
        );

        assert!(descriptor.visible);
        assert_eq!(descriptor.preferred_display_index, 3);
        assert_eq!(descriptor.shared_body_height, Some(180.0));
        assert_eq!(descriptor.timeline, Some(timeline));
    }

    #[test]
    fn timeline_descriptor_for_animation_derives_card_direction() {
        let mut animation = crate::native_panel_core::PanelAnimationDescriptor {
            kind: crate::native_panel_core::PanelAnimationKind::Open,
            canvas_height: 140.0,
            visible_height: 120.0,
            width_progress: 0.5,
            height_progress: 0.75,
            shoulder_progress: 1.0,
            drop_progress: 0.25,
            cards_progress: 0.8,
        };

        assert!(native_panel_timeline_descriptor_for_animation(animation).cards_entering);

        animation.kind = crate::native_panel_core::PanelAnimationKind::SurfaceSwitch;
        assert!(native_panel_timeline_descriptor_for_animation(animation).cards_entering);

        animation.kind = crate::native_panel_core::PanelAnimationKind::Close;
        assert!(!native_panel_timeline_descriptor_for_animation(animation).cards_entering);
    }
}
