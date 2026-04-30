use crate::{
    native_panel_core::{PanelLayout, PanelRenderState},
    native_panel_scene::{PanelRuntimeRenderState, PanelScene},
};

use super::descriptors::{
    NativePanelHostWindowDescriptor, NativePanelHostWindowState, NativePanelPointerRegion,
    NativePanelPointerRegionInput, NativePanelTimelineDescriptor,
};
use super::presentation_model::{
    NativePanelPresentationModel, NativePanelResolvedPresentation,
    resolve_native_panel_presentation, resolve_native_panel_presentation_model,
};
use super::render_commands::NativePanelRenderCommandBundle;
use super::runtime_scene_cache::{
    NativePanelRuntimeSceneCache, cache_render_command_bundle, cache_scene_runtime,
    cached_presentation_model, cached_runtime_render_state, cached_scene,
};

pub(crate) trait NativePanelCachedRendererBackend {
    type Error;

    fn scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache;

    fn timeline_descriptor_slot(&mut self) -> &mut Option<NativePanelTimelineDescriptor>;

    fn host_window_descriptor_slot(&mut self) -> &mut Option<NativePanelHostWindowDescriptor>;

    fn host_window_state_slot(&mut self) -> &mut Option<NativePanelHostWindowState>;

    fn pointer_regions_slot(&mut self) -> &mut Vec<NativePanelPointerRegion>;

    fn presentation_model_slot(&mut self) -> Option<&mut Option<NativePanelPresentationModel>> {
        None
    }

    fn set_cached_visible(&mut self, _visible: bool) {}

    fn after_scene_runtime_cached(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn after_timeline_descriptor_cached(
        &mut self,
        _descriptor: NativePanelTimelineDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn after_render_command_bundle_cached(
        &mut self,
        _bundle: &NativePanelRenderCommandBundle,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub(crate) fn cache_scene_runtime_on_renderer<R>(
    renderer: &mut R,
    scene: &PanelScene,
    runtime: PanelRuntimeRenderState,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    cache_scene_runtime(renderer.scene_cache_mut(), scene.clone(), runtime);
    if let Some(slot) = renderer.presentation_model_slot() {
        *slot = None;
    }
    renderer.after_scene_runtime_cached()
}

pub(crate) fn cache_timeline_descriptor_on_renderer<R>(
    renderer: &mut R,
    descriptor: NativePanelTimelineDescriptor,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    *renderer.timeline_descriptor_slot() = Some(descriptor);
    renderer.after_timeline_descriptor_cached(descriptor)
}

pub(crate) fn cache_host_window_state_on_renderer<R>(
    renderer: &mut R,
    state: NativePanelHostWindowState,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    *renderer.host_window_state_slot() = Some(state);
    Ok(())
}

pub(crate) fn cache_host_window_descriptor_on_renderer<R>(
    renderer: &mut R,
    descriptor: NativePanelHostWindowDescriptor,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    *renderer.host_window_descriptor_slot() = Some(descriptor);
    Ok(())
}

pub(crate) fn cache_pointer_regions_on_renderer<R>(
    renderer: &mut R,
    regions: &[NativePanelPointerRegion],
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    *renderer.pointer_regions_slot() = regions.to_vec();
    Ok(())
}

pub(crate) fn cache_render_command_bundle_on_renderer<R>(
    renderer: &mut R,
    bundle: &NativePanelRenderCommandBundle,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    cache_render_command_bundle(renderer.scene_cache_mut(), bundle);
    *renderer.pointer_regions_slot() = bundle.pointer_regions.clone();
    if let Some(slot) = renderer.presentation_model_slot() {
        *slot = Some(resolve_native_panel_presentation_model(bundle));
    }
    renderer.after_render_command_bundle_cached(bundle)
}

pub(crate) fn cache_resolved_presentation_on_renderer<R>(
    renderer: &mut R,
    resolved: NativePanelResolvedPresentation,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    cache_render_command_bundle(renderer.scene_cache_mut(), &resolved.bundle);
    *renderer.pointer_regions_slot() = resolved.bundle.pointer_regions.clone();
    if let Some(slot) = renderer.presentation_model_slot() {
        *slot = Some(resolved.presentation);
    }
    renderer.after_render_command_bundle_cached(&resolved.bundle)
}

pub(crate) fn resolve_and_cache_presentation_from_scene_cache_on_renderer<R>(
    renderer: &mut R,
    layout: PanelLayout,
    render_state: PanelRenderState,
    pointer_region_input: Option<NativePanelPointerRegionInput>,
) -> Result<bool, R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    let (scene, runtime) = {
        let cache = renderer.scene_cache_mut();
        (
            cached_scene(cache),
            cached_runtime_render_state(cache).unwrap_or_default(),
        )
    };
    let Some(scene) = scene else {
        return Ok(false);
    };
    let resolved = resolve_native_panel_presentation(
        layout,
        &scene,
        runtime,
        render_state,
        pointer_region_input,
    );
    cache_resolved_presentation_on_renderer(renderer, resolved)?;
    Ok(true)
}

pub(crate) fn sync_cached_presentation_model_slot(
    slot: &mut Option<NativePanelPresentationModel>,
    cache: &NativePanelRuntimeSceneCache,
) {
    *slot = cached_presentation_model(cache);
}

pub(crate) fn resolve_cached_presentation_model(
    slot: Option<&NativePanelPresentationModel>,
    cache: &NativePanelRuntimeSceneCache,
) -> Option<NativePanelPresentationModel> {
    slot.cloned().or_else(|| cached_presentation_model(cache))
}

pub(crate) fn native_panel_presentation_cards_visible(
    primary: Option<&NativePanelPresentationModel>,
    fallback: Option<&NativePanelPresentationModel>,
) -> bool {
    primary
        .or(fallback)
        .is_some_and(NativePanelPresentationModel::card_stack_visible)
}

pub(crate) fn sync_cached_visibility_on_renderer<R>(
    renderer: &mut R,
    visible: bool,
) -> Result<(), R::Error>
where
    R: NativePanelCachedRendererBackend,
{
    renderer.set_cached_visible(visible);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        native_panel_presentation_cards_visible,
        resolve_and_cache_presentation_from_scene_cache_on_renderer,
        resolve_cached_presentation_model, sync_cached_presentation_model_slot,
    };
    use crate::{
        native_panel_core::{
            PanelGeometryMetrics, PanelLayoutInput, PanelRect, PanelRenderStateInput, PanelState,
            resolve_panel_layout, resolve_panel_render_state,
        },
        native_panel_renderer::{
            descriptors::{
                NativePanelHostWindowDescriptor, NativePanelHostWindowState,
                NativePanelPointerRegion, NativePanelTimelineDescriptor,
            },
            presentation_model::{
                NativePanelPresentationModel, build_native_panel_presentation_model,
            },
            renderer_backend::NativePanelCachedRendererBackend,
            runtime_scene_cache::{NativePanelRuntimeSceneCache, cache_scene_runtime},
        },
        native_panel_scene::{PanelRuntimeRenderState, PanelSceneBuildInput, build_panel_scene},
    };

    fn snapshot() -> echoisland_runtime::RuntimeSnapshot {
        echoisland_runtime::RuntimeSnapshot {
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
        }
    }

    #[derive(Default)]
    struct TestCachedRendererBackend {
        scene_cache: NativePanelRuntimeSceneCache,
        pointer_regions: Vec<NativePanelPointerRegion>,
        presentation_model: Option<NativePanelPresentationModel>,
        timeline_descriptor: Option<NativePanelTimelineDescriptor>,
        host_window_descriptor: Option<NativePanelHostWindowDescriptor>,
        host_window_state: Option<NativePanelHostWindowState>,
    }

    impl NativePanelCachedRendererBackend for TestCachedRendererBackend {
        type Error = String;

        fn scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache {
            &mut self.scene_cache
        }

        fn timeline_descriptor_slot(&mut self) -> &mut Option<NativePanelTimelineDescriptor> {
            &mut self.timeline_descriptor
        }

        fn host_window_descriptor_slot(&mut self) -> &mut Option<NativePanelHostWindowDescriptor> {
            &mut self.host_window_descriptor
        }

        fn host_window_state_slot(&mut self) -> &mut Option<NativePanelHostWindowState> {
            &mut self.host_window_state
        }

        fn pointer_regions_slot(&mut self) -> &mut Vec<NativePanelPointerRegion> {
            &mut self.pointer_regions
        }

        fn presentation_model_slot(&mut self) -> Option<&mut Option<NativePanelPresentationModel>> {
            Some(&mut self.presentation_model)
        }
    }

    #[test]
    fn cached_presentation_helpers_rebuild_from_scene_cache() {
        let scene = build_panel_scene(
            &PanelState::default(),
            &snapshot(),
            &PanelSceneBuildInput::default(),
        );
        let mut cache = NativePanelRuntimeSceneCache::default();
        cache.last_scene = Some(scene.clone());

        let mut slot = None;
        sync_cached_presentation_model_slot(&mut slot, &cache);
        let resolved = resolve_cached_presentation_model(slot.as_ref(), &cache)
            .expect("presentation from cache");

        assert_eq!(
            resolved.compact_bar.headline.text,
            scene.compact_bar.headline.text
        );
    }

    #[test]
    fn presentation_cards_visible_prefers_primary_then_fallback() {
        let scene = build_panel_scene(
            &PanelState::default(),
            &snapshot(),
            &PanelSceneBuildInput::default(),
        );

        let mut visible = build_native_panel_presentation_model(&scene);
        visible.card_stack.visible = true;
        let mut hidden = build_native_panel_presentation_model(&scene);
        hidden.card_stack.visible = false;

        assert!(native_panel_presentation_cards_visible(
            Some(&visible),
            Some(&hidden)
        ));
        assert!(native_panel_presentation_cards_visible(
            None,
            Some(&visible)
        ));
        assert!(!native_panel_presentation_cards_visible(
            Some(&hidden),
            Some(&visible)
        ));
    }

    #[test]
    fn resolve_and_cache_renderer_presentation_uses_cached_scene_runtime() {
        let scene = build_panel_scene(
            &PanelState {
                expanded: true,
                ..PanelState::default()
            },
            &snapshot(),
            &PanelSceneBuildInput::default(),
        );
        let layout = resolve_panel_layout(PanelLayoutInput {
            screen_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0,
            },
            metrics: PanelGeometryMetrics {
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
        });
        let runtime = PanelRuntimeRenderState {
            transitioning: true,
            ..PanelRuntimeRenderState::default()
        };
        let render_state = resolve_panel_render_state(PanelRenderStateInput {
            shared_expanded_enabled: false,
            shell_visible: layout.shell_visible,
            separator_visibility: layout.separator_visibility,
            bar_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 0.0,
            cards_height: layout.cards_frame.height,
            status_surface_active: false,
            content_visibility: 1.0,
            transitioning: runtime.transitioning,
            headline_emphasized: scene.compact_bar.headline.emphasized,
            edge_actions_visible: scene.compact_bar.actions_visible,
        });
        let mut renderer = TestCachedRendererBackend::default();

        cache_scene_runtime(&mut renderer.scene_cache, scene.clone(), runtime);

        let resolved = resolve_and_cache_presentation_from_scene_cache_on_renderer(
            &mut renderer,
            layout,
            render_state,
            None,
        )
        .expect("resolve cached presentation");

        assert!(resolved);
        assert_eq!(
            renderer.presentation_model.as_ref().map(|model| model
                .compact_bar
                .headline
                .text
                .as_str()),
            Some(scene.compact_bar.headline.text.as_str())
        );
        assert_eq!(
            renderer
                .scene_cache
                .last_render_command_bundle
                .as_ref()
                .map(|bundle| bundle.runtime.transitioning),
            Some(true)
        );
    }
}
