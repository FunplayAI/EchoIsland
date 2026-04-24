use crate::{
    native_panel_core::{
        HoverTransition, PanelAnimationDescriptor, PanelHitTarget, PanelInteractionCommand,
        PanelLayout, PanelPoint, PanelRect, point_in_rect, resolve_native_panel_host_frame,
    },
    native_panel_scene::{PanelScene, PanelSceneBuildInput, SceneHitTarget},
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelRuntimeInputContext {
    pub(crate) display_count: usize,
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
    descriptor.visible = visible;
}

pub(crate) fn sync_native_panel_host_window_screen_frame(
    descriptor: &mut NativePanelHostWindowDescriptor,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
) {
    descriptor.preferred_display_index = preferred_display_index;
    descriptor.screen_frame = screen_frame;
}

pub(crate) fn sync_native_panel_host_window_shared_body_height(
    descriptor: &mut NativePanelHostWindowDescriptor,
    shared_body_height: Option<f64>,
) {
    descriptor.shared_body_height = shared_body_height;
}

pub(crate) fn sync_native_panel_host_window_timeline(
    descriptor: &mut NativePanelHostWindowDescriptor,
    timeline: Option<NativePanelTimelineDescriptor>,
) {
    descriptor.timeline = timeline;
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NativePanelPointerRegionFrameOverrides {
    pub(crate) settings_action: Option<PanelRect>,
    pub(crate) quit_action: Option<PanelRect>,
}

impl NativePanelPointerRegionFrameOverrides {
    fn edge_action_frame(self, action: NativePanelEdgeAction) -> Option<PanelRect> {
        match action {
            NativePanelEdgeAction::Settings => self.settings_action,
            NativePanelEdgeAction::Quit => self.quit_action,
        }
    }
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
    OpenReleasePage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NativePanelPointerInputOutcome {
    Hover(Option<HoverTransition>),
    Click(Option<NativePanelPlatformEvent>),
}

pub(crate) trait NativePanelPlatformEventHandler {
    type Error;

    fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error>;

    fn toggle_settings_surface(&mut self) -> Result<(), Self::Error>;

    fn quit_application(&mut self) -> Result<(), Self::Error>;

    fn cycle_display(&mut self) -> Result<(), Self::Error>;

    fn toggle_completion_sound(&mut self) -> Result<(), Self::Error>;

    fn toggle_mascot(&mut self) -> Result<(), Self::Error>;

    fn open_release_page(&mut self) -> Result<(), Self::Error>;
}

impl From<SceneHitTarget> for PanelHitTarget {
    fn from(value: SceneHitTarget) -> Self {
        Self {
            action: value.action,
            value: value.value,
        }
    }
}

pub(crate) fn dispatch_native_panel_platform_event<H>(
    handler: &mut H,
    event: NativePanelPlatformEvent,
) -> Result<(), H::Error>
where
    H: NativePanelPlatformEventHandler,
{
    match event {
        NativePanelPlatformEvent::FocusSession(session_id) => handler.focus_session(session_id),
        NativePanelPlatformEvent::ToggleSettingsSurface => handler.toggle_settings_surface(),
        NativePanelPlatformEvent::QuitApplication => handler.quit_application(),
        NativePanelPlatformEvent::CycleDisplay => handler.cycle_display(),
        NativePanelPlatformEvent::ToggleCompletionSound => handler.toggle_completion_sound(),
        NativePanelPlatformEvent::ToggleMascot => handler.toggle_mascot(),
        NativePanelPlatformEvent::OpenReleasePage => handler.open_release_page(),
    }
}

pub(crate) fn dispatch_native_panel_platform_events<H>(
    handler: &mut H,
    events: impl IntoIterator<Item = NativePanelPlatformEvent>,
) -> Result<(), H::Error>
where
    H: NativePanelPlatformEventHandler,
{
    for event in events {
        dispatch_native_panel_platform_event(handler, event)?;
    }
    Ok(())
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
    frame_overrides: Option<NativePanelPointerRegionFrameOverrides>,
) -> Vec<NativePanelPointerRegion> {
    let mut regions = Vec::new();

    push_region(
        &mut regions,
        absolute_panel_rect(layout, layout.content_frame),
        NativePanelPointerRegionKind::Shell,
    );
    push_region(
        &mut regions,
        absolute_panel_rect(layout, layout.pill_frame),
        NativePanelPointerRegionKind::CompactBar,
    );

    if layout.shell_visible {
        push_region(
            &mut regions,
            absolute_panel_rect(layout, layout.expanded_frame),
            NativePanelPointerRegionKind::Shell,
        );
        push_region(
            &mut regions,
            absolute_panel_rect(layout, layout.cards_frame),
            NativePanelPointerRegionKind::CardsContainer,
        );
        if scene.compact_bar.actions_visible {
            push_edge_action_regions(&mut regions, layout, frame_overrides.unwrap_or_default());
        }
        push_scene_hit_target_regions(&mut regions, layout, scene);
    }

    regions
}

fn push_edge_action_regions(
    regions: &mut Vec<NativePanelPointerRegion>,
    layout: PanelLayout,
    frame_overrides: NativePanelPointerRegionFrameOverrides,
) {
    let pill = absolute_panel_rect(layout, layout.pill_frame);
    let action_width = (pill.width * 0.18).clamp(36.0, 58.0);
    let settings_frame = frame_overrides
        .edge_action_frame(NativePanelEdgeAction::Settings)
        .unwrap_or(PanelRect {
            x: pill.x,
            y: pill.y,
            width: action_width,
            height: pill.height,
        });
    let quit_frame = frame_overrides
        .edge_action_frame(NativePanelEdgeAction::Quit)
        .unwrap_or(PanelRect {
            x: pill.x + pill.width - action_width,
            y: pill.y,
            width: action_width,
            height: pill.height,
        });
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

fn push_scene_hit_target_regions(
    regions: &mut Vec<NativePanelPointerRegion>,
    layout: PanelLayout,
    scene: &PanelScene,
) {
    if scene.hit_targets.is_empty() {
        return;
    }

    let cards = absolute_panel_rect(layout, layout.cards_frame);
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
        NativePanelEdgeAction, NativePanelPointerRegion, NativePanelPointerRegionFrameOverrides,
        NativePanelPointerRegionKind,
    };
    use super::{
        NativePanelHostWindowDescriptor, NativePanelPlatformEvent, NativePanelPointerInput,
        NativePanelPointerInputOutcome, NativePanelTimelineDescriptor,
        native_panel_hit_target_at_point, native_panel_host_window_descriptor,
        native_panel_host_window_frame, native_panel_platform_event_at_point,
        native_panel_platform_event_for_interaction_command,
        native_panel_platform_event_for_pointer_input,
        native_panel_platform_event_for_pointer_region, native_panel_pointer_input_outcome,
        native_panel_pointer_inside_for_input, native_panel_pointer_inside_regions,
        native_panel_pointer_region_at_point, native_panel_timeline_descriptor,
        native_panel_timeline_descriptor_for_animation, resolve_native_panel_pointer_regions,
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
    fn pointer_regions_use_default_edge_action_frames_without_platform_overrides() {
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
        let action_width = (pill.width * 0.18).clamp(36.0, 58.0);
        assert_eq!(
            settings_frame,
            PanelRect {
                x: pill.x,
                y: pill.y,
                width: action_width,
                height: pill.height,
            }
        );
    }

    #[test]
    fn pointer_regions_accept_platform_edge_action_frame_overrides() {
        let layout = pointer_test_layout();
        let scene = pointer_test_scene();
        let overrides = NativePanelPointerRegionFrameOverrides {
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
        };

        let regions = resolve_native_panel_pointer_regions(layout, &scene, Some(overrides));

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
            crate::native_panel_renderer::NativePanelHostWindowState {
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
