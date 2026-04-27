use std::sync::{Condvar, Mutex, OnceLock};

use crate::native_panel_renderer::facade::{
    descriptor::NativePanelHostWindowState,
    shell::{NativePanelHostShellCommandBackend, apply_native_panel_host_shell_command},
};

#[cfg(windows)]
use super::paint_backend::WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY;
use super::{
    paint_backend::WindowsNativePanelPaintPlan,
    window_shell::{WindowsNativePanelShellCommand, WindowsNativePanelShellPaintJob},
};

#[derive(Clone, Debug, Default)]
pub(super) struct WindowsNativePanelPlatformLoopState {
    pub(super) applied_command_count: usize,
    pub(super) create_count: usize,
    pub(super) destroy_count: usize,
    pub(super) show_count: usize,
    pub(super) hide_count: usize,
    pub(super) last_raw_window_handle: Option<isize>,
    pub(super) last_window_state: Option<NativePanelHostWindowState>,
    pub(super) last_visible: Option<bool>,
    pub(super) redraw_request_count: usize,
    pub(super) last_ignores_mouse_events: Option<bool>,
    pub(super) processed_window_message_count: usize,
    pub(super) last_window_message_id: Option<u32>,
    pub(super) paint_dispatch_count: usize,
    pub(super) last_painted_job: Option<WindowsNativePanelShellPaintJob>,
    pub(super) last_paint_plan: Option<WindowsNativePanelPaintPlan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsNativePanelQueuedWindowMessage {
    pub(super) hwnd: isize,
    pub(super) message_id: u32,
    pub(super) lparam: isize,
}

static WINDOWS_NATIVE_PANEL_WINDOW_MESSAGE_QUEUE: OnceLock<
    Mutex<Vec<WindowsNativePanelQueuedWindowMessage>>,
> = OnceLock::new();

fn windows_native_panel_window_message_queue()
-> &'static Mutex<Vec<WindowsNativePanelQueuedWindowMessage>> {
    WINDOWS_NATIVE_PANEL_WINDOW_MESSAGE_QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) fn queue_windows_native_panel_window_message(
    hwnd: isize,
    message_id: u32,
    lparam: isize,
) {
    if let Ok(mut queue) = windows_native_panel_window_message_queue().lock() {
        queue.push(WindowsNativePanelQueuedWindowMessage {
            hwnd,
            message_id,
            lparam,
        });
    }
}

pub(super) fn take_windows_native_panel_window_messages(
    raw_window_handle: Option<isize>,
) -> Vec<WindowsNativePanelQueuedWindowMessage> {
    let Some(raw_window_handle) = raw_window_handle else {
        return Vec::new();
    };
    let Ok(mut queue) = windows_native_panel_window_message_queue().lock() else {
        return Vec::new();
    };
    let mut drained = Vec::new();
    let mut retained = Vec::with_capacity(queue.len());
    for message in queue.drain(..) {
        if message.hwnd == raw_window_handle {
            drained.push(message);
        } else {
            retained.push(message);
        }
    }
    *queue = retained;
    drained
}

#[cfg(windows)]
extern "system" fn native_panel_window_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::{
        Graphics::Gdi::{BeginPaint, EndPaint},
        UI::WindowsAndMessaging::DefWindowProcW,
    };

    match msg {
        super::window_shell::WINDOWS_WM_PAINT => unsafe {
            let mut paint = MaybeUninit::zeroed();
            let hdc = BeginPaint(hwnd, paint.as_mut_ptr());
            if !hdc.is_null() {
                EndPaint(hwnd, paint.as_ptr());
            }
            queue_windows_native_panel_window_message(hwnd as isize, msg, lparam);
            0
        },
        super::window_shell::WINDOWS_WM_MOUSEMOVE
        | super::window_shell::WINDOWS_WM_LBUTTONUP
        | super::window_shell::WINDOWS_WM_MOUSELEAVE => {
            if msg == super::window_shell::WINDOWS_WM_MOUSEMOVE {
                track_windows_native_panel_mouse_leave(hwnd);
            }
            queue_windows_native_panel_window_message(hwnd as isize, msg, lparam);
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(windows)]
fn track_windows_native_panel_mouse_leave(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
    };

    let mut event = TRACKMOUSEEVENT {
        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };
    unsafe {
        let _ = TrackMouseEvent(&mut event);
    }
}

impl WindowsNativePanelPlatformLoopState {
    pub(super) fn consume_shell_command(
        &mut self,
        raw_window_handle: &mut Option<isize>,
        command: WindowsNativePanelShellCommand,
    ) -> Result<(), String> {
        apply_native_panel_host_shell_command(self, raw_window_handle, command)
    }
}

impl NativePanelHostShellCommandBackend for WindowsNativePanelPlatformLoopState {
    type RawWindowHandle = isize;
    type Error = String;

    fn create_shell_window(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
    ) -> Result<Option<Self::RawWindowHandle>, Self::Error> {
        let raw_window_handle = apply_windows_native_window_create(raw_window_handle)?;
        self.create_count += 1;
        Ok(raw_window_handle)
    }

    fn destroy_shell_window(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
    ) -> Result<Option<Self::RawWindowHandle>, Self::Error> {
        let raw_window_handle = apply_windows_native_window_destroy(raw_window_handle)?;
        self.destroy_count += 1;
        self.last_visible = Some(false);
        Ok(raw_window_handle)
    }

    fn set_shell_window_visible(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
        visible: bool,
    ) -> Result<(), Self::Error> {
        apply_windows_native_window_visibility(raw_window_handle, visible)?;
        if visible {
            self.show_count += 1;
        } else {
            self.hide_count += 1;
        }
        self.last_visible = Some(visible);
        Ok(())
    }

    fn sync_shell_window_state(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
        window_state: NativePanelHostWindowState,
    ) -> Result<(), Self::Error> {
        apply_windows_native_window_state(raw_window_handle, window_state)?;
        self.last_window_state = Some(window_state);
        Ok(())
    }

    fn sync_shell_mouse_event_passthrough(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
        ignores_mouse_events: bool,
    ) -> Result<(), Self::Error> {
        apply_windows_native_mouse_event_passthrough(raw_window_handle, ignores_mouse_events)?;
        self.last_ignores_mouse_events = Some(ignores_mouse_events);
        Ok(())
    }

    fn request_shell_redraw(
        &mut self,
        raw_window_handle: Option<Self::RawWindowHandle>,
    ) -> Result<(), Self::Error> {
        apply_windows_native_window_redraw(raw_window_handle)?;
        self.redraw_request_count += 1;
        Ok(())
    }

    fn record_shell_command_applied(&mut self, raw_window_handle: Option<Self::RawWindowHandle>) {
        self.last_raw_window_handle = raw_window_handle;
        self.applied_command_count += 1;
    }
}

#[cfg(windows)]
fn apply_windows_native_window_create(
    raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    use std::{iter, ptr};
    use windows_sys::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, LWA_ALPHA, LWA_COLORKEY, RegisterClassW, SetLayeredWindowAttributes,
            WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    };

    if raw_window_handle.is_some() {
        return Ok(raw_window_handle);
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(iter::once(0)).collect()
    }

    static WINDOW_CLASS: OnceLock<Result<Vec<u16>, String>> = OnceLock::new();
    let class_name = WINDOW_CLASS.get_or_init(|| {
        let class_name = wide_null("EchoIslandNativePanelWindow");
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        let class = WNDCLASSW {
            lpfnWndProc: Some(native_panel_window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..unsafe { std::mem::zeroed() }
        };
        let atom = unsafe { RegisterClassW(&class) };
        if atom == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(class_name)
    });
    let class_name = class_name.as_ref().map_err(Clone::clone)?;
    let window_name = wide_null("EchoIsland Native Panel");
    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED | WS_EX_TOPMOST,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            0 as _,
            0 as _,
            instance,
            ptr::null_mut(),
        )
    };
    if hwnd.is_null() {
        return Err(std::io::Error::last_os_error().to_string());
    }
    unsafe {
        let _ = SetLayeredWindowAttributes(
            hwnd,
            WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY,
            255,
            LWA_ALPHA | LWA_COLORKEY,
        );
    }
    Ok(Some(hwnd as isize))
}

#[cfg(not(windows))]
fn apply_windows_native_window_create(
    raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    use std::sync::atomic::{AtomicIsize, Ordering};

    static NEXT_FAKE_HWND: AtomicIsize = AtomicIsize::new(1);

    Ok(Some(raw_window_handle.unwrap_or_else(|| {
        NEXT_FAKE_HWND.fetch_add(1, Ordering::Relaxed)
    })))
}

#[cfg(windows)]
fn apply_windows_native_window_destroy(
    raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow;

    let Some(hwnd) = raw_window_handle else {
        return Ok(None);
    };
    unsafe {
        let _ = DestroyWindow(hwnd as _);
    }
    Ok(None)
}

#[cfg(not(windows))]
fn apply_windows_native_window_destroy(
    _raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    Ok(None)
}

#[cfg(windows)]
fn apply_windows_native_window_visibility(
    raw_window_handle: Option<isize>,
    visible: bool,
) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNA, ShowWindow};

    let Some(hwnd) = raw_window_handle else {
        return Ok(());
    };
    unsafe {
        ShowWindow(hwnd as _, if visible { SW_SHOWNA } else { SW_HIDE });
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_windows_native_window_visibility(
    _raw_window_handle: Option<isize>,
    _visible: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn apply_windows_native_window_state(
    raw_window_handle: Option<isize>,
    window_state: NativePanelHostWindowState,
) -> Result<(), String> {
    use std::io;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SetWindowPos,
    };

    let Some(hwnd) = raw_window_handle else {
        return Ok(());
    };
    let Some(frame) = window_state.frame else {
        return Ok(());
    };
    let ok = unsafe {
        SetWindowPos(
            hwnd as _,
            HWND_TOPMOST,
            frame.x.round() as i32,
            frame.y.round() as i32,
            frame.width.round() as i32,
            frame.height.round() as i32,
            SWP_NOOWNERZORDER | SWP_NOACTIVATE,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_windows_native_window_state(
    _raw_window_handle: Option<isize>,
    _window_state: NativePanelHostWindowState,
) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn apply_windows_native_window_redraw(raw_window_handle: Option<isize>) -> Result<(), String> {
    use std::io;
    use windows_sys::Win32::Graphics::Gdi::{InvalidateRect, UpdateWindow};

    let Some(hwnd) = raw_window_handle else {
        return Ok(());
    };
    let ok = unsafe { InvalidateRect(hwnd as _, std::ptr::null(), 0) };
    if ok == 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    unsafe {
        let _ = UpdateWindow(hwnd as _);
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_windows_native_window_redraw(_raw_window_handle: Option<isize>) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn apply_windows_native_mouse_event_passthrough(
    raw_window_handle: Option<isize>,
    ignores_mouse_events: bool,
) -> Result<(), String> {
    use std::io;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos,
        WS_EX_TRANSPARENT,
    };

    let Some(hwnd) = raw_window_handle else {
        return Ok(());
    };
    let current_style = unsafe { GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE) } as u32;
    let next_style = if ignores_mouse_events {
        current_style | WS_EX_TRANSPARENT
    } else {
        current_style & !WS_EX_TRANSPARENT
    };
    if next_style == current_style {
        return Ok(());
    }

    unsafe {
        SetWindowLongPtrW(hwnd as _, GWL_EXSTYLE, next_style as isize);
    }
    let ok = unsafe {
        SetWindowPos(
            hwnd as _,
            0 as _,
            0,
            0,
            0,
            0,
            SWP_NOMOVE
                | SWP_NOSIZE
                | SWP_NOZORDER
                | SWP_NOOWNERZORDER
                | SWP_NOACTIVATE
                | SWP_FRAMECHANGED,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_windows_native_mouse_event_passthrough(
    _raw_window_handle: Option<isize>,
    _ignores_mouse_events: bool,
) -> Result<(), String> {
    Ok(())
}

#[derive(Debug, Default)]
struct WindowsNativePanelPlatformLoopThreadState {
    thread_started: bool,
    thread_id: Option<u32>,
    wake_generation: u64,
    processed_generation: u64,
}

#[derive(Debug, Default)]
struct WindowsNativePanelPlatformLoopController {
    state: Mutex<WindowsNativePanelPlatformLoopThreadState>,
    condvar: Condvar,
}

static WINDOWS_NATIVE_PANEL_PLATFORM_LOOP_CONTROLLER: OnceLock<
    WindowsNativePanelPlatformLoopController,
> = OnceLock::new();

fn windows_native_panel_platform_loop_controller()
-> &'static WindowsNativePanelPlatformLoopController {
    WINDOWS_NATIVE_PANEL_PLATFORM_LOOP_CONTROLLER
        .get_or_init(WindowsNativePanelPlatformLoopController::default)
}

#[cfg(windows)]
const WINDOWS_NATIVE_PANEL_LOOP_WAKE_MESSAGE: u32 = 0x8001;

pub(super) fn ensure_windows_native_platform_loop_thread(
    pump_runtime_once: fn() -> Result<(), String>,
) {
    let controller = windows_native_panel_platform_loop_controller();
    let mut state = match controller.state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    if state.thread_started {
        return;
    }
    state.thread_started = true;
    std::thread::spawn(move || run_windows_native_platform_loop_thread(pump_runtime_once));
    #[cfg(windows)]
    while state.thread_id.is_none() {
        state = match controller.condvar.wait(state) {
            Ok(guard) => guard,
            Err(_) => return,
        };
    }
}

pub(super) fn platform_loop_thread_started() -> bool {
    windows_native_panel_platform_loop_controller()
        .state
        .lock()
        .map(|state| state.thread_started)
        .unwrap_or(false)
}

pub(super) fn wake_windows_native_platform_loop() {
    let controller = windows_native_panel_platform_loop_controller();
    if let Ok(mut state) = controller.state.lock() {
        if !state.thread_started {
            return;
        }
        state.wake_generation += 1;
        #[cfg(windows)]
        if let Some(thread_id) = state.thread_id {
            unsafe {
                let _ = windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                    thread_id,
                    WINDOWS_NATIVE_PANEL_LOOP_WAKE_MESSAGE,
                    0,
                    0,
                );
            }
        }
        #[cfg(not(windows))]
        controller.condvar.notify_one();
    }
}

#[cfg(windows)]
fn run_windows_native_platform_loop_thread(pump_runtime_once: fn() -> Result<(), String>) {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::{
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, TranslateMessage,
        },
    };

    unsafe {
        let mut bootstrap = MaybeUninit::<MSG>::zeroed();
        let _ = PeekMessageW(bootstrap.as_mut_ptr(), 0 as _, 0, 0, PM_NOREMOVE);
    }

    let controller = windows_native_panel_platform_loop_controller();
    if let Ok(mut state) = controller.state.lock() {
        state.thread_id = Some(unsafe { GetCurrentThreadId() });
        controller.condvar.notify_all();
    } else {
        return;
    }

    loop {
        let mut message = unsafe { std::mem::zeroed::<MSG>() };
        let result = unsafe { GetMessageW(&mut message, 0 as _, 0, 0) };
        if result <= 0 {
            break;
        }

        if message.message == WINDOWS_NATIVE_PANEL_LOOP_WAKE_MESSAGE {
            let wake_generation = controller
                .state
                .lock()
                .ok()
                .map(|state| state.wake_generation)
                .unwrap_or(0);
            let _ = pump_runtime_once();
            if let Ok(mut state) = controller.state.lock() {
                state.processed_generation = wake_generation;
                controller.condvar.notify_all();
            } else {
                return;
            }
            continue;
        }

        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let _ = pump_runtime_once();
        if let Ok(mut state) = controller.state.lock() {
            state.processed_generation = state.wake_generation;
            controller.condvar.notify_all();
        } else {
            return;
        }
    }
}

#[cfg(not(windows))]
fn run_windows_native_platform_loop_thread(pump_runtime_once: fn() -> Result<(), String>) {
    loop {
        let wake_generation = {
            let controller = windows_native_panel_platform_loop_controller();
            let mut state = match controller.state.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            while state.wake_generation == state.processed_generation {
                state = match controller.condvar.wait(state) {
                    Ok(guard) => guard,
                    Err(_) => return,
                };
            }
            state.wake_generation
        };

        let _ = pump_runtime_once();

        let controller = windows_native_panel_platform_loop_controller();
        if let Ok(mut state) = controller.state.lock() {
            state.processed_generation = wake_generation;
            controller.condvar.notify_all();
        } else {
            return;
        }
    }
}

#[cfg(test)]
pub(super) fn windows_native_platform_loop_generations() -> Option<(u64, u64)> {
    windows_native_panel_platform_loop_controller()
        .state
        .lock()
        .ok()
        .map(|state| (state.wake_generation, state.processed_generation))
}

#[cfg(test)]
pub(super) fn wait_windows_native_platform_loop_processed_at_least(
    target_generation: u64,
    timeout_ms: u64,
) -> bool {
    use std::time::Duration;

    let controller = windows_native_panel_platform_loop_controller();
    let Ok(state) = controller.state.lock() else {
        return false;
    };
    let waited =
        controller
            .condvar
            .wait_timeout_while(state, Duration::from_millis(timeout_ms), |state| {
                state.processed_generation < target_generation
            });
    match waited {
        Ok((state, _)) => state.processed_generation >= target_generation,
        Err(_) => false,
    }
}
