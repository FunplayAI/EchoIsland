use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use super::panel_refs::{native_panel_handles, native_panel_state};
use super::panel_transition_entry::{
    begin_native_panel_surface_transition, begin_native_panel_transition,
};
use super::panel_types::{NativePanelHandles, NativePanelRenderPayload};
use crate::native_panel_renderer::facade::{
    command::NativePanelRuntimeDispatchMode,
    runtime::{
        dispatch_native_panel_runtime_payload_with_handles,
        dispatch_native_panel_runtime_render_payload_if_available,
    },
    shell::dispatch_native_panel_on_platform_thread,
    transition::{
        NativePanelTransitionRequest, dispatch_native_panel_transition_request_or_fallback_via,
        dispatch_native_panel_transition_request_with_snapshot_via,
    },
};

fn native_panel_runtime_handles() -> Option<NativePanelHandles> {
    native_panel_handles()
}

fn current_native_panel_render_payload() -> Option<NativePanelRenderPayload> {
    native_panel_state().and_then(|state_mutex| {
        state_mutex
            .lock()
            .ok()
            .and_then(|state| state.render_payload())
    })
}

pub(super) fn dispatch_native_panel_render_payload<R: tauri::Runtime>(
    app: &AppHandle<R>,
    payload: NativePanelRenderPayload,
) -> Result<(), String> {
    dispatch_native_panel_runtime_payload_with_handles(
        app,
        native_panel_runtime_handles(),
        NativePanelRuntimeDispatchMode::Scheduled,
        payload,
        |app, handles, payload| {
            dispatch_native_panel_on_platform_thread(app, move || unsafe {
                crate::macos_native_test_panel::panel_snapshot::apply_native_panel_render_payload(
                    handles, payload,
                );
            })
        },
        |_app, handles, payload| unsafe {
            crate::macos_native_test_panel::panel_snapshot::apply_native_panel_render_payload(
                handles, payload,
            );
        },
    )
}

pub(super) fn refresh_native_panel_render_payload_from_runtime_state<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<bool, String> {
    dispatch_native_panel_runtime_render_payload_if_available(
        app,
        current_native_panel_render_payload(),
        dispatch_native_panel_render_payload,
    )
}

pub(super) fn dispatch_native_panel_transition_request_immediate_with_snapshot<
    R: tauri::Runtime,
>(
    app: AppHandle<R>,
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
) -> bool {
    dispatch_optional_native_panel_transition_request(
        &app,
        request,
        snapshot,
        NativePanelRuntimeDispatchMode::Immediate,
    )
    .unwrap_or(false)
}

pub(super) fn dispatch_native_panel_transition_request<R: tauri::Runtime>(
    app: &AppHandle<R>,
    request: NativePanelTransitionRequest,
    snapshot: RuntimeSnapshot,
    mode: NativePanelRuntimeDispatchMode,
) -> Result<(), String> {
    dispatch_native_panel_runtime_payload_with_handles(
        app,
        native_panel_runtime_handles(),
        mode,
        (request, snapshot),
        |app, handles, (request, snapshot)| {
            let app_for_transition = app.clone();
            dispatch_native_panel_on_platform_thread(app, move || unsafe {
                match request {
                    NativePanelTransitionRequest::Open => {
                        begin_native_panel_transition(
                            app_for_transition.clone(),
                            handles,
                            snapshot,
                            true,
                        );
                    }
                    NativePanelTransitionRequest::Close => {
                        begin_native_panel_transition(
                            app_for_transition.clone(),
                            handles,
                            snapshot,
                            false,
                        );
                    }
                    NativePanelTransitionRequest::SurfaceSwitch => {
                        begin_native_panel_surface_transition(
                            app_for_transition.clone(),
                            handles,
                            snapshot,
                        );
                    }
                }
            })
        },
        move |app, handles, (request, snapshot)| unsafe {
            match request {
                NativePanelTransitionRequest::Open => {
                    begin_native_panel_transition(app, handles, snapshot, true);
                }
                NativePanelTransitionRequest::Close => {
                    begin_native_panel_transition(app, handles, snapshot, false);
                }
                NativePanelTransitionRequest::SurfaceSwitch => {
                    begin_native_panel_surface_transition(app, handles, snapshot);
                }
            }
        },
    )
}

pub(super) fn dispatch_optional_native_panel_transition_request<R: tauri::Runtime>(
    app: &AppHandle<R>,
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
    mode: NativePanelRuntimeDispatchMode,
) -> Result<bool, String> {
    dispatch_native_panel_transition_request_with_snapshot_via(
        request,
        snapshot,
        |request, snapshot| dispatch_native_panel_transition_request(app, request, snapshot, mode),
    )
}

pub(super) fn dispatch_optional_native_panel_transition_request_or_fallback(
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
    dispatch: impl FnOnce(NativePanelTransitionRequest, RuntimeSnapshot) -> Result<(), String>,
    fallback: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    dispatch_native_panel_transition_request_or_fallback_via(request, snapshot, dispatch, fallback)
}

pub(super) fn dispatch_native_panel_transition_request_or_rerender<R: tauri::Runtime>(
    app: &AppHandle<R>,
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
) -> Result<(), String> {
    dispatch_optional_native_panel_transition_request_or_fallback(
        request,
        snapshot,
        |request, snapshot| {
            dispatch_native_panel_transition_request(
                app,
                request,
                snapshot,
                NativePanelRuntimeDispatchMode::Scheduled,
            )
        },
        || {
            let _ = refresh_native_panel_render_payload_from_runtime_state(app)?;
            Ok(())
        },
    )
}

pub(super) fn dispatch_native_panel_transition_request_or_apply_render_payload<
    R: tauri::Runtime,
>(
    app: &AppHandle<R>,
    request: Option<NativePanelTransitionRequest>,
    snapshot: Option<RuntimeSnapshot>,
    payload: NativePanelRenderPayload,
) -> Result<(), String> {
    dispatch_optional_native_panel_transition_request_or_fallback(
        request,
        snapshot,
        |request, snapshot| {
            dispatch_native_panel_transition_request(
                app,
                request,
                snapshot,
                NativePanelRuntimeDispatchMode::Scheduled,
            )
        },
        || dispatch_native_panel_render_payload(app, payload),
    )
}
