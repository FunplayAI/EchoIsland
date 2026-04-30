use std::{
    sync::{Condvar, Mutex, OnceLock},
    time::{Duration, Instant},
};

use crate::{
    native_panel_core::PanelPoint,
    native_panel_renderer::facade::{
        descriptor::{NativePanelHostWindowState, NativePanelPointerRegion},
        shell::{NativePanelHostShellCommandBackend, apply_native_panel_host_shell_command},
    },
};
#[cfg(all(windows, not(test)))]
use tracing::info;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowsNativePanelPointerPollingSample {
    pub(super) point: PanelPoint,
}

use super::hit_region::{WindowsNativePanelHitTest, resolve_windows_native_panel_hit_test};

use super::dpi::{WindowsDpiScale, WindowsPhysicalRect, resolve_windows_dpi_scale_for_window};

#[cfg(all(windows, not(test)))]
use super::hit_region::WindowsNativePanelHitTest as WindowsNativePanelPlatformHitTest;

#[cfg(all(windows, not(test)))]
use super::layered_window::apply_windows_layered_window_initial_attributes;
#[cfg(all(windows, not(test)))]
use super::paint_backend::{
    WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY,
    windows_native_panel_composition_mode_for_preferred_painter,
};
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
    pub(super) topmost_reassert_count: usize,
    pub(super) last_ignores_mouse_events: Option<bool>,
    pub(super) processed_window_message_count: usize,
    pub(super) last_window_message_id: Option<u32>,
    pub(super) paint_dispatch_count: usize,
    pub(super) last_painted_job: Option<WindowsNativePanelShellPaintJob>,
    pub(super) last_paint_plan: Option<WindowsNativePanelPaintPlan>,
    pub(super) last_physical_window_rect: Option<WindowsPhysicalRect>,
    pub(super) last_surface_dpi_scale: Option<WindowsDpiScale>,
    pub(super) surface_resource_revision: u64,
    pub(super) last_paint_surface_resource_revision: Option<u64>,
    pub(super) paint_surface_resource_rebuild_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsNativePanelQueuedWindowMessage {
    pub(super) hwnd: isize,
    pub(super) message_id: u32,
    pub(super) lparam: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsNativeWindowPositioningBehavior {
    pub(super) topmost: bool,
    pub(super) no_activate: bool,
    pub(super) preserve_existing_z_order: bool,
}

pub(super) fn windows_native_window_positioning_behavior() -> WindowsNativeWindowPositioningBehavior
{
    WindowsNativeWindowPositioningBehavior {
        topmost: true,
        no_activate: true,
        preserve_existing_z_order: false,
    }
}

static WINDOWS_NATIVE_PANEL_WINDOW_MESSAGE_QUEUE: OnceLock<
    Mutex<Vec<WindowsNativePanelQueuedWindowMessage>>,
> = OnceLock::new();

static WINDOWS_NATIVE_PANEL_HIT_REGION_CACHE: OnceLock<
    Mutex<Vec<(isize, Vec<NativePanelPointerRegion>)>>,
> = OnceLock::new();

fn windows_native_panel_window_message_queue()
-> &'static Mutex<Vec<WindowsNativePanelQueuedWindowMessage>> {
    WINDOWS_NATIVE_PANEL_WINDOW_MESSAGE_QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

fn windows_native_panel_hit_region_cache()
-> &'static Mutex<Vec<(isize, Vec<NativePanelPointerRegion>)>> {
    WINDOWS_NATIVE_PANEL_HIT_REGION_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) fn sync_windows_native_panel_hit_regions(
    raw_window_handle: Option<isize>,
    regions: &[NativePanelPointerRegion],
) {
    let Some(raw_window_handle) = raw_window_handle else {
        return;
    };
    let Ok(mut cache) = windows_native_panel_hit_region_cache().lock() else {
        return;
    };
    if let Some((_, cached_regions)) = cache
        .iter_mut()
        .find(|(handle, _)| *handle == raw_window_handle)
    {
        *cached_regions = regions.to_vec();
        return;
    }
    cache.push((raw_window_handle, regions.to_vec()));
}

pub(super) fn clear_windows_native_panel_hit_regions(raw_window_handle: Option<isize>) {
    let Some(raw_window_handle) = raw_window_handle else {
        return;
    };
    let Ok(mut cache) = windows_native_panel_hit_region_cache().lock() else {
        return;
    };
    cache.retain(|(handle, _)| *handle != raw_window_handle);
}

pub(super) fn resolve_windows_native_panel_cached_hit_test(
    raw_window_handle: isize,
    point: PanelPoint,
) -> WindowsNativePanelHitTest {
    let Ok(cache) = windows_native_panel_hit_region_cache().lock() else {
        return WindowsNativePanelHitTest::Transparent;
    };
    let Some((_, regions)) = cache
        .iter()
        .find(|(handle, _)| *handle == raw_window_handle)
    else {
        return WindowsNativePanelHitTest::Transparent;
    };
    resolve_windows_native_panel_hit_test(regions, point)
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

#[cfg(test)]
pub(super) fn clear_windows_native_panel_window_messages(raw_window_handle: Option<isize>) {
    let _ = take_windows_native_panel_window_messages(raw_window_handle);
}

#[cfg(all(windows, not(test)))]
extern "system" fn native_panel_window_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::{
        Graphics::Gdi::{BeginPaint, EndPaint},
        UI::WindowsAndMessaging::{
            DefWindowProcW, HTCLIENT, HTTRANSPARENT, IDC_ARROW, LoadCursorW, SetCursor,
            WM_NCHITTEST, WM_SETCURSOR,
        },
    };

    match msg {
        WM_SETCURSOR => unsafe {
            let cursor = LoadCursorW(std::ptr::null_mut(), IDC_ARROW);
            if !cursor.is_null() {
                SetCursor(cursor);
            }
            1
        },
        WM_NCHITTEST => {
            let point = panel_point_from_screen_lparam_for_window(hwnd, lparam);
            match resolve_windows_native_panel_cached_hit_test(hwnd as isize, point) {
                WindowsNativePanelPlatformHitTest::Client => HTCLIENT as isize,
                WindowsNativePanelPlatformHitTest::Transparent => HTTRANSPARENT as isize,
            }
        }
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
            let lparam = match msg {
                super::window_shell::WINDOWS_WM_MOUSEMOVE
                | super::window_shell::WINDOWS_WM_LBUTTONUP => {
                    logical_client_lparam_for_window(hwnd, lparam)
                }
                _ => lparam,
            };
            queue_windows_native_panel_window_message(hwnd as isize, msg, lparam);
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(all(windows, not(test)))]
fn logical_client_lparam_for_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    lparam: isize,
) -> isize {
    let physical_x = (lparam as u32 & 0xFFFF) as u16 as i16 as i32;
    let physical_y = ((lparam as u32 >> 16) & 0xFFFF) as u16 as i16 as i32;
    let point = resolve_windows_dpi_scale_for_window(Some(hwnd as isize))
        .point_to_logical(physical_x, physical_y);
    pack_windows_client_lparam(point.x.round() as i32, point.y.round() as i32)
}

#[cfg(all(windows, not(test)))]
fn pack_windows_client_lparam(x: i32, y: i32) -> isize {
    let low = (x as i16 as u16) as u32;
    let high = ((y as i16 as u16) as u32) << 16;
    (low | high) as isize
}

#[cfg(all(windows, not(test)))]
fn panel_point_from_screen_lparam_for_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    lparam: isize,
) -> PanelPoint {
    use windows_sys::Win32::{Foundation::POINT, Graphics::Gdi::ScreenToClient};

    let mut point = POINT {
        x: (lparam as u32 & 0xFFFF) as u16 as i16 as i32,
        y: ((lparam as u32 >> 16) & 0xFFFF) as u16 as i16 as i32,
    };
    unsafe {
        let _ = ScreenToClient(hwnd, &mut point);
    }
    resolve_windows_dpi_scale_for_window(Some(hwnd as isize)).point_to_logical(point.x, point.y)
}

#[cfg(all(windows, not(test)))]
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

    pub(super) fn sync_surface_resource_rect(
        &mut self,
        physical_rect: Option<WindowsPhysicalRect>,
    ) {
        self.sync_surface_resource_state(physical_rect, WindowsDpiScale::default());
    }

    pub(super) fn sync_surface_resource_state(
        &mut self,
        physical_rect: Option<WindowsPhysicalRect>,
        dpi_scale: WindowsDpiScale,
    ) {
        if self.last_physical_window_rect == physical_rect
            && self.last_surface_dpi_scale == Some(dpi_scale)
        {
            return;
        }
        self.last_physical_window_rect = physical_rect;
        self.last_surface_dpi_scale = Some(dpi_scale);
        self.surface_resource_revision = self.surface_resource_revision.saturating_add(1);
    }

    pub(super) fn sync_paint_surface_resources_for_current_revision(&mut self) {
        if self.last_paint_surface_resource_revision == Some(self.surface_resource_revision) {
            return;
        }
        self.last_paint_surface_resource_revision = Some(self.surface_resource_revision);
        self.paint_surface_resource_rebuild_count += 1;
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
        clear_windows_native_panel_hit_regions(raw_window_handle);
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
        let surface_state =
            resolve_windows_native_window_surface_state(raw_window_handle, window_state);
        apply_windows_native_window_state(raw_window_handle, window_state)?;
        self.sync_surface_resource_state(surface_state.physical_rect, surface_state.dpi_scale);
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
        apply_windows_native_window_topmost(raw_window_handle)?;
        if raw_window_handle.is_some() {
            self.topmost_reassert_count += 1;
        }
        apply_windows_native_window_redraw(raw_window_handle)?;
        self.redraw_request_count += 1;
        Ok(())
    }

    fn record_shell_command_applied(&mut self, raw_window_handle: Option<Self::RawWindowHandle>) {
        self.last_raw_window_handle = raw_window_handle;
        self.applied_command_count += 1;
    }
}

#[cfg(all(windows, not(test)))]
fn apply_windows_native_window_topmost(raw_window_handle: Option<isize>) -> Result<(), String> {
    use std::io;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowPos,
    };

    let Some(hwnd) = raw_window_handle else {
        return Ok(());
    };
    let ok = unsafe {
        SetWindowPos(
            hwnd as _,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER | SWP_NOACTIVATE,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[cfg(any(not(windows), test))]
fn apply_windows_native_window_topmost(_raw_window_handle: Option<isize>) -> Result<(), String> {
    Ok(())
}

#[cfg(all(windows, not(test)))]
fn apply_windows_native_window_create(
    raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    use std::{iter, ptr};
    use windows_sys::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, IDC_ARROW, LoadCursorW, RegisterClassW, WNDCLASSW, WS_EX_LAYERED,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
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
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
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
    apply_windows_layered_window_initial_attributes(
        hwnd,
        windows_native_panel_composition_mode_for_preferred_painter(),
        WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY,
    )?;
    Ok(Some(hwnd as isize))
}

#[cfg(any(not(windows), test))]
fn apply_windows_native_window_create(
    raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    use std::sync::atomic::{AtomicIsize, Ordering};

    static NEXT_FAKE_HWND: AtomicIsize = AtomicIsize::new(1);

    Ok(Some(raw_window_handle.unwrap_or_else(|| {
        NEXT_FAKE_HWND.fetch_add(1, Ordering::Relaxed)
    })))
}

#[cfg(all(windows, not(test)))]
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

#[cfg(any(not(windows), test))]
fn apply_windows_native_window_destroy(
    _raw_window_handle: Option<isize>,
) -> Result<Option<isize>, String> {
    Ok(None)
}

#[cfg(all(windows, not(test)))]
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

#[cfg(any(not(windows), test))]
fn apply_windows_native_window_visibility(
    _raw_window_handle: Option<isize>,
    _visible: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(all(windows, not(test)))]
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
    let frame = resolve_windows_dpi_scale_for_window(raw_window_handle).rect_to_physical(frame);
    let ok = unsafe {
        SetWindowPos(
            hwnd as _,
            HWND_TOPMOST,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
            SWP_NOOWNERZORDER | SWP_NOACTIVATE,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WindowsNativePanelSurfaceState {
    physical_rect: Option<WindowsPhysicalRect>,
    dpi_scale: WindowsDpiScale,
}

fn resolve_windows_native_window_surface_state(
    raw_window_handle: Option<isize>,
    window_state: NativePanelHostWindowState,
) -> WindowsNativePanelSurfaceState {
    let dpi_scale = resolve_windows_dpi_scale_for_window(raw_window_handle);
    WindowsNativePanelSurfaceState {
        physical_rect: window_state
            .frame
            .map(|frame| dpi_scale.rect_to_physical(frame)),
        dpi_scale,
    }
}

#[cfg(any(not(windows), test))]
fn apply_windows_native_window_state(
    _raw_window_handle: Option<isize>,
    _window_state: NativePanelHostWindowState,
) -> Result<(), String> {
    Ok(())
}

#[cfg(all(windows, not(test)))]
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

#[cfg(any(not(windows), test))]
fn apply_windows_native_window_redraw(_raw_window_handle: Option<isize>) -> Result<(), String> {
    Ok(())
}

#[cfg(all(windows, not(test)))]
fn apply_windows_native_mouse_event_passthrough(
    raw_window_handle: Option<isize>,
    ignores_mouse_events: bool,
) -> Result<(), String> {
    let _ = (raw_window_handle, ignores_mouse_events);
    // Point-level WM_NCHITTEST owns passthrough. Toggling WS_EX_TRANSPARENT
    // would make the whole native panel ignore input, including the island.
    Ok(())
}

#[cfg(any(not(windows), test))]
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

#[cfg(all(windows, not(test)))]
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
    #[cfg(all(windows, not(test)))]
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
        #[cfg(all(windows, not(test)))]
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
        #[cfg(any(not(windows), test))]
        controller.condvar.notify_one();
    }
}

#[derive(Debug, Default)]
struct WindowsNativePanelDelayedWakeState {
    thread_started: bool,
    deadline: Option<Instant>,
}

#[derive(Debug, Default)]
struct WindowsNativePanelDelayedWakeController {
    state: Mutex<WindowsNativePanelDelayedWakeState>,
    condvar: Condvar,
}

static WINDOWS_NATIVE_PANEL_DELAYED_WAKE_CONTROLLER: OnceLock<
    WindowsNativePanelDelayedWakeController,
> = OnceLock::new();

fn windows_native_panel_delayed_wake_controller() -> &'static WindowsNativePanelDelayedWakeController
{
    WINDOWS_NATIVE_PANEL_DELAYED_WAKE_CONTROLLER
        .get_or_init(WindowsNativePanelDelayedWakeController::default)
}

pub(super) fn schedule_windows_native_platform_loop_wake(delay_ms: u64) {
    let controller = windows_native_panel_delayed_wake_controller();
    let mut state = match controller.state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    let deadline = Instant::now() + Duration::from_millis(delay_ms);
    state.deadline = Some(match state.deadline {
        Some(current) => current.min(deadline),
        None => deadline,
    });
    if !state.thread_started {
        state.thread_started = true;
        std::thread::spawn(run_windows_native_platform_loop_delayed_wake_thread);
    }
    controller.condvar.notify_one();
}

#[cfg(all(windows, not(test)))]
pub(super) fn current_windows_native_panel_pointer_polling_sample(
    raw_window_handle: Option<isize>,
) -> Option<WindowsNativePanelPointerPollingSample> {
    use windows_sys::Win32::{
        Foundation::POINT, Graphics::Gdi::ScreenToClient, UI::WindowsAndMessaging::GetCursorPos,
    };

    let hwnd = raw_window_handle?;
    let mut point = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok == 0 {
        return None;
    }
    unsafe {
        let _ = ScreenToClient(hwnd as _, &mut point);
    }
    Some(WindowsNativePanelPointerPollingSample {
        point: resolve_windows_dpi_scale_for_window(raw_window_handle)
            .point_to_logical(point.x, point.y),
    })
}

#[cfg(any(not(windows), test))]
pub(super) fn current_windows_native_panel_pointer_polling_sample(
    _raw_window_handle: Option<isize>,
) -> Option<WindowsNativePanelPointerPollingSample> {
    None
}

fn run_windows_native_platform_loop_delayed_wake_thread() {
    loop {
        let controller = windows_native_panel_delayed_wake_controller();
        let mut state = match controller.state.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        while state.deadline.is_none() {
            state = match controller.condvar.wait(state) {
                Ok(guard) => guard,
                Err(_) => return,
            };
        }

        let deadline = state.deadline.expect("checked deadline");
        let now = Instant::now();
        if now < deadline {
            let timeout = deadline.saturating_duration_since(now);
            let waited = controller.condvar.wait_timeout(state, timeout);
            match waited {
                Ok((next_state, _)) => {
                    drop(next_state);
                    continue;
                }
                Err(_) => return,
            }
        }

        state.deadline = None;
        drop(state);
        wake_windows_native_platform_loop();
    }
}

#[cfg(all(windows, not(test)))]
fn run_windows_native_platform_loop_thread(pump_runtime_once: fn() -> Result<(), String>) {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::{
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{
            DispatchMessageW, MSG, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_NOREMOVE,
            PM_REMOVE, PeekMessageW, QS_ALLINPUT, TranslateMessage, WM_QUIT,
        },
    };

    const WAIT_TIMEOUT_STATUS: u32 = 258;

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
    if windows_native_hover_probe_enabled() {
        info!("windows native platform loop thread started");
    }

    loop {
        loop {
            let mut message = unsafe { std::mem::zeroed::<MSG>() };
            let has_message = unsafe { PeekMessageW(&mut message, 0 as _, 0, 0, PM_REMOVE) };
            if has_message == 0 {
                break;
            }
            if message.message == WM_QUIT {
                return;
            }
            if message.message == WINDOWS_NATIVE_PANEL_LOOP_WAKE_MESSAGE {
                pump_windows_native_platform_loop_and_notify(controller, pump_runtime_once);
                continue;
            }

            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            pump_windows_native_platform_loop_and_notify(controller, pump_runtime_once);
        }

        let wait_result = unsafe {
            MsgWaitForMultipleObjectsEx(
                0,
                std::ptr::null(),
                crate::native_panel_core::HOVER_POLL_MS as u32,
                QS_ALLINPUT,
                MWMO_INPUTAVAILABLE,
            )
        };
        if wait_result == WAIT_TIMEOUT_STATUS {
            pump_windows_native_platform_loop_and_notify(controller, pump_runtime_once);
        }
    }
}

#[cfg(all(windows, not(test)))]
fn windows_native_hover_probe_enabled() -> bool {
    std::env::var("ECHOISLAND_WINDOWS_HOVER_PROBE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

#[cfg(all(windows, not(test)))]
fn pump_windows_native_platform_loop_and_notify(
    controller: &WindowsNativePanelPlatformLoopController,
    pump_runtime_once: fn() -> Result<(), String>,
) {
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
    }
}

#[cfg(any(not(windows), test))]
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
