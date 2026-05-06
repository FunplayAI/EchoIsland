use std::time::Instant;

use chrono::{DateTime, Utc};
use echoisland_runtime::RuntimeSnapshot;

use crate::native_panel_core::{
    HoverTransition, PanelInteractionCommand, PanelPoint, PanelRect, PanelState,
};

use super::descriptors::{
    NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
    NativePanelPointerPointState, NativePanelPointerRegion, NativePanelRuntimeCommandHandler,
    NativePanelRuntimeInputDescriptor,
};
use super::runtime_backend::{
    NativePanelRuntimeSceneSyncResult, apply_runtime_scene_sync_result_to_host,
    sync_runtime_scene_bundle_from_input_descriptor,
};
use super::runtime_click::{
    dispatch_native_panel_click_command_at_point_with_handler,
    dispatch_native_panel_click_command_with_handler,
    dispatch_queued_native_panel_platform_events_with_handler,
    resolve_native_panel_click_command_for_pointer_state,
};
use super::runtime_hover::{
    sync_native_panel_hover_expansion_state_for_state,
    sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor,
    sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor,
    sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor,
    sync_native_panel_hover_interaction_at_point_for_state,
    sync_native_panel_hover_interaction_for_pointer_input_for_state,
    sync_native_panel_hover_interaction_for_state,
};
use super::runtime_interaction::{
    NativePanelClickInteractionHost, NativePanelClickStateBridge, NativePanelCoreStateBridge,
    NativePanelHostInteractionState, NativePanelHostInteractionStateBridge,
    NativePanelHostPollingInteractionResult, NativePanelHoverFallbackFrames,
    NativePanelHoverFallbackState, NativePanelHoverInteractionHost, NativePanelHoverSyncResult,
    NativePanelPointerInputRuntimeBridge, NativePanelPointerRegionInteractionBridge,
    NativePanelPollingHostFacts, NativePanelPollingInteractionInput,
    NativePanelPollingInteractionResult, NativePanelPrimaryPointerStateBridge,
    NativePanelQueuedPlatformEventBridge, NativePanelQueuedPlatformEventSource,
    NativePanelSceneRuntimeBridge, NativePanelSettingsSurfaceSnapshotUpdate,
    NativePanelSettingsSurfaceToggleResult, native_panel_click_state_slots,
    record_native_panel_focus_click_session, resolve_native_panel_last_focus_click,
};
use super::runtime_pointer_input::{
    handle_native_panel_pointer_input_with_handler,
    handle_optional_native_panel_pointer_input_with_handler,
};
use super::runtime_polling::{
    native_panel_interactive_inside_for_polling_input,
    native_panel_interactive_inside_from_host_facts, native_panel_polling_interaction_input,
    native_panel_polling_interaction_input_from_host_facts,
    resolve_native_panel_host_interaction_state, resolve_native_panel_hover_fallback_frames,
    resolve_native_panel_hover_fallback_state,
    sync_native_panel_host_polling_interaction_for_state,
    sync_native_panel_mouse_passthrough_for_interactive_inside,
    sync_native_panel_polling_interaction_for_state,
};
use super::runtime_scene_cache::NativePanelRuntimeSceneCache;
use super::runtime_scene_sync::rerender_runtime_scene_sync_result_to_host_on_transition_for_state_with_input_descriptor;
use super::runtime_settings_surface::{
    mutate_native_panel_core_state,
    resolve_native_panel_settings_surface_snapshot_update_for_state,
    toggle_native_panel_settings_surface_transition_request_for_state,
};
use super::runtime_transition_slots::{
    apply_native_panel_hover_sync_result_slot,
    apply_native_panel_runtime_scene_sync_result_with_transition_slot,
    apply_native_panel_settings_surface_toggle_result_slot,
};
use super::traits::{NativePanelHost, NativePanelRenderer, NativePanelSceneHost};
use super::transition_controller::native_panel_transition_request_for_surface_change;

#[cfg(test)]
mod tests {
    use super::{
        NativePanelClickInteractionHost, NativePanelClickStateBridge, NativePanelCoreStateBridge,
        NativePanelHostInteractionStateBridge, NativePanelHoverFallbackFrames,
        NativePanelHoverFallbackState, NativePanelHoverInteractionHost, NativePanelHoverSyncResult,
        NativePanelPointerInputRuntimeBridge, NativePanelPointerRegionInteractionBridge,
        NativePanelPollingHostFacts, NativePanelPollingInteractionInput,
        NativePanelPrimaryPointerStateBridge, NativePanelQueuedPlatformEventBridge,
        NativePanelQueuedPlatformEventSource, apply_native_panel_hover_sync_result_slot,
        dispatch_native_panel_click_command_at_point_with_handler,
        dispatch_native_panel_click_command_with_handler,
        dispatch_queued_native_panel_platform_events_with_handler,
        handle_native_panel_pointer_input_with_handler, native_panel_click_state_slots,
        native_panel_polling_interaction_input,
        native_panel_polling_interaction_input_from_host_facts,
        record_native_panel_focus_click_session,
        resolve_native_panel_click_command_for_pointer_state,
        resolve_native_panel_hover_fallback_frames, resolve_native_panel_last_focus_click,
        resolve_native_panel_settings_surface_snapshot_update_for_state,
        sync_native_panel_host_polling_interaction_for_state,
        sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor,
        sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor,
        sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor,
        sync_native_panel_hover_interaction_at_point_for_state,
        sync_native_panel_hover_interaction_for_pointer_input_for_state,
        sync_native_panel_hover_interaction_for_state,
        sync_native_panel_polling_interaction_for_state,
        toggle_native_panel_settings_surface_transition_request_for_state,
    };
    use crate::native_panel_core::{
        ExpandedSurface, HoverTransition, LastFocusClick, PanelHitAction, PanelHitTarget,
        PanelInteractionCommand, PanelPoint, PanelRect, PanelState,
    };
    use crate::native_panel_renderer::{
        descriptors::{
            NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPlatformEvent,
            NativePanelPointerInput, NativePanelPointerPointState, NativePanelPointerRegion,
            NativePanelPointerRegionKind, NativePanelRuntimeCommandCapability,
            NativePanelRuntimeCommandHandler, NativePanelRuntimeInputDescriptor,
            NativePanelTimelineDescriptor,
        },
        runtime_scene_cache::NativePanelRuntimeSceneCache,
        traits::{NativePanelHost, NativePanelSceneHost},
        transition_controller::NativePanelTransitionRequest,
        visual_plan::{NativePanelVisualDisplayMode, NativePanelVisualPlanInput},
    };
    use crate::native_panel_scene::{
        PanelRuntimeRenderState, PanelScene, PanelSceneBuildInput, SceneMascotPose,
    };
    use chrono::Utc;
    use std::time::{Duration, Instant};

    #[derive(Default)]
    struct TestClickState {
        expanded: bool,
        transitioning: bool,
        primary_mouse_down: bool,
        last_focus_click: Option<(String, Instant)>,
    }

    impl NativePanelClickStateBridge for TestClickState {
        fn click_expanded(&self) -> bool {
            self.expanded
        }

        fn click_transitioning(&self) -> bool {
            self.transitioning
        }

        fn click_last_focus_click(&self) -> Option<LastFocusClick<'_>> {
            resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
        }

        fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
            record_native_panel_focus_click_session(&mut self.last_focus_click, session_id, now);
        }
    }

    impl NativePanelCoreStateBridge for TestClickState {
        fn snapshot_core_panel_state(&self) -> PanelState {
            PanelState {
                expanded: self.expanded,
                transitioning: self.transitioning,
                ..PanelState::default()
            }
        }

        fn apply_core_panel_state(&mut self, core: PanelState) {
            self.expanded = core.expanded;
            self.transitioning = core.transitioning;
        }
    }

    fn visual_input(display_mode: NativePanelVisualDisplayMode) -> NativePanelVisualPlanInput {
        NativePanelVisualPlanInput {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 20.0,
                    width: 320.0,
                    height: 160.0,
                }),
                visible: display_mode != NativePanelVisualDisplayMode::Hidden,
                preferred_display_index: 0,
            },
            display_mode,
            surface: ExpandedSurface::Default,
            panel_frame: PanelRect {
                x: 100.0,
                y: 20.0,
                width: 320.0,
                height: 160.0,
            },
            compact_bar_frame: PanelRect {
                x: 40.0,
                y: 12.0,
                width: 240.0,
                height: 36.0,
            },
            left_shoulder_frame: PanelRect {
                x: 34.0,
                y: 42.0,
                width: 6.0,
                height: 6.0,
            },
            right_shoulder_frame: PanelRect {
                x: 280.0,
                y: 42.0,
                width: 6.0,
                height: 6.0,
            },
            shoulder_progress: 0.0,
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            card_stack_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            card_stack_content_height: 160.0,
            shell_frame: PanelRect {
                x: 20.0,
                y: 0.0,
                width: 280.0,
                height: 160.0,
            },
            headline_text: "EchoIsland".to_string(),
            headline_emphasized: false,
            active_count: "1".to_string(),
            active_count_elapsed_ms: 0,
            total_count: "3".to_string(),
            separator_visibility: 0.0,
            cards_visible: false,
            card_count: 0,
            cards: Vec::new(),
            glow_visible: false,
            action_buttons_visible: false,
            action_buttons: Vec::new(),
            completion_count: 0,
            mascot_elapsed_ms: 0,
            mascot_pose: SceneMascotPose::Idle,
            mascot_debug_mode_enabled: false,
        }
    }

    impl NativePanelPrimaryPointerStateBridge for TestClickState {
        fn primary_pointer_down(&self) -> bool {
            self.primary_mouse_down
        }

        fn set_primary_pointer_down(&mut self, down: bool) {
            self.primary_mouse_down = down;
        }
    }

    #[derive(Default)]
    struct RecordingHandler {
        handled: Vec<NativePanelPlatformEvent>,
    }

    struct TestInteractionHost {
        pointer_state: NativePanelPointerPointState,
        cards_visible: bool,
        hover_inside: bool,
        hover_inside_for_input: Option<bool>,
        rerender_count: usize,
        last_scene: Option<PanelScene>,
        host_descriptor: NativePanelHostWindowDescriptor,
        render_error: Option<String>,
    }

    impl Default for TestInteractionHost {
        fn default() -> Self {
            Self {
                pointer_state: NativePanelPointerPointState {
                    inside: false,
                    platform_event: None,
                    hit_target: None,
                },
                cards_visible: false,
                hover_inside: false,
                hover_inside_for_input: Some(false),
                rerender_count: 0,
                last_scene: None,
                host_descriptor: NativePanelHostWindowDescriptor::default(),
                render_error: None,
            }
        }
    }

    impl NativePanelClickInteractionHost for TestInteractionHost {
        fn click_pointer_state_at_point(&self, _point: PanelPoint) -> NativePanelPointerPointState {
            self.pointer_state.clone()
        }

        fn click_cards_visible(&self) -> bool {
            self.cards_visible
        }
    }

    impl NativePanelHoverInteractionHost for TestInteractionHost {
        fn hover_inside_at_point(&self, _point: PanelPoint) -> bool {
            self.hover_inside
        }

        fn hover_inside_for_input(&self, _input: NativePanelPointerInput) -> Option<bool> {
            self.hover_inside_for_input
        }
    }

    impl NativePanelHost for TestInteractionHost {
        type Error = String;
        type Renderer = Self;

        fn renderer(&mut self) -> &mut Self::Renderer {
            self
        }

        fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
            self.host_descriptor
        }

        fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
            &mut self.host_descriptor
        }

        fn window_state(&self) -> NativePanelHostWindowState {
            self.host_descriptor.window_state(None)
        }

        fn show(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn hide(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl crate::native_panel_renderer::traits::NativePanelRenderer for TestInteractionHost {
        type Error = String;

        fn render_scene(
            &mut self,
            scene: &PanelScene,
            _runtime: PanelRuntimeRenderState,
        ) -> Result<(), Self::Error> {
            if let Some(error) = self.render_error.clone() {
                return Err(error);
            }
            self.rerender_count += 1;
            self.last_scene = Some(scene.clone());
            Ok(())
        }

        fn apply_timeline_descriptor(
            &mut self,
            _descriptor: NativePanelTimelineDescriptor,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn sync_host_window_state(
            &mut self,
            _state: NativePanelHostWindowState,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn sync_screen_frame(
            &mut self,
            _screen_frame: Option<PanelRect>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn record_host_window_descriptor(
            &mut self,
            _descriptor: NativePanelHostWindowDescriptor,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn sync_pointer_regions(
            &mut self,
            _regions: &[NativePanelPointerRegion],
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn set_visible(&mut self, _visible: bool) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl NativePanelSceneHost for TestInteractionHost {}

    #[derive(Default)]
    struct TestQueuedPlatformEventSource {
        pending_events: Vec<NativePanelPlatformEvent>,
    }

    impl NativePanelQueuedPlatformEventSource for TestQueuedPlatformEventSource {
        fn take_queued_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
            std::mem::take(&mut self.pending_events)
        }
    }

    impl NativePanelRuntimeCommandCapability for RecordingHandler {
        type Error = String;

        fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::FocusSession(session_id));
            Ok(())
        }

        fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::ToggleSettingsSurface);
            Ok(())
        }

        fn quit_application(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::QuitApplication);
            Ok(())
        }

        fn cycle_display(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::CycleDisplay);
            Ok(())
        }

        fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::ToggleCompletionSound);
            Ok(())
        }

        fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::ToggleMascot);
            Ok(())
        }

        fn mascot_debug_click(&mut self) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::MascotDebugClick);
            Ok(())
        }

        fn open_settings_location(&mut self) -> Result<(), Self::Error> {
            self.handled
                .push(NativePanelPlatformEvent::OpenSettingsLocation);
            Ok(())
        }

        fn open_release_page(&mut self) -> Result<(), Self::Error> {
            self.handled.push(NativePanelPlatformEvent::OpenReleasePage);
            Ok(())
        }
    }

    #[test]
    fn shared_click_resolution_records_focus_target_and_suppresses_duplicate_clicks() {
        let now = Instant::now();
        let pointer_state = NativePanelPointerPointState {
            inside: true,
            platform_event: Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string(),
            )),
            hit_target: Some(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        };
        let mut state = TestClickState {
            expanded: true,
            transitioning: false,
            primary_mouse_down: false,
            last_focus_click: None,
        };

        let first = resolve_native_panel_click_command_for_pointer_state(
            &mut state,
            &pointer_state,
            true,
            true,
            now,
            500,
        );
        let duplicate = resolve_native_panel_click_command_for_pointer_state(
            &mut state,
            &pointer_state,
            true,
            true,
            now + Duration::from_millis(100),
            500,
        );

        assert_eq!(
            first,
            PanelInteractionCommand::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            })
        );
        assert_eq!(duplicate, PanelInteractionCommand::None);
    }

    #[test]
    fn shared_click_state_slots_record_focus_clicks() {
        let now = Instant::now();
        let panel_state = PanelState {
            expanded: true,
            transitioning: false,
            ..PanelState::default()
        };
        let mut last_focus_click = None;
        let mut slots = native_panel_click_state_slots(&panel_state, &mut last_focus_click);

        assert!(slots.click_expanded());
        assert!(!slots.click_transitioning());
        assert!(slots.click_last_focus_click().is_none());

        slots.record_click_focus_session("session-1".to_string(), now);

        let recorded = slots
            .click_last_focus_click()
            .expect("recorded focus click should exist");
        assert_eq!(recorded.session_id, "session-1");
        assert_eq!(recorded.clicked_at, now);
    }

    #[test]
    fn settings_surface_toggle_returns_surface_switch_when_expanded() {
        let mut state = PanelState {
            expanded: true,
            transitioning: false,
            ..PanelState::default()
        };

        let result = toggle_native_panel_settings_surface_transition_request_for_state(&mut state);

        assert!(result.changed);
        assert_eq!(
            result.transition_request,
            Some(NativePanelTransitionRequest::SurfaceSwitch)
        );
    }

    #[test]
    fn settings_surface_toggle_skips_animation_when_collapsed() {
        let mut state = PanelState {
            expanded: false,
            transitioning: false,
            ..PanelState::default()
        };

        let result = toggle_native_panel_settings_surface_transition_request_for_state(&mut state);

        assert!(result.changed);
        assert_eq!(result.transition_request, None);
    }

    #[test]
    fn settings_surface_snapshot_update_carries_snapshot_when_state_changes() {
        let snapshot = echoisland_runtime::RuntimeSnapshot {
            status: "idle".to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        };
        let mut state = PanelState {
            expanded: true,
            transitioning: false,
            ..PanelState::default()
        };

        let update = resolve_native_panel_settings_surface_snapshot_update_for_state(
            &mut state,
            Some(snapshot.clone()),
        )
        .expect("settings surface update");

        assert_eq!(update.snapshot, Some(snapshot));
        assert_eq!(
            update.transition_request,
            Some(NativePanelTransitionRequest::SurfaceSwitch)
        );
    }

    #[test]
    fn shared_click_dispatch_maps_command_into_runtime_handler() {
        let mut handler = RecordingHandler::default();

        let event = dispatch_native_panel_click_command_with_handler(
            &mut handler,
            PanelInteractionCommand::HitTarget(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            }),
        )
        .expect("dispatch click command");

        assert_eq!(
            event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            handler.handled,
            vec![NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )]
        );
    }

    #[test]
    fn shared_click_dispatch_at_point_uses_pointer_host_state() {
        let now = Instant::now();
        let mut state = TestClickState {
            expanded: true,
            transitioning: false,
            primary_mouse_down: false,
            last_focus_click: None,
        };
        let host = TestInteractionHost {
            pointer_state: NativePanelPointerPointState {
                inside: true,
                platform_event: Some(NativePanelPlatformEvent::FocusSession(
                    "session-1".to_string(),
                )),
                hit_target: Some(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                }),
            },
            cards_visible: false,
            ..TestInteractionHost::default()
        };
        let mut handler = RecordingHandler::default();

        let event = dispatch_native_panel_click_command_at_point_with_handler(
            &mut state,
            &host,
            PanelPoint { x: 20.0, y: 20.0 },
            now,
            500,
            &mut handler,
        )
        .expect("dispatch click at point");

        assert_eq!(
            event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            handler.handled,
            vec![NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            )]
        );
    }

    #[test]
    fn shared_hover_sync_returns_transition_and_request_together() {
        let now = Instant::now();
        let mut state = PanelState {
            expanded: false,
            pointer_inside_since: Some(now - Duration::from_millis(700)),
            ..PanelState::default()
        };

        let result = sync_native_panel_hover_interaction_for_state(&mut state, true, now, 600);

        assert_eq!(result.transition, Some(HoverTransition::Expand));
        assert_eq!(result.request, Some(NativePanelTransitionRequest::Open));
        assert!(state.expanded);
    }

    #[test]
    fn shared_hover_sync_without_transition_preserves_pending_open_request() {
        let mut request_slot = Some(NativePanelTransitionRequest::Open);
        let result = apply_native_panel_hover_sync_result_slot(
            &mut request_slot,
            NativePanelHoverSyncResult {
                transition: None,
                request: None,
            },
        );

        assert_eq!(result, None);
        assert_eq!(request_slot, Some(NativePanelTransitionRequest::Open));
    }

    #[test]
    fn shared_hover_sync_at_point_uses_host_hover_resolution() {
        let now = Instant::now();
        let mut state = PanelState {
            expanded: false,
            pointer_inside_since: Some(now - Duration::from_millis(700)),
            ..PanelState::default()
        };
        let host = TestInteractionHost {
            hover_inside: true,
            ..TestInteractionHost::default()
        };

        let transition = sync_native_panel_hover_interaction_at_point_for_state(
            &mut state,
            &host,
            PanelPoint { x: 10.0, y: 10.0 },
            now,
            600,
        );

        assert_eq!(transition, Some(HoverTransition::Expand));
        assert!(state.expanded);
    }

    #[test]
    fn shared_hover_sync_for_pointer_input_uses_host_input_resolution() {
        let now = Instant::now();
        let mut state = PanelState {
            expanded: false,
            pointer_inside_since: Some(now - Duration::from_millis(700)),
            ..PanelState::default()
        };
        let host = TestInteractionHost {
            hover_inside_for_input: Some(true),
            ..TestInteractionHost::default()
        };

        let transition = sync_native_panel_hover_interaction_for_pointer_input_for_state(
            &mut state,
            &host,
            NativePanelPointerInput::Move(PanelPoint { x: 10.0, y: 10.0 }),
            now,
            600,
        );

        assert_eq!(transition, Some(HoverTransition::Expand));
        assert!(state.expanded);
    }

    #[test]
    fn shared_hover_rerender_at_point_returns_request_and_refreshes_scene() {
        let now = Instant::now();
        let mut state = PanelState {
            expanded: false,
            pointer_inside_since: Some(now - Duration::from_millis(700)),
            ..PanelState::default()
        };
        let mut host = TestInteractionHost {
            hover_inside: true,
            ..TestInteractionHost::default()
        };
        let mut cache = NativePanelRuntimeSceneCache::default();
        cache.last_snapshot = Some(echoisland_runtime::RuntimeSnapshot {
            status: "idle".to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        });
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };

        let result =
            sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor(
                &mut host,
                &mut cache,
                &mut state,
                PanelPoint { x: 10.0, y: 10.0 },
                now,
                600,
                &input,
            )
            .expect("hover rerender at point");

        assert_eq!(result.transition, Some(HoverTransition::Expand));
        assert_eq!(result.request, Some(NativePanelTransitionRequest::Open));
        assert_eq!(host.rerender_count, 1);
        assert!(host.last_scene.is_some());
    }

    #[test]
    fn shared_hover_rerender_without_snapshot_does_not_clear_badge_or_expand() {
        let now = Instant::now();
        let mut state = PanelState {
            expanded: false,
            pointer_inside_since: Some(now - Duration::from_millis(700)),
            completion_badge_items: vec![crate::native_panel_core::CompletionBadgeItem {
                session_id: "session-1".to_string(),
                completed_at: Utc::now(),
                last_user_prompt: Some("prompt".to_string()),
                last_assistant_message: Some("done".to_string()),
            }],
            ..PanelState::default()
        };
        let mut host = TestInteractionHost {
            hover_inside: true,
            ..TestInteractionHost::default()
        };
        let mut cache = NativePanelRuntimeSceneCache::default();
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };

        let result =
            sync_native_panel_hover_interaction_and_rerender_at_point_with_input_descriptor(
                &mut host,
                &mut cache,
                &mut state,
                PanelPoint { x: 10.0, y: 10.0 },
                now,
                600,
                &input,
            )
            .expect("hover without snapshot");

        assert_eq!(result.transition, None);
        assert_eq!(result.request, None);
        assert_eq!(host.rerender_count, 0);
        assert!(!state.expanded);
        assert_eq!(state.completion_badge_items.len(), 1);
    }

    #[test]
    fn shared_hover_close_rerender_error_restores_previous_panel_state() {
        let now = Instant::now();
        let mut state = PanelState {
            expanded: true,
            pointer_outside_since: Some(now - Duration::from_millis(700)),
            ..PanelState::default()
        };
        let original = state.clone();
        let mut host = TestInteractionHost {
            render_error: Some("render failed".to_string()),
            ..TestInteractionHost::default()
        };
        let mut cache = NativePanelRuntimeSceneCache::default();
        cache.last_snapshot = Some(echoisland_runtime::RuntimeSnapshot {
            status: "idle".to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        });
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };

        let error =
            sync_native_panel_hover_interaction_and_rerender_for_inside_with_input_descriptor(
                &mut host, &mut cache, &mut state, false, now, 600, &input,
            )
            .expect_err("render error should propagate");

        assert_eq!(error, "render failed");
        assert_eq!(state.expanded, original.expanded);
        assert_eq!(state.transitioning, original.transitioning);
        assert_eq!(state.status_auto_expanded, original.status_auto_expanded);
        assert_eq!(state.surface_mode, original.surface_mode);
        assert_eq!(state.pointer_inside_since, original.pointer_inside_since);
        assert_eq!(state.pointer_outside_since, original.pointer_outside_since);
    }

    #[test]
    fn shared_hover_rerender_for_pointer_input_propagates_none_when_host_has_no_inside_state() {
        let now = Instant::now();
        let mut state = PanelState::default();
        let mut host = TestInteractionHost {
            hover_inside_for_input: None,
            ..TestInteractionHost::default()
        };
        let mut cache = NativePanelRuntimeSceneCache::default();
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };

        let result =
            sync_native_panel_hover_interaction_and_rerender_for_pointer_input_with_input_descriptor(
                &mut host,
                &mut cache,
                &mut state,
                NativePanelPointerInput::Leave,
                now,
                600,
                &input,
            )
            .expect("hover rerender pointer input");

        assert_eq!(result, None);
        assert_eq!(host.rerender_count, 0);
    }

    #[test]
    fn pointer_region_bridge_exposes_shared_click_and_hover_behavior() {
        struct PointerRegionBridgeHost {
            pointer_regions: Vec<NativePanelPointerRegion>,
            cards_visible: bool,
        }

        impl NativePanelPointerRegionInteractionBridge for PointerRegionBridgeHost {
            fn interaction_pointer_regions(&self) -> &[NativePanelPointerRegion] {
                &self.pointer_regions
            }

            fn interaction_cards_visible(&self) -> bool {
                self.cards_visible
            }
        }

        let host = PointerRegionBridgeHost {
            pointer_regions: vec![NativePanelPointerRegion {
                frame: PanelRect {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 24.0,
                },
                kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                }),
            }],
            cards_visible: true,
        };

        assert_eq!(
            host.click_pointer_state_at_point(PanelPoint { x: 20.0, y: 20.0 }),
            NativePanelPointerPointState {
                inside: true,
                platform_event: Some(NativePanelPlatformEvent::FocusSession(
                    "session-1".to_string()
                )),
                hit_target: Some(PanelHitTarget {
                    action: PanelHitAction::FocusSession,
                    value: "session-1".to_string(),
                }),
            }
        );
        assert!(host.click_cards_visible());
        assert!(host.hover_inside_at_point(PanelPoint { x: 20.0, y: 20.0 }));
        assert_eq!(
            host.hover_inside_for_input(NativePanelPointerInput::Move(PanelPoint {
                x: 200.0,
                y: 200.0,
            })),
            Some(false)
        );
    }

    #[test]
    fn shared_queued_platform_event_dispatch_drains_and_runs_handler() {
        let mut source = TestQueuedPlatformEventSource {
            pending_events: vec![
                NativePanelPlatformEvent::ToggleSettingsSurface,
                NativePanelPlatformEvent::QuitApplication,
            ],
        };
        let mut handler = RecordingHandler::default();

        dispatch_queued_native_panel_platform_events_with_handler(&mut source, &mut handler)
            .expect("dispatch queued events");

        assert_eq!(
            handler.handled,
            vec![
                NativePanelPlatformEvent::ToggleSettingsSurface,
                NativePanelPlatformEvent::QuitApplication,
            ]
        );
        assert!(source.pending_events.is_empty());
    }

    #[test]
    fn shared_queued_platform_event_bridge_queues_region_and_point_events() {
        struct TestQueuedBridge {
            pending_events: Vec<NativePanelPlatformEvent>,
            pointer_regions: Vec<NativePanelPointerRegion>,
        }

        impl NativePanelQueuedPlatformEventBridge for TestQueuedBridge {
            fn queued_platform_events_mut(&mut self) -> &mut Vec<NativePanelPlatformEvent> {
                &mut self.pending_events
            }

            fn queued_pointer_regions(&self) -> &[NativePanelPointerRegion] {
                &self.pointer_regions
            }
        }

        let mut bridge = TestQueuedBridge {
            pending_events: Vec::new(),
            pointer_regions: vec![
                NativePanelPointerRegion {
                    frame: PanelRect {
                        x: 20.0,
                        y: 20.0,
                        width: 80.0,
                        height: 40.0,
                    },
                    kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                        action: PanelHitAction::FocusSession,
                        value: "session-1".to_string(),
                    }),
                },
                NativePanelPointerRegion {
                    frame: PanelRect {
                        x: 140.0,
                        y: 140.0,
                        width: 40.0,
                        height: 40.0,
                    },
                    kind: NativePanelPointerRegionKind::EdgeAction(
                        crate::native_panel_renderer::descriptors::NativePanelEdgeAction::Quit,
                    ),
                },
            ],
        };

        let first_region = bridge.pointer_regions[0].clone();
        assert_eq!(
            bridge.queue_platform_event_for_pointer_region(&first_region),
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            bridge.queue_platform_event_at_point(PanelPoint { x: 150.0, y: 150.0 }),
            Some(NativePanelPlatformEvent::QuitApplication)
        );
        assert_eq!(
            bridge.pending_events,
            vec![
                NativePanelPlatformEvent::FocusSession("session-1".to_string()),
                NativePanelPlatformEvent::QuitApplication,
            ]
        );
    }

    #[test]
    fn shared_polling_interaction_uses_fallback_hit_testing_and_updates_pointer_button_state() {
        let now = Instant::now();
        let mut state = TestClickState {
            expanded: false,
            transitioning: false,
            primary_mouse_down: false,
            last_focus_click: None,
        };

        let result = sync_native_panel_polling_interaction_for_state(
            &mut state,
            native_panel_polling_interaction_input(
                NativePanelPointerPointState {
                    inside: false,
                    platform_event: None,
                    hit_target: None,
                },
                false,
                NativePanelHoverFallbackState {
                    interactive_inside: true,
                    hover_inside: true,
                },
                true,
                false,
                None,
            ),
            now,
            600,
            500,
        );

        assert!(result.interactive_inside);
        assert_eq!(result.click_command, PanelInteractionCommand::None);
        assert!(state.primary_mouse_down);
    }

    #[test]
    fn shared_hover_fallback_frames_follow_visual_input() {
        let compact = resolve_native_panel_hover_fallback_frames(&visual_input(
            NativePanelVisualDisplayMode::Compact,
        ));

        assert_eq!(
            compact.interactive_pill_frame,
            PanelRect {
                x: 40.0,
                y: 12.0,
                width: 240.0,
                height: 36.0,
            }
        );
        assert_eq!(
            compact.hover_pill_frame,
            PanelRect {
                x: 100.0,
                y: 20.0,
                width: 320.0,
                height: 160.0,
            }
        );
        assert_eq!(compact.interactive_expanded_frame, None);

        let expanded = resolve_native_panel_hover_fallback_frames(&visual_input(
            NativePanelVisualDisplayMode::Expanded,
        ));

        assert_eq!(
            expanded.interactive_expanded_frame,
            Some(PanelRect {
                x: 20.0,
                y: 0.0,
                width: 280.0,
                height: 160.0,
            })
        );
    }

    #[test]
    fn shared_polling_input_from_host_facts_resolves_pointer_and_hover_state() {
        let input =
            native_panel_polling_interaction_input_from_host_facts(NativePanelPollingHostFacts {
                pointer: PanelPoint { x: 48.0, y: 22.0 },
                pointer_regions: &[NativePanelPointerRegion {
                    frame: PanelRect {
                        x: 0.0,
                        y: 0.0,
                        width: 120.0,
                        height: 40.0,
                    },
                    kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
                        action: PanelHitAction::FocusSession,
                        value: "session-1".to_string(),
                    }),
                }],
                hover_frames: NativePanelHoverFallbackFrames {
                    interactive_pill_frame: PanelRect {
                        x: 0.0,
                        y: 0.0,
                        width: 120.0,
                        height: 40.0,
                    },
                    hover_pill_frame: PanelRect {
                        x: 4.0,
                        y: 4.0,
                        width: 140.0,
                        height: 60.0,
                    },
                    interactive_expanded_frame: None,
                },
                primary_mouse_down: true,
                cards_visible: true,
                snapshot: None,
            });

        assert!(input.pointer_state.inside);
        assert_eq!(
            input.pointer_state.platform_event,
            Some(NativePanelPlatformEvent::FocusSession(
                "session-1".to_string()
            ))
        );
        assert_eq!(
            input.pointer_state.hit_target,
            Some(PanelHitTarget {
                action: PanelHitAction::FocusSession,
                value: "session-1".to_string(),
            })
        );
        assert!(input.pointer_regions_available);
        assert_eq!(
            input.fallback_hover,
            NativePanelHoverFallbackState {
                interactive_inside: true,
                hover_inside: true,
            }
        );
        assert!(input.primary_mouse_down);
        assert!(input.cards_visible);
    }

    #[test]
    fn shared_polling_interaction_emits_hover_transition_bundle_when_snapshot_exists() {
        let now = Instant::now();
        let state = PanelState {
            expanded: false,
            pointer_inside_since: Some(now - Duration::from_millis(700)),
            ..PanelState::default()
        };
        #[derive(Default)]
        struct TestPollingState {
            core: PanelState,
            primary_mouse_down: bool,
            last_focus_click: Option<(String, Instant)>,
        }
        impl NativePanelCoreStateBridge for TestPollingState {
            fn snapshot_core_panel_state(&self) -> PanelState {
                self.core.clone()
            }
            fn apply_core_panel_state(&mut self, core: PanelState) {
                self.core = core;
            }
        }
        impl NativePanelClickStateBridge for TestPollingState {
            fn click_expanded(&self) -> bool {
                self.core.expanded
            }
            fn click_transitioning(&self) -> bool {
                self.core.transitioning
            }
            fn click_last_focus_click(&self) -> Option<LastFocusClick<'_>> {
                resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
            }
            fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
                record_native_panel_focus_click_session(
                    &mut self.last_focus_click,
                    session_id,
                    now,
                );
            }
        }
        impl NativePanelPrimaryPointerStateBridge for TestPollingState {
            fn primary_pointer_down(&self) -> bool {
                self.primary_mouse_down
            }
            fn set_primary_pointer_down(&mut self, down: bool) {
                self.primary_mouse_down = down;
            }
        }
        let snapshot = echoisland_runtime::RuntimeSnapshot {
            status: "idle".to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        };
        let mut polling_state = TestPollingState {
            core: state,
            ..TestPollingState::default()
        };

        let result = sync_native_panel_polling_interaction_for_state(
            &mut polling_state,
            native_panel_polling_interaction_input(
                NativePanelPointerPointState {
                    inside: true,
                    platform_event: None,
                    hit_target: None,
                },
                true,
                NativePanelHoverFallbackState::default(),
                false,
                false,
                Some(snapshot.clone()),
            ),
            now,
            600,
            500,
        );

        assert_eq!(
            result.transition_request,
            Some(NativePanelTransitionRequest::Open)
        );
        assert_eq!(result.transition_snapshot, Some(snapshot));
        assert!(polling_state.core.expanded);
    }

    #[test]
    fn shared_polling_interaction_does_not_commit_hover_transition_without_snapshot() {
        let now = Instant::now();

        #[derive(Default)]
        struct TestPollingState {
            core: PanelState,
            primary_mouse_down: bool,
            last_focus_click: Option<(String, Instant)>,
        }

        impl NativePanelCoreStateBridge for TestPollingState {
            fn snapshot_core_panel_state(&self) -> PanelState {
                self.core.clone()
            }

            fn apply_core_panel_state(&mut self, core: PanelState) {
                self.core = core;
            }
        }

        impl NativePanelClickStateBridge for TestPollingState {
            fn click_expanded(&self) -> bool {
                self.core.expanded
            }

            fn click_transitioning(&self) -> bool {
                self.core.transitioning
            }

            fn click_last_focus_click(&self) -> Option<LastFocusClick<'_>> {
                resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
            }

            fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
                record_native_panel_focus_click_session(
                    &mut self.last_focus_click,
                    session_id,
                    now,
                );
            }
        }

        impl NativePanelPrimaryPointerStateBridge for TestPollingState {
            fn primary_pointer_down(&self) -> bool {
                self.primary_mouse_down
            }

            fn set_primary_pointer_down(&mut self, down: bool) {
                self.primary_mouse_down = down;
            }
        }

        let mut polling_state = TestPollingState {
            core: PanelState {
                expanded: false,
                pointer_inside_since: Some(now - Duration::from_millis(700)),
                ..PanelState::default()
            },
            ..TestPollingState::default()
        };

        let result = sync_native_panel_polling_interaction_for_state(
            &mut polling_state,
            native_panel_polling_interaction_input(
                NativePanelPointerPointState {
                    inside: true,
                    platform_event: None,
                    hit_target: None,
                },
                true,
                NativePanelHoverFallbackState::default(),
                false,
                false,
                None,
            ),
            now,
            600,
            500,
        );

        assert_eq!(result.transition_request, None);
        assert_eq!(result.transition_snapshot, None);
        assert!(!polling_state.core.expanded);
        assert!(
            polling_state
                .core
                .pointer_inside_since
                .is_some_and(|entered_at| now.duration_since(entered_at).as_millis() >= 600)
        );
    }

    #[test]
    fn shared_host_polling_interaction_updates_mouse_passthrough_state() {
        let now = Instant::now();

        #[derive(Default)]
        struct TestHostPollingState {
            core: PanelState,
            primary_mouse_down: bool,
            last_focus_click: Option<(String, Instant)>,
            ignores_mouse_events: bool,
        }

        impl NativePanelCoreStateBridge for TestHostPollingState {
            fn snapshot_core_panel_state(&self) -> PanelState {
                self.core.clone()
            }

            fn apply_core_panel_state(&mut self, core: PanelState) {
                self.core = core;
            }
        }

        impl NativePanelClickStateBridge for TestHostPollingState {
            fn click_expanded(&self) -> bool {
                self.core.expanded
            }

            fn click_transitioning(&self) -> bool {
                self.core.transitioning
            }

            fn click_last_focus_click(&self) -> Option<LastFocusClick<'_>> {
                resolve_native_panel_last_focus_click(self.last_focus_click.as_ref())
            }

            fn record_click_focus_session(&mut self, session_id: String, now: Instant) {
                record_native_panel_focus_click_session(
                    &mut self.last_focus_click,
                    session_id,
                    now,
                );
            }
        }

        impl NativePanelPrimaryPointerStateBridge for TestHostPollingState {
            fn primary_pointer_down(&self) -> bool {
                self.primary_mouse_down
            }

            fn set_primary_pointer_down(&mut self, down: bool) {
                self.primary_mouse_down = down;
            }
        }

        impl NativePanelHostInteractionStateBridge for TestHostPollingState {
            fn host_ignores_mouse_events(&self) -> bool {
                self.ignores_mouse_events
            }

            fn set_host_ignores_mouse_events(&mut self, ignores_mouse_events: bool) {
                self.ignores_mouse_events = ignores_mouse_events;
            }
        }

        let mut state = TestHostPollingState {
            ignores_mouse_events: true,
            ..Default::default()
        };

        let result = sync_native_panel_host_polling_interaction_for_state(
            &mut state,
            native_panel_polling_interaction_input(
                NativePanelPointerPointState {
                    inside: false,
                    platform_event: None,
                    hit_target: None,
                },
                false,
                NativePanelHoverFallbackState {
                    interactive_inside: true,
                    hover_inside: true,
                },
                false,
                false,
                None,
            ),
            now,
            600,
            500,
        );

        assert!(result.interactive_inside);
        assert!(!result.next_ignores_mouse_events);
        assert!(result.sync_mouse_event_passthrough);
        assert!(!state.ignores_mouse_events);
    }

    #[test]
    fn shared_pointer_input_handler_routes_hover_path() {
        #[derive(Default)]
        struct TestPointerRuntimeState {
            recorded_inputs: Vec<NativePanelPointerInput>,
            passthrough_inputs: Vec<NativePanelPointerInput>,
            hover_calls: Vec<NativePanelPointerInput>,
            click_points: Vec<PanelPoint>,
        }

        impl NativePanelPointerInputRuntimeBridge for TestPointerRuntimeState {
            type Error = String;

            fn sync_mouse_passthrough_for_pointer_input(&mut self, input: NativePanelPointerInput) {
                self.passthrough_inputs.push(input);
            }

            fn record_pointer_input(&mut self, input: NativePanelPointerInput) {
                self.recorded_inputs.push(input);
            }

            fn sync_hover_and_refresh_for_pointer_input(
                &mut self,
                input: NativePanelPointerInput,
                _now: Instant,
                _runtime_input: &NativePanelRuntimeInputDescriptor,
            ) -> Result<Option<HoverTransition>, Self::Error> {
                self.hover_calls.push(input);
                Ok(Some(HoverTransition::Expand))
            }

            fn dispatch_click_command_for_pointer_point<H>(
                &mut self,
                point: PanelPoint,
                _now: Instant,
                _handler: &mut H,
            ) -> Result<Option<NativePanelPlatformEvent>, Self::Error>
            where
                H: NativePanelRuntimeCommandHandler<Error = Self::Error>,
            {
                self.click_points.push(point);
                Ok(None)
            }
        }

        #[derive(Default)]
        struct TestPointerHandler;

        impl NativePanelRuntimeCommandCapability for TestPointerHandler {
            type Error = String;

            fn focus_session(&mut self, _session_id: String) -> Result<(), Self::Error> {
                Ok(())
            }

            fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn quit_application(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn cycle_display(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn mascot_debug_click(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn open_settings_location(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn open_release_page(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
        }

        let mut state = TestPointerRuntimeState::default();
        let mut handler = TestPointerHandler;
        let input_event = NativePanelPointerInput::Move(PanelPoint { x: 12.0, y: 18.0 });

        let outcome = handle_native_panel_pointer_input_with_handler(
            &mut state,
            input_event,
            Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("route hover pointer input");

        assert_eq!(
            outcome,
            crate::native_panel_renderer::descriptors::NativePanelPointerInputOutcome::Hover(Some(
                HoverTransition::Expand
            ))
        );
        assert_eq!(state.passthrough_inputs, vec![input_event]);
        assert_eq!(state.recorded_inputs, vec![input_event]);
        assert_eq!(state.hover_calls, vec![input_event]);
        assert!(state.click_points.is_empty());
    }

    #[test]
    fn shared_pointer_input_handler_routes_click_path() {
        #[derive(Default)]
        struct TestPointerRuntimeState {
            recorded_inputs: Vec<NativePanelPointerInput>,
            passthrough_inputs: Vec<NativePanelPointerInput>,
            hover_calls: Vec<NativePanelPointerInput>,
            click_points: Vec<PanelPoint>,
        }

        impl NativePanelPointerInputRuntimeBridge for TestPointerRuntimeState {
            type Error = String;

            fn sync_mouse_passthrough_for_pointer_input(&mut self, input: NativePanelPointerInput) {
                self.passthrough_inputs.push(input);
            }

            fn record_pointer_input(&mut self, input: NativePanelPointerInput) {
                self.recorded_inputs.push(input);
            }

            fn sync_hover_and_refresh_for_pointer_input(
                &mut self,
                input: NativePanelPointerInput,
                _now: Instant,
                _runtime_input: &NativePanelRuntimeInputDescriptor,
            ) -> Result<Option<HoverTransition>, Self::Error> {
                self.hover_calls.push(input);
                Ok(None)
            }

            fn dispatch_click_command_for_pointer_point<H>(
                &mut self,
                point: PanelPoint,
                _now: Instant,
                _handler: &mut H,
            ) -> Result<Option<NativePanelPlatformEvent>, Self::Error>
            where
                H: NativePanelRuntimeCommandHandler<Error = Self::Error>,
            {
                self.click_points.push(point);
                Ok(Some(NativePanelPlatformEvent::QuitApplication))
            }
        }

        #[derive(Default)]
        struct TestPointerHandler;

        impl NativePanelRuntimeCommandCapability for TestPointerHandler {
            type Error = String;

            fn focus_session(&mut self, _session_id: String) -> Result<(), Self::Error> {
                Ok(())
            }

            fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn quit_application(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn cycle_display(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn mascot_debug_click(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn open_settings_location(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn open_release_page(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
        }

        let mut state = TestPointerRuntimeState::default();
        let mut handler = TestPointerHandler;
        let input_event = NativePanelPointerInput::Click(PanelPoint { x: 24.0, y: 36.0 });

        let outcome = handle_native_panel_pointer_input_with_handler(
            &mut state,
            input_event,
            Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("route click pointer input");

        assert_eq!(
            outcome,
            crate::native_panel_renderer::descriptors::NativePanelPointerInputOutcome::Click(Some(
                NativePanelPlatformEvent::QuitApplication
            ))
        );
        assert_eq!(state.passthrough_inputs, vec![input_event]);
        assert_eq!(state.recorded_inputs, vec![input_event]);
        assert!(state.hover_calls.is_empty());
        assert_eq!(state.click_points, vec![PanelPoint { x: 24.0, y: 36.0 }]);
    }
}
