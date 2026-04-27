use crate::native_panel_core::HoverTransition;

use super::descriptors::NativePanelRuntimeInputDescriptor;
use super::host_runtime_facade::NativePanelRuntimeDispatchMode;
use super::runtime_backend::{
    NativePanelRuntimeSceneSyncResult, apply_runtime_scene_sync_result_to_host,
};
use super::runtime_interaction::{
    NativePanelHoverSyncResult, NativePanelSceneRuntimeBridge,
    NativePanelSettingsSurfaceToggleResult,
};
use super::runtime_scene_cache::NativePanelRuntimeSceneCache;
use super::traits::{NativePanelHost, NativePanelSceneHost};
use super::transition_controller::{
    NativePanelTransitionRequest, native_panel_transition_request_for_snapshot_sync,
};

pub(crate) fn dispatch_native_panel_transition_request_slot(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    request: NativePanelTransitionRequest,
    _: NativePanelRuntimeDispatchMode,
) {
    *request_slot = Some(request);
}

pub(crate) fn dispatch_optional_native_panel_transition_request_slot(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    request: Option<NativePanelTransitionRequest>,
    mode: NativePanelRuntimeDispatchMode,
) -> bool {
    match request {
        Some(request) => {
            dispatch_native_panel_transition_request_slot(request_slot, request, mode);
            true
        }
        None => {
            *request_slot = None;
            false
        }
    }
}

pub(crate) fn dispatch_immediate_native_panel_transition_request_slot(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    request: Option<NativePanelTransitionRequest>,
) -> bool {
    dispatch_optional_native_panel_transition_request_slot(
        request_slot,
        request,
        NativePanelRuntimeDispatchMode::Immediate,
    )
}

pub(crate) fn apply_native_panel_transition_result_slot<T>(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    request: Option<NativePanelTransitionRequest>,
    value: T,
) -> T {
    dispatch_immediate_native_panel_transition_request_slot(request_slot, request);
    value
}

pub(crate) fn apply_native_panel_runtime_scene_sync_result_with_transition_slot<H>(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    sync_result: NativePanelRuntimeSceneSyncResult,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<(), H::Error>
where
    H: NativePanelSceneHost,
{
    dispatch_immediate_native_panel_transition_request_slot(
        request_slot,
        native_panel_transition_request_for_snapshot_sync(&sync_result.snapshot_sync),
    );
    apply_runtime_scene_sync_result_to_host(host, cache, sync_result, input).map(|_| ())
}

pub(crate) fn apply_native_panel_settings_surface_toggle_result_slot(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    result: NativePanelSettingsSurfaceToggleResult,
) -> bool {
    apply_native_panel_transition_result_slot(
        request_slot,
        result.transition_request,
        result.changed,
    )
}

pub(crate) fn apply_native_panel_hover_sync_result_slot(
    request_slot: &mut Option<NativePanelTransitionRequest>,
    hover_sync: NativePanelHoverSyncResult,
) -> Option<HoverTransition> {
    apply_native_panel_transition_result_slot(
        request_slot,
        hover_sync.request,
        hover_sync.transition,
    )
}

pub(crate) fn apply_native_panel_runtime_scene_sync_result_for_runtime<R>(
    runtime: &mut R,
    sync_result: NativePanelRuntimeSceneSyncResult,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<(), <R::Host as NativePanelHost>::Error>
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|request_slot, host, cache, _state| {
        apply_native_panel_runtime_scene_sync_result_with_transition_slot(
            request_slot,
            host,
            cache,
            sync_result,
            input,
        )
    })
}

pub(crate) fn apply_native_panel_settings_surface_toggle_result_for_runtime<R>(
    runtime: &mut R,
    result: NativePanelSettingsSurfaceToggleResult,
) -> bool
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|request_slot, _host, _cache, _state| {
        apply_native_panel_settings_surface_toggle_result_slot(request_slot, result)
    })
}

pub(crate) fn apply_native_panel_hover_sync_result_for_runtime<R>(
    runtime: &mut R,
    hover_sync: NativePanelHoverSyncResult,
) -> Option<HoverTransition>
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|request_slot, _host, _cache, _state| {
        apply_native_panel_hover_sync_result_slot(request_slot, hover_sync)
    })
}

pub(crate) fn sync_native_panel_hover_and_refresh_for_runtime<R>(
    runtime: &mut R,
    resolve: impl FnOnce(
        &mut R::Host,
        &mut NativePanelRuntimeSceneCache,
        &mut R::State,
    ) -> Result<
        Option<NativePanelHoverSyncResult>,
        <R::Host as NativePanelHost>::Error,
    >,
) -> Result<Option<HoverTransition>, <R::Host as NativePanelHost>::Error>
where
    R: NativePanelSceneRuntimeBridge,
{
    runtime.with_runtime_scene_slots(|request_slot, host, cache, state| {
        let hover_sync = resolve(host, cache, state)?;
        Ok(hover_sync.and_then(|hover_sync| {
            apply_native_panel_hover_sync_result_slot(request_slot, hover_sync)
        }))
    })
}
