use chrono::{DateTime, Utc};
use echoisland_runtime::RuntimeSnapshot;

use crate::native_panel_core::PanelState;

use super::descriptors::NativePanelRuntimeInputDescriptor;
use super::runtime_backend::{
    NativePanelRuntimeSceneSyncResult, apply_runtime_scene_sync_result_to_host,
    sync_runtime_scene_bundle_from_input_descriptor,
};
use super::runtime_interaction::{
    NativePanelCoreStateBridge, NativePanelSceneRuntimeBridge,
    NativePanelSettingsSurfaceToggleResult,
};
use super::runtime_scene_cache::NativePanelRuntimeSceneCache;
use super::runtime_settings_surface::{
    mutate_native_panel_core_state,
    toggle_native_panel_settings_surface_transition_request_for_state,
};
use super::runtime_transition_slots::{
    apply_native_panel_runtime_scene_sync_result_with_transition_slot,
    apply_native_panel_settings_surface_toggle_result_slot,
};
use super::traits::{NativePanelHost, NativePanelSceneHost};

pub(crate) fn sync_runtime_scene_bundle_from_state_input<S>(
    state: &mut S,
    raw_snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
    now: DateTime<Utc>,
) -> NativePanelRuntimeSceneSyncResult
where
    S: NativePanelCoreStateBridge,
{
    let mut core = state.snapshot_core_panel_state();
    let sync_result =
        sync_runtime_scene_bundle_from_input_descriptor(&mut core, raw_snapshot, input, now);
    state.apply_core_panel_state(core);
    sync_result
}

pub(crate) fn sync_runtime_scene_bundle_for_runtime_with_input<R>(
    runtime: &mut R,
    raw_snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<(), <R::Host as NativePanelHost>::Error>
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|request_slot, host, cache, state| {
        let sync_result =
            sync_runtime_scene_bundle_from_state_input(state, raw_snapshot, input, Utc::now());
        apply_native_panel_runtime_scene_sync_result_with_transition_slot(
            request_slot,
            host,
            cache,
            sync_result,
            input,
        )
    })
}

pub(crate) fn sync_and_apply_runtime_scene_from_state_input_descriptor<S, H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    raw_snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
    now: DateTime<Utc>,
) -> Result<(), H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost,
{
    let sync_result = sync_runtime_scene_bundle_from_state_input(state, raw_snapshot, input, now);
    apply_runtime_scene_sync_result_to_host(host, cache, sync_result, input)?;
    Ok(())
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor<S, H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<bool, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost,
{
    let Some(snapshot) = cache.last_snapshot.clone() else {
        return Ok(false);
    };
    let sync_result =
        sync_runtime_scene_bundle_from_state_input(state, &snapshot, input, Utc::now());
    apply_runtime_scene_sync_result_to_host(host, cache, sync_result, input)?;
    Ok(true)
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_for_runtime_with_input_descriptor<R>(
    runtime: &mut R,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<bool, <R::Host as NativePanelHost>::Error>
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|_request_slot, host, cache, state| {
        rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor(
            host, cache, state, input,
        )
    })
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_on_transition_for_state_with_input_descriptor<
    S,
    H,
    T,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    transition: Option<T>,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<Option<T>, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost,
{
    if transition.is_some() {
        rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor(
            host, cache, state, input,
        )?;
    }
    Ok(transition)
}

pub(crate) fn mutate_panel_state_and_rerender_runtime_scene_sync_result_for_state_with_input_descriptor<
    S,
    H,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    input: &NativePanelRuntimeInputDescriptor,
    mutate: impl FnOnce(&mut PanelState) -> bool,
) -> Result<bool, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost,
{
    if !mutate_native_panel_core_state(state, mutate) {
        return Ok(false);
    }
    let _ = rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor(
        host, cache, state, input,
    )?;
    Ok(true)
}

pub(crate) fn toggle_native_panel_settings_surface_and_rerender_for_state_with_input_descriptor<
    S,
    H,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    state: &mut S,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<NativePanelSettingsSurfaceToggleResult, H::Error>
where
    S: NativePanelCoreStateBridge,
    H: NativePanelSceneHost,
{
    let result = toggle_native_panel_settings_surface_transition_request_for_state(state);
    if !result.changed {
        return Ok(result);
    }
    let _ = rerender_runtime_scene_sync_result_to_host_for_state_with_input_descriptor(
        host, cache, state, input,
    )?;
    Ok(result)
}

pub(crate) fn toggle_native_panel_settings_surface_and_rerender_for_runtime_with_input_descriptor<
    R,
>(
    runtime: &mut R,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<bool, <R::Host as NativePanelHost>::Error>
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|request_slot, host, cache, state| {
        let result =
            toggle_native_panel_settings_surface_and_rerender_for_state_with_input_descriptor(
                host, cache, state, input,
            )?;
        Ok(apply_native_panel_settings_surface_toggle_result_slot(
            request_slot,
            result,
        ))
    })
}
