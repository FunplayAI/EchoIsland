use echoisland_runtime::RuntimeSnapshot;

use crate::{
    native_panel_core::{ExpandedSurface, PanelLayout, PanelRect, PanelRenderState, PanelState},
    native_panel_scene::{
        PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelScene, PanelSceneBuildInput,
        build_panel_scene, resolve_panel_runtime_render_state,
    },
};

use super::descriptors::{
    NativePanelPointerRegion, NativePanelPointerRegionInput, NativePanelRuntimeInputDescriptor,
};
use super::host_runtime_facade::{
    native_panel_host_display_reposition,
    native_panel_host_display_reposition_from_input_descriptor,
};
use super::presentation_model::{
    NativePanelPresentationModel, NativePanelResolvedPresentation, NativePanelSnapshotRenderPlan,
    resolve_native_panel_presentation, resolve_native_panel_presentation_model_for_scene,
    resolve_native_panel_snapshot_render_plan_for_scene,
};
use super::render_commands::NativePanelRenderCommandBundle;
use super::runtime_interaction::NativePanelCoreStateBridge;
use super::traits::NativePanelSceneHost;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePanelRuntimeSceneCacheKey {
    pub(crate) expanded: bool,
    pub(crate) transitioning: bool,
    pub(crate) surface_mode: ExpandedSurface,
    pub(crate) scene_input: PanelSceneBuildInput,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct NativePanelRuntimeSceneCache {
    pub(crate) last_snapshot: Option<RuntimeSnapshot>,
    pub(crate) last_cache_key: Option<NativePanelRuntimeSceneCacheKey>,
    pub(crate) last_scene: Option<PanelScene>,
    pub(crate) last_runtime_render_state: Option<PanelRuntimeRenderState>,
    pub(crate) last_render_command_bundle: Option<NativePanelRenderCommandBundle>,
}

pub(crate) trait NativePanelRuntimeSceneStateBridge: NativePanelCoreStateBridge {
    fn runtime_scene_cache(&self) -> &NativePanelRuntimeSceneCache;

    fn runtime_scene_current_snapshot(&self) -> Option<&RuntimeSnapshot>;
}

pub(crate) trait NativePanelRuntimeSceneMutableStateBridge:
    NativePanelRuntimeSceneStateBridge
{
    fn runtime_scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache;

    fn runtime_pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion>;
}

pub(crate) fn native_panel_runtime_scene_cache_key(
    panel_state: &PanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelRuntimeSceneCacheKey {
    NativePanelRuntimeSceneCacheKey {
        expanded: panel_state.expanded,
        transitioning: panel_state.transitioning,
        surface_mode: panel_state.surface_mode,
        scene_input: input.scene_input.clone(),
    }
}

pub(crate) fn native_panel_runtime_scene_cache_key_for_state_bridge<S>(
    state: &S,
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelRuntimeSceneCacheKey
where
    S: NativePanelCoreStateBridge,
{
    native_panel_runtime_scene_cache_key(&state.snapshot_core_panel_state(), input)
}

pub(crate) fn build_native_panel_scene_for_state_bridge_with_input<S>(
    state: &S,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> PanelScene
where
    S: NativePanelCoreStateBridge,
{
    build_panel_scene(
        &state.snapshot_core_panel_state(),
        snapshot,
        &input.scene_input,
    )
}

pub(crate) fn resolve_native_panel_runtime_render_state_for_state_bridge_with_input<S>(
    state: &S,
    input: &NativePanelRuntimeInputDescriptor,
) -> PanelRuntimeRenderState
where
    S: NativePanelRuntimeSceneStateBridge,
{
    let cache_key = native_panel_runtime_scene_cache_key_for_state_bridge(state, input);
    cached_runtime_render_state_for_key(state.runtime_scene_cache(), &cache_key).unwrap_or_else(
        || {
            resolve_panel_runtime_render_state(
                &state.snapshot_core_panel_state(),
                state.runtime_scene_current_snapshot(),
                &input.scene_input,
            )
        },
    )
}

fn runtime_scene_snapshot_matches_state_or_cache<S>(state: &S, snapshot: &RuntimeSnapshot) -> bool
where
    S: NativePanelRuntimeSceneStateBridge,
{
    state.runtime_scene_current_snapshot() == Some(snapshot)
        || state.runtime_scene_cache().last_snapshot.as_ref() == Some(snapshot)
}

pub(crate) fn resolve_native_panel_scene_for_state_bridge_with_input<S>(
    state: &S,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<PanelScene>
where
    S: NativePanelRuntimeSceneStateBridge,
{
    let cache_key = native_panel_runtime_scene_cache_key_for_state_bridge(state, input);
    cached_scene_for_key(state.runtime_scene_cache(), &cache_key).or_else(|| {
        state.runtime_scene_current_snapshot().map(|snapshot| {
            build_native_panel_scene_for_state_bridge_with_input(state, snapshot, input)
        })
    })
}

pub(crate) fn resolve_native_panel_scene_for_state_bridge_and_snapshot_with_input<S>(
    state: &S,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<PanelScene>
where
    S: NativePanelRuntimeSceneStateBridge,
{
    let cache_key = native_panel_runtime_scene_cache_key_for_state_bridge(state, input);
    if runtime_scene_snapshot_matches_state_or_cache(state, snapshot) {
        if let Some(scene) = cached_scene_for_key(state.runtime_scene_cache(), &cache_key) {
            return Some(scene);
        }
    }

    Some(build_native_panel_scene_for_state_bridge_with_input(
        state, snapshot, input,
    ))
}

pub(crate) fn resolve_native_panel_render_command_bundle_for_state_bridge_and_snapshot_with_input<
    S,
>(
    state: &S,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<NativePanelRenderCommandBundle>
where
    S: NativePanelRuntimeSceneStateBridge,
{
    let cache_key = native_panel_runtime_scene_cache_key_for_state_bridge(state, input);
    runtime_scene_snapshot_matches_state_or_cache(state, snapshot)
        .then(|| cached_render_command_bundle_for_key(state.runtime_scene_cache(), &cache_key))
        .flatten()
}

pub(crate) fn resolve_current_native_panel_render_command_bundle_for_state_bridge_with_input<S>(
    state: &S,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<NativePanelRenderCommandBundle>
where
    S: NativePanelRuntimeSceneStateBridge,
{
    state.runtime_scene_current_snapshot().and_then(|snapshot| {
        resolve_native_panel_render_command_bundle_for_state_bridge_and_snapshot_with_input(
            state, snapshot, input,
        )
    })
}

pub(crate) fn resolve_native_panel_presentation_model_for_state_bridge_and_snapshot_with_input<S>(
    state: &S,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<NativePanelPresentationModel>
where
    S: NativePanelRuntimeSceneStateBridge,
{
    let cache_key = native_panel_runtime_scene_cache_key_for_state_bridge(state, input);

    if runtime_scene_snapshot_matches_state_or_cache(state, snapshot) {
        if let Some(model) =
            cached_presentation_model_for_key(state.runtime_scene_cache(), &cache_key)
        {
            return Some(model);
        }
    }

    let render_command_bundle =
        resolve_native_panel_render_command_bundle_for_state_bridge_and_snapshot_with_input(
            state, snapshot, input,
        );
    let scene = render_command_bundle
        .as_ref()
        .map(|bundle| bundle.scene.clone())
        .or_else(|| {
            resolve_native_panel_scene_for_state_bridge_and_snapshot_with_input(
                state, snapshot, input,
            )
        })?;
    Some(resolve_native_panel_presentation_model_for_scene(
        &scene,
        render_command_bundle.as_ref(),
    ))
}

pub(crate) fn resolve_current_native_panel_presentation_model_for_state_bridge_with_input<S>(
    state: &S,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<NativePanelPresentationModel>
where
    S: NativePanelRuntimeSceneStateBridge,
{
    state.runtime_scene_current_snapshot().and_then(|snapshot| {
        resolve_native_panel_presentation_model_for_state_bridge_and_snapshot_with_input(
            state, snapshot, input,
        )
    })
}

pub(crate) fn resolve_and_cache_native_panel_presentation_for_state_bridge_with_input<S>(
    state: &mut S,
    input: &NativePanelRuntimeInputDescriptor,
    layout: PanelLayout,
    render_state: PanelRenderState,
    pointer_region_input: Option<NativePanelPointerRegionInput>,
) -> Option<NativePanelResolvedPresentation>
where
    S: NativePanelRuntimeSceneMutableStateBridge,
{
    let scene = resolve_native_panel_scene_for_state_bridge_with_input(state, input)?;
    let runtime =
        resolve_native_panel_runtime_render_state_for_state_bridge_with_input(state, input);
    let resolved = resolve_native_panel_presentation(
        layout,
        &scene,
        runtime,
        render_state,
        pointer_region_input,
    );
    cache_render_command_bundle_for_state_bridge_with_input(state, input, &resolved.bundle);
    Some(resolved)
}

pub(crate) fn resolve_native_panel_snapshot_render_plan_for_state_bridge_snapshot_with_input<S>(
    state: &S,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelSnapshotRenderPlan
where
    S: NativePanelRuntimeSceneStateBridge,
{
    let render_command_bundle =
        resolve_native_panel_render_command_bundle_for_state_bridge_and_snapshot_with_input(
            state, snapshot, input,
        );
    let scene = render_command_bundle
        .as_ref()
        .map(|bundle| bundle.scene.clone())
        .or_else(|| {
            resolve_native_panel_scene_for_state_bridge_and_snapshot_with_input(
                state, snapshot, input,
            )
        })
        .expect("scene");
    resolve_native_panel_snapshot_render_plan_for_scene(scene, render_command_bundle)
}

pub(crate) fn cache_runtime_scene(
    cache: &mut NativePanelRuntimeSceneCache,
    snapshot: RuntimeSnapshot,
    scene: PanelScene,
    runtime_render_state: PanelRuntimeRenderState,
) {
    cache_runtime_scene_with_key(cache, snapshot, None, scene, runtime_render_state);
}

pub(crate) fn cache_runtime_scene_with_key(
    cache: &mut NativePanelRuntimeSceneCache,
    snapshot: RuntimeSnapshot,
    cache_key: Option<NativePanelRuntimeSceneCacheKey>,
    scene: PanelScene,
    runtime_render_state: PanelRuntimeRenderState,
) {
    cache.last_snapshot = Some(snapshot);
    cache_scene_runtime_with_key(cache, cache_key, scene, runtime_render_state);
}

pub(crate) fn cache_scene_runtime(
    cache: &mut NativePanelRuntimeSceneCache,
    scene: PanelScene,
    runtime_render_state: PanelRuntimeRenderState,
) {
    cache_scene_runtime_with_key(cache, None, scene, runtime_render_state);
}

pub(crate) fn cache_scene_runtime_with_key(
    cache: &mut NativePanelRuntimeSceneCache,
    cache_key: Option<NativePanelRuntimeSceneCacheKey>,
    scene: PanelScene,
    runtime_render_state: PanelRuntimeRenderState,
) {
    cache.last_cache_key = cache_key;
    cache.last_scene = Some(scene);
    cache.last_runtime_render_state = Some(runtime_render_state);
    cache.last_render_command_bundle = None;
}

pub(crate) fn cache_render_command_bundle(
    cache: &mut NativePanelRuntimeSceneCache,
    bundle: &NativePanelRenderCommandBundle,
) {
    cache_render_command_bundle_with_key(cache, None, bundle);
}

pub(crate) fn cache_render_command_bundle_with_key(
    cache: &mut NativePanelRuntimeSceneCache,
    cache_key: Option<NativePanelRuntimeSceneCacheKey>,
    bundle: &NativePanelRenderCommandBundle,
) {
    cache_scene_runtime_with_key(cache, cache_key, bundle.scene.clone(), bundle.runtime);
    cache.last_render_command_bundle = Some(bundle.clone());
}

pub(crate) fn cache_render_command_bundle_for_state_bridge_with_input<S>(
    state: &mut S,
    input: &NativePanelRuntimeInputDescriptor,
    bundle: &NativePanelRenderCommandBundle,
) where
    S: NativePanelRuntimeSceneMutableStateBridge,
{
    let cache_key = native_panel_runtime_scene_cache_key_for_state_bridge(state, input);
    cache_render_command_bundle_with_key(state.runtime_scene_cache_mut(), Some(cache_key), bundle);
    *state.runtime_pointer_regions_mut() = bundle.pointer_regions.clone();
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

pub(crate) fn cached_scene_for_key(
    cache: &NativePanelRuntimeSceneCache,
    cache_key: &NativePanelRuntimeSceneCacheKey,
) -> Option<PanelScene> {
    (cache.last_cache_key.as_ref() == Some(cache_key))
        .then(|| cached_scene(cache))
        .flatten()
}

pub(crate) fn cached_runtime_render_state_for_key(
    cache: &NativePanelRuntimeSceneCache,
    cache_key: &NativePanelRuntimeSceneCacheKey,
) -> Option<PanelRuntimeRenderState> {
    (cache.last_cache_key.as_ref() == Some(cache_key))
        .then(|| cached_runtime_render_state(cache))
        .flatten()
}

pub(crate) fn cached_render_command_bundle_for_key(
    cache: &NativePanelRuntimeSceneCache,
    cache_key: &NativePanelRuntimeSceneCacheKey,
) -> Option<NativePanelRenderCommandBundle> {
    (cache.last_cache_key.as_ref() == Some(cache_key))
        .then(|| cache.last_render_command_bundle.clone())
        .flatten()
}

pub(crate) fn cached_presentation_model(
    cache: &NativePanelRuntimeSceneCache,
) -> Option<NativePanelPresentationModel> {
    cache.last_scene.as_ref().map(|scene| {
        resolve_native_panel_presentation_model_for_scene(
            scene,
            cache.last_render_command_bundle.as_ref(),
        )
    })
}

pub(crate) fn cached_presentation_model_for_key(
    cache: &NativePanelRuntimeSceneCache,
    cache_key: &NativePanelRuntimeSceneCacheKey,
) -> Option<NativePanelPresentationModel> {
    (cache.last_cache_key.as_ref() == Some(cache_key))
        .then(|| cached_presentation_model(cache))
        .flatten()
}

pub(crate) fn apply_runtime_scene_bundle_to_host<H: NativePanelSceneHost>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    bundle: PanelRuntimeSceneBundle,
    preferred_display_index: usize,
    screen_frame: Option<PanelRect>,
) -> Result<(), H::Error> {
    host.sync_scene_with_payload(
        &bundle.scene,
        bundle.runtime_render_state,
        native_panel_host_display_reposition(preferred_display_index, screen_frame),
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
    let Some(snapshot) = cache.last_snapshot.clone() else {
        return Ok(false);
    };
    let bundle = rebuild_bundle(&snapshot);
    host.sync_scene_with_payload(
        &bundle.scene,
        bundle.runtime_render_state,
        native_panel_host_display_reposition_from_input_descriptor(input),
    )?;
    cache_runtime_scene(
        cache,
        bundle.displayed_snapshot,
        bundle.scene,
        bundle.runtime_render_state,
    );
    Ok(true)
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
    if transition.is_some() {
        let _ = rerender_runtime_scene_cache_to_host_with_input_descriptor(
            host,
            cache,
            input,
            rebuild_bundle,
        )?;
    }
    Ok(transition)
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelRuntimeSceneCache, NativePanelRuntimeSceneMutableStateBridge,
        NativePanelRuntimeSceneStateBridge, apply_runtime_scene_bundle_to_host,
        build_native_panel_scene_for_state_bridge_with_input, cache_render_command_bundle,
        cache_render_command_bundle_for_state_bridge_with_input,
        cache_render_command_bundle_with_key, cache_runtime_scene, cache_scene_runtime_with_key,
        cached_render_command_bundle_for_key, cached_runtime_render_state, cached_scene,
        cached_scene_for_key, native_panel_runtime_scene_cache_key,
        native_panel_runtime_scene_cache_key_for_state_bridge,
        rerender_runtime_scene_cache_to_host, rerender_runtime_scene_cache_to_host_on_transition,
        resolve_and_cache_native_panel_presentation_for_state_bridge_with_input,
        resolve_current_native_panel_presentation_model_for_state_bridge_with_input,
        resolve_current_native_panel_render_command_bundle_for_state_bridge_with_input,
        resolve_native_panel_presentation_model_for_state_bridge_and_snapshot_with_input,
        resolve_native_panel_runtime_render_state_for_state_bridge_with_input,
        resolve_native_panel_scene_for_state_bridge_and_snapshot_with_input,
        resolve_native_panel_snapshot_render_plan_for_state_bridge_snapshot_with_input,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelRect, PanelSettingsState, PanelState},
        native_panel_renderer::{
            descriptors::{
                NativePanelHostWindowDescriptor, NativePanelHostWindowState,
                NativePanelPointerRegion, NativePanelPointerRegionInput,
                NativePanelRuntimeInputDescriptor,
            },
            render_commands::{
                NativePanelRenderCommandBundle, resolve_native_panel_render_command_bundle,
            },
            runtime_interaction::NativePanelCoreStateBridge,
            traits::{NativePanelHost, NativePanelRenderer, NativePanelSceneHost},
        },
        native_panel_scene::{
            CompactBarScene, PanelRuntimeRenderState, PanelRuntimeSceneBundle, PanelScene,
            PanelSceneBuildInput, SceneMascotPose, SceneText, SessionSurfaceScene,
            StatusSurfaceDefaultState, StatusSurfaceDisplayMode, StatusSurfaceQueueState,
            StatusSurfaceScene, SurfaceScene, build_panel_scene, build_settings_surface_scene,
            surface_scene_mode,
        },
    };
    use echoisland_runtime::RuntimeSnapshot;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestTransition {
        Changed,
    }

    #[derive(Clone, Default)]
    struct TestRuntimeStateBridge {
        panel_state: PanelState,
        last_snapshot: Option<RuntimeSnapshot>,
        scene_cache: NativePanelRuntimeSceneCache,
        pointer_regions: Vec<NativePanelPointerRegion>,
    }

    impl NativePanelCoreStateBridge for TestRuntimeStateBridge {
        fn snapshot_core_panel_state(&self) -> PanelState {
            self.panel_state.clone()
        }

        fn apply_core_panel_state(&mut self, core: PanelState) {
            self.panel_state = core;
        }
    }

    impl NativePanelRuntimeSceneStateBridge for TestRuntimeStateBridge {
        fn runtime_scene_cache(&self) -> &NativePanelRuntimeSceneCache {
            &self.scene_cache
        }

        fn runtime_scene_current_snapshot(&self) -> Option<&RuntimeSnapshot> {
            self.last_snapshot.as_ref()
        }
    }

    impl NativePanelRuntimeSceneMutableStateBridge for TestRuntimeStateBridge {
        fn runtime_scene_cache_mut(&mut self) -> &mut NativePanelRuntimeSceneCache {
            &mut self.scene_cache
        }

        fn runtime_pointer_regions_mut(&mut self) -> &mut Vec<NativePanelPointerRegion> {
            &mut self.pointer_regions
        }
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

    #[test]
    fn keyed_render_command_bundle_lookup_requires_matching_state() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let stale_key = native_panel_runtime_scene_cache_key(&PanelState::default(), &input);
        let current_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Settings,
                ..PanelState::default()
            },
            &input,
        );
        let mut cache = NativePanelRuntimeSceneCache::default();
        let bundle = test_render_command_bundle("bundle", true);

        cache_render_command_bundle_with_key(&mut cache, Some(stale_key.clone()), &bundle);

        assert!(cached_render_command_bundle_for_key(&cache, &current_key).is_none());
        assert!(cached_render_command_bundle_for_key(&cache, &stale_key).is_some());
    }

    #[test]
    fn keyed_scene_lookup_requires_matching_state() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let stale_key = native_panel_runtime_scene_cache_key(&PanelState::default(), &input);
        let current_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Settings,
                ..PanelState::default()
            },
            &input,
        );
        let mut cache = NativePanelRuntimeSceneCache::default();
        let bundle = test_bundle("scene");

        cache_scene_runtime_with_key(
            &mut cache,
            Some(stale_key.clone()),
            bundle.scene,
            bundle.runtime_render_state,
        );

        assert!(cached_scene_for_key(&cache, &current_key).is_none());
        assert!(cached_scene_for_key(&cache, &stale_key).is_some());
    }

    #[test]
    fn cache_key_distinguishes_default_and_settings_surface() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let default_key = native_panel_runtime_scene_cache_key(&PanelState::default(), &input);
        let settings_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Settings,
                ..PanelState::default()
            },
            &input,
        );

        assert_ne!(default_key, settings_key);
    }

    #[test]
    fn cache_key_distinguishes_status_and_settings_surface() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let status_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Status,
                ..PanelState::default()
            },
            &input,
        );
        let settings_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Settings,
                ..PanelState::default()
            },
            &input,
        );

        assert_ne!(status_key, settings_key);
    }

    #[test]
    fn cache_key_distinguishes_settings_and_default_surface() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let settings_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Settings,
                ..PanelState::default()
            },
            &input,
        );
        let default_key = native_panel_runtime_scene_cache_key(
            &PanelState {
                expanded: true,
                surface_mode: ExpandedSurface::Default,
                ..PanelState::default()
            },
            &input,
        );

        assert_ne!(settings_key, default_key);
    }

    #[test]
    fn state_bridge_helpers_reuse_cached_scene_bundle_and_presentation() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let snapshot = RuntimeSnapshot {
            status: "current".to_string(),
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
        let bundle = test_render_command_bundle("bundle", true);
        let panel_state = PanelState {
            expanded: true,
            ..PanelState::default()
        };
        let cache_key = native_panel_runtime_scene_cache_key(&panel_state, &input);
        let mut state = TestRuntimeStateBridge {
            panel_state,
            last_snapshot: Some(snapshot.clone()),
            scene_cache: NativePanelRuntimeSceneCache {
                last_snapshot: Some(snapshot.clone()),
                ..NativePanelRuntimeSceneCache::default()
            },
            ..Default::default()
        };
        cache_render_command_bundle_with_key(&mut state.scene_cache, Some(cache_key), &bundle);

        let scene = resolve_native_panel_scene_for_state_bridge_and_snapshot_with_input(
            &state, &snapshot, &input,
        )
        .expect("scene");
        let presentation =
            resolve_native_panel_presentation_model_for_state_bridge_and_snapshot_with_input(
                &state, &snapshot, &input,
            )
            .expect("presentation");
        let plan = resolve_native_panel_snapshot_render_plan_for_state_bridge_snapshot_with_input(
            &state, &snapshot, &input,
        );

        assert_eq!(scene.compact_bar.headline.text, "bundle");
        assert_eq!(presentation.compact_bar.headline.text, "bundle");
        assert_eq!(
            plan.compact_bar_command(bundle.layout.pill_frame)
                .headline
                .text,
            "bundle"
        );
    }

    #[test]
    fn state_bridge_helpers_rebuild_from_current_snapshot_when_cache_misses() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let snapshot = RuntimeSnapshot {
            status: "rebuilt".to_string(),
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
        let state = TestRuntimeStateBridge {
            panel_state: PanelState::default(),
            last_snapshot: Some(snapshot.clone()),
            scene_cache: NativePanelRuntimeSceneCache::default(),
            ..Default::default()
        };

        let bridge_key = native_panel_runtime_scene_cache_key_for_state_bridge(&state, &input);
        let rebuilt_scene =
            build_native_panel_scene_for_state_bridge_with_input(&state, &snapshot, &input);
        let rebuilt_runtime =
            resolve_native_panel_runtime_render_state_for_state_bridge_with_input(&state, &input);
        let direct_scene = build_panel_scene(&state.panel_state, &snapshot, &input.scene_input);

        assert_eq!(
            bridge_key,
            native_panel_runtime_scene_cache_key(&state.panel_state, &input)
        );
        assert_eq!(rebuilt_scene.surface, direct_scene.surface);
        assert_eq!(
            rebuilt_scene.compact_bar.headline.text,
            direct_scene.compact_bar.headline.text
        );
        assert!(!rebuilt_runtime.transitioning);
    }

    #[test]
    fn mutable_state_bridge_cache_helper_syncs_pointer_regions() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let panel_state = PanelState {
            expanded: true,
            ..PanelState::default()
        };
        let bundle = test_render_command_bundle("bundle", true);
        let mut state = TestRuntimeStateBridge {
            panel_state,
            ..Default::default()
        };

        cache_render_command_bundle_for_state_bridge_with_input(&mut state, &input, &bundle);

        assert_eq!(state.pointer_regions.len(), bundle.pointer_regions.len());
        assert_eq!(
            state
                .scene_cache
                .last_render_command_bundle
                .as_ref()
                .map(|cached| cached.compact_bar.headline.text.as_str()),
            Some("bundle")
        );
    }

    #[test]
    fn mutable_state_bridge_resolve_and_cache_presentation_syncs_bundle_and_regions() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let snapshot = RuntimeSnapshot {
            status: "resolved".to_string(),
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
        let mut state = TestRuntimeStateBridge {
            panel_state: PanelState {
                expanded: true,
                ..PanelState::default()
            },
            last_snapshot: Some(snapshot),
            ..Default::default()
        };
        let scene = build_native_panel_scene_for_state_bridge_with_input(
            &state,
            state.last_snapshot.as_ref().expect("snapshot"),
            &input,
        );
        let bundle = test_render_command_bundle(&scene.compact_bar.headline.text, false);

        let resolved = resolve_and_cache_native_panel_presentation_for_state_bridge_with_input(
            &mut state,
            &input,
            bundle.layout,
            bundle.render_state,
            Some(NativePanelPointerRegionInput::default()),
        )
        .expect("resolved presentation");

        assert_eq!(
            state.pointer_regions.len(),
            resolved.bundle.pointer_regions.len()
        );
        assert_eq!(
            state
                .scene_cache
                .last_render_command_bundle
                .as_ref()
                .map(|cached| cached.compact_bar.headline.text.as_str()),
            Some(bundle.compact_bar.headline.text.as_str())
        );
    }

    #[test]
    fn current_state_bridge_bundle_helper_uses_current_snapshot() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let snapshot = RuntimeSnapshot {
            status: "current".to_string(),
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
        let panel_state = PanelState {
            expanded: true,
            ..PanelState::default()
        };
        let cache_key = native_panel_runtime_scene_cache_key(&panel_state, &input);
        let bundle = test_render_command_bundle("bundle", true);
        let mut state = TestRuntimeStateBridge {
            panel_state,
            last_snapshot: Some(snapshot.clone()),
            scene_cache: NativePanelRuntimeSceneCache {
                last_snapshot: Some(snapshot),
                ..Default::default()
            },
            ..Default::default()
        };
        cache_render_command_bundle_with_key(&mut state.scene_cache, Some(cache_key), &bundle);

        let current_bundle =
            resolve_current_native_panel_render_command_bundle_for_state_bridge_with_input(
                &state, &input,
            )
            .expect("current bundle");

        assert_eq!(current_bundle.compact_bar.headline.text, "bundle");
    }

    #[test]
    fn current_state_bridge_presentation_helper_uses_current_snapshot() {
        let input = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let snapshot = RuntimeSnapshot {
            status: "current".to_string(),
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
        let panel_state = PanelState {
            expanded: true,
            ..PanelState::default()
        };
        let cache_key = native_panel_runtime_scene_cache_key(&panel_state, &input);
        let bundle = test_render_command_bundle("bundle", true);
        let mut state = TestRuntimeStateBridge {
            panel_state,
            last_snapshot: Some(snapshot.clone()),
            scene_cache: NativePanelRuntimeSceneCache {
                last_snapshot: Some(snapshot),
                ..Default::default()
            },
            ..Default::default()
        };
        cache_render_command_bundle_with_key(&mut state.scene_cache, Some(cache_key), &bundle);

        let presentation =
            resolve_current_native_panel_presentation_model_for_state_bridge_with_input(
                &state, &input,
            )
            .expect("current presentation");

        assert_eq!(presentation.compact_bar.headline.text, "bundle");
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
                    &[crate::native_panel_scene::fallback_panel_display_option()],
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
    ) -> NativePanelRenderCommandBundle {
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
