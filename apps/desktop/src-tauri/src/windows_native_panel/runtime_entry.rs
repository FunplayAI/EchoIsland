use tauri::AppHandle;

use crate::native_panel_renderer::facade::{
    command::NativePanelPlatformEvent, descriptor::NativePanelRuntimeInputDescriptor,
};

use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::Notify;

use super::{
    WindowsNativePanelRuntime,
    platform_loop::{
        ensure_windows_native_platform_loop_thread, platform_loop_thread_started,
        wake_windows_native_platform_loop,
    },
    runtime_input::windows_runtime_input_descriptor,
};

static WINDOWS_NATIVE_PANEL_RUNTIME: OnceLock<Mutex<WindowsNativePanelRuntime>> = OnceLock::new();
static WINDOWS_NATIVE_PANEL_EVENT_DISPATCH_NOTIFY: OnceLock<Arc<Notify>> = OnceLock::new();

pub(super) fn windows_native_panel_runtime() -> &'static Mutex<WindowsNativePanelRuntime> {
    WINDOWS_NATIVE_PANEL_RUNTIME.get_or_init(|| Mutex::new(WindowsNativePanelRuntime::default()))
}

pub(super) fn pump_windows_native_panel_runtime_once() -> Result<(), String> {
    let result = {
        let mut runtime = windows_native_panel_runtime()
            .lock()
            .map_err(|_| "windows native panel runtime poisoned".to_string())?;
        runtime.pump_platform_loop()
    };
    notify_windows_native_panel_event_dispatcher();
    result
}

pub(super) fn windows_native_panel_event_dispatch_notifier() -> Arc<Notify> {
    WINDOWS_NATIVE_PANEL_EVENT_DISPATCH_NOTIFY
        .get_or_init(|| Arc::new(Notify::new()))
        .clone()
}

pub(super) fn notify_windows_native_panel_event_dispatcher() {
    windows_native_panel_event_dispatch_notifier().notify_one();
}

pub(super) fn with_windows_native_panel_runtime<T>(
    f: impl FnOnce(&mut WindowsNativePanelRuntime) -> Result<T, String>,
) -> Result<T, String> {
    let value = {
        let mut runtime = windows_native_panel_runtime()
            .lock()
            .map_err(|_| "windows native panel runtime poisoned".to_string())?;
        f(&mut runtime)?
    };
    if platform_loop_thread_started() {
        wake_windows_native_platform_loop();
    } else {
        pump_windows_native_panel_runtime_once()?;
    }
    Ok(value)
}

pub(super) fn with_windows_native_panel_runtime_input<R, T>(
    app: &AppHandle<R>,
    f: impl FnOnce(
        &mut WindowsNativePanelRuntime,
        &NativePanelRuntimeInputDescriptor,
    ) -> Result<T, String>,
) -> Result<T, String>
where
    R: tauri::Runtime,
{
    let input = windows_runtime_input_descriptor(app);
    with_windows_native_panel_runtime(|runtime| f(runtime, &input))
}

pub(super) fn drain_windows_native_panel_platform_events()
-> Result<Vec<NativePanelPlatformEvent>, String> {
    let mut runtime = windows_native_panel_runtime()
        .lock()
        .map_err(|_| "windows native panel runtime poisoned".to_string())?;
    Ok(runtime.take_queued_platform_events())
}

pub(super) fn spawn_platform_loops_internal() {
    ensure_windows_native_platform_loop_thread(pump_windows_native_panel_runtime_once);
    let _ = with_windows_native_panel_runtime(|runtime| {
        runtime.host.record_platform_loop_spawn();
        Ok(())
    });
}
