use crate::{
    native_panel_core::{
        ExpandedSurface, PanelAnimationDescriptor, PanelGeometryMetrics, PanelLayout,
        PanelLayoutInput, PanelRect, PanelRenderState, PanelRenderStateInput, resolve_panel_layout,
        resolve_panel_render_state,
    },
    native_panel_renderer::facade::{
        descriptor::{
            NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPointerRegion,
            NativePanelTimelineDescriptor,
        },
        presentation::NativePanelPresentationModel,
        renderer::{
            NativePanelCachedRendererBackend, NativePanelRenderCommandBundle, NativePanelRenderer,
            NativePanelRuntimeSceneCache, cache_host_window_descriptor_on_renderer,
            cache_host_window_state_on_renderer, cache_pointer_regions_on_renderer,
            cache_render_command_bundle_on_renderer, cache_scene_runtime_on_renderer,
            cache_timeline_descriptor_on_renderer, cached_runtime_render_state, cached_scene,
            resolve_and_cache_presentation_from_scene_cache_on_renderer,
            resolve_cached_presentation_model,
        },
    },
    native_panel_scene::{PanelRuntimeRenderState, PanelScene},
};

use super::{WINDOWS_FALLBACK_PANEL_SCREEN_FRAME, host_runtime::WindowsNativePanelHost};

#[derive(Default)]
pub(crate) struct WindowsNativePanelRenderer {
    pub(super) scene_cache: NativePanelRuntimeSceneCache,
    pub(super) last_screen_frame: Option<PanelRect>,
    pub(super) last_animation_descriptor: Option<PanelAnimationDescriptor>,
    pub(super) last_timeline_descriptor: Option<NativePanelTimelineDescriptor>,
    pub(super) last_host_window_descriptor: Option<NativePanelHostWindowDescriptor>,
    pub(super) last_layout: Option<PanelLayout>,
    pub(super) last_render_state: Option<PanelRenderState>,
    pub(super) last_window_state: Option<NativePanelHostWindowState>,
    pub(super) last_pointer_regions: Vec<NativePanelPointerRegion>,
    pub(super) last_presentation_model: Option<NativePanelPresentationModel>,
}

impl NativePanelCachedRendererBackend for WindowsNativePanelRenderer {
    type Error = String;

    fn scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache {
        &mut self.scene_cache
    }

    fn timeline_descriptor_slot(&mut self) -> &mut Option<NativePanelTimelineDescriptor> {
        &mut self.last_timeline_descriptor
    }

    fn host_window_descriptor_slot(&mut self) -> &mut Option<NativePanelHostWindowDescriptor> {
        &mut self.last_host_window_descriptor
    }

    fn host_window_state_slot(&mut self) -> &mut Option<NativePanelHostWindowState> {
        &mut self.last_window_state
    }

    fn pointer_regions_slot(&mut self) -> &mut Vec<NativePanelPointerRegion> {
        &mut self.last_pointer_regions
    }

    fn presentation_model_slot(&mut self) -> Option<&mut Option<NativePanelPresentationModel>> {
        Some(&mut self.last_presentation_model)
    }

    fn after_scene_runtime_cached(&mut self) -> Result<(), Self::Error> {
        self.refresh_cached_render_inputs();
        Ok(())
    }

    fn after_timeline_descriptor_cached(
        &mut self,
        descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_animation_descriptor = Some(descriptor.animation);
        self.refresh_cached_render_inputs();
        Ok(())
    }

    fn after_render_command_bundle_cached(
        &mut self,
        bundle: &NativePanelRenderCommandBundle,
    ) -> Result<(), Self::Error> {
        self.last_layout = Some(bundle.layout);
        self.last_render_state = Some(bundle.render_state);
        Ok(())
    }
}

impl NativePanelRenderer for WindowsNativePanelRenderer {
    type Error = String;

    fn render_scene(
        &mut self,
        scene: &PanelScene,
        runtime: PanelRuntimeRenderState,
    ) -> Result<(), Self::Error> {
        cache_scene_runtime_on_renderer(self, scene, runtime)
    }

    fn apply_animation_descriptor(
        &mut self,
        descriptor: PanelAnimationDescriptor,
    ) -> Result<(), Self::Error> {
        self.last_animation_descriptor = Some(descriptor);
        self.refresh_cached_render_inputs();
        Ok(())
    }

    fn apply_timeline_descriptor(
        &mut self,
        descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        cache_timeline_descriptor_on_renderer(self, descriptor)
    }

    fn sync_host_window_state(
        &mut self,
        state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error> {
        cache_host_window_state_on_renderer(self, state)
    }

    fn sync_screen_frame(&mut self, screen_frame: Option<PanelRect>) -> Result<(), Self::Error> {
        self.update_screen_frame(screen_frame);
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
        cache_host_window_descriptor_on_renderer(self, descriptor)
    }

    fn sync_pointer_regions(
        &mut self,
        regions: &[NativePanelPointerRegion],
    ) -> Result<(), Self::Error> {
        cache_pointer_regions_on_renderer(self, regions)
    }

    fn apply_render_command_bundle(
        &mut self,
        bundle: &NativePanelRenderCommandBundle,
    ) -> Result<(), Self::Error> {
        cache_render_command_bundle_on_renderer(self, bundle)
    }

    fn set_visible(&mut self, _visible: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl WindowsNativePanelRenderer {
    pub(super) fn current_presentation_model(&self) -> Option<NativePanelPresentationModel> {
        resolve_cached_presentation_model(self.last_presentation_model.as_ref(), &self.scene_cache)
    }

    pub(super) fn update_screen_frame(&mut self, screen_frame: Option<PanelRect>) {
        self.last_screen_frame = screen_frame;
        self.refresh_cached_render_inputs();
    }

    fn refresh_cached_render_inputs(&mut self) {
        let Some(descriptor) = self.last_animation_descriptor else {
            return;
        };
        let screen_frame = self
            .last_screen_frame
            .unwrap_or(WINDOWS_FALLBACK_PANEL_SCREEN_FRAME);
        let layout = resolve_panel_layout(PanelLayoutInput {
            screen_frame,
            metrics: PanelGeometryMetrics {
                compact_height: crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT,
                compact_width: crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH,
                expanded_width: crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
                panel_width: crate::native_panel_core::DEFAULT_PANEL_CANVAS_WIDTH,
            },
            canvas_height: descriptor.canvas_height,
            visible_height: descriptor.visible_height,
            bar_progress: descriptor.width_progress,
            height_progress: descriptor.height_progress,
            drop_progress: descriptor.drop_progress,
            content_visibility: descriptor.cards_progress,
            collapsed_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
            drop_distance: crate::native_panel_core::PANEL_DROP_DISTANCE,
            content_top_gap: crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP,
            content_bottom_inset: crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET,
            cards_side_inset: crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
            shoulder_size: crate::native_panel_core::COMPACT_SHOULDER_SIZE,
            separator_side_inset: crate::native_panel_core::EXPANDED_SEPARATOR_SIDE_INSET,
        });
        let status_surface_active = cached_scene(&self.scene_cache)
            .as_ref()
            .is_some_and(|scene| scene.surface == ExpandedSurface::Status);
        let runtime = cached_runtime_render_state(&self.scene_cache).unwrap_or_default();
        let render_state = resolve_panel_render_state(PanelRenderStateInput {
            shared_expanded_enabled: false,
            shell_visible: layout.shell_visible,
            separator_visibility: layout.separator_visibility,
            bar_progress: descriptor.width_progress,
            height_progress: descriptor.height_progress,
            cards_height: layout.cards_frame.height,
            status_surface_active,
            content_visibility: descriptor.cards_progress,
            transitioning: runtime.transitioning,
            headline_emphasized: runtime.shell_scene.headline_emphasized,
            edge_actions_visible: runtime.shell_scene.edge_actions_visible,
        });

        if cached_scene(&self.scene_cache).is_none() {
            self.last_layout = Some(layout);
            self.last_render_state = Some(render_state);
            self.last_pointer_regions = Vec::new();
            self.last_presentation_model = None;
            self.scene_cache.last_render_command_bundle = None;
            return;
        }
        let _ = resolve_and_cache_presentation_from_scene_cache_on_renderer(
            self,
            layout,
            render_state,
            None,
        );
    }
}

impl WindowsNativePanelHost {
    pub(super) fn renderer_ref(&self) -> &WindowsNativePanelRenderer {
        &self.renderer
    }
}
