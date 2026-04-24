use crate::{
    native_panel_core::{DEFAULT_COMPACT_PILL_WIDTH, DEFAULT_EXPANDED_PILL_WIDTH, PanelRect},
    native_panel_renderer::{
        NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
        NativePanelPlatformEvent, NativePanelPointerRegion, NativePanelRenderCommandBundle,
        NativePanelRenderer, NativePanelRuntimeSceneCache, NativePanelTimelineDescriptor,
        cache_render_command_bundle, cache_scene_runtime, native_panel_host_window_frame,
    },
    native_panel_scene::{PanelRuntimeRenderState, PanelScene},
};

use super::panel_types::NativePanelState;
use objc2_foundation::NSRect;

#[derive(Default)]
pub(super) struct MacosNativePanelRendererAdapter {
    pub(super) scene_cache: NativePanelRuntimeSceneCache,
    pub(super) last_timeline_descriptor: Option<NativePanelTimelineDescriptor>,
    pub(super) last_host_window_descriptor: Option<NativePanelHostWindowDescriptor>,
    pub(super) last_host_window_state: Option<NativePanelHostWindowState>,
    pub(super) last_pointer_regions: Vec<NativePanelPointerRegion>,
    pub(super) visible: bool,
}

impl NativePanelRenderer for MacosNativePanelRendererAdapter {
    type Error = String;

    fn render_scene(
        &mut self,
        scene: &PanelScene,
        runtime: PanelRuntimeRenderState,
    ) -> Result<(), Self::Error> {
        cache_scene_runtime(&mut self.scene_cache, scene.clone(), runtime);
        Ok(())
    }

    fn apply_timeline_descriptor(
        &mut self,
        descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_timeline_descriptor = Some(descriptor);
        Ok(())
    }

    fn sync_host_window_state(
        &mut self,
        state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error> {
        self.last_host_window_state = Some(state);
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
        descriptor: NativePanelHostWindowDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_host_window_descriptor = Some(descriptor);
        Ok(())
    }

    fn sync_pointer_regions(
        &mut self,
        regions: &[NativePanelPointerRegion],
    ) -> Result<(), Self::Error> {
        self.last_pointer_regions = regions.to_vec();
        Ok(())
    }

    fn record_render_command_bundle(
        &mut self,
        bundle: &NativePanelRenderCommandBundle,
    ) -> Result<(), Self::Error> {
        cache_render_command_bundle(&mut self.scene_cache, bundle);
        Ok(())
    }

    fn set_visible(&mut self, visible: bool) -> Result<(), Self::Error> {
        self.visible = visible;
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct MacosNativePanelHostAdapter {
    pub(super) renderer: MacosNativePanelRendererAdapter,
    pub(super) descriptor: NativePanelHostWindowDescriptor,
    pub(super) frame: Option<PanelRect>,
    pending_events: Vec<NativePanelPlatformEvent>,
}

impl NativePanelHost for MacosNativePanelHostAdapter {
    type Error = String;
    type Renderer = MacosNativePanelRendererAdapter;

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
        self.descriptor
            .window_state(self.resolved_window_frame().or(self.frame))
    }

    fn show(&mut self) -> Result<(), Self::Error> {
        self.descriptor.visible = true;
        self.sync_renderer_host_window_descriptor()
    }

    fn hide(&mut self) -> Result<(), Self::Error> {
        self.descriptor.visible = false;
        self.sync_renderer_host_window_descriptor()
    }

    fn take_platform_events(&mut self) -> Vec<NativePanelPlatformEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

#[allow(dead_code)]
impl MacosNativePanelHostAdapter {
    pub(super) fn sync_from_native_panel_state(
        &mut self,
        state: &NativePanelState,
        frame: Option<NSRect>,
    ) {
        self.descriptor = state.host_window_descriptor;
        self.frame = frame.map(ns_rect_to_panel_rect);
        self.renderer.scene_cache = state.scene_cache.clone();
        if self.renderer.scene_cache.last_snapshot.is_none() {
            self.renderer.scene_cache.last_snapshot = state.last_snapshot.clone();
        }
    }

    fn resolved_window_frame(&self) -> Option<PanelRect> {
        native_panel_host_window_frame(
            self.descriptor,
            self.descriptor.screen_frame?,
            DEFAULT_COMPACT_PILL_WIDTH,
            DEFAULT_EXPANDED_PILL_WIDTH,
        )
    }
}

#[allow(dead_code)]
pub(super) fn ns_rect_to_panel_rect(rect: NSRect) -> PanelRect {
    PanelRect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use echoisland_runtime::RuntimeSnapshot;
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::MacosNativePanelHostAdapter;
    use crate::{
        macos_native_test_panel::{
            mascot::NativeMascotRuntime, panel_types::NativeExpandedSurface,
        },
        native_panel_renderer::{
            NativePanelHost, NativePanelHostWindowDescriptor, NativePanelRenderer,
            NativePanelTimelineDescriptor, resolve_native_panel_render_command_bundle,
        },
        native_panel_scene::{PanelRuntimeRenderState, PanelSceneBuildInput, build_panel_scene},
    };

    fn panel_state() -> crate::macos_native_test_panel::panel_types::NativePanelState {
        crate::macos_native_test_panel::panel_types::NativePanelState {
            expanded: false,
            transitioning: false,
            transition_cards_progress: 0.0,
            transition_cards_entering: false,
            skip_next_close_card_exit: false,
            last_raw_snapshot: None,
            last_snapshot: Some(RuntimeSnapshot {
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
            }),
            scene_cache: crate::native_panel_renderer::NativePanelRuntimeSceneCache::default(),
            status_queue: Vec::new(),
            completion_badge_items: Vec::new(),
            pending_permission_card: None,
            pending_question_card: None,
            status_auto_expanded: false,
            surface_mode: NativeExpandedSurface::Default,
            shared_body_height: None,
            host_window_descriptor: NativePanelHostWindowDescriptor {
                visible: true,
                preferred_display_index: 2,
                screen_frame: None,
                shared_body_height: Some(180.0),
                timeline: Some(NativePanelTimelineDescriptor {
                    animation: crate::native_panel_core::PanelAnimationDescriptor {
                        kind: crate::native_panel_core::PanelAnimationKind::Open,
                        canvas_height: 160.0,
                        visible_height: 120.0,
                        width_progress: 0.5,
                        height_progress: 0.75,
                        shoulder_progress: 1.0,
                        drop_progress: 0.25,
                        cards_progress: 0.8,
                    },
                    cards_entering: true,
                }),
            },
            pointer_inside_since: None,
            pointer_outside_since: None,
            primary_mouse_down: false,
            last_focus_click: None,
            pointer_regions: Vec::new(),
            mascot_runtime: NativeMascotRuntime::new(Instant::now()),
        }
    }

    #[test]
    fn macos_host_adapter_projects_native_state_descriptor_and_frame() {
        let state = panel_state();
        let mut host = MacosNativePanelHostAdapter::default();

        host.sync_from_native_panel_state(
            &state,
            Some(NSRect::new(
                NSPoint::new(10.0, 20.0),
                NSSize::new(300.0, 200.0),
            )),
        );

        assert_eq!(host.host_window_descriptor(), state.host_window_descriptor);
        assert_eq!(
            host.renderer
                .scene_cache
                .last_snapshot
                .as_ref()
                .map(|snapshot| snapshot.status.as_str()),
            Some("idle")
        );
        assert_eq!(
            host.window_state(),
            state
                .host_window_descriptor
                .window_state(Some(crate::native_panel_core::PanelRect {
                    x: 10.0,
                    y: 20.0,
                    width: 300.0,
                    height: 200.0,
                }))
        );
    }

    #[test]
    fn macos_host_adapter_prefers_shared_descriptor_frame_when_available() {
        let mut state = panel_state();
        state.host_window_descriptor.screen_frame = Some(crate::native_panel_core::PanelRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        });
        let mut host = MacosNativePanelHostAdapter::default();

        host.sync_from_native_panel_state(
            &state,
            Some(NSRect::new(
                NSPoint::new(10.0, 20.0),
                NSSize::new(300.0, 200.0),
            )),
        );

        assert_eq!(
            host.window_state(),
            state
                .host_window_descriptor
                .window_state(Some(crate::native_panel_core::PanelRect {
                    x: 586.0,
                    y: 740.0,
                    width: 268.0,
                    height: 160.0,
                }))
        );
    }

    #[test]
    fn macos_host_adapter_syncs_descriptor_into_renderer() {
        let state = panel_state();
        let mut host = MacosNativePanelHostAdapter::default();
        host.sync_from_native_panel_state(&state, None);

        host.show().expect("show adapter host");

        assert_eq!(
            host.renderer.last_host_window_state,
            Some(state.host_window_descriptor.window_state(None))
        );
        assert_eq!(
            host.renderer.last_host_window_descriptor,
            Some(state.host_window_descriptor)
        );
        assert_eq!(
            host.renderer.last_timeline_descriptor,
            state.host_window_descriptor.timeline
        );
        assert!(host.renderer.visible);

        host.hide().expect("hide adapter host");
        assert!(!host.renderer.visible);
    }

    #[test]
    fn macos_renderer_adapter_stores_scene_in_shared_cache() {
        let mut host = MacosNativePanelHostAdapter::default();
        let snapshot = RuntimeSnapshot {
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
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &snapshot,
            &PanelSceneBuildInput::default(),
        );
        let runtime = PanelRuntimeRenderState {
            transitioning: true,
            ..PanelRuntimeRenderState::default()
        };

        host.renderer
            .render_scene(&scene, runtime)
            .expect("render scene into adapter cache");

        assert_eq!(
            host.renderer
                .scene_cache
                .last_scene
                .as_ref()
                .map(|cached| cached.surface),
            Some(scene.surface)
        );
        assert_eq!(
            host.renderer.scene_cache.last_runtime_render_state,
            Some(runtime)
        );
    }

    #[test]
    fn macos_renderer_adapter_stores_render_command_bundle_in_shared_cache() {
        let mut host = MacosNativePanelHostAdapter::default();
        let snapshot = RuntimeSnapshot {
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
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState {
                expanded: true,
                ..Default::default()
            },
            &snapshot,
            &PanelSceneBuildInput::default(),
        );
        let runtime = PanelRuntimeRenderState::default();
        let layout = crate::native_panel_core::resolve_panel_layout(
            crate::native_panel_core::PanelLayoutInput {
                screen_frame: crate::native_panel_core::PanelRect {
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
                transitioning: false,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            },
        );
        let bundle =
            resolve_native_panel_render_command_bundle(layout, &scene, runtime, render_state, None);

        host.renderer
            .apply_render_command_bundle(&bundle)
            .expect("cache render command bundle");

        assert_eq!(
            host.renderer
                .scene_cache
                .last_render_command_bundle
                .as_ref()
                .map(|cached| cached.compact_bar.headline.text.as_str()),
            Some(scene.compact_bar.headline.text.as_str())
        );
        assert_eq!(host.renderer.last_pointer_regions, bundle.pointer_regions);
    }
}
