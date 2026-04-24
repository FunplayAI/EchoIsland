use crate::{
    native_panel_core::{PanelAnimationDescriptor, PanelRect},
    native_panel_scene::{PanelRuntimeRenderState, PanelScene},
};

use super::{
    NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPlatformEvent,
    NativePanelPointerRegion, NativePanelRenderCommandBundle, NativePanelTimelineDescriptor,
    sync_native_panel_host_window_screen_frame, sync_native_panel_host_window_shared_body_height,
    sync_native_panel_host_window_timeline,
};

pub(crate) trait NativePanelRenderer {
    type Error;

    fn render_scene(
        &mut self,
        scene: &PanelScene,
        runtime: PanelRuntimeRenderState,
    ) -> Result<(), Self::Error>;

    fn apply_animation_descriptor(
        &mut self,
        _descriptor: PanelAnimationDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply_timeline_descriptor(
        &mut self,
        descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        self.apply_animation_descriptor(descriptor.animation)
    }

    fn sync_host_window_state(
        &mut self,
        _state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn sync_screen_frame(&mut self, _screen_frame: Option<PanelRect>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn sync_shared_body_height(
        &mut self,
        _shared_body_height: Option<f64>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn record_host_window_descriptor(
        &mut self,
        _descriptor: NativePanelHostWindowDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn sync_host_window_descriptor(
        &mut self,
        descriptor: NativePanelHostWindowDescriptor,
        state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error> {
        self.record_host_window_descriptor(descriptor)?;
        self.sync_screen_frame(descriptor.screen_frame)?;
        self.sync_shared_body_height(descriptor.shared_body_height)?;
        self.sync_host_window_state(state)?;
        if let Some(timeline) = descriptor.timeline {
            self.apply_timeline_descriptor(timeline)?;
        }
        self.set_visible(descriptor.visible)
    }

    fn sync_pointer_regions(
        &mut self,
        _regions: &[NativePanelPointerRegion],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply_render_command_bundle(
        &mut self,
        bundle: &NativePanelRenderCommandBundle,
    ) -> Result<(), Self::Error> {
        self.render_scene(&bundle.scene, bundle.runtime)?;
        self.sync_pointer_regions(&bundle.pointer_regions)
    }

    fn set_visible(&mut self, visible: bool) -> Result<(), Self::Error>;
}

pub(crate) trait NativePanelHost {
    type Error;
    type Renderer: NativePanelRenderer<Error = Self::Error>;

    fn renderer(&mut self) -> &mut Self::Renderer;

    fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor;

    fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor;

    fn window_state(&self) -> NativePanelHostWindowState;

    fn show(&mut self) -> Result<(), Self::Error>;

    fn hide(&mut self) -> Result<(), Self::Error>;

    fn create(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn sync_renderer_window_state(&mut self) -> Result<(), Self::Error> {
        let state = self.window_state();
        self.renderer().sync_host_window_state(state)
    }

    fn sync_renderer_host_window_descriptor(&mut self) -> Result<(), Self::Error> {
        let descriptor = self.host_window_descriptor();
        let state = self.window_state();
        self.renderer()
            .sync_host_window_descriptor(descriptor, state)
    }

    fn after_host_window_descriptor_updated(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_host_window_descriptor(
        &mut self,
        update: impl FnOnce(&mut NativePanelHostWindowDescriptor),
    ) -> Result<(), Self::Error> {
        update(self.host_window_descriptor_mut());
        self.after_host_window_descriptor_updated()?;
        self.sync_renderer_host_window_descriptor()
    }

    fn sync_pointer_regions(
        &mut self,
        regions: &[NativePanelPointerRegion],
    ) -> Result<(), Self::Error> {
        self.renderer().sync_pointer_regions(regions)
    }

    fn reposition_to_display(
        &mut self,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) -> Result<(), Self::Error> {
        self.create()?;
        self.update_host_window_descriptor(|descriptor| {
            sync_native_panel_host_window_screen_frame(
                descriptor,
                preferred_display_index,
                screen_frame,
            );
        })
    }

    fn set_shared_body_height(&mut self, body_height: f64) -> Result<(), Self::Error> {
        self.update_host_window_descriptor(|descriptor| {
            sync_native_panel_host_window_shared_body_height(descriptor, Some(body_height));
        })
    }

    fn apply_timeline_descriptor(
        &mut self,
        descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        self.create()?;
        self.update_host_window_descriptor(|host_descriptor| {
            sync_native_panel_host_window_timeline(host_descriptor, Some(descriptor));
        })
    }

    fn take_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        Vec::new()
    }
}

pub(crate) trait NativePanelSceneHost: NativePanelHost {
    fn sync_scene_window_descriptor(
        &mut self,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) -> Result<(), Self::Error> {
        self.create()?;
        self.update_host_window_descriptor(|descriptor| {
            sync_native_panel_host_window_screen_frame(
                descriptor,
                preferred_display_index,
                screen_frame,
            );
        })
    }

    fn sync_scene_descriptor(
        &mut self,
        scene: &PanelScene,
        runtime: PanelRuntimeRenderState,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) -> Result<(), Self::Error> {
        self.sync_scene_window_descriptor(preferred_display_index, screen_frame)?;
        self.renderer().render_scene(scene, runtime)
    }

    fn sync_scene(
        &mut self,
        scene: &PanelScene,
        runtime: PanelRuntimeRenderState,
        preferred_display_index: usize,
        screen_frame: Option<PanelRect>,
    ) -> Result<(), Self::Error> {
        self.sync_scene_descriptor(scene, runtime, preferred_display_index, screen_frame)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
        NativePanelRenderer, NativePanelSceneHost,
    };
    use crate::{
        native_panel_core::PanelRect,
        native_panel_scene::{
            CompactBarScene, PanelRuntimeRenderState, PanelScene, SceneMascotPose, SceneText,
            SessionSurfaceScene, StatusSurfaceDefaultState, StatusSurfaceDisplayMode,
            StatusSurfaceQueueState, StatusSurfaceScene, SurfaceScene,
        },
    };

    #[derive(Default)]
    struct TestRenderer {
        rendered_scene: Option<PanelScene>,
        rendered_runtime: Option<PanelRuntimeRenderState>,
        visible: bool,
    }

    impl NativePanelRenderer for TestRenderer {
        type Error = String;

        fn render_scene(
            &mut self,
            scene: &PanelScene,
            runtime: PanelRuntimeRenderState,
        ) -> Result<(), Self::Error> {
            self.rendered_scene = Some(scene.clone());
            self.rendered_runtime = Some(runtime);
            Ok(())
        }

        fn set_visible(&mut self, visible: bool) -> Result<(), Self::Error> {
            self.visible = visible;
            Ok(())
        }
    }

    #[derive(Default)]
    struct TestHost {
        renderer: TestRenderer,
        descriptor: NativePanelHostWindowDescriptor,
        create_calls: usize,
    }

    impl NativePanelHost for TestHost {
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

        fn create(&mut self) -> Result<(), Self::Error> {
            self.create_calls += 1;
            Ok(())
        }
    }

    impl NativePanelSceneHost for TestHost {}

    #[test]
    fn sync_scene_window_descriptor_updates_screen_frame() {
        let mut host = TestHost::default();
        let screen_frame = Some(PanelRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 120.0,
        });

        host.sync_scene_window_descriptor(2, screen_frame)
            .expect("sync scene window descriptor");

        assert_eq!(host.create_calls, 1);
        assert_eq!(host.descriptor.preferred_display_index, 2);
        assert_eq!(host.descriptor.screen_frame, screen_frame);
    }

    #[test]
    fn sync_scene_descriptor_updates_window_then_renders() {
        let mut host = TestHost::default();
        let scene = test_scene();
        let runtime = PanelRuntimeRenderState {
            transitioning: true,
            ..PanelRuntimeRenderState::default()
        };

        host.sync_scene_descriptor(
            &scene,
            runtime,
            1,
            Some(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            }),
        )
        .expect("sync scene descriptor");

        assert_eq!(host.create_calls, 1);
        assert_eq!(
            host.renderer.rendered_scene.as_ref().map(|it| it.surface),
            Some(scene.surface)
        );
        assert_eq!(host.renderer.rendered_runtime, Some(runtime));
        assert_eq!(host.descriptor.preferred_display_index, 1);
    }

    fn test_scene() -> PanelScene {
        PanelScene {
            surface: crate::native_panel_core::ExpandedSurface::Default,
            compact_bar: CompactBarScene {
                headline: SceneText {
                    text: "idle".to_string(),
                    emphasized: false,
                },
                active_count: "0".to_string(),
                total_count: "0".to_string(),
                completion_count: 0,
                actions_visible: false,
            },
            surface_scene: SurfaceScene {
                mode: crate::native_panel_scene::surface_scene_mode(
                    crate::native_panel_core::ExpandedSurface::Default,
                ),
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
            settings_surface: crate::native_panel_scene::build_settings_surface_scene(
                1,
                crate::native_panel_core::PanelSettingsState::default(),
                "0.0.0",
            ),
            cards: vec![],
            glow: None,
            mascot_pose: SceneMascotPose::Idle,
            hit_targets: vec![],
            nodes: vec![],
        }
    }
}
