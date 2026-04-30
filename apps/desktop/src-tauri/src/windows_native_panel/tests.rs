use super::WindowsNativePanelDrawFrame;
use crate::{
    native_panel_core::{
        ACTIVE_COUNT_SCROLL_HOLD_MS, CompletionBadgeItem, ExpandedSurface, HoverTransition,
        PanelAnimationDescriptor, PanelAnimationKind, PanelHitAction, PanelHitTarget,
        PanelInteractionCommand, PanelPoint, PanelRect, PanelState,
    },
    native_panel_renderer::facade::{
        command::{
            NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
            NativePanelRuntimeCommandCapability,
            dispatch_queued_native_panel_platform_events_with_handler,
        },
        descriptor::{
            NativePanelEdgeAction, NativePanelHostWindowState, NativePanelPointerRegion,
            NativePanelPointerRegionKind, NativePanelRuntimeInputDescriptor,
            NativePanelTimelineDescriptor,
        },
        host::{NativePanelHost, NativePanelRuntimeHostController, NativePanelSceneHost},
        interaction::{
            NativePanelClickStateBridge, NativePanelCoreStateBridge,
            NativePanelHostInteractionStateBridge, NativePanelPointerInputRuntimeBridge,
            NativePanelPrimaryPointerStateBridge, NativePanelQueuedPlatformEventBridge,
        },
        presentation::{
            NativePanelActionButtonsPresentation, NativePanelCardStackPresentation,
            NativePanelCompactBarPresentation, NativePanelMascotPresentation,
            NativePanelPresentationMetrics, NativePanelPresentationModel,
            NativePanelShellPresentation, NativePanelVisualDisplayMode,
            native_panel_visual_plan_input_from_presentation,
        },
        renderer::{
            NativePanelRenderer, NativePanelRuntimeSceneMutableStateBridge,
            NativePanelRuntimeSceneStateBridge, NativePanelSceneRuntimeBridge,
            cache_render_command_bundle_for_state_bridge_with_input,
            resolve_current_native_panel_render_command_bundle_for_state_bridge_with_input,
        },
        runtime::sync_runtime_scene_bundle_from_input_descriptor,
        shell::{
            NativePanelHostShellLifecycle, NativePanelHostShellRuntimePump,
            NativePanelPlatformWindowMessagePump,
        },
        transition::NativePanelTransitionRequest,
        visual::{
            NativePanelVisualColor, NativePanelVisualPrimitive, NativePanelVisualTextWeight,
            resolve_native_panel_visual_plan,
        },
    },
    native_panel_scene::{
        PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelSceneBuildInput, SceneCard,
        SceneMascotPose, build_panel_scene,
    },
};
use chrono::Utc;
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};
use std::{
    sync::{Mutex, MutexGuard, OnceLock},
    time::{Duration, Instant},
};

fn window_message_queue_test_guard() -> MutexGuard<'static, ()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn sync_test_pointer_regions(
    runtime: &mut super::WindowsNativePanelRuntime,
    regions: Vec<NativePanelPointerRegion>,
) {
    runtime.host.renderer.last_pointer_regions = regions;
    runtime
        .host
        .present_renderer_state()
        .expect("present test pointer regions");
}

fn snapshot() -> RuntimeSnapshot {
    RuntimeSnapshot {
        status: "idle".to_string(),
        primary_source: "codex".to_string(),
        active_session_count: 1,
        total_session_count: 1,
        pending_permission_count: 0,
        pending_question_count: 0,
        pending_permission: None,
        pending_question: None,
        pending_permissions: vec![],
        pending_questions: vec![],
        sessions: vec![],
    }
}

fn runtime_input_descriptor() -> NativePanelRuntimeInputDescriptor {
    NativePanelRuntimeInputDescriptor {
        scene_input: PanelSceneBuildInput::default(),
        screen_frame: Some(PanelRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        }),
    }
}

fn test_runtime_scene_bundle(
    panel_state: &mut PanelState,
    raw_snapshot: &RuntimeSnapshot,
    input: &PanelSceneBuildInput,
) -> PanelRuntimeSceneBundle {
    sync_runtime_scene_bundle_from_input_descriptor(
        panel_state,
        raw_snapshot,
        &NativePanelRuntimeInputDescriptor {
            scene_input: input.clone(),
            screen_frame: None,
        },
        Utc::now(),
    )
    .bundle
}

fn shell_draw_frame(
    pointer_regions: Vec<NativePanelPointerRegion>,
    expanded_cards_visible: bool,
) -> WindowsNativePanelDrawFrame {
    let panel_frame = PanelRect {
        x: 100.0,
        y: 50.0,
        width: 320.0,
        height: 120.0,
    };
    WindowsNativePanelDrawFrame {
        window_state: NativePanelHostWindowState {
            frame: Some(panel_frame),
            visible: true,
            preferred_display_index: 0,
        },
        pointer_regions,
        presentation_model: expanded_cards_visible.then(|| NativePanelPresentationModel {
            panel_frame,
            content_frame: PanelRect {
                x: 110.0,
                y: 90.0,
                width: 300.0,
                height: 70.0,
            },
            shell: NativePanelShellPresentation {
                surface: ExpandedSurface::Default,
                frame: PanelRect {
                    x: 100.0,
                    y: 70.0,
                    width: 320.0,
                    height: 100.0,
                },
                visible: true,
                separator_visibility: 1.0,
                shared_visible: true,
            },
            compact_bar: NativePanelCompactBarPresentation {
                frame: PanelRect {
                    x: 110.0,
                    y: 60.0,
                    width: 300.0,
                    height: 24.0,
                },
                left_shoulder_frame: PanelRect {
                    x: 104.0,
                    y: 78.0,
                    width: 6.0,
                    height: 6.0,
                },
                right_shoulder_frame: PanelRect {
                    x: 410.0,
                    y: 78.0,
                    width: 6.0,
                    height: 6.0,
                },
                shoulder_progress: 0.0,
                headline: crate::native_panel_scene::SceneText {
                    text: "Approval waiting".to_string(),
                    emphasized: false,
                },
                active_count: "1".to_string(),
                total_count: "1".to_string(),
                completion_count: 0,
                headline_emphasized: false,
                actions_visible: false,
            },
            card_stack: NativePanelCardStackPresentation {
                frame: PanelRect {
                    x: 110.0,
                    y: 90.0,
                    width: 300.0,
                    height: 70.0,
                },
                surface: ExpandedSurface::Default,
                cards: Vec::new(),
                content_height: 70.0,
                body_height: 70.0,
                visible: true,
            },
            mascot: NativePanelMascotPresentation {
                pose: SceneMascotPose::Idle,
            },
            glow: None,
            action_buttons: NativePanelActionButtonsPresentation {
                visible: false,
                buttons: Vec::new(),
            },
            metrics: NativePanelPresentationMetrics {
                expanded_content_height: 70.0,
                expanded_body_height: 70.0,
            },
        }),
    }
}

fn pending_permission_snapshot(session_id: &str) -> RuntimeSnapshot {
    pending_permission_snapshot_with_request("req-1", session_id)
}

fn pending_permission_snapshot_with_request(request_id: &str, session_id: &str) -> RuntimeSnapshot {
    let pending = PendingPermissionView {
        request_id: request_id.to_string(),
        session_id: session_id.to_string(),
        source: "claude".to_string(),
        tool_name: Some("Bash".to_string()),
        tool_description: Some("Run command".to_string()),
        requested_at: Utc::now(),
    };
    let mut snapshot = snapshot();
    snapshot.pending_permission_count = 1;
    snapshot.pending_permission = Some(pending.clone());
    snapshot.pending_permissions = vec![pending];
    snapshot
}

fn pending_question_snapshot_with_request(request_id: &str, session_id: &str) -> RuntimeSnapshot {
    let pending = PendingQuestionView {
        request_id: request_id.to_string(),
        session_id: session_id.to_string(),
        source: "claude".to_string(),
        header: Some("Pick one".to_string()),
        text: "Choose the deployment target".to_string(),
        options: vec!["Local".to_string(), "Staging".to_string()],
        requested_at: Utc::now(),
    };
    let mut snapshot = snapshot();
    snapshot.pending_question_count = 1;
    snapshot.pending_question = Some(pending.clone());
    snapshot.pending_questions = vec![pending];
    snapshot
}

fn session_snapshot_view(session_id: &str) -> SessionSnapshotView {
    SessionSnapshotView {
        session_id: session_id.to_string(),
        source: "codex".to_string(),
        project_name: Some("Blender Addon".to_string()),
        cwd: None,
        model: Some("gpt-5.5".to_string()),
        terminal_app: None,
        terminal_bundle: None,
        host_app: None,
        window_title: None,
        tty: None,
        terminal_pid: None,
        cli_pid: None,
        iterm_session_id: None,
        kitty_window_id: None,
        tmux_env: None,
        tmux_pane: None,
        tmux_client_tty: None,
        status: "thinking".to_string(),
        current_tool: None,
        tool_description: None,
        last_user_prompt: Some("Review the addon panel layout".to_string()),
        last_assistant_message: Some("Checking the current implementation".to_string()),
        tool_history_count: 0,
        tool_history: Vec::new(),
        last_activity: Utc::now(),
    }
}

fn sessions_snapshot(count: usize) -> RuntimeSnapshot {
    let mut snapshot = snapshot();
    snapshot.status = "active".to_string();
    snapshot.active_session_count = count;
    snapshot.total_session_count = count;
    snapshot.sessions = (0..count)
        .map(|index| session_snapshot_view(&format!("session-{}", index + 1)))
        .collect();
    snapshot
}

#[test]
fn windows_runtime_and_host_satisfy_shared_native_traits() {
    fn assert_runtime<T>()
    where
        T: NativePanelClickStateBridge
            + NativePanelCoreStateBridge
            + NativePanelHostInteractionStateBridge
            + NativePanelHostShellRuntimePump
            + NativePanelPlatformWindowMessagePump
            + NativePanelPointerInputRuntimeBridge
            + NativePanelPrimaryPointerStateBridge
            + NativePanelRuntimeSceneMutableStateBridge
            + NativePanelRuntimeSceneStateBridge
            + NativePanelSceneRuntimeBridge,
    {
    }

    fn assert_host<H>()
    where
        H: NativePanelHost
            + NativePanelRuntimeHostController
            + NativePanelSceneHost
            + NativePanelQueuedPlatformEventBridge,
    {
    }

    assert_runtime::<super::WindowsNativePanelRuntime>();
    assert_host::<super::WindowsNativePanelHost>();
}

#[test]
fn windows_native_default_enable_preflight_uses_shared_runtime_pipeline() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    runtime.create_panel().expect("create native panel");
    assert!(runtime.host.window.descriptor.visible);
    assert_eq!(
        runtime.host.shell.lifecycle(),
        NativePanelHostShellLifecycle::Visible
    );
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");
    runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("sync snapshot through shared runtime");

    let scene = runtime
        .scene_cache
        .last_scene
        .as_ref()
        .expect("shared scene cached");
    assert!(!scene.cards.is_empty());
    assert!(runtime.host.renderer.last_layout.is_some());
    assert!(runtime.host.renderer.last_render_state.is_some());
    assert!(!runtime.host.renderer.last_pointer_regions.is_empty());

    let first_region = runtime.host.renderer.last_pointer_regions[0].clone();
    let point = PanelPoint {
        x: first_region.frame.x + first_region.frame.width / 2.0,
        y: first_region.frame.y + first_region.frame.height / 2.0,
    };
    let mut handler = RecordingEventHandler::default();
    let outcome = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Move(point),
            Instant::now(),
            &input,
            &mut handler,
        )
        .expect("route pointer input through shared helper");
    assert!(matches!(outcome, NativePanelPointerInputOutcome::Hover(_)));

    runtime
        .toggle_settings_surface_with_input(&input)
        .expect("toggle settings through shared runtime");
    assert!(runtime.host.renderer.last_layout.is_some());

    runtime
        .set_shared_expanded_body_height(180.0)
        .expect("route shared body height through host facade");
    assert_eq!(
        runtime.host.window.descriptor.shared_body_height,
        Some(180.0)
    );

    runtime
        .pump_platform_loop()
        .expect("pump shared shell commands");
    assert!(runtime.platform_loop.applied_command_count > 0);
}

#[test]
fn windows_runtime_first_snapshot_renders_without_seeded_animation_descriptor() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    runtime.create_panel().expect("create native panel");
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("sync first snapshot");

    assert!(runtime.host.renderer.last_animation_descriptor.is_some());
    assert!(runtime.host.renderer.last_layout.is_some());
    assert!(runtime.host.renderer.last_render_state.is_some());
    assert!(runtime.host.renderer.last_window_state.is_some());
    assert!(
        runtime
            .host
            .renderer
            .last_window_state
            .is_some_and(|state| state
                .frame
                .is_some_and(|frame| { frame.width > 1.0 && frame.height > 1.0 && state.visible }))
    );
}

#[test]
fn windows_runtime_snapshot_sync_exposes_shared_message_sound_reminder() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    let sync = runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("sync snapshot through shared runtime")
        .expect("snapshot sync result");

    assert!(sync.reminder.play_sound);
}

#[test]
fn windows_runtime_auto_pops_question_status_card() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    let sync = runtime
        .sync_snapshot_bundle(
            &pending_question_snapshot_with_request("question-1", "session-1"),
            &input,
        )
        .expect("sync question snapshot through shared runtime")
        .expect("snapshot sync result");

    assert!(sync.reminder.play_sound);
    assert!(sync.reminder.show_status_card);
    assert_eq!(sync.panel_transition, Some(true));
    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("question status presentation");
    assert_eq!(presentation.shell.surface, ExpandedSurface::Status);
    assert_eq!(presentation.compact_bar.headline.text, "Question waiting");
    assert!(
        presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusQuestion { .. }))
    );
}

#[test]
fn windows_runtime_keeps_mixed_approval_and_question_status_cards() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    let mut snapshot = pending_permission_snapshot_with_request("req-1", "session-1");
    let question = pending_question_snapshot_with_request("question-1", "session-2");
    snapshot.pending_question_count = question.pending_question_count;
    snapshot.pending_question = question.pending_question.clone();
    snapshot.pending_questions = question.pending_questions.clone();

    runtime
        .sync_snapshot_bundle(&snapshot, &input)
        .expect("sync mixed status snapshot")
        .expect("snapshot sync result");

    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("mixed status presentation");
    assert_eq!(presentation.shell.surface, ExpandedSurface::Status);
    assert_eq!(presentation.compact_bar.headline.text, "Requests waiting");
    assert_eq!(presentation.card_stack.cards.len(), 2);
    assert!(matches!(
        presentation.card_stack.cards[0],
        SceneCard::StatusApproval { .. }
    ));
    assert!(matches!(
        presentation.card_stack.cards[1],
        SceneCard::StatusQuestion { .. }
    ));
}

#[test]
fn windows_runtime_pump_refreshes_status_queue_from_last_raw_snapshot() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("seed status queue");
    assert!(!runtime.panel_state.status_queue.is_empty());
    runtime.panel_state.status_queue[0].expires_at = Instant::now() - Duration::from_millis(1);

    runtime
        .pump_platform_loop()
        .expect("pump status queue refresh");

    assert!(runtime.panel_state.status_queue.is_empty());
}

#[test]
fn windows_runtime_status_queue_refresh_does_not_cancel_pending_open_transition() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            visible_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            width_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("seed collapsed animation descriptor");

    runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("seed status queue");
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open)
    );

    runtime.pump_platform_loop().expect("pump pending open");

    assert_eq!(
        runtime
            .host
            .renderer
            .last_animation_descriptor
            .map(|descriptor| descriptor.kind),
        Some(PanelAnimationKind::Open)
    );
}

#[test]
fn windows_scaffold_consumes_shared_scene_bundle() {
    let mut state = PanelState::default();
    let bundle =
        test_runtime_scene_bundle(&mut state, &snapshot(), &PanelSceneBuildInput::default());
    let scene = bundle.scene;
    let runtime_render_state = bundle.runtime_render_state;

    assert!(!scene.cards.is_empty());
    assert!(matches!(
        scene.mascot_pose,
        SceneMascotPose::Idle | SceneMascotPose::Running | SceneMascotPose::Hidden
    ));
    assert!(
        scene
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::Empty))
    );
    assert!(!runtime_render_state.transitioning);
}

#[test]
fn windows_host_lifecycle_tracks_create_show_hide() {
    let mut host = super::WindowsNativePanelHost::default();

    assert_eq!(
        host.window.lifecycle,
        super::host_window::WindowsNativePanelWindowLifecycle::NotCreated
    );
    assert_eq!(
        host.shell.lifecycle(),
        NativePanelHostShellLifecycle::Detached
    );
    assert!(!host.window.descriptor.visible);

    host.show().expect("show host");
    assert_eq!(
        host.window.lifecycle,
        super::host_window::WindowsNativePanelWindowLifecycle::Created
    );
    assert_eq!(
        host.shell.lifecycle(),
        NativePanelHostShellLifecycle::Visible
    );
    assert!(host.window.descriptor.visible);
    assert_eq!(
        host.renderer.last_window_state,
        Some(NativePanelHostWindowState {
            frame: None,
            visible: true,
            preferred_display_index: 0,
        })
    );

    host.reposition_to_display(2, None)
        .expect("reposition host");
    assert_eq!(host.window.descriptor.preferred_display_index, 2);
    assert_eq!(
        host.renderer.last_window_state,
        Some(NativePanelHostWindowState {
            frame: None,
            visible: true,
            preferred_display_index: 2,
        })
    );

    host.set_shared_body_height(320.0)
        .expect("sync shared body height");
    assert_eq!(host.window.descriptor.shared_body_height, Some(320.0));
    assert_eq!(
        host.renderer.last_host_window_descriptor,
        Some(host.window.descriptor)
    );

    host.hide().expect("hide host");
    assert_eq!(
        host.window.lifecycle,
        super::host_window::WindowsNativePanelWindowLifecycle::Created
    );
    assert_eq!(
        host.shell.lifecycle(),
        NativePanelHostShellLifecycle::Hidden
    );
    assert!(!host.window.descriptor.visible);
    assert_eq!(
        host.renderer.last_window_state,
        Some(NativePanelHostWindowState {
            frame: None,
            visible: false,
            preferred_display_index: 2,
        })
    );
}

#[test]
fn windows_host_shell_commands_track_lifecycle_and_reposition() {
    let mut host = super::WindowsNativePanelHost::default();

    host.show().expect("show host");
    host.reposition_to_display(1, None)
        .expect("reposition host");
    host.hide().expect("hide host");

    let commands = host.take_pending_shell_commands();

    assert!(commands.iter().any(|command| matches!(
        command,
        super::window_shell::WindowsNativePanelShellCommand::Create
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        super::window_shell::WindowsNativePanelShellCommand::Show
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        super::window_shell::WindowsNativePanelShellCommand::Hide
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        super::window_shell::WindowsNativePanelShellCommand::SyncWindowState(
            NativePanelHostWindowState {
                preferred_display_index: 1,
                ..
            }
        )
    )));
}

#[test]
fn windows_renderer_caches_shared_animation_descriptor() {
    let mut host = super::WindowsNativePanelHost::default();
    let descriptor = PanelAnimationDescriptor {
        kind: PanelAnimationKind::Open,
        canvas_height: 180.0,
        visible_height: 120.0,
        width_progress: 0.5,
        height_progress: 0.0,
        shoulder_progress: 1.0,
        drop_progress: 0.0,
        cards_progress: 0.25,
    };

    host.apply_animation_descriptor(descriptor)
        .expect("apply descriptor");

    assert_eq!(host.renderer.last_animation_descriptor, Some(descriptor));
    assert_eq!(
        host.renderer.last_timeline_descriptor,
        Some(NativePanelTimelineDescriptor {
            animation: descriptor,
            cards_entering: true,
        })
    );
    assert_eq!(
        host.renderer.last_host_window_descriptor,
        Some(host.window.descriptor)
    );
    assert_eq!(
        host.window.descriptor.timeline,
        Some(NativePanelTimelineDescriptor {
            animation: descriptor,
            cards_entering: true,
        })
    );
    assert!(host.window.last_frame.is_some());
    assert_eq!(
        host.renderer.last_window_state,
        Some(host.window.window_state())
    );
    assert_eq!(
        host.window.lifecycle,
        super::host_window::WindowsNativePanelWindowLifecycle::Created
    );
}

#[test]
fn windows_renderer_caches_pointer_regions_from_host_trait() {
    let mut host = super::WindowsNativePanelHost::default();
    let regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::Shell,
    }];

    host.sync_pointer_regions(&regions)
        .expect("sync pointer regions");

    assert_eq!(host.renderer.last_pointer_regions, regions);
}

#[test]
fn windows_host_queues_platform_events_from_pointer_regions() {
    let mut host = super::WindowsNativePanelHost::default();
    let frame = PanelRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 40.0,
    };

    host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
        frame,
        kind: NativePanelPointerRegionKind::Shell,
    });
    host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
        frame,
        kind: NativePanelPointerRegionKind::HitTarget(PanelHitTarget {
            action: PanelHitAction::FocusSession,
            value: "session-1".to_string(),
        }),
    });
    host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
        frame,
        kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings),
    });
    host.queue_platform_event_for_pointer_region(&NativePanelPointerRegion {
        frame,
        kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Quit),
    });

    assert_eq!(
        host.take_platform_events(),
        vec![
            NativePanelPlatformEvent::FocusSession("session-1".to_string()),
            NativePanelPlatformEvent::ToggleSettingsSurface,
            NativePanelPlatformEvent::QuitApplication,
        ]
    );
    assert!(host.take_platform_events().is_empty());
}

#[test]
fn windows_host_queues_platform_event_by_point_from_cached_regions() {
    let mut host = super::WindowsNativePanelHost::default();
    host.renderer.last_pointer_regions = vec![
        NativePanelPointerRegion {
            frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            },
            kind: NativePanelPointerRegionKind::CardsContainer,
        },
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
            kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Quit),
        },
    ];

    assert_eq!(
        host.queue_platform_event_at_point(PanelPoint { x: 30.0, y: 30.0 }),
        Some(NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        ))
    );
    assert_eq!(
        host.queue_platform_event_at_point(PanelPoint { x: 150.0, y: 150.0 }),
        Some(NativePanelPlatformEvent::QuitApplication)
    );
    assert_eq!(
        host.queue_platform_event_at_point(PanelPoint { x: 190.0, y: 190.0 }),
        None
    );
    assert_eq!(
        host.take_platform_events(),
        vec![
            NativePanelPlatformEvent::FocusSession("session-1".to_string()),
            NativePanelPlatformEvent::QuitApplication,
        ]
    );
}

#[test]
fn windows_runtime_syncs_hover_expand_from_cached_regions() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];

    let transition = runtime.sync_hover_at_point(PanelPoint { x: 30.0, y: 30.0 }, now);

    assert_eq!(transition, Some(HoverTransition::Expand));
    assert!(runtime.panel_state.expanded);
    assert!(runtime.panel_state.pointer_outside_since.is_none());
}

#[test]
fn windows_runtime_syncs_hover_collapse_outside_cached_regions() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.panel_state.pointer_outside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];

    let transition = runtime.sync_hover_at_point(PanelPoint { x: 180.0, y: 180.0 }, now);

    assert_eq!(transition, Some(HoverTransition::Collapse));
    assert!(!runtime.panel_state.expanded);
    assert!(runtime.panel_state.pointer_inside_since.is_none());
}

#[test]
fn windows_runtime_reposition_to_selected_display_uses_input_descriptor() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    runtime
        .reposition_to_selected_display_with_input(&input)
        .expect("reposition runtime to selected display");

    assert_eq!(
        runtime.host.window.descriptor.preferred_display_index,
        input.selected_display_index()
    );
    assert_eq!(
        runtime.host.window.descriptor.screen_frame,
        input.screen_frame
    );
}

#[test]
fn windows_runtime_set_shared_body_height_updates_host_descriptor() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime
        .set_shared_expanded_body_height(240.0)
        .expect("set shared body height");

    assert_eq!(
        runtime.host.window.descriptor.shared_body_height,
        Some(240.0)
    );
    assert_eq!(
        runtime.host.renderer.last_host_window_descriptor,
        Some(runtime.host.window.descriptor)
    );
}

#[test]
fn windows_runtime_expanded_target_height_prefers_current_native_content_over_stale_shared_height()
{
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime
        .set_shared_expanded_body_height(240.0)
        .expect("set stale shared body height");
    runtime.host.renderer.last_presentation_model =
        shell_draw_frame(Vec::new(), true).presentation_model;

    let expected_height = crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT
        + crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP
        + 70.0
        + crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET;

    assert_eq!(runtime.resolved_expanded_target_height(), expected_height);
}

#[test]
fn windows_runtime_expanded_target_height_prefers_latest_scene_over_stale_presentation_slot() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(&sessions_snapshot(3), &input)
        .expect("sync three session snapshot");
    runtime.host.renderer.last_presentation_model =
        shell_draw_frame(Vec::new(), true).presentation_model;

    let target_height = runtime.resolved_expanded_target_height();
    let stale_height = crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT
        + crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP
        + 70.0
        + crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET;

    assert!(target_height > stale_height);
}

#[test]
fn windows_host_presenter_prefers_latest_scene_over_stale_presentation_slot() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("sync empty state snapshot");
    runtime.host.renderer.last_presentation_model =
        shell_draw_frame(Vec::new(), true).presentation_model;

    runtime
        .host
        .present_renderer_state()
        .expect("present renderer state");
    let presented = runtime
        .host
        .window
        .presented_presentation_model
        .as_ref()
        .expect("presented model");

    assert_eq!(presented.card_stack.cards.len(), 1);
    assert!(matches!(presented.card_stack.cards[0], SceneCard::Empty));
}

#[test]
fn windows_runtime_hover_expand_refreshes_cached_scene_from_last_snapshot() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(pending_permission_snapshot("session-1"));
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 140.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];

    let transition = runtime
        .sync_hover_and_refresh_at_point_with_input(
            PanelPoint { x: 30.0, y: 30.0 },
            now,
            &runtime_input_descriptor(),
        )
        .expect("expand and refresh");

    assert_eq!(transition, Some(HoverTransition::Expand));
    assert!(runtime.panel_state.expanded);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open)
    );
    assert!(runtime.scene_cache.last_scene.is_some());
    assert!(runtime.scene_cache.last_runtime_render_state.is_some());
    assert!(runtime.host.renderer.scene_cache.last_scene.is_some());
    assert!(
        runtime
            .host
            .renderer
            .scene_cache
            .last_runtime_render_state
            .is_some()
    );
    assert!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .is_some_and(|scene| {
                scene.hit_targets.iter().any(|target| {
                    target.action == PanelHitAction::FocusSession && target.value == "session-1"
                })
            })
    );
}

#[test]
fn windows_runtime_hover_collapse_refreshes_cached_scene_from_last_snapshot() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.panel_state.expanded = true;
    runtime.panel_state.pointer_outside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: 120.0,
            visible_height: 120.0,
            width_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("seed animation descriptor");
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];

    let transition = runtime
        .sync_hover_and_refresh_at_point_with_input(
            PanelPoint { x: 180.0, y: 180.0 },
            now,
            &runtime_input_descriptor(),
        )
        .expect("collapse and refresh");

    assert_eq!(transition, Some(HoverTransition::Collapse));
    assert!(!runtime.panel_state.expanded);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Close)
    );
    assert!(runtime.scene_cache.last_scene.is_some());
    assert!(runtime.host.renderer.scene_cache.last_scene.is_some());
    assert!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .is_some_and(|scene| scene.compact_bar.actions_visible)
    );
    assert!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .is_some_and(|scene| scene.hit_targets.is_empty())
    );
}

#[test]
fn windows_runtime_hover_transition_without_snapshot_keeps_collapsed_state() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];

    let transition = runtime
        .sync_hover_and_refresh_at_point_with_input(
            PanelPoint { x: 30.0, y: 30.0 },
            now,
            &runtime_input_descriptor(),
        )
        .expect("hover without snapshot");

    assert_eq!(transition, None);
    assert!(!runtime.panel_state.expanded);
    assert!(runtime.scene_cache.last_scene.is_none());
    assert!(runtime.host.renderer.scene_cache.last_scene.is_none());
}

#[test]
fn windows_runtime_polling_hover_without_snapshot_keeps_collapsed_state() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 20.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        false,
    ));
    assert!(runtime.host.consume_presenter_into_shell());

    let interaction = runtime
        .sync_host_polling_interaction_and_refresh(
            PanelPoint { x: 30.0, y: 30.0 },
            false,
            now,
            &runtime_input_descriptor(),
        )
        .expect("poll hover without snapshot")
        .expect("polling facts exist");

    assert!(interaction.interactive_inside);
    assert_eq!(interaction.transition_request, None);
    assert_eq!(runtime.last_transition_request, None);
    assert!(!runtime.panel_state.expanded);
    assert!(runtime.scene_cache.last_scene.is_none());
    assert!(runtime.host.renderer.scene_cache.last_scene.is_none());
}

#[test]
fn windows_runtime_toggle_settings_surface_updates_cached_scene() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.panel_state.expanded = true;
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");

    let changed = runtime
        .toggle_settings_surface_with_input(&runtime_input_descriptor())
        .expect("toggle settings surface");

    assert!(changed);
    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Settings);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::SurfaceSwitch)
    );
    assert_eq!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Settings)
    );
    assert_eq!(
        runtime
            .host
            .renderer
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Settings)
    );
}

#[test]
fn windows_runtime_toggle_settings_surface_cycles_back_to_default() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.panel_state.expanded = true;
    runtime.panel_state.surface_mode = ExpandedSurface::Settings;
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");

    let changed = runtime
        .toggle_settings_surface_with_input(&runtime_input_descriptor())
        .expect("toggle settings surface");

    assert!(changed);
    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Default);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::SurfaceSwitch)
    );
    assert_eq!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Default)
    );
}

#[test]
fn windows_runtime_toggle_settings_surface_from_status_updates_cached_scene() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");
    runtime
        .sync_snapshot_bundle(
            &pending_permission_snapshot_with_request("req-1", "session-1"),
            &runtime_input_descriptor(),
        )
        .expect("seed status surface snapshot");
    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Status);

    let changed = runtime
        .toggle_settings_surface_with_input(&runtime_input_descriptor())
        .expect("toggle settings surface from status");

    assert!(changed);
    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Settings);
    assert!(!runtime.panel_state.status_queue.is_empty());
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::SurfaceSwitch)
    );
    assert_eq!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Settings)
    );
    assert_eq!(
        runtime
            .host
            .renderer
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Settings)
    );
}

#[test]
fn windows_runtime_sync_snapshot_can_return_from_settings_to_status_on_new_item() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");
    runtime
        .sync_snapshot_bundle(
            &pending_permission_snapshot_with_request("req-1", "session-1"),
            &runtime_input_descriptor(),
        )
        .expect("seed status surface snapshot");
    runtime
        .toggle_settings_surface_with_input(&runtime_input_descriptor())
        .expect("switch to settings");
    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Settings);

    runtime
        .sync_snapshot_bundle(
            &pending_permission_snapshot_with_request("req-2", "session-2"),
            &runtime_input_descriptor(),
        )
        .expect("sync new status item");

    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Status);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::SurfaceSwitch)
    );
    assert_eq!(
        runtime
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Status)
    );
    assert_eq!(
        runtime
            .host
            .renderer
            .scene_cache
            .last_scene
            .as_ref()
            .map(|scene| scene.surface),
        Some(ExpandedSurface::Status)
    );
}

#[test]
fn windows_runtime_dispatches_click_command_at_point_through_handler() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.host.renderer.last_pointer_regions = vec![
        NativePanelPointerRegion {
            frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            },
            kind: NativePanelPointerRegionKind::CardsContainer,
        },
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
    ];
    let mut handler = RecordingEventHandler::default();

    let event = runtime
        .dispatch_click_command_at_point_with_handler(
            PanelPoint { x: 30.0, y: 30.0 },
            Instant::now(),
            &mut handler,
        )
        .expect("dispatch point event");

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
    assert!(runtime.host.pending_events.is_empty());
}

#[test]
fn windows_runtime_dispatches_edge_action_click_during_open_transition() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.panel_state.transitioning = true;
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 40.0,
            height: 32.0,
        },
        kind: NativePanelPointerRegionKind::EdgeAction(NativePanelEdgeAction::Settings),
    }];
    let mut handler = RecordingEventHandler::default();

    let event = runtime
        .dispatch_click_command_at_point_with_handler(
            PanelPoint { x: 30.0, y: 30.0 },
            Instant::now(),
            &mut handler,
        )
        .expect("dispatch edge action during transition");

    assert_eq!(event, Some(NativePanelPlatformEvent::ToggleSettingsSurface));
    assert_eq!(
        handler.handled,
        vec![NativePanelPlatformEvent::ToggleSettingsSurface]
    );
    assert!(runtime.host.pending_events.is_empty());
}

#[test]
fn windows_runtime_dispatches_queued_platform_events_through_handler() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.host.pending_events = vec![
        NativePanelPlatformEvent::FocusSession("session-1".to_string()),
        NativePanelPlatformEvent::ToggleCompletionSound,
    ];
    let mut handler = RecordingEventHandler::default();

    dispatch_queued_native_panel_platform_events_with_handler(&mut runtime.host, &mut handler)
        .expect("dispatch queued runtime events");

    assert_eq!(
        handler.handled,
        vec![
            NativePanelPlatformEvent::FocusSession("session-1".to_string()),
            NativePanelPlatformEvent::ToggleCompletionSound,
        ]
    );
    assert!(runtime.host.pending_events.is_empty());
}

#[test]
fn windows_runtime_can_drain_queued_platform_events_without_dispatching_under_lock() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.host.pending_events = vec![
        NativePanelPlatformEvent::ToggleSettingsSurface,
        NativePanelPlatformEvent::CycleDisplay,
    ];

    let events = runtime.take_queued_platform_events();

    assert_eq!(
        events,
        vec![
            NativePanelPlatformEvent::ToggleSettingsSurface,
            NativePanelPlatformEvent::CycleDisplay,
        ]
    );
    assert!(runtime.host.pending_events.is_empty());
}

#[test]
fn windows_runtime_pointer_event_dispatch_is_noop_when_point_has_no_target() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 60.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];
    let mut handler = RecordingEventHandler::default();

    let event = runtime
        .dispatch_click_command_at_point_with_handler(
            PanelPoint { x: 10.0, y: 10.0 },
            Instant::now(),
            &mut handler,
        )
        .expect("dispatch empty point event");

    assert_eq!(event, None);
    assert!(handler.handled.is_empty());
    assert!(runtime.host.pending_events.is_empty());
}

#[test]
fn windows_runtime_focus_click_dispatch_is_debounced() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
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
    }];
    let mut handler = RecordingEventHandler::default();

    let first = runtime
        .dispatch_click_command_at_point_with_handler(
            PanelPoint { x: 30.0, y: 30.0 },
            now,
            &mut handler,
        )
        .expect("dispatch first focus click");
    let duplicate = runtime
        .dispatch_click_command_at_point_with_handler(
            PanelPoint { x: 30.0, y: 30.0 },
            now + Duration::from_millis(100),
            &mut handler,
        )
        .expect("dispatch duplicate focus click");

    assert_eq!(
        first,
        Some(NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        ))
    );
    assert_eq!(duplicate, None);
    assert_eq!(
        handler.handled,
        vec![NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        )]
    );
}

#[test]
fn windows_runtime_window_message_pointer_leave_collapses_and_refreshes() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.panel_state.expanded = true;
    runtime.panel_state.pointer_outside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: 120.0,
            visible_height: 120.0,
            width_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("seed animation descriptor");
    let mut handler = RecordingEventHandler::default();

    let outcome = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Leave,
            now,
            &runtime_input_descriptor(),
            &mut handler,
        )
        .expect("handle pointer leave");

    assert_eq!(
        outcome,
        NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Collapse))
    );
    assert!(!runtime.panel_state.expanded);
    assert!(runtime.scene_cache.last_scene.is_some());
    assert!(runtime.host.renderer.scene_cache.last_scene.is_some());
    assert!(handler.handled.is_empty());
}

#[test]
fn windows_runtime_window_message_click_dispatches_hit_target_event() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
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
    }];
    let mut handler = RecordingEventHandler::default();

    let outcome = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Click(PanelPoint { x: 30.0, y: 30.0 }),
            std::time::Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("handle pointer click");

    assert_eq!(
        outcome,
        NativePanelPointerInputOutcome::Click(Some(NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        )))
    );
    assert_eq!(
        handler.handled,
        vec![NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        )]
    );
}

#[test]
fn windows_runtime_clicks_visual_settings_button_center() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime.panel_state.expanded = true;

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed expanded descriptor");
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("seed expanded scene");
    runtime
        .host
        .present_renderer_state()
        .expect("present expanded frame");
    runtime
        .pump_platform_loop()
        .expect("sync expanded frame into shell");

    let window_state = runtime
        .host
        .window
        .presented_window_state
        .expect("presented window state");
    let surface_height = window_state.frame.expect("window frame").height;
    let presentation = runtime
        .host
        .window
        .presented_presentation_model
        .as_ref()
        .expect("presented presentation");
    let visual_input = native_panel_visual_plan_input_from_presentation(
        window_state,
        NativePanelVisualDisplayMode::Expanded,
        Some(presentation),
    );
    let plan = resolve_native_panel_visual_plan(&visual_input);
    let settings_icon_center = plan
        .primitives
        .iter()
        .find_map(|primitive| match primitive {
            NativePanelVisualPrimitive::Text {
                origin,
                max_width,
                text,
                color,
                size,
                weight,
                ..
            } if text == "\u{E713}"
                && *color == NativePanelVisualColor::rgb(245, 247, 252)
                && *size == 16
                && *weight == NativePanelVisualTextWeight::Normal =>
            {
                Some(PanelPoint {
                    x: origin.x + max_width / 2.0,
                    y: surface_height - origin.y - 12.0,
                })
            }
            _ => None,
        })
        .expect("settings icon text primitive");

    assert_eq!(
        runtime
            .host
            .shell
            .pointer_state_at_point(settings_icon_center)
            .platform_event,
        Some(NativePanelPlatformEvent::ToggleSettingsSurface)
    );
    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            settings_icon_center
        ),
        super::hit_region::WindowsNativePanelHitTest::Client
    );

    let mut handler = RecordingEventHandler::default();
    let outcome = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Click(settings_icon_center),
            Instant::now(),
            &input,
            &mut handler,
        )
        .expect("click settings icon center");

    assert_eq!(
        outcome,
        NativePanelPointerInputOutcome::Click(Some(
            NativePanelPlatformEvent::ToggleSettingsSurface
        ))
    );
    assert_eq!(
        handler.handled,
        vec![NativePanelPlatformEvent::ToggleSettingsSurface]
    );
}

#[test]
fn windows_runtime_window_message_helper_decodes_and_expands_hover() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(pending_permission_snapshot("session-1"));
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 140.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];
    let mut handler = RecordingEventHandler::default();

    let outcome = runtime
        .handle_window_message_with_handler(
            super::window_shell::WINDOWS_WM_MOUSEMOVE,
            ((30_i32 as u32 as u64) | ((30_i32 as u32 as u64) << 16)) as isize,
            now,
            &runtime_input_descriptor(),
            &mut handler,
        )
        .expect("handle decoded move message");

    assert_eq!(
        outcome,
        Some(NativePanelPointerInputOutcome::Hover(Some(
            HoverTransition::Expand
        )))
    );
    assert_eq!(
        runtime.host.shell.last_pointer_input(),
        Some(NativePanelPointerInput::Move(PanelPoint {
            x: 30.0,
            y: 30.0
        }))
    );
    assert!(runtime.panel_state.expanded);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open)
    );
    assert!(handler.handled.is_empty());
}

#[test]
fn windows_runtime_keeps_pending_hover_open_after_badge_clearing_mousemove() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime.panel_state.completion_badge_items = vec![CompletionBadgeItem {
        session_id: "session-1".to_string(),
        completed_at: Utc::now(),
        last_user_prompt: Some("prompt".to_string()),
        last_assistant_message: Some("done".to_string()),
    }];
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];
    let mut handler = RecordingEventHandler::default();

    let first = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Move(PanelPoint { x: 30.0, y: 30.0 }),
            now,
            &runtime_input_descriptor(),
            &mut handler,
        )
        .expect("first hover clears badge and requests open");
    let second = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Move(PanelPoint { x: 30.0, y: 30.0 }),
            now + Duration::from_millis(1),
            &runtime_input_descriptor(),
            &mut handler,
        )
        .expect("second hover before animation starts");

    assert_eq!(
        first,
        NativePanelPointerInputOutcome::Hover(Some(HoverTransition::Expand))
    );
    assert_eq!(second, NativePanelPointerInputOutcome::Hover(None));
    assert!(runtime.panel_state.completion_badge_items.is_empty());
    assert!(runtime.panel_state.expanded);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open)
    );
}

#[test]
fn windows_runtime_window_message_expands_hover_after_presenting_shared_absolute_regions() {
    let now = std::time::Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(pending_permission_snapshot("session-1"));
    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100));
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    runtime.host.window.present(
        NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 100.0,
                y: 50.0,
                width: 320.0,
                height: 120.0,
            }),
            visible: true,
            preferred_display_index: 0,
        },
        &[NativePanelPointerRegion {
            frame: PanelRect {
                x: 110.0,
                y: 110.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        None,
    );
    let frame = runtime
        .host
        .window
        .take_pending_draw_frame()
        .expect("pending draw frame");
    runtime.host.presenter.present(frame);
    runtime
        .pump_platform_loop()
        .expect("present shared regions");
    assert_eq!(
        runtime.host.shell.pointer_regions(),
        &[NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 20.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }]
    );
    let mut handler = RecordingEventHandler::default();

    let outcome = runtime
        .handle_window_message_with_handler(
            super::window_shell::WINDOWS_WM_MOUSEMOVE,
            ((30_i32 as u32 as u64) | ((30_i32 as u32 as u64) << 16)) as isize,
            now,
            &runtime_input_descriptor(),
            &mut handler,
        )
        .expect("handle decoded move message");

    assert_eq!(
        outcome,
        Some(NativePanelPointerInputOutcome::Hover(Some(
            HoverTransition::Expand
        )))
    );
    assert!(runtime.panel_state.expanded);
}

#[test]
fn windows_runtime_window_message_helper_decodes_and_dispatches_click() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
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
    }];
    let mut handler = RecordingEventHandler::default();

    let outcome = runtime
        .handle_window_message_with_handler(
            super::window_shell::WINDOWS_WM_LBUTTONUP,
            ((30_i32 as u32 as u64) | ((30_i32 as u32 as u64) << 16)) as isize,
            std::time::Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("handle decoded click message");

    assert_eq!(
        outcome,
        Some(NativePanelPointerInputOutcome::Click(Some(
            NativePanelPlatformEvent::FocusSession("session-1".to_string())
        )))
    );
    assert_eq!(
        handler.handled,
        vec![NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        )]
    );
}

#[derive(Default)]
struct RecordingEventHandler {
    handled: Vec<NativePanelPlatformEvent>,
}

impl NativePanelRuntimeCommandCapability for RecordingEventHandler {
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
fn windows_host_dispatches_queued_platform_events_through_handler() {
    let mut host = super::WindowsNativePanelHost::default();
    host.pending_events = vec![
        NativePanelPlatformEvent::FocusSession("session-1".to_string()),
        NativePanelPlatformEvent::ToggleCompletionSound,
        NativePanelPlatformEvent::ToggleMascot,
        NativePanelPlatformEvent::OpenSettingsLocation,
        NativePanelPlatformEvent::OpenReleasePage,
    ];
    let mut handler = RecordingEventHandler::default();

    dispatch_queued_native_panel_platform_events_with_handler(&mut host, &mut handler)
        .expect("dispatch queued events");

    assert_eq!(
        handler.handled,
        vec![
            NativePanelPlatformEvent::FocusSession("session-1".to_string()),
            NativePanelPlatformEvent::ToggleCompletionSound,
            NativePanelPlatformEvent::ToggleMascot,
            NativePanelPlatformEvent::OpenSettingsLocation,
            NativePanelPlatformEvent::OpenReleasePage,
        ]
    );
    assert!(host.pending_events.is_empty());
}

#[test]
fn windows_renderer_caches_scene_and_resolves_shared_render_inputs() {
    let mut panel_state = PanelState::default();
    let bundle = test_runtime_scene_bundle(
        &mut panel_state,
        &snapshot(),
        &PanelSceneBuildInput::default(),
    );
    let scene = bundle.scene;
    let runtime_render_state = bundle.runtime_render_state;
    let mut renderer = super::WindowsNativePanelRenderer::default();

    renderer.update_screen_frame(Some(PanelRect {
        x: 100.0,
        y: 50.0,
        width: 1000.0,
        height: 700.0,
    }));
    renderer
        .render_scene(&scene, runtime_render_state)
        .expect("render scene");
    renderer
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 140.0,
            width_progress: 0.5,
            height_progress: 0.75,
            shoulder_progress: 1.0,
            drop_progress: 0.25,
            cards_progress: 0.8,
        })
        .expect("apply descriptor");

    assert_eq!(
        renderer
            .scene_cache
            .last_scene
            .as_ref()
            .map(|cached| cached.surface),
        Some(scene.surface)
    );
    assert_eq!(
        renderer.scene_cache.last_runtime_render_state,
        Some(runtime_render_state)
    );
    assert_eq!(
        renderer.last_layout,
        Some(crate::native_panel_core::PanelLayout {
            panel_frame: PanelRect {
                x: 390.0,
                y: 570.0,
                width: 420.0,
                height: 180.0,
            },
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 420.0,
                height: 180.0,
            },
            pill_frame: PanelRect {
                x: 76.0,
                y: 141.875,
                width: 268.0,
                height: 37.0,
            },
            left_shoulder_frame: PanelRect {
                x: 70.0,
                y: 172.875,
                width: 6.0,
                height: 6.0,
            },
            right_shoulder_frame: PanelRect {
                x: 344.0,
                y: 172.875,
                width: 6.0,
                height: 6.0,
            },
            expanded_frame: PanelRect {
                x: 76.0,
                y: 65.46875,
                width: 268.0,
                height: 113.40625,
            },
            cards_frame: PanelRect {
                x: 10.0,
                y: 10.0,
                width: 248.0,
                height: 57.40625,
            },
            separator_frame: PanelRect {
                x: 14.0,
                y: 75.90625,
                width: 240.0,
                height: 1.0,
            },
            shared_content_frame: PanelRect {
                x: 476.0,
                y: 645.46875,
                width: 248.0,
                height: 57.40625,
            },
            shell_visible: true,
            separator_visibility: 0.66,
        })
    );

    let render_state = renderer.last_render_state.expect("cached render state");
    assert!(!render_state.shared.enabled);
    assert!(!render_state.shared.visible);
    assert_eq!(
        render_state.layer_style.headline_emphasized,
        runtime_render_state.shell_scene.headline_emphasized
    );
    assert_eq!(
        render_state.layer_style.edge_actions_visible,
        runtime_render_state.shell_scene.edge_actions_visible
    );
    assert!(
        renderer
            .last_pointer_regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CardsContainer))
    );
    let command_bundle = renderer
        .scene_cache
        .last_render_command_bundle
        .as_ref()
        .expect("cached render command bundle");
    assert_eq!(
        command_bundle.compact_bar.frame,
        command_bundle.layout.pill_frame
    );
    assert_eq!(
        command_bundle.compact_bar.headline.text,
        scene.compact_bar.headline.text
    );
    assert_eq!(
        command_bundle.card_stack.frame,
        command_bundle.layout.cards_frame
    );
    assert_eq!(command_bundle.card_stack.cards.len(), scene.cards.len());
    assert_eq!(command_bundle.mascot.pose, scene.mascot_pose);
}

#[test]
fn windows_renderer_resolves_pointer_regions_from_shared_scene_and_layout() {
    let mut panel_state = PanelState::default();
    let bundle = test_runtime_scene_bundle(
        &mut panel_state,
        &pending_permission_snapshot("session-1"),
        &PanelSceneBuildInput::default(),
    );
    let scene = bundle.scene;
    let runtime_render_state = bundle.runtime_render_state;
    let mut renderer = super::WindowsNativePanelRenderer::default();

    renderer.update_screen_frame(Some(PanelRect {
        x: 100.0,
        y: 50.0,
        width: 1000.0,
        height: 700.0,
    }));
    renderer
        .render_scene(&scene, runtime_render_state)
        .expect("render scene");
    renderer
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 140.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("apply descriptor");

    assert!(
        renderer
            .last_pointer_regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CompactBar))
    );
    assert!(
        renderer
            .last_pointer_regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::CardsContainer))
    );
    assert!(renderer.last_pointer_regions.iter().any(|region| matches!(
        &region.kind,
        NativePanelPointerRegionKind::HitTarget(target)
            if target.action == PanelHitAction::FocusSession
                && target.value == "session-1"
    )));
}

#[test]
fn windows_renderer_caches_complete_render_commands() {
    let mut expanded_state = PanelState {
        expanded: true,
        ..PanelState::default()
    };
    let expanded_bundle = test_runtime_scene_bundle(
        &mut expanded_state,
        &snapshot(),
        &PanelSceneBuildInput::default(),
    );
    let mut renderer = super::WindowsNativePanelRenderer::default();
    renderer
        .render_scene(&expanded_bundle.scene, expanded_bundle.runtime_render_state)
        .expect("render expanded scene");
    renderer
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("apply expanded descriptor");

    let expanded_command = renderer
        .scene_cache
        .last_render_command_bundle
        .as_ref()
        .expect("expanded render command");
    let expanded_presentation = renderer
        .last_presentation_model
        .as_ref()
        .expect("expanded presentation model");
    assert!(expanded_command.compact_bar.actions_visible);
    assert_eq!(
        expanded_command.card_stack.cards.len(),
        expanded_bundle.scene.cards.len()
    );
    assert_eq!(
        expanded_command.mascot.pose,
        expanded_bundle.scene.mascot_pose
    );
    assert_eq!(expanded_command.action_buttons.len(), 2);
    assert!(expanded_command.glow.is_none());
    assert_eq!(
        expanded_presentation.panel_frame,
        expanded_command.layout.panel_frame
    );
    assert_eq!(
        expanded_presentation.compact_bar.frame,
        expanded_command.compact_bar.frame
    );

    let completion_state = PanelState {
        completion_badge_items: vec![CompletionBadgeItem {
            session_id: "session-1".to_string(),
            completed_at: Utc::now(),
            last_user_prompt: None,
            last_assistant_message: Some("Done".to_string()),
        }],
        ..PanelState::default()
    };
    let completion_scene = build_panel_scene(
        &completion_state,
        &snapshot(),
        &PanelSceneBuildInput::default(),
    );
    renderer
        .render_scene(&completion_scene, PanelRuntimeRenderState::default())
        .expect("render completion scene");
    renderer
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: 80.0,
            visible_height: 80.0,
            width_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("apply completion descriptor");

    let completion_command = renderer
        .scene_cache
        .last_render_command_bundle
        .as_ref()
        .expect("completion render command");
    assert!(completion_command.glow.is_some());
    assert!(
        renderer
            .last_presentation_model
            .as_ref()
            .and_then(|presentation| presentation.glow.as_ref())
            .is_some()
    );
    assert_eq!(
        completion_command.compact_bar.completion_count,
        completion_scene.compact_bar.completion_count
    );
}

#[test]
fn windows_runtime_scene_state_bridge_syncs_current_bundle_and_pointer_regions() {
    let input = runtime_input_descriptor();
    let mut runtime = super::WindowsNativePanelRuntime {
        panel_state: PanelState {
            expanded: true,
            ..PanelState::default()
        },
        ..Default::default()
    };
    let bundle =
        test_runtime_scene_bundle(&mut runtime.panel_state, &snapshot(), &input.scene_input);

    runtime
        .host
        .renderer
        .render_scene(&bundle.scene, bundle.runtime_render_state)
        .expect("render scene");
    runtime
        .host
        .renderer
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("apply descriptor");

    let rendered_bundle = runtime
        .host
        .renderer
        .scene_cache
        .last_render_command_bundle
        .clone()
        .expect("rendered bundle");
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.host.renderer.last_pointer_regions.clear();

    cache_render_command_bundle_for_state_bridge_with_input(&mut runtime, &input, &rendered_bundle);

    let current_bundle =
        resolve_current_native_panel_render_command_bundle_for_state_bridge_with_input(
            &runtime, &input,
        )
        .expect("current bundle");

    assert_eq!(
        runtime.host.renderer.last_pointer_regions.len(),
        rendered_bundle.pointer_regions.len()
    );
    assert_eq!(
        current_bundle.compact_bar.headline.text,
        rendered_bundle.compact_bar.headline.text
    );
}

#[test]
fn windows_host_presents_renderer_state_into_window() {
    let mut host = super::WindowsNativePanelHost::default();
    let scene = build_panel_scene(
        &PanelState {
            expanded: true,
            ..PanelState::default()
        },
        &snapshot(),
        &PanelSceneBuildInput::default(),
    );
    host.renderer.last_window_state = Some(NativePanelHostWindowState {
        frame: Some(PanelRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 120.0,
        }),
        visible: true,
        preferred_display_index: 1,
    });
    host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];
    host.renderer.scene_cache.last_scene = Some(scene.clone());

    host.present_renderer_state()
        .expect("present renderer state");

    assert!(host.presenter.redraw_requested());
    let draw_frame = host.take_pending_draw_frame().expect("pending draw frame");

    assert_eq!(
        host.window.presented_window_state,
        host.renderer.last_window_state
    );
    assert_eq!(
        host.window.pointer_regions(&[]),
        &[NativePanelPointerRegion {
            frame: PanelRect {
                x: 10.0,
                y: 80.0,
                width: 80.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }]
    );
    assert_eq!(
        host.window
            .presented_presentation_model
            .as_ref()
            .map(|presentation| presentation.compact_bar.headline.text.as_str()),
        Some(scene.compact_bar.headline.text.as_str())
    );
    assert_eq!(
        draw_frame.window_state,
        host.window.presented_window_state.unwrap()
    );
    assert_eq!(
        draw_frame
            .presentation_model
            .as_ref()
            .map(|presentation| presentation.compact_bar.headline.text.as_str()),
        Some(scene.compact_bar.headline.text.as_str())
    );
    assert!(!host.presenter.redraw_requested());
    assert!(host.take_pending_draw_frame().is_none());
}

#[test]
fn windows_runtime_user_hide_blocks_snapshot_refresh_until_show() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    runtime.hide_panel().expect("hide panel");
    runtime.pump_platform_loop().expect("pump hide");

    assert!(runtime.user_hidden);
    assert_eq!(runtime.platform_loop.last_visible, Some(false));

    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("hidden snapshot sync");
    runtime.pump_platform_loop().expect("pump hidden refresh");

    assert!(runtime.user_hidden);
    assert_eq!(runtime.platform_loop.last_visible, Some(false));

    runtime.create_panel().expect("show panel");
    runtime.pump_platform_loop().expect("pump show");

    assert!(!runtime.user_hidden);
    assert_eq!(runtime.platform_loop.last_visible, Some(true));
}

#[test]
fn windows_host_shell_can_consume_presenter_frame() {
    let mut host = super::WindowsNativePanelHost::default();
    host.presenter.present(WindowsNativePanelDrawFrame {
        window_state: NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 12.0,
                y: 24.0,
                width: 256.0,
                height: 96.0,
            }),
            visible: true,
            preferred_display_index: 0,
        },
        pointer_regions: Vec::new(),
        presentation_model: None,
    });

    assert!(host.consume_presenter_into_shell());
    assert_eq!(host.shell.redraw_requests(), 1);
    assert_eq!(
        host.shell
            .last_frame()
            .and_then(|frame| frame.window_state.frame)
            .map(|frame| frame.width),
        Some(256.0)
    );
    assert!(host.shell.pending_paint_job().is_some());
    assert!(!host.consume_presenter_into_shell());
}

#[test]
fn windows_host_shell_paints_pending_presenter_frame() {
    let mut host = super::WindowsNativePanelHost::default();
    host.presenter.present(WindowsNativePanelDrawFrame {
        window_state: NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 100.0,
            }),
            visible: true,
            preferred_display_index: 0,
        },
        pointer_regions: Vec::new(),
        presentation_model: None,
    });

    let result = host.consume_presenter_into_shell_result();

    assert!(result.redraw_requested);
    assert!(result.paint_queued);
    assert!(host.shell.pending_paint_job().is_some());
    let paint_job = host.shell.paint_next_frame().expect("paint job");
    assert_eq!(
        paint_job.display_mode,
        NativePanelVisualDisplayMode::Compact
    );
    assert_eq!(host.shell.paint_pass_count(), 1);
    assert_eq!(
        host.shell
            .last_painted_job()
            .map(|job| job.panel_frame.width),
        Some(300.0)
    );
}

#[test]
fn windows_runtime_records_pointer_input_on_window_event_path() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let mut handler = RecordingEventHandler::default();

    let _ = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Move(PanelPoint { x: 8.0, y: 16.0 }),
            std::time::Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("handle move");

    assert_eq!(
        runtime.host.shell.last_pointer_input(),
        Some(NativePanelPointerInput::Move(PanelPoint {
            x: 8.0,
            y: 16.0
        }))
    );
}

#[test]
fn windows_runtime_pointer_move_syncs_mouse_passthrough_state() {
    let mut runtime = super::WindowsNativePanelRuntime {
        ignores_mouse_events: true,
        ..Default::default()
    };
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 40.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        false,
    ));
    let _ = runtime.host.consume_presenter_into_shell_result();
    let mut handler = RecordingEventHandler::default();

    let _ = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Move(PanelPoint { x: 20.0, y: 20.0 }),
            Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("handle move");

    assert!(!runtime.ignores_mouse_events);
    assert_eq!(runtime.host.shell.last_ignores_mouse_events(), Some(false));
    assert!(
        runtime
            .host
            .take_pending_shell_commands()
            .into_iter()
            .any(|command| matches!(
                command,
                super::window_shell::WindowsNativePanelShellCommand::SyncMouseEventPassthrough(
                    false
                )
            ))
    );
}

#[test]
fn windows_runtime_pointer_leave_syncs_mouse_passthrough_state() {
    let mut runtime = super::WindowsNativePanelRuntime {
        ignores_mouse_events: false,
        ..Default::default()
    };
    let mut handler = RecordingEventHandler::default();

    let _ = runtime
        .handle_pointer_input_with_handler(
            NativePanelPointerInput::Leave,
            Instant::now(),
            &NativePanelRuntimeInputDescriptor {
                scene_input: PanelSceneBuildInput::default(),
                screen_frame: None,
            },
            &mut handler,
        )
        .expect("handle leave");

    assert!(runtime.ignores_mouse_events);
    assert_eq!(runtime.host.shell.last_ignores_mouse_events(), Some(true));
    assert!(
        runtime
            .host
            .take_pending_shell_commands()
            .into_iter()
            .any(|command| matches!(
                command,
                super::window_shell::WindowsNativePanelShellCommand::SyncMouseEventPassthrough(
                    true
                )
            ))
    );
}

#[test]
fn windows_runtime_host_polling_interaction_updates_passthrough_state() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime {
        ignores_mouse_events: true,
        ..Default::default()
    };
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 110.0,
                y: 60.0,
                width: 100.0,
                height: 30.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        false,
    ));
    let present = runtime.host.consume_presenter_into_shell_result();

    assert!(present.display_updated);

    let interaction = runtime
        .sync_host_polling_interaction(PanelPoint { x: 120.0, y: 70.0 }, false, now)
        .expect("polling interaction");

    assert!(interaction.interactive_inside);
    assert_eq!(interaction.click_command, PanelInteractionCommand::None);
    assert!(!interaction.next_ignores_mouse_events);
    assert!(interaction.sync_mouse_event_passthrough);
    assert!(!runtime.ignores_mouse_events);
    assert_eq!(runtime.host.shell.last_ignores_mouse_events(), Some(false));
    assert!(
        runtime
            .host
            .take_pending_shell_commands()
            .into_iter()
            .any(|command| matches!(
                command,
                super::window_shell::WindowsNativePanelShellCommand::SyncMouseEventPassthrough(
                    false
                )
            ))
    );
}

#[test]
fn windows_runtime_host_polling_interaction_marks_completion_viewed_on_hover_expand() {
    let now = Instant::now();
    let input = runtime_input_descriptor();
    let mut runtime = super::WindowsNativePanelRuntime {
        panel_state: PanelState {
            pointer_inside_since: Some(
                now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100),
            ),
            completion_badge_items: vec![CompletionBadgeItem {
                session_id: "session-1".to_string(),
                completed_at: Utc::now(),
                last_user_prompt: None,
                last_assistant_message: Some("Done".to_string()),
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("sync snapshot");
    runtime.panel_state.completion_badge_items = vec![CompletionBadgeItem {
        session_id: "session-1".to_string(),
        completed_at: Utc::now(),
        last_user_prompt: None,
        last_assistant_message: Some("Done".to_string()),
    }];
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 110.0,
                y: 60.0,
                width: 100.0,
                height: 30.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        false,
    ));
    let present = runtime.host.consume_presenter_into_shell_result();

    assert!(present.display_updated);

    let interaction = runtime
        .sync_host_polling_interaction_and_refresh(
            PanelPoint { x: 120.0, y: 70.0 },
            false,
            now,
            &input,
        )
        .expect("polling interaction")
        .expect("polling facts");

    assert!(interaction.interactive_inside);
    assert_eq!(
        interaction.transition_request,
        Some(NativePanelTransitionRequest::Open)
    );
    assert!(runtime.panel_state.expanded);
    assert!(runtime.panel_state.completion_badge_items.is_empty());
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open)
    );
}

#[test]
fn windows_runtime_hover_expanded_panel_switches_to_new_completion_status_message() {
    let now = Instant::now();
    let input = runtime_input_descriptor();
    let mut running_snapshot = sessions_snapshot(1);
    running_snapshot.sessions[0].status = "Running".to_string();
    running_snapshot.sessions[0].last_assistant_message = Some("Working".to_string());
    let mut completed_snapshot = running_snapshot.clone();
    completed_snapshot.sessions[0].status = "Idle".to_string();
    completed_snapshot.sessions[0].last_activity = Utc::now();
    completed_snapshot.sessions[0].last_assistant_message = Some("Done".to_string());
    let mut runtime = super::WindowsNativePanelRuntime {
        panel_state: PanelState {
            expanded: true,
            pointer_inside_since: Some(
                now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100),
            ),
            surface_mode: ExpandedSurface::Default,
            ..Default::default()
        },
        ..Default::default()
    };

    runtime
        .sync_snapshot_bundle(&running_snapshot, &input)
        .expect("seed running snapshot");
    runtime
        .sync_snapshot_bundle(&completed_snapshot, &input)
        .expect("sync completion snapshot");

    assert!(runtime.panel_state.expanded);
    assert!(runtime.panel_state.status_auto_expanded);
    assert_eq!(runtime.panel_state.surface_mode, ExpandedSurface::Status);
    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("completion status presentation");
    assert_eq!(presentation.card_stack.surface, ExpandedSurface::Status);
    assert!(
        presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusCompletion { .. }))
    );
    assert_eq!(presentation.mascot.pose, SceneMascotPose::MessageBubble);
}

#[test]
fn windows_runtime_host_polling_interaction_resolves_hit_target_click() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime {
        panel_state: PanelState {
            expanded: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let target = PanelHitTarget {
        action: PanelHitAction::FocusSession,
        value: "session-1".to_string(),
    };
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 110.0,
                y: 90.0,
                width: 200.0,
                height: 50.0,
            },
            kind: NativePanelPointerRegionKind::HitTarget(target.clone()),
        }],
        true,
    ));
    let present = runtime.host.consume_presenter_into_shell_result();

    assert!(present.display_updated);

    let interaction = runtime
        .sync_host_polling_interaction(PanelPoint { x: 140.0, y: 110.0 }, true, now)
        .expect("polling interaction");

    assert!(interaction.interactive_inside);
    assert_eq!(
        interaction.click_command,
        PanelInteractionCommand::HitTarget(target)
    );
    assert!(!interaction.next_ignores_mouse_events);
    assert!(!interaction.sync_mouse_event_passthrough);
    assert!(runtime.primary_pointer_down);
    assert!(
        runtime
            .last_focus_click
            .as_ref()
            .is_some_and(|(session_id, _)| session_id == "session-1")
    );
    assert!(
        !runtime
            .host
            .take_pending_shell_commands()
            .into_iter()
            .any(|command| matches!(
                command,
                super::window_shell::WindowsNativePanelShellCommand::SyncMouseEventPassthrough(_)
            ))
    );
}

#[test]
fn windows_runtime_pump_platform_loop_consumes_passthrough_command() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime {
        ignores_mouse_events: true,
        ..Default::default()
    };
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 110.0,
                y: 60.0,
                width: 100.0,
                height: 30.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        false,
    ));
    let _ = runtime.host.consume_presenter_into_shell_result();
    let _ = runtime
        .sync_host_polling_interaction(PanelPoint { x: 120.0, y: 70.0 }, false, now)
        .expect("polling interaction");

    assert_eq!(runtime.host.shell.last_ignores_mouse_events(), Some(false));

    runtime.pump_platform_loop().expect("pump platform loop");

    assert_eq!(runtime.platform_loop.last_ignores_mouse_events, Some(false));
    assert_eq!(runtime.platform_loop.redraw_request_count, 1);
    assert!(runtime.host.take_pending_shell_commands().is_empty());
}

#[test]
fn windows_runtime_pump_platform_loop_auto_consumes_presenter_frame() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime
        .host
        .presenter
        .present(shell_draw_frame(Vec::new(), false));
    runtime.create_panel().expect("create panel");

    runtime.pump_platform_loop().expect("pump platform loop");

    assert_eq!(runtime.host.shell.redraw_requests(), 1);
    assert_eq!(runtime.platform_loop.redraw_request_count, 1);
    assert!(
        runtime.host.shell.pending_paint_job().is_some()
            || runtime.platform_loop.last_painted_job.is_some()
    );
}

#[test]
fn windows_platform_hit_region_cache_uses_shared_pointer_regions() {
    let hwnd = 4242;
    let regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 10.0,
            width: 120.0,
            height: 36.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];

    super::platform_loop::sync_windows_native_panel_hit_regions(Some(hwnd), &regions);

    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            PanelPoint { x: 40.0, y: 20.0 }
        ),
        super::hit_region::WindowsNativePanelHitTest::Client
    );
    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            PanelPoint { x: 4.0, y: 4.0 }
        ),
        super::hit_region::WindowsNativePanelHitTest::Transparent
    );

    super::platform_loop::clear_windows_native_panel_hit_regions(Some(hwnd));

    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            PanelPoint { x: 40.0, y: 20.0 }
        ),
        super::hit_region::WindowsNativePanelHitTest::Transparent
    );
}

#[test]
fn windows_runtime_pump_platform_loop_syncs_hit_regions_after_presenter_frame() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    runtime.host.presenter.present(shell_draw_frame(
        vec![NativePanelPointerRegion {
            frame: PanelRect {
                x: 110.0,
                y: 60.0,
                width: 100.0,
                height: 30.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
        false,
    ));

    runtime
        .pump_platform_loop()
        .expect("pump presenter hit regions");

    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            PanelPoint { x: 120.0, y: 70.0 }
        ),
        super::hit_region::WindowsNativePanelHitTest::Client
    );
    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            PanelPoint { x: 10.0, y: 10.0 }
        ),
        super::hit_region::WindowsNativePanelHitTest::Transparent
    );
}

#[test]
fn windows_platform_loop_surface_resource_revision_tracks_physical_rect_changes() {
    let mut state = super::platform_loop::WindowsNativePanelPlatformLoopState::default();
    let first = Some(super::dpi::WindowsPhysicalRect {
        x: -1600,
        y: 0,
        width: 316,
        height: 100,
    });
    let second = Some(super::dpi::WindowsPhysicalRect {
        x: -1600,
        y: 0,
        width: 380,
        height: 120,
    });

    state.sync_surface_resource_rect(first);
    assert_eq!(state.surface_resource_revision, 1);
    assert_eq!(state.last_physical_window_rect, first);

    state.sync_surface_resource_rect(first);
    assert_eq!(state.surface_resource_revision, 1);

    state.sync_surface_resource_rect(second);
    assert_eq!(state.surface_resource_revision, 2);
    assert_eq!(state.last_physical_window_rect, second);
}

#[test]
fn windows_platform_loop_records_physical_window_rect_from_sync_command() {
    let mut state = super::platform_loop::WindowsNativePanelPlatformLoopState::default();
    let mut raw_window_handle = Some(1);
    let window_state = NativePanelHostWindowState {
        frame: Some(PanelRect {
            x: -1280.0,
            y: -16.0,
            width: 253.0,
            height: 80.0,
        }),
        visible: true,
        preferred_display_index: 1,
    };

    state
        .consume_shell_command(
            &mut raw_window_handle,
            super::window_shell::WindowsNativePanelShellCommand::SyncWindowState(window_state),
        )
        .expect("sync window state");

    assert_eq!(
        state.last_physical_window_rect,
        Some(super::dpi::WindowsPhysicalRect {
            x: -1280,
            y: -16,
            width: 253,
            height: 80,
        })
    );
    assert_eq!(state.surface_resource_revision, 1);
}

#[test]
fn windows_native_window_state_positioning_keeps_panel_topmost_without_activation() {
    let behavior = super::platform_loop::windows_native_window_positioning_behavior();

    assert!(behavior.topmost);
    assert!(behavior.no_activate);
    assert!(!behavior.preserve_existing_z_order);
}

#[test]
fn windows_platform_loop_surface_resource_revision_tracks_dpi_scale_changes() {
    let mut state = super::platform_loop::WindowsNativePanelPlatformLoopState::default();
    let physical_rect = Some(super::dpi::WindowsPhysicalRect {
        x: 100,
        y: 40,
        width: 300,
        height: 120,
    });

    state.sync_surface_resource_state(physical_rect, super::dpi::WindowsDpiScale::from_scale(1.0));
    assert_eq!(state.surface_resource_revision, 1);
    assert_eq!(
        state.last_surface_dpi_scale,
        Some(super::dpi::WindowsDpiScale::from_scale(1.0))
    );

    state.sync_surface_resource_state(physical_rect, super::dpi::WindowsDpiScale::from_scale(1.0));
    assert_eq!(state.surface_resource_revision, 1);

    state.sync_surface_resource_state(physical_rect, super::dpi::WindowsDpiScale::from_scale(1.5));
    assert_eq!(state.surface_resource_revision, 2);
    assert_eq!(
        state.last_surface_dpi_scale,
        Some(super::dpi::WindowsDpiScale::from_scale(1.5))
    );
}

#[test]
fn windows_platform_loop_tracks_negative_origin_physical_rect_after_dpi_change() {
    let mut state = super::platform_loop::WindowsNativePanelPlatformLoopState::default();
    let logical_frame = PanelRect {
        x: -1280.0,
        y: -24.0,
        width: 253.0,
        height: 80.0,
    };

    state.sync_surface_resource_state(
        Some(super::dpi::WindowsDpiScale::from_scale(1.25).rect_to_physical(logical_frame)),
        super::dpi::WindowsDpiScale::from_scale(1.25),
    );
    assert_eq!(
        state.last_physical_window_rect,
        Some(super::dpi::WindowsPhysicalRect {
            x: -1600,
            y: -30,
            width: 316,
            height: 100,
        })
    );
    assert_eq!(state.surface_resource_revision, 1);

    state.sync_surface_resource_state(
        Some(super::dpi::WindowsDpiScale::from_scale(1.5).rect_to_physical(logical_frame)),
        super::dpi::WindowsDpiScale::from_scale(1.5),
    );
    assert_eq!(
        state.last_physical_window_rect,
        Some(super::dpi::WindowsPhysicalRect {
            x: -1920,
            y: -36,
            width: 380,
            height: 120,
        })
    );
    assert_eq!(state.surface_resource_revision, 2);
}

#[test]
fn windows_runtime_display_reposition_updates_platform_physical_rect() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 120.0,
            visible_height: 120.0,
            width_progress: 1.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("seed descriptor");
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");

    runtime
        .host
        .reposition_to_display(
            2,
            Some(PanelRect {
                x: -1280.0,
                y: 0.0,
                width: 1280.0,
                height: 720.0,
            }),
        )
        .expect("reposition display");
    runtime.pump_platform_loop().expect("pump reposition");

    assert_eq!(
        runtime.platform_loop.last_physical_window_rect,
        Some(super::dpi::WindowsPhysicalRect {
            x: -850,
            y: 0,
            width: 420,
            height: 120,
        })
    );
    assert_eq!(
        runtime
            .platform_loop
            .last_window_state
            .expect("last window state")
            .preferred_display_index,
        2
    );
    assert!(runtime.platform_loop.surface_resource_revision >= 2);
}

#[test]
fn windows_animation_scheduler_starts_open_transition_and_queues_redraw() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime
        .sync_snapshot_bundle(
            &pending_permission_snapshot("session-1"),
            &runtime_input_descriptor(),
        )
        .expect("sync transition snapshot");
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open)
    );
    let _ = runtime.host.presenter.take_redraw_frame();

    let frame = runtime
        .advance_animation_frame_at(now)
        .expect("advance animation")
        .expect("opening frame");

    assert_eq!(frame.descriptor.animation.kind, PanelAnimationKind::Open);
    assert!(frame.continue_animating);
    assert!(runtime.panel_state.transitioning);
    assert!(runtime.animation_scheduler.is_active());
    assert!(runtime.host.presenter.redraw_requested());
}

#[test]
fn windows_animation_scheduler_finishes_close_transition_without_redraw_loop() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed expanded descriptor");
    runtime.last_transition_request = Some(NativePanelTransitionRequest::Close);
    let first = runtime
        .advance_animation_frame_at(now)
        .expect("advance close")
        .expect("closing frame");
    assert_eq!(first.descriptor.animation.kind, PanelAnimationKind::Close);
    assert!(first.continue_animating);
    assert!(runtime.animation_scheduler.is_active());
    let _ = runtime.host.presenter.take_redraw_frame();

    let mut final_frame = None;
    for step in 1..=first
        .total_ms
        .div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS)
        + 1
    {
        let frame = runtime
            .advance_animation_frame_at(
                now + Duration::from_millis(
                    first.total_ms + step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                ),
            )
            .expect("advance close frame")
            .expect("closing frame");
        if !frame.continue_animating {
            final_frame = Some(frame);
            break;
        }
    }
    let final_frame = final_frame.expect("terminal frame");
    assert!(!final_frame.continue_animating);
    assert!(!runtime.animation_scheduler.is_active());
    assert!(!runtime.panel_state.transitioning);
    let _ = runtime.host.presenter.take_redraw_frame();

    let idle = runtime
        .advance_animation_frame_at(now + Duration::from_millis(first.total_ms + 32))
        .expect("idle advance");
    assert!(idle.is_none());
    assert!(!runtime.host.presenter.redraw_requested());
}

#[test]
fn windows_runtime_refreshes_active_count_marquee_paint_job_without_scene_sync() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let now = Instant::now();
    let mut frame = shell_draw_frame(Vec::new(), true);
    frame
        .presentation_model
        .as_mut()
        .expect("presentation model")
        .compact_bar
        .active_count = "23".to_string();
    runtime.host.presenter.present(frame);
    runtime.host.consume_presenter_into_shell_result();
    runtime.active_count_marquee_started_at =
        Some(now - Duration::from_millis(ACTIVE_COUNT_SCROLL_HOLD_MS as u64 + 10));

    assert!(runtime.refresh_active_count_marquee_frame_at(now));

    let paint_job = runtime
        .host
        .shell
        .pending_paint_job()
        .expect("marquee paint job");
    assert_eq!(paint_job.active_count, "23");
    assert!(paint_job.active_count_elapsed_ms >= ACTIVE_COUNT_SCROLL_HOLD_MS);
    assert!(runtime.host.shell.redraw_requests() > 0);
}

#[test]
fn windows_runtime_refreshes_mascot_animation_paint_job_without_scene_sync() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let now = Instant::now();
    let mut frame = shell_draw_frame(Vec::new(), true);
    frame
        .presentation_model
        .as_mut()
        .expect("presentation model")
        .mascot
        .pose = SceneMascotPose::Idle;
    runtime.host.presenter.present(frame);
    runtime.host.consume_presenter_into_shell_result();
    runtime.mascot_animation_started_at =
        Some(now - Duration::from_millis(crate::native_panel_core::MASCOT_ANIMATION_REFRESH_MS));

    assert!(runtime.refresh_mascot_animation_frame_at(now));

    let paint_job = runtime
        .host
        .shell
        .pending_paint_job()
        .expect("mascot paint job");
    assert_eq!(paint_job.mascot_pose, SceneMascotPose::Idle);
    assert!(paint_job.mascot_elapsed_ms > 0);
    assert!(runtime.host.shell.redraw_requests() > 0);
}

#[test]
fn windows_runtime_skips_lightweight_refreshes_during_panel_transition() {
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let now = Instant::now();
    let mut frame = shell_draw_frame(Vec::new(), true);
    let presentation = frame
        .presentation_model
        .as_mut()
        .expect("presentation model");
    presentation.compact_bar.active_count = "23".to_string();
    presentation.mascot.pose = SceneMascotPose::Idle;
    runtime.host.presenter.present(frame);
    runtime.host.consume_presenter_into_shell_result();
    let _ = runtime.host.shell.paint_next_frame();
    runtime.panel_state.transitioning = true;
    runtime.active_count_marquee_started_at =
        Some(now - Duration::from_millis(ACTIVE_COUNT_SCROLL_HOLD_MS as u64 + 10));
    runtime.mascot_animation_started_at =
        Some(now - Duration::from_millis(crate::native_panel_core::MASCOT_ANIMATION_REFRESH_MS));

    assert!(!runtime.refresh_active_count_marquee_frame_at(now));
    assert!(!runtime.refresh_mascot_animation_frame_at(now));
    assert!(runtime.host.shell.pending_paint_job().is_none());
    assert!(runtime.active_count_marquee_started_at.is_none());
    assert!(runtime.mascot_animation_started_at.is_none());
}

#[test]
fn windows_animation_scheduler_preserves_status_cards_when_auto_status_close_skips_default_rebuild()
{
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("seed auto status surface");
    runtime
        .host
        .present_renderer_state()
        .expect("present status surface");
    let presented_status_cards = runtime
        .host
        .window
        .presented_presentation_model
        .as_ref()
        .expect("presented status model")
        .card_stack
        .cards
        .clone();
    assert!(
        presented_status_cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusApproval { .. }))
    );
    runtime.panel_state.status_queue[0].expires_at = Instant::now() - Duration::from_millis(1);
    runtime
        .refresh_status_queue_from_last_raw_snapshot_with_input(&input)
        .expect("refresh expired status queue");
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Close)
    );
    assert!(runtime.panel_state.skip_next_close_card_exit);
    let pre_close_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("pre-close presentation");
    assert!(!pre_close_presentation.compact_bar.actions_visible);
    assert!(!pre_close_presentation.action_buttons.visible);
    assert!(
        pre_close_presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusApproval { .. }))
    );
    let pre_close_frame = runtime
        .host
        .take_pending_draw_frame()
        .expect("pre-close draw frame");
    let pre_close_frame_presentation = pre_close_frame
        .presentation_model
        .as_ref()
        .expect("pre-close frame presentation");
    assert!(!pre_close_frame_presentation.compact_bar.actions_visible);
    assert!(!pre_close_frame_presentation.action_buttons.visible);
    assert!(
        pre_close_frame_presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusApproval { .. }))
    );
    runtime.host.presenter.present(pre_close_frame);
    runtime
        .pump_platform_loop()
        .expect("pump pre-close frame into shell");
    let shell_job = runtime
        .host
        .shell
        .pending_paint_job()
        .expect("pre-close shell paint job");
    assert!(!shell_job.action_buttons_visible);
    assert_eq!(
        runtime
            .last_animation_descriptor
            .expect("pump started close animation")
            .kind,
        PanelAnimationKind::Close
    );
    assert!(!runtime.panel_state.skip_next_close_card_exit);
    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("suppressed close presentation");
    assert!(!presentation.compact_bar.actions_visible);
    assert!(!presentation.action_buttons.visible);
    assert!(presentation.card_stack.visible);
    assert_eq!(
        presentation.card_stack.cards.len(),
        presented_status_cards.len()
    );
    assert!(
        presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusApproval { .. })),
        "unexpected close cards: {:?}",
        presentation.card_stack.cards
    );
    assert!(
        !presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::Empty))
    );

    let next_frame = runtime
        .advance_animation_frame_at(
            Instant::now()
                + Duration::from_millis(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS),
        )
        .expect("advance next close frame")
        .expect("next closing frame");
    assert_eq!(
        next_frame.descriptor.animation.kind,
        PanelAnimationKind::Close
    );
    let next_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("next close presentation");
    assert!(!next_presentation.compact_bar.actions_visible);
    assert!(!next_presentation.action_buttons.visible);
}

#[test]
fn windows_default_hover_close_transition_keeps_edge_actions_for_retract_animation() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime.panel_state.expanded = true;
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("seed expanded default surface");
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed fully expanded descriptor");
    let expanded_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("expanded default presentation");
    assert!(expanded_presentation.compact_bar.actions_visible);
    assert!(expanded_presentation.action_buttons.visible);

    runtime.last_transition_request = Some(NativePanelTransitionRequest::Close);
    runtime
        .advance_animation_frame_at(now)
        .expect("advance close")
        .expect("close frame");

    let closing_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("closing presentation");
    assert_eq!(closing_presentation.shell.surface, ExpandedSurface::Default);
    assert_eq!(
        closing_presentation.compact_bar.headline.text,
        "No active tasks"
    );
    assert!(closing_presentation.compact_bar.actions_visible);
    assert!(closing_presentation.action_buttons.visible);
    assert!(
        runtime
            .host
            .renderer
            .last_pointer_regions
            .iter()
            .any(|region| matches!(region.kind, NativePanelPointerRegionKind::EdgeAction(_)))
    );
}

#[test]
fn windows_default_close_delayed_wake_keeps_cards_and_actions_during_catch_up() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime.panel_state.expanded = true;
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("seed expanded default surface");
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed fully expanded descriptor");
    runtime.last_transition_request = Some(NativePanelTransitionRequest::Close);

    let first_frame = runtime
        .advance_animation_frame_at(now)
        .expect("start close")
        .expect("first close frame");
    let delayed_frame = runtime
        .advance_animation_frame_at(now + Duration::from_millis(first_frame.total_ms + 250))
        .expect("delayed close")
        .expect("delayed close catch-up frame");

    assert!(delayed_frame.continue_animating);
    assert!(delayed_frame.descriptor.animation.cards_progress < 1.0);
    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("catch-up presentation");
    assert_eq!(presentation.shell.surface, ExpandedSurface::Default);
    assert!(presentation.card_stack.visible);
    assert!(!presentation.card_stack.cards.is_empty());
    assert!(presentation.compact_bar.actions_visible);
    assert!(presentation.action_buttons.visible);
}

#[test]
fn windows_second_hover_cycle_delayed_wake_preserves_open_and_close_stages() {
    fn finish_animation(
        runtime: &mut super::WindowsNativePanelRuntime,
        start: Instant,
        total_ms: u64,
    ) {
        for step in 1..=total_ms.div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS) + 2 {
            let Some(frame) = runtime
                .advance_animation_frame_at(
                    start
                        + Duration::from_millis(
                            step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                        ),
                )
                .expect("advance animation")
            else {
                break;
            };
            if !frame.continue_animating {
                break;
            }
        }
    }

    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("seed default snapshot");

    runtime.panel_state.pointer_inside_since =
        Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
    assert_eq!(
        runtime
            .sync_hover_and_refresh_inside_with_input(true, now, &input)
            .expect("first hover open"),
        Some(HoverTransition::Expand)
    );
    let first_open = runtime
        .advance_animation_frame_at(now)
        .expect("start first open")
        .expect("first open frame");
    finish_animation(&mut runtime, now, first_open.total_ms);

    let first_close_at = now + Duration::from_millis(first_open.total_ms + 200);
    runtime.panel_state.pointer_outside_since =
        Some(first_close_at - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
    assert_eq!(
        runtime
            .sync_hover_and_refresh_inside_with_input(false, first_close_at, &input)
            .expect("first hover close"),
        Some(HoverTransition::Collapse)
    );
    let first_close = runtime
        .advance_animation_frame_at(first_close_at)
        .expect("start first close")
        .expect("first close frame");
    finish_animation(&mut runtime, first_close_at, first_close.total_ms);
    assert!(!runtime.panel_state.expanded);
    assert!(!runtime.panel_state.transitioning);

    let second_open_at = first_close_at + Duration::from_millis(first_close.total_ms + 200);
    runtime.panel_state.pointer_inside_since =
        Some(second_open_at - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
    assert_eq!(
        runtime
            .sync_hover_and_refresh_inside_with_input(true, second_open_at, &input)
            .expect("second hover open"),
        Some(HoverTransition::Expand)
    );
    let second_open = runtime
        .advance_animation_frame_at(second_open_at)
        .expect("start second open")
        .expect("second open frame");
    let delayed_second_open = runtime
        .advance_animation_frame_at(
            second_open_at + Duration::from_millis(second_open.total_ms + 250),
        )
        .expect("delayed second open")
        .expect("second open catch-up");
    assert!(delayed_second_open.continue_animating);
    assert!(
        delayed_second_open.descriptor.animation.width_progress < 0.7,
        "second open should not skip the compact width expansion"
    );
    finish_animation(&mut runtime, second_open_at, second_open.total_ms);

    let second_close_at = second_open_at + Duration::from_millis(second_open.total_ms + 200);
    runtime.panel_state.pointer_outside_since =
        Some(second_close_at - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
    assert_eq!(
        runtime
            .sync_hover_and_refresh_inside_with_input(false, second_close_at, &input)
            .expect("second hover close"),
        Some(HoverTransition::Collapse)
    );
    let second_close = runtime
        .advance_animation_frame_at(second_close_at)
        .expect("start second close")
        .expect("second close frame");
    let delayed_second_close = runtime
        .advance_animation_frame_at(
            second_close_at + Duration::from_millis(second_close.total_ms + 250),
        )
        .expect("delayed second close")
        .expect("second close catch-up");

    assert!(delayed_second_close.continue_animating);
    assert!(delayed_second_close.descriptor.animation.cards_progress < 0.5);
    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("second close presentation");
    assert!(presentation.card_stack.visible);
    assert!(presentation.compact_bar.actions_visible);
    assert!(presentation.action_buttons.visible);
}

#[test]
fn windows_repeated_hover_cycles_keep_transition_stages_stable() {
    fn finish_animation(
        runtime: &mut super::WindowsNativePanelRuntime,
        start: Instant,
        total_ms: u64,
    ) {
        for step in 1..=total_ms.div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS) + 2 {
            let Some(frame) = runtime
                .advance_animation_frame_at(
                    start
                        + Duration::from_millis(
                            step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                        ),
                )
                .expect("advance animation")
            else {
                break;
            };
            if !frame.continue_animating {
                break;
            }
        }
    }

    let mut now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("seed default snapshot");

    for cycle in 0..5 {
        runtime.panel_state.pointer_inside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
        assert_eq!(
            runtime
                .sync_hover_and_refresh_inside_with_input(true, now, &input)
                .expect("hover open"),
            Some(HoverTransition::Expand),
            "open transition missing at cycle {cycle}"
        );
        let open = runtime
            .advance_animation_frame_at(now)
            .expect("start open")
            .expect("open frame");
        let delayed_open = runtime
            .advance_animation_frame_at(now + Duration::from_millis(open.total_ms + 250))
            .expect("delayed open")
            .expect("open catch-up");
        assert!(
            delayed_open.continue_animating,
            "open ended early at cycle {cycle}"
        );
        assert!(
            delayed_open.descriptor.animation.width_progress < 0.7,
            "open skipped width stage at cycle {cycle}"
        );
        finish_animation(&mut runtime, now, open.total_ms);

        now += Duration::from_millis(open.total_ms + 240);
        runtime.panel_state.pointer_outside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
        assert_eq!(
            runtime
                .sync_hover_and_refresh_inside_with_input(false, now, &input)
                .expect("hover close"),
            Some(HoverTransition::Collapse),
            "close transition missing at cycle {cycle}"
        );
        let close = runtime
            .advance_animation_frame_at(now)
            .expect("start close")
            .expect("close frame");
        let delayed_close = runtime
            .advance_animation_frame_at(now + Duration::from_millis(close.total_ms + 250))
            .expect("delayed close")
            .expect("close catch-up");
        assert!(
            delayed_close.continue_animating,
            "close ended early at cycle {cycle}"
        );
        assert!(
            delayed_close.descriptor.animation.cards_progress < 0.5,
            "close skipped card stage at cycle {cycle}"
        );
        let presentation = runtime
            .host
            .renderer
            .latest_scene_presentation_model()
            .expect("close presentation");
        assert!(
            presentation.card_stack.visible,
            "cards hidden during close at cycle {cycle}"
        );
        assert!(
            presentation.action_buttons.visible,
            "actions hidden during close at cycle {cycle}"
        );
        finish_animation(&mut runtime, now, close.total_ms);
        now += Duration::from_millis(close.total_ms + 240);
    }
}

#[test]
fn windows_second_hover_leave_keeps_expanded_shell_until_close_animation_starts() {
    fn finish_animation(
        runtime: &mut super::WindowsNativePanelRuntime,
        start: Instant,
        total_ms: u64,
    ) {
        for step in 1..=total_ms.div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS) + 2 {
            let Some(frame) = runtime
                .advance_animation_frame_at(
                    start
                        + Duration::from_millis(
                            step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                        ),
                )
                .expect("advance animation")
            else {
                break;
            };
            runtime.host.consume_presenter_into_shell_result();
            if !frame.continue_animating {
                break;
            }
        }
    }

    let mut now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    runtime
        .sync_snapshot_bundle(&snapshot(), &input)
        .expect("seed default snapshot");
    runtime.host.consume_presenter_into_shell_result();

    for cycle in 0..2 {
        runtime.panel_state.pointer_inside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
        assert_eq!(
            runtime
                .sync_hover_and_refresh_inside_with_input(true, now, &input)
                .expect("hover open"),
            Some(HoverTransition::Expand),
            "open transition missing at cycle {cycle}"
        );
        runtime.host.consume_presenter_into_shell_result();
        let open = runtime
            .advance_animation_frame_at(now)
            .expect("start open")
            .expect("open frame");
        runtime.host.consume_presenter_into_shell_result();
        finish_animation(&mut runtime, now, open.total_ms);

        now += Duration::from_millis(open.total_ms + 240);
        runtime.panel_state.pointer_outside_since =
            Some(now - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 1));
        assert_eq!(
            runtime
                .sync_hover_and_refresh_inside_with_input(false, now, &input)
                .expect("hover close"),
            Some(HoverTransition::Collapse),
            "close transition missing at cycle {cycle}"
        );
        let pre_close_present = runtime.host.consume_presenter_into_shell_result();
        assert!(
            pre_close_present.paint_queued,
            "pre-close paint was not queued at cycle {cycle}"
        );
        let pre_close = runtime
            .host
            .shell
            .pending_paint_job()
            .expect("pre-close paint job");
        assert_eq!(
            pre_close.display_mode,
            NativePanelVisualDisplayMode::Expanded,
            "pre-close display mode changed before animation at cycle {cycle}"
        );
        assert!(
            pre_close.action_buttons_visible,
            "pre-close actions hidden before animation at cycle {cycle}"
        );
        assert!(
            pre_close.cards_visible,
            "pre-close cards hidden before animation at cycle {cycle}"
        );

        let close = runtime
            .advance_animation_frame_at(now)
            .expect("start close")
            .expect("close frame");
        runtime.host.consume_presenter_into_shell_result();
        let close_start = runtime
            .host
            .shell
            .pending_paint_job()
            .expect("close-start paint job");
        assert_eq!(
            close_start.display_mode,
            NativePanelVisualDisplayMode::Expanded,
            "close-start display mode changed too early at cycle {cycle}"
        );
        assert!(
            close_start.action_buttons_visible,
            "close-start actions hidden too early at cycle {cycle}"
        );
        assert!(
            close_start.cards_visible,
            "close-start cards hidden too early at cycle {cycle}"
        );

        finish_animation(&mut runtime, now, close.total_ms);
        now += Duration::from_millis(close.total_ms + 240);
    }
}

#[test]
fn windows_runtime_replaces_stale_action_button_paint_before_status_close() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");

    runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("seed auto status surface");
    runtime
        .host
        .present_renderer_state()
        .expect("present status surface");
    let mut stale_frame = runtime
        .host
        .take_pending_draw_frame()
        .expect("status draw frame");
    let stale_presentation = stale_frame
        .presentation_model
        .as_mut()
        .expect("stale frame presentation");
    stale_presentation.compact_bar.actions_visible = true;
    stale_presentation.action_buttons.visible = true;
    runtime.host.presenter.present(stale_frame);
    runtime.host.consume_presenter_into_shell_result();
    assert!(
        runtime
            .host
            .shell
            .pending_paint_job()
            .is_some_and(|job| job.action_buttons_visible)
    );

    super::clear_windows_native_panel_window_messages(Some(hwnd));
    super::queue_windows_native_panel_window_message(hwnd, super::WINDOWS_WM_PAINT, 0);
    runtime.panel_state.status_queue[0].expires_at = Instant::now() - Duration::from_millis(1);

    runtime
        .pump_platform_loop()
        .expect("pump expired status close");

    assert_eq!(
        runtime.platform_loop.last_window_message_id,
        Some(super::WINDOWS_WM_PAINT)
    );
    assert!(
        runtime
            .platform_loop
            .last_painted_job
            .as_ref()
            .is_some_and(|job| !job.action_buttons_visible)
    );
    assert_eq!(
        runtime
            .last_animation_descriptor
            .expect("status close animation")
            .kind,
        PanelAnimationKind::Close
    );
}

#[test]
fn windows_status_queue_refresh_during_close_keeps_preserved_status_cards() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(&pending_permission_snapshot("session-1"), &input)
        .expect("seed auto status surface");
    runtime
        .host
        .present_renderer_state()
        .expect("present status surface");
    let presented_status_cards = runtime
        .host
        .window
        .presented_presentation_model
        .as_ref()
        .expect("presented status model")
        .card_stack
        .cards
        .clone();

    runtime.panel_state.status_queue[0].expires_at = Instant::now() - Duration::from_millis(1);
    runtime
        .refresh_status_queue_from_last_raw_snapshot_with_input(&input)
        .expect("refresh expired status queue");
    runtime
        .advance_animation_frame_at(now)
        .expect("advance auto status close")
        .expect("closing frame");
    runtime
        .host
        .present_renderer_state()
        .expect("present closing frame");

    runtime
        .refresh_status_queue_from_last_raw_snapshot_with_input(&input)
        .expect("refresh while close is active");

    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("closing presentation after refresh");
    assert!(!presentation.compact_bar.actions_visible);
    assert!(!presentation.action_buttons.visible);
    assert_eq!(
        presentation.card_stack.cards.len(),
        presented_status_cards.len()
    );
    assert!(
        presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusApproval { .. }))
    );
    assert!(
        !presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::Empty))
    );
}

#[test]
fn windows_status_queue_new_request_during_close_reopens_status_after_close() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    let input = runtime_input_descriptor();
    runtime
        .sync_snapshot_bundle(
            &pending_permission_snapshot_with_request("req-1", "session-1"),
            &input,
        )
        .expect("seed first status surface");
    runtime
        .host
        .present_renderer_state()
        .expect("present first status surface");

    runtime.panel_state.status_queue[0].expires_at = Instant::now() - Duration::from_millis(1);
    runtime
        .refresh_status_queue_from_last_raw_snapshot_with_input(&input)
        .expect("expire first status item");
    let close_frame = runtime
        .advance_animation_frame_at(now)
        .expect("advance status close")
        .expect("closing frame");
    runtime
        .host
        .present_renderer_state()
        .expect("present closing status frame");

    runtime
        .sync_snapshot_bundle(
            &pending_question_snapshot_with_request("question-2", "session-2"),
            &input,
        )
        .expect("sync new status while close is active");

    let closing_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("closing presentation after new status");
    assert!(
        closing_presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusApproval { .. })),
        "active close should keep the old status card stack"
    );
    assert!(
        !closing_presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusQuestion { .. })),
        "new status should wait until the close finishes"
    );

    let mut finished_close = false;
    for step in 1..=close_frame
        .total_ms
        .div_ceil(crate::native_panel_core::PANEL_ANIMATION_FRAME_MS)
        + 1
    {
        let frame = runtime
            .advance_animation_frame_at(
                now + Duration::from_millis(
                    close_frame.total_ms
                        + step * crate::native_panel_core::PANEL_ANIMATION_FRAME_MS,
                ),
            )
            .expect("finish close frame")
            .expect("closing frame");
        if !frame.continue_animating {
            finished_close = true;
            break;
        }
    }
    assert!(finished_close);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Open),
        "new status should schedule a reopen after close finishes"
    );

    let open_frame = runtime
        .advance_animation_frame_at(now + Duration::from_millis(close_frame.total_ms + 17))
        .expect("start reopen")
        .expect("opening new status frame");
    assert_eq!(
        open_frame.descriptor.animation.kind,
        PanelAnimationKind::Open
    );
    let reopened_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("reopened status presentation");
    assert!(
        reopened_presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusQuestion { .. }))
    );
}

#[test]
fn windows_completion_status_close_keeps_message_bubble_until_cards_exit() {
    let now = Instant::now();
    let input = runtime_input_descriptor();
    let mut running_snapshot = sessions_snapshot(1);
    running_snapshot.sessions[0].status = "Running".to_string();
    running_snapshot.sessions[0].last_assistant_message = Some("Working".to_string());
    let mut completed_snapshot = running_snapshot.clone();
    completed_snapshot.sessions[0].status = "Idle".to_string();
    completed_snapshot.sessions[0].last_activity = Utc::now();
    completed_snapshot.sessions[0].last_assistant_message = Some("Done".to_string());
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime
        .sync_snapshot_bundle(&running_snapshot, &input)
        .expect("seed running snapshot");
    runtime
        .sync_snapshot_bundle(&completed_snapshot, &input)
        .expect("show completion status");
    runtime
        .host
        .present_renderer_state()
        .expect("present completion status");
    let completion_presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("completion status presentation");
    assert_eq!(
        completion_presentation.mascot.pose,
        SceneMascotPose::MessageBubble
    );
    assert!(!completion_presentation.compact_bar.actions_visible);
    assert!(!completion_presentation.action_buttons.visible);

    runtime.panel_state.status_queue[0].expires_at = Instant::now() - Duration::from_millis(1);
    runtime
        .refresh_status_queue_from_last_raw_snapshot_with_input(&input)
        .expect("refresh expired status queue");
    runtime
        .advance_animation_frame_at(now)
        .expect("advance auto status close")
        .expect("closing frame");
    runtime
        .host
        .present_renderer_state()
        .expect("present closing frame");
    runtime
        .refresh_status_queue_from_last_raw_snapshot_with_input(&input)
        .expect("refresh while completion close is active");

    let presentation = runtime
        .host
        .renderer
        .latest_scene_presentation_model()
        .expect("closing completion presentation");
    assert_eq!(presentation.mascot.pose, SceneMascotPose::MessageBubble);
    assert!(!presentation.compact_bar.actions_visible);
    assert!(!presentation.action_buttons.visible);
    assert!(
        presentation
            .card_stack
            .cards
            .iter()
            .any(|card| matches!(card, SceneCard::StatusCompletion { .. }))
    );
}

#[test]
fn windows_animation_scheduler_runs_surface_switch_only_while_active() {
    let now = Instant::now();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed expanded descriptor");
    runtime.last_transition_request = Some(NativePanelTransitionRequest::SurfaceSwitch);

    let frame = runtime
        .advance_animation_frame_at(now)
        .expect("advance surface switch")
        .expect("surface switch frame");

    assert_eq!(
        frame.descriptor.animation.kind,
        PanelAnimationKind::SurfaceSwitch
    );
    assert!(frame.continue_animating);
    assert!(runtime.animation_scheduler.is_active());
}

#[test]
fn windows_animation_scheduler_idle_state_does_not_redraw_continuously() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    let frame = runtime
        .advance_animation_frame_at(Instant::now())
        .expect("advance idle");

    assert!(frame.is_none());
    assert!(!runtime.animation_scheduler.is_active());
    assert!(!runtime.host.presenter.redraw_requested());
}

#[test]
fn windows_renderer_treats_close_card_progress_as_exit_progress() {
    let mut state = PanelState::default();
    let bundle = test_runtime_scene_bundle(
        &mut state,
        &pending_permission_snapshot("session-1"),
        &PanelSceneBuildInput::default(),
    );
    let mut renderer = super::WindowsNativePanelRenderer::default();

    renderer
        .render_scene(&bundle.scene, bundle.runtime_render_state)
        .expect("render scene");
    renderer
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: 180.0,
            visible_height: 180.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 0.0,
        })
        .expect("apply close descriptor");

    assert_eq!(
        renderer.last_layout.expect("layout").separator_visibility,
        0.88
    );
}

#[test]
fn windows_runtime_pump_platform_loop_tracks_window_state_command() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump platform loop");

    assert_eq!(
        runtime.platform_loop.last_window_state,
        Some(runtime.host.window.window_state())
    );
    assert!(runtime.platform_loop.applied_command_count > 0);
    assert!(runtime.host.take_pending_shell_commands().is_empty());
}

#[test]
fn windows_runtime_pump_platform_loop_backfills_shell_raw_window_handle() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    assert_eq!(runtime.host.shell.raw_window_handle(), None);

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump platform loop");

    assert!(runtime.host.shell.raw_window_handle().is_some());
    assert_eq!(
        runtime.platform_loop.last_raw_window_handle,
        runtime.host.shell.raw_window_handle()
    );
}

#[test]
fn windows_runtime_pump_platform_loop_clears_shell_raw_window_handle_on_destroy() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::platform_loop::sync_windows_native_panel_hit_regions(
        Some(hwnd),
        &[NativePanelPointerRegion {
            frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            kind: NativePanelPointerRegionKind::CompactBar,
        }],
    );

    runtime.host.shell.destroy();
    runtime.pump_platform_loop().expect("pump destroy");

    assert_eq!(runtime.host.shell.raw_window_handle(), None);
    assert_eq!(runtime.platform_loop.last_raw_window_handle, None);
    assert_eq!(
        super::platform_loop::resolve_windows_native_panel_cached_hit_test(
            hwnd,
            PanelPoint { x: 10.0, y: 10.0 }
        ),
        super::hit_region::WindowsNativePanelHitTest::Transparent
    );
}

#[test]
fn windows_runtime_pump_window_messages_consumes_paint_job() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");

    runtime.host.presenter.present(WindowsNativePanelDrawFrame {
        window_state: NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 100.0,
            }),
            visible: true,
            preferred_display_index: 0,
        },
        pointer_regions: Vec::new(),
        presentation_model: None,
    });
    runtime.pump_platform_loop().expect("pump presenter frame");

    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::clear_windows_native_panel_window_messages(Some(hwnd));
    super::queue_windows_native_panel_window_message(hwnd, super::WINDOWS_WM_PAINT, 0);

    runtime
        .pump_window_messages()
        .expect("pump window messages");

    assert_eq!(
        runtime.platform_loop.last_window_message_id,
        Some(super::WINDOWS_WM_PAINT)
    );
    assert!(runtime.platform_loop.paint_dispatch_count >= 1);
    assert!(
        runtime
            .platform_loop
            .last_paint_plan
            .as_ref()
            .is_some_and(|plan| !plan.hidden && !plan.primitives.is_empty())
    );
    assert_eq!(
        runtime
            .platform_loop
            .last_painted_job
            .as_ref()
            .map(|job| job.panel_frame.width),
        Some(300.0)
    );
    assert_eq!(
        runtime.platform_loop.last_paint_surface_resource_revision,
        Some(runtime.platform_loop.surface_resource_revision)
    );
    assert_eq!(
        runtime.platform_loop.paint_surface_resource_rebuild_count,
        1
    );
    assert!(runtime.host.shell.pending_paint_job().is_none());
}

#[test]
fn windows_runtime_pump_window_messages_queues_click_event() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    sync_test_pointer_regions(
        &mut runtime,
        vec![NativePanelPointerRegion {
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
        }],
    );

    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::clear_windows_native_panel_window_messages(Some(hwnd));
    super::queue_windows_native_panel_window_message(
        hwnd,
        super::window_shell::WINDOWS_WM_LBUTTONUP,
        ((30_i32 as u32 as u64) | ((30_i32 as u32 as u64) << 16)) as isize,
    );

    runtime
        .pump_window_messages()
        .expect("pump window messages");

    assert_eq!(
        runtime.platform_loop.last_window_message_id,
        Some(super::window_shell::WINDOWS_WM_LBUTTONUP)
    );
    assert_eq!(
        runtime.host.take_platform_events(),
        vec![NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        )]
    );
}

#[test]
fn windows_runtime_pump_window_messages_routes_move_message_into_pointer_path() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(pending_permission_snapshot("session-1"));
    runtime.panel_state.pointer_inside_since = Some(
        Instant::now() - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100),
    );
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Open,
            canvas_height: 180.0,
            visible_height: 140.0,
            width_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        })
        .expect("seed animation descriptor");
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");

    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::clear_windows_native_panel_window_messages(Some(hwnd));
    super::queue_windows_native_panel_window_message(
        hwnd,
        super::window_shell::WINDOWS_WM_MOUSEMOVE,
        ((30_i32 as u32 as u64) | ((30_i32 as u32 as u64) << 16)) as isize,
    );

    runtime
        .pump_window_messages()
        .expect("pump window messages");

    assert_eq!(
        runtime.platform_loop.last_window_message_id,
        Some(super::window_shell::WINDOWS_WM_MOUSEMOVE)
    );
    assert!(runtime.platform_loop.processed_window_message_count >= 1);
    assert_eq!(
        runtime.host.shell.last_pointer_input(),
        Some(NativePanelPointerInput::Move(PanelPoint {
            x: 30.0,
            y: 30.0
        }))
    );
    assert!(runtime.host.take_platform_events().is_empty());
}

#[test]
fn windows_runtime_pump_window_messages_leave_collapses_and_refreshes() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.panel_state.expanded = true;
    runtime.panel_state.pointer_outside_since = Some(
        Instant::now() - Duration::from_millis(crate::native_panel_core::HOVER_DELAY_MS + 100),
    );
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: 120.0,
            visible_height: 120.0,
            width_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("seed animation descriptor");
    runtime.host.renderer.last_pointer_regions = vec![NativePanelPointerRegion {
        frame: PanelRect {
            x: 20.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        kind: NativePanelPointerRegionKind::CompactBar,
    }];
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");

    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::clear_windows_native_panel_window_messages(Some(hwnd));
    super::queue_windows_native_panel_window_message(
        hwnd,
        super::window_shell::WINDOWS_WM_MOUSELEAVE,
        0,
    );

    runtime
        .pump_window_messages()
        .expect("pump window messages");

    assert_eq!(
        runtime.platform_loop.last_window_message_id,
        Some(super::window_shell::WINDOWS_WM_MOUSELEAVE)
    );
    assert_eq!(
        runtime.host.shell.last_pointer_input(),
        Some(NativePanelPointerInput::Leave)
    );
    assert!(!runtime.panel_state.expanded);
    assert_eq!(
        runtime.last_transition_request,
        Some(NativePanelTransitionRequest::Close)
    );
    assert!(runtime.scene_cache.last_scene.is_some());
    assert!(runtime.host.take_platform_events().is_empty());
}

#[test]
fn windows_runtime_pump_platform_loop_processes_leave_before_unstarted_hover_open() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.scene_cache.last_snapshot = Some(snapshot());
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    runtime
        .host
        .apply_animation_descriptor(PanelAnimationDescriptor {
            kind: PanelAnimationKind::Close,
            canvas_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            visible_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            width_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        })
        .expect("seed collapsed animation descriptor");
    runtime.panel_state.expanded = true;
    runtime.last_transition_request = Some(NativePanelTransitionRequest::Open);

    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::clear_windows_native_panel_window_messages(Some(hwnd));
    super::queue_windows_native_panel_window_message(
        hwnd,
        super::window_shell::WINDOWS_WM_MOUSELEAVE,
        0,
    );

    runtime
        .pump_platform_loop()
        .expect("pump leave before unstarted open");

    assert_eq!(
        runtime.host.shell.last_pointer_input(),
        Some(NativePanelPointerInput::Leave)
    );
    assert_eq!(runtime.last_transition_request, None);
    assert!(!runtime.panel_state.expanded);
    assert!(!runtime.panel_state.transitioning);
    assert_eq!(
        runtime
            .host
            .renderer
            .last_animation_descriptor
            .map(|descriptor| descriptor.kind),
        Some(PanelAnimationKind::Close)
    );
}

#[test]
fn windows_runtime_pump_window_messages_debounces_focus_clicks() {
    let _guard = window_message_queue_test_guard();
    let mut runtime = super::WindowsNativePanelRuntime::default();
    runtime.panel_state.expanded = true;
    runtime.create_panel().expect("create panel");
    runtime.pump_platform_loop().expect("pump create");
    sync_test_pointer_regions(
        &mut runtime,
        vec![NativePanelPointerRegion {
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
        }],
    );

    let hwnd = runtime.host.shell.raw_window_handle().expect("shell hwnd");
    super::clear_windows_native_panel_window_messages(Some(hwnd));
    let click_lparam = ((30_i32 as u32 as u64) | ((30_i32 as u32 as u64) << 16)) as isize;
    super::queue_windows_native_panel_window_message(
        hwnd,
        super::window_shell::WINDOWS_WM_LBUTTONUP,
        click_lparam,
    );
    super::queue_windows_native_panel_window_message(
        hwnd,
        super::window_shell::WINDOWS_WM_LBUTTONUP,
        click_lparam,
    );

    runtime
        .pump_window_messages()
        .expect("pump window messages");

    assert_eq!(
        runtime.platform_loop.last_window_message_id,
        Some(super::window_shell::WINDOWS_WM_LBUTTONUP)
    );
    assert!(runtime.platform_loop.processed_window_message_count >= 2);
    assert_eq!(
        runtime.host.take_platform_events(),
        vec![NativePanelPlatformEvent::FocusSession(
            "session-1".to_string()
        )]
    );
    assert!(
        runtime
            .last_focus_click
            .as_ref()
            .is_some_and(|(session_id, _)| session_id == "session-1")
    );
}

#[test]
fn windows_runtime_pump_platform_loop_tracks_lifecycle_and_redraw_commands() {
    let mut runtime = super::WindowsNativePanelRuntime::default();

    runtime.host.show().expect("show host");
    runtime.host.shell.request_redraw();
    runtime.host.hide().expect("hide host");
    runtime.host.shell.destroy();
    runtime.pump_platform_loop().expect("pump platform loop");

    assert_eq!(runtime.platform_loop.create_count, 1);
    assert_eq!(runtime.platform_loop.show_count, 1);
    assert_eq!(runtime.platform_loop.hide_count, 1);
    assert_eq!(runtime.platform_loop.destroy_count, 1);
    assert_eq!(runtime.platform_loop.redraw_request_count, 1);
    assert_eq!(runtime.platform_loop.topmost_reassert_count, 1);
    assert_eq!(runtime.platform_loop.last_visible, Some(false));
    assert!(runtime.host.take_pending_shell_commands().is_empty());
}

#[test]
fn windows_spawn_platform_loops_marks_shell_state() {
    let before = super::runtime_entry::with_windows_native_panel_runtime(|runtime| {
        Ok(runtime.host.shell.platform_loop_spawn_count())
    })
    .expect("inspect pre-spawn runtime");

    super::runtime_entry::spawn_platform_loops_internal();

    super::runtime_entry::with_windows_native_panel_runtime(|runtime| {
        assert!(runtime.host.shell.platform_loop_started());
        assert!(runtime.host.shell.platform_loop_spawn_count() > before);
        Ok(())
    })
    .expect("inspect runtime");
}

#[test]
fn windows_spawn_platform_loops_background_thread_drains_public_api_work() {
    super::runtime_entry::spawn_platform_loops_internal();
    let before = super::windows_native_platform_loop_generations()
        .expect("platform loop generations")
        .0;

    super::create_native_panel().expect("create native panel");

    let after_create = super::windows_native_platform_loop_generations()
        .expect("platform loop generations")
        .0;
    assert!(after_create > before);
    assert!(super::wait_windows_native_platform_loop_processed_at_least(
        after_create,
        1000
    ));

    super::runtime_entry::with_windows_native_panel_runtime(|runtime| {
        assert!(runtime.host.shell.raw_window_handle().is_some());
        Ok(())
    })
    .expect("inspect runtime");
}

#[test]
fn windows_host_recomputes_cached_frame_when_display_changes() {
    let mut host = super::WindowsNativePanelHost::default();
    let descriptor = PanelAnimationDescriptor {
        kind: PanelAnimationKind::Open,
        canvas_height: 120.0,
        visible_height: 120.0,
        width_progress: 1.0,
        height_progress: 0.0,
        shoulder_progress: 0.0,
        drop_progress: 0.0,
        cards_progress: 0.0,
    };

    host.apply_animation_descriptor(descriptor)
        .expect("apply descriptor");
    host.reposition_to_display(
        1,
        Some(PanelRect {
            x: 500.0,
            y: 100.0,
            width: 800.0,
            height: 600.0,
        }),
    )
    .expect("reposition host");

    assert_eq!(host.window.descriptor.preferred_display_index, 1);
    assert_eq!(
        host.window.last_frame,
        Some(PanelRect {
            x: 690.0,
            y: 100.0,
            width: 420.0,
            height: 120.0,
        })
    );
    assert_eq!(
        host.renderer.last_window_state,
        Some(host.window.window_state())
    );
}

#[test]
fn windows_window_frame_uses_canvas_width_to_contain_local_layout() {
    let descriptor = PanelAnimationDescriptor {
        kind: PanelAnimationKind::Open,
        canvas_height: 120.0,
        visible_height: 160.0,
        width_progress: 0.25,
        height_progress: 0.0,
        shoulder_progress: 0.0,
        drop_progress: 0.0,
        cards_progress: 0.0,
    };

    let frame = super::host_window::resolve_windows_panel_window_frame(
        descriptor,
        PanelRect {
            x: 100.0,
            y: 50.0,
            width: 1000.0,
            height: 700.0,
        },
        200.0,
        400.0,
    );

    assert_eq!(
        frame,
        PanelRect {
            x: 390.0,
            y: 50.0,
            width: 420.0,
            height: 160.0,
        }
    );
}

#[cfg(not(windows))]
#[test]
fn windows_native_ui_remains_disabled_on_non_windows() {
    assert!(!super::native_ui_enabled());
}

#[cfg(windows)]
#[test]
fn windows_native_ui_is_enabled_by_default_on_windows() {
    assert!(super::facade::windows_native_ui_enabled_from_env(
        true, None
    ));
}

#[test]
fn windows_native_ui_env_parser_preserves_default_and_fallback_override() {
    assert!(!super::facade::windows_native_ui_enabled_from_env(
        false, None
    ));
    assert!(super::facade::windows_native_ui_enabled_from_env(
        true, None
    ));
    assert!(!super::facade::windows_native_ui_enabled_from_env(
        true,
        Some("0".to_string())
    ));
    assert!(!super::facade::windows_native_ui_enabled_from_env(
        true,
        Some("off".to_string())
    ));
    assert!(super::facade::windows_native_ui_enabled_from_env(
        false,
        Some("1".to_string())
    ));
}
