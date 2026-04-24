use echoisland_runtime::RuntimeSnapshot;

use crate::{
    native_panel_core::PanelRect,
    native_panel_scene::{PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelScene},
};

use super::{NativePanelRuntimeInputDescriptor, NativePanelSceneHost};

#[derive(Clone, Debug, Default)]
pub(crate) struct NativePanelRuntimeSceneCache {
    pub(crate) last_snapshot: Option<RuntimeSnapshot>,
    pub(crate) last_scene: Option<PanelScene>,
    pub(crate) last_runtime_render_state: Option<PanelRuntimeRenderState>,
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
    cache.last_snapshot = Some(bundle.displayed_snapshot);
    cache.last_scene = Some(bundle.scene);
    cache.last_runtime_render_state = Some(bundle.runtime_render_state);
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
        rerender_runtime_scene_cache_to_host, rerender_runtime_scene_cache_to_host_on_transition,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelRect, PanelSettingsState},
        native_panel_renderer::{
            NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
            NativePanelRenderer, NativePanelSceneHost,
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
}
