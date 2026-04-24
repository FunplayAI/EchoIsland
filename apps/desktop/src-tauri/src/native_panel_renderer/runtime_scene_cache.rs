use echoisland_runtime::RuntimeSnapshot;

use crate::{
    native_panel_core::PanelRect,
    native_panel_scene::{PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelScene},
};

use super::{
    NativePanelRenderCommandBundle, NativePanelRuntimeInputDescriptor, NativePanelSceneHost,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct NativePanelRuntimeSceneCache {
    pub(crate) last_snapshot: Option<RuntimeSnapshot>,
    pub(crate) last_scene: Option<PanelScene>,
    pub(crate) last_runtime_render_state: Option<PanelRuntimeRenderState>,
    pub(crate) last_render_command_bundle: Option<NativePanelRenderCommandBundle>,
}

pub(crate) fn cache_runtime_scene(
    cache: &mut NativePanelRuntimeSceneCache,
    snapshot: RuntimeSnapshot,
    scene: PanelScene,
    runtime_render_state: PanelRuntimeRenderState,
) {
    cache.last_snapshot = Some(snapshot);
    cache_scene_runtime(cache, scene, runtime_render_state);
}

pub(crate) fn cache_scene_runtime(
    cache: &mut NativePanelRuntimeSceneCache,
    scene: PanelScene,
    runtime_render_state: PanelRuntimeRenderState,
) {
    cache.last_scene = Some(scene);
    cache.last_runtime_render_state = Some(runtime_render_state);
    cache.last_render_command_bundle = None;
}

pub(crate) fn cache_render_command_bundle(
    cache: &mut NativePanelRuntimeSceneCache,
    bundle: &NativePanelRenderCommandBundle,
) {
    cache_scene_runtime(cache, bundle.scene.clone(), bundle.runtime);
    cache.last_render_command_bundle = Some(bundle.clone());
}

pub(crate) fn cached_scene(cache: &NativePanelRuntimeSceneCache) -> Option<PanelScene> {
    cache
        .last_render_command_bundle
        .as_ref()
        .map(|bundle| bundle.scene.clone())
        .or_else(|| cache.last_scene.clone())
}

pub(crate) fn cached_runtime_render_state(
    cache: &NativePanelRuntimeSceneCache,
) -> Option<PanelRuntimeRenderState> {
    cache
        .last_render_command_bundle
        .as_ref()
        .map(|bundle| bundle.runtime)
        .or(cache.last_runtime_render_state)
}

pub(crate) fn apply_runtime_scene_bundle_to_host<H: NativePanelSceneHost>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    bundle: PanelRuntimeSceneBundle,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
) -> Result<(), H::Error> {
    host.sync_scene(
        &bundle.scene,
        bundle.runtime_render_state,
        preferred_display_index,
        screen_frame,
    )?;
    cache_runtime_scene(
        cache,
        bundle.displayed_snapshot,
        bundle.scene,
        bundle.runtime_render_state,
    );
    Ok(())
}

pub(crate) fn rerender_runtime_scene_cache_to_host<H: NativePanelSceneHost>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
    rebuild_bundle: impl FnOnce(&RuntimeSnapshot) -> PanelRuntimeSceneBundle,
) -> Result<bool, H::Error> {
    let Some(snapshot) = cache.last_snapshot.clone() else {
        return Ok(false);
    };
    let bundle = rebuild_bundle(&snapshot);
    apply_runtime_scene_bundle_to_host(host, cache, bundle, preferred_display_index, screen_frame)?;
    Ok(true)
}

pub(crate) fn rerender_runtime_scene_cache_to_host_with_input_descriptor<
    H: NativePanelSceneHost,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    input: &NativePanelRuntimeInputDescriptor,
    rebuild_bundle: impl FnOnce(&RuntimeSnapshot) -> PanelRuntimeSceneBundle,
) -> Result<bool, H::Error> {
    rerender_runtime_scene_cache_to_host(
        host,
        cache,
        input.selected_display_index(),
        input.screen_frame,
        rebuild_bundle,
    )
}

pub(crate) fn rerender_runtime_scene_cache_to_host_on_transition<H, T>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    transition: Option<T>,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
    rebuild_bundle: impl FnOnce(&RuntimeSnapshot) -> PanelRuntimeSceneBundle,
) -> Result<Option<T>, H::Error>
where
    H: NativePanelSceneHost,
{
    if transition.is_some() {
        rerender_runtime_scene_cache_to_host(
            host,
            cache,
            preferred_display_index,
            screen_frame,
            rebuild_bundle,
        )?;
    }
    Ok(transition)
}

pub(crate) fn rerender_runtime_scene_cache_to_host_on_transition_with_input_descriptor<H, T>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    transition: Option<T>,
    input: &NativePanelRuntimeInputDescriptor,
    rebuild_bundle: impl FnOnce(&RuntimeSnapshot) -> PanelRuntimeSceneBundle,
) -> Result<Option<T>, H::Error>
where
    H: NativePanelSceneHost,
{
    rerender_runtime_scene_cache_to_host_on_transition(
        host,
        cache,
        transition,
        input.selected_display_index(),
        input.screen_frame,
        rebuild_bundle,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelRuntimeSceneCache, apply_runtime_scene_bundle_to_host,
        cache_render_command_bundle, cache_runtime_scene, cached_runtime_render_state,
        cached_scene, rerender_runtime_scene_cache_to_host,
        rerender_runtime_scene_cache_to_host_on_transition,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelRect, PanelSettingsState},
        native_panel_renderer::{
            NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
            NativePanelRenderer, NativePanelSceneHost, resolve_native_panel_render_command_bundle,
        },
        native_panel_scene::{
            CompactBarScene, PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelScene,
            SceneMascotPose, SceneText, SessionSurfaceScene, StatusSurfaceDefaultState,
            StatusSurfaceDisplayMode, StatusSurfaceQueueState, StatusSurfaceScene, SurfaceScene,
            build_settings_surface_scene, surface_scene_mode,
        },
    };
    use echoisland_runtime::RuntimeSnapshot;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestTransition {
        Changed,
    }

    #[derive(Default)]
    struct TestRenderer;

    impl NativePanelRenderer for TestRenderer {
        type Error = String;

        fn render_scene(
            &mut self,
            _scene: &PanelScene,
            _runtime: PanelRuntimeRenderState,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn set_visible(&mut self, _visible: bool) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct TestSceneHost {
        renderer: TestRenderer,
        descriptor: NativePanelHostWindowDescriptor,
        synced_scene: Option<PanelScene>,
        synced_runtime: Option<PanelRuntimeRenderState>,
        synced_preferred_display_index: Option<usize>,
        synced_screen_frame: Option<Option<PanelRect>>,
    }

    impl NativePanelHost for TestSceneHost {
        type Error = String;
        type Renderer = TestRenderer;

        fn renderer(&mut self) -> &mut Self::Renderer {
            &mut self.renderer
        }

        fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
            self.descriptor
        }

        fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
            &mut self.descriptor
        }

        fn window_state(&self) -> NativePanelHostWindowState {
            NativePanelHostWindowState::default()
        }

        fn show(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn hide(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl NativePanelSceneHost for TestSceneHost {
        fn sync_scene(
            &mut self,
            scene: &PanelScene,
            runtime: PanelRuntimeRenderState,
            preferred_display_index: usize,
            screen_frame: Option<PanelRect>,
        ) -> Result<(), Self::Error> {
            self.synced_scene = Some(scene.clone());
            self.synced_runtime = Some(runtime);
            self.synced_preferred_display_index = Some(preferred_display_index);
            self.synced_screen_frame = Some(screen_frame);
            Ok(())
        }
    }

    #[test]
    fn runtime_scene_bundle_apply_syncs_host_and_updates_cache() {
        let mut host = TestSceneHost::default();
        let mut cache = NativePanelRuntimeSceneCache::default();
        let bundle = test_bundle("idle");
        let screen_frame = Some(PanelRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 120.0,
        });

        apply_runtime_scene_bundle_to_host(&mut host, &mut cache, bundle, 2, screen_frame)
            .expect("apply runtime bundle");

        assert_eq!(host.synced_preferred_display_index, Some(2));
        assert_eq!(host.synced_screen_frame, Some(screen_frame));
        assert!(host.synced_scene.is_some());
        assert_eq!(
            host.synced_runtime,
            Some(PanelRuntimeRenderState {
                transitioning: true,
                ..PanelRuntimeRenderState::default()
            })
        );
        assert!(cache.last_snapshot.is_some());
        assert!(cache.last_scene.is_some());
        assert_eq!(
            cache.last_runtime_render_state,
            Some(PanelRuntimeRenderState {
                transitioning: true,
                ..PanelRuntimeRenderState::default()
            })
        );
        assert!(cache.last_render_command_bundle.is_none());
    }

    #[test]
    fn runtime_scene_cache_rerender_rebuilds_from_cached_snapshot() {
        let mut host = TestSceneHost::default();
        let mut cache = NativePanelRuntimeSceneCache {
            last_snapshot: Some(RuntimeSnapshot {
                status: "cached".to_string(),
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
            }),
            ..NativePanelRuntimeSceneCache::default()
        };

        let rerendered =
            rerender_runtime_scene_cache_to_host(&mut host, &mut cache, 1, None, |snapshot| {
                test_bundle(&snapshot.status)
            })
            .expect("rerender cached scene");

        assert!(rerendered);
        assert!(
            cache
                .last_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.status == "cached")
        );
        assert!(
            host.synced_scene
                .as_ref()
                .is_some_and(|scene| scene.compact_bar.headline.text == "cached")
        );
    }

    #[test]
    fn runtime_scene_cache_rerender_skips_without_cached_snapshot() {
        let mut host = TestSceneHost::default();
        let mut cache = NativePanelRuntimeSceneCache::default();

        let rerendered =
            rerender_runtime_scene_cache_to_host(&mut host, &mut cache, 0, None, |_| {
                unreachable!("should not rebuild without cached snapshot")
            })
            .expect("skip rerender without snapshot");

        assert!(!rerendered);
        assert!(host.synced_scene.is_none());
        assert!(cache.last_scene.is_none());
    }

    #[test]
    fn runtime_scene_cache_transition_rerender_rebuilds_only_when_changed() {
        let mut host = TestSceneHost::default();
        let mut cache = NativePanelRuntimeSceneCache {
            last_snapshot: Some(RuntimeSnapshot {
                status: "cached".to_string(),
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
            }),
            ..NativePanelRuntimeSceneCache::default()
        };

        let transition = rerender_runtime_scene_cache_to_host_on_transition(
            &mut host,
            &mut cache,
            Some(TestTransition::Changed),
            1,
            None,
            |snapshot| test_bundle(&snapshot.status),
        )
        .expect("rerender on transition");

        assert_eq!(transition, Some(TestTransition::Changed));
        assert!(
            host.synced_scene
                .as_ref()
                .is_some_and(|scene| scene.compact_bar.headline.text == "cached")
        );
    }

    #[test]
    fn runtime_scene_cache_transition_rerender_skips_without_change() {
        let mut host = TestSceneHost::default();
        let mut cache = NativePanelRuntimeSceneCache {
            last_snapshot: Some(RuntimeSnapshot {
                status: "cached".to_string(),
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
            }),
            ..NativePanelRuntimeSceneCache::default()
        };

        let transition = rerender_runtime_scene_cache_to_host_on_transition(
            &mut host,
            &mut cache,
            None::<TestTransition>,
            1,
            None,
            |_| unreachable!("should not rebuild when transition is absent"),
        )
        .expect("skip rerender without transition");

        assert!(transition.is_none());
        assert!(host.synced_scene.is_none());
        assert!(cache.last_scene.is_none());
    }

    #[test]
    fn caching_runtime_scene_clears_stale_render_command_bundle() {
        let mut cache = NativePanelRuntimeSceneCache::default();
        let stale_bundle = test_render_command_bundle("stale", true);
        cache_render_command_bundle(&mut cache, &stale_bundle);

        cache_runtime_scene(
            &mut cache,
            RuntimeSnapshot {
                status: "fresh".to_string(),
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
            },
            test_bundle("fresh").scene,
            PanelRuntimeRenderState::default(),
        );

        assert!(cache.last_render_command_bundle.is_none());
        assert_eq!(
            cached_scene(&cache).map(|scene| scene.compact_bar.headline.text),
            Some("fresh".to_string())
        );
    }

    #[test]
    fn cached_scene_and_runtime_prefer_render_command_bundle() {
        let mut cache = NativePanelRuntimeSceneCache {
            last_scene: Some(test_bundle("fallback").scene),
            last_runtime_render_state: Some(PanelRuntimeRenderState::default()),
            ..NativePanelRuntimeSceneCache::default()
        };
        let bundle = test_render_command_bundle("bundle", true);

        cache_render_command_bundle(&mut cache, &bundle);

        assert_eq!(
            cached_scene(&cache).map(|scene| scene.compact_bar.headline.text),
            Some("bundle".to_string())
        );
        assert_eq!(
            cached_runtime_render_state(&cache).map(|runtime| runtime.transitioning),
            Some(true)
        );
    }

    fn test_bundle(status: &str) -> PanelRuntimeSceneBundle {
        PanelRuntimeSceneBundle {
            scene: PanelScene {
                surface: ExpandedSurface::Default,
                compact_bar: CompactBarScene {
                    headline: SceneText {
                        text: status.to_string(),
                        emphasized: false,
                    },
                    active_count: "0".to_string(),
                    total_count: "0".to_string(),
                    completion_count: 0,
                    actions_visible: false,
                },
                surface_scene: SurfaceScene {
                    mode: surface_scene_mode(ExpandedSurface::Default),
                    headline_text: "Idle".to_string(),
                    headline_emphasized: false,
                    edge_actions_visible: false,
                },
                status_surface: StatusSurfaceScene {
                    cards: vec![],
                    display_mode: StatusSurfaceDisplayMode::Hidden,
                    default_state: StatusSurfaceDefaultState::default(),
                    queue_state: StatusSurfaceQueueState::default(),
                    completion_badge_count: 0,
                    show_completion_glow: false,
                },
                session_surface: SessionSurfaceScene { cards: vec![] },
                settings_surface: build_settings_surface_scene(
                    1,
                    PanelSettingsState::default(),
                    "0.0.0",
                ),
                cards: vec![],
                glow: None,
                mascot_pose: SceneMascotPose::Idle,
                hit_targets: vec![],
                nodes: vec![],
            },
            runtime_render_state: PanelRuntimeRenderState {
                transitioning: true,
                ..PanelRuntimeRenderState::default()
            },
            displayed_snapshot: RuntimeSnapshot {
                status: status.to_string(),
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
            },
        }
    }

    fn test_render_command_bundle(
        status: &str,
        transitioning: bool,
    ) -> crate::native_panel_renderer::NativePanelRenderCommandBundle {
        let scene = test_bundle(status).scene;
        let layout = crate::native_panel_core::resolve_panel_layout(
            crate::native_panel_core::PanelLayoutInput {
                screen_frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1440.0,
                    height: 900.0,
                },
                metrics: crate::native_panel_core::PanelGeometryMetrics {
                    compact_height: crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT,
                    compact_width: crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH,
                    expanded_width: crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
                    panel_width: crate::native_panel_core::DEFAULT_PANEL_CANVAS_WIDTH,
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
            },
        );
        let runtime = PanelRuntimeRenderState {
            transitioning,
            ..PanelRuntimeRenderState::default()
        };
        let render_state = crate::native_panel_core::resolve_panel_render_state(
            crate::native_panel_core::PanelRenderStateInput {
                shared_expanded_enabled: false,
                shell_visible: layout.shell_visible,
                separator_visibility: layout.separator_visibility,
                bar_progress: 1.0,
                height_progress: 1.0,
                cards_height: layout.cards_frame.height,
                status_surface_active: false,
                content_visibility: 1.0,
                transitioning,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            },
        );

        resolve_native_panel_render_command_bundle(layout, &scene, runtime, render_state, None)
    }
}
