use std::{sync::atomic::AtomicBool, time::Instant};

use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::{
    native_panel_core::{HoverTransition, PanelAnimationDescriptor, PanelPoint},
    native_panel_renderer::facade::{
        command::{
            NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
            NativePanelRuntimeDispatchMode,
            dispatch_drained_native_panel_platform_events_with_app_handle,
            run_native_panel_pointer_input_with_queued_command_dispatch,
            run_native_panel_runtime_with_queued_command_dispatch,
            spawn_native_panel_platform_event_dispatch_loop,
            spawn_native_panel_platform_loops_with_event_dispatch,
        },
        env::native_panel_enabled_from_env_value,
        host::{NativePanelHost, hide_main_webview_window_when_native_ui_enabled},
    },
    notification_sound::play_message_card_sound,
};

use super::runtime_entry::{
    drain_windows_native_panel_platform_events, spawn_platform_loops_internal,
    windows_native_panel_event_dispatch_notifier, with_windows_native_panel_runtime,
    with_windows_native_panel_runtime_input,
};

static WINDOWS_NATIVE_PANEL_EVENT_DISPATCH_LOOP_STARTED: AtomicBool = AtomicBool::new(false);

pub(crate) fn current_windows_native_panel_runtime_backend()
-> super::runtime_backend::WindowsNativePanelRuntimeBackendFacade {
    super::runtime_backend::current_windows_native_panel_runtime_backend()
}

pub(crate) fn native_ui_enabled() -> bool {
    windows_native_ui_enabled_by_default()
}

pub(crate) fn create_native_panel() -> Result<(), String> {
    with_windows_native_panel_runtime(|runtime| runtime.create_panel())
}

pub(crate) fn hide_main_webview_window<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    hide_main_webview_window_when_native_ui_enabled(app, native_ui_enabled)
}

pub(crate) fn spawn_platform_loops<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    spawn_native_panel_platform_loops_with_event_dispatch(
        app,
        spawn_platform_loops_internal,
        spawn_windows_native_panel_event_dispatch_loop,
    );
}

pub(crate) fn dispatch_queued_native_panel_platform_events<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
) -> Result<(), String> {
    dispatch_drained_native_panel_platform_events_with_app_handle(
        app,
        drain_windows_native_panel_platform_events,
        toggle_native_panel_settings_surface,
        NativePanelRuntimeDispatchMode::Immediate,
    )
}

fn spawn_windows_native_panel_event_dispatch_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    spawn_native_panel_platform_event_dispatch_loop(
        &WINDOWS_NATIVE_PANEL_EVENT_DISPATCH_LOOP_STARTED,
        app,
        windows_native_panel_event_dispatch_notifier(),
        dispatch_queued_native_panel_platform_events,
        "failed to dispatch Windows native panel event",
    );
}

pub(crate) fn update_native_panel_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    let sync = with_windows_native_panel_runtime_input(app, |runtime, input| {
        runtime.sync_snapshot_bundle(snapshot, input)
    })?;
    if sync.is_some_and(|sync| sync.reminder.play_sound) {
        play_message_card_sound();
    }
    Ok(())
}

pub(crate) fn hide_native_panel<R: tauri::Runtime>(_: &AppHandle<R>) -> Result<(), String> {
    with_windows_native_panel_runtime(|runtime| runtime.hide_panel())
}

pub(super) fn toggle_native_panel_settings_surface<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    with_windows_native_panel_runtime_input(app, |runtime, input| {
        runtime
            .toggle_settings_surface_with_input(input)
            .map(|_| ())
    })
}

pub(crate) fn refresh_native_panel_from_last_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    with_windows_native_panel_runtime(|runtime| {
        runtime.rerender_from_last_snapshot(app).map(|_| ())
    })
}

pub(crate) fn handle_native_panel_pointer_move<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    point: PanelPoint,
    now: Instant,
) -> Result<Option<HoverTransition>, String> {
    Ok(
        handle_windows_native_panel_pointer_input(app, NativePanelPointerInput::Move(point), now)?
            .into_hover_transition(),
    )
}

pub(crate) fn handle_native_panel_pointer_leave<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    now: Instant,
) -> Result<Option<HoverTransition>, String> {
    Ok(
        handle_windows_native_panel_pointer_input(app, NativePanelPointerInput::Leave, now)?
            .into_hover_transition(),
    )
}

pub(crate) fn handle_native_panel_pointer_click<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    point: PanelPoint,
) -> Result<Option<NativePanelPlatformEvent>, String> {
    Ok(handle_windows_native_panel_pointer_input(
        app,
        NativePanelPointerInput::Click(point),
        Instant::now(),
    )?
    .into_click_event())
}

pub(crate) fn handle_native_panel_window_message<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    message_id: u32,
    lparam: isize,
    now: Instant,
) -> Result<Option<NativePanelPointerInputOutcome>, String> {
    let outcome = run_native_panel_runtime_with_queued_command_dispatch(
        app,
        |handler| {
            with_windows_native_panel_runtime_input(app, |runtime, input| {
                runtime.handle_window_message_with_handler(message_id, lparam, now, input, handler)
            })
        },
        toggle_native_panel_settings_surface,
        NativePanelRuntimeDispatchMode::Immediate,
    )?;
    Ok(outcome)
}

pub(crate) fn reposition_native_panel_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    with_windows_native_panel_runtime_input(app, |runtime, input| {
        runtime.reposition_to_selected_display_with_input(input)
    })
}

pub(crate) fn set_shared_expanded_body_height<R: tauri::Runtime>(
    _: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    with_windows_native_panel_runtime(|runtime| {
        runtime.set_shared_expanded_body_height(body_height)
    })
}

pub(crate) fn apply_native_panel_animation_descriptor(
    descriptor: PanelAnimationDescriptor,
) -> Result<(), String> {
    with_windows_native_panel_runtime(|runtime| {
        runtime.host.apply_animation_descriptor(descriptor)?;
        runtime.last_animation_descriptor = Some(descriptor);
        Ok(())
    })
}

fn handle_windows_native_panel_pointer_input<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    input_event: NativePanelPointerInput,
    now: Instant,
) -> Result<NativePanelPointerInputOutcome, String> {
    run_native_panel_pointer_input_with_queued_command_dispatch(
        app,
        input_event,
        now,
        |input_event, now, handler| {
            with_windows_native_panel_runtime_input(app, |runtime, input| {
                runtime.handle_pointer_input_with_handler(input_event, now, input, handler)
            })
        },
        toggle_native_panel_settings_surface,
        NativePanelRuntimeDispatchMode::Immediate,
    )
}

#[cfg(windows)]
fn windows_native_ui_enabled_by_default() -> bool {
    windows_native_ui_enabled_from_env(true, std::env::var("ECHOISLAND_WINDOWS_NATIVE_UI").ok())
}

#[cfg(not(windows))]
fn windows_native_ui_enabled_by_default() -> bool {
    false
}

pub(super) fn windows_native_ui_enabled_from_env(
    default_enabled: bool,
    value: Option<String>,
) -> bool {
    native_panel_enabled_from_env_value(default_enabled, value)
}
