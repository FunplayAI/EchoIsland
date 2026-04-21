use crate::app_settings::current_app_settings;
use crate::display_settings::{DisplayOption, display_key_for_monitor, resolve_preferred_display_index};
use tauri::window::Color;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewWindow};

const COMPACT_WIDTH: f64 = 420.0;
const COMPACT_HEIGHT: f64 = 80.0;
const BAR_STAGE_WIDTH: f64 = 420.0;
const BAR_STAGE_HEIGHT: f64 = 80.0;
const COMPACT_HIT_HEIGHT: f64 = 40.0;
const PANEL_STAGE_WIDTH: f64 = 420.0;
const EXPANDED_VISIBLE_WIDTH: f64 = 364.0;
const EXPANDED_WIDTH: f64 = 784.0;
const EXPANDED_HEIGHT: f64 = 560.0;
const TOP_MARGIN: f64 = 0.0;
const COMPACT_HIT_WIDTH: f64 = 265.0;
const EXPANDED_HIT_WIDTH: f64 = EXPANDED_VISIBLE_WIDTH;

pub fn apply_island_window_mode<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    expanded: bool,
) -> tauri::Result<()> {
    let (width, height) = if expanded {
        (EXPANDED_WIDTH, EXPANDED_HEIGHT)
    } else {
        (COMPACT_WIDTH, COMPACT_HEIGHT)
    };

    apply_overlay_window_flags(window)?;
    apply_transparent_background(window)?;
    window.set_size(LogicalSize::new(width, height))?;
    position_island_window(window, width, expanded)?;
    apply_island_hit_region(window, IslandRegion::from_mode(expanded, height))
}

pub fn apply_island_bar_stage<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    apply_overlay_window_flags(window)?;
    apply_transparent_background(window)?;
    window.set_size(LogicalSize::new(BAR_STAGE_WIDTH, BAR_STAGE_HEIGHT))?;
    position_island_window(window, BAR_STAGE_WIDTH, false)?;
    apply_island_hit_region(window, IslandRegion::bar_stage())
}

pub fn apply_island_panel_stage<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    height: f64,
) -> tauri::Result<()> {
    apply_overlay_window_flags(window)?;
    apply_transparent_background(window)?;
    let panel_height = height.max(120.0);
    window.set_size(LogicalSize::new(PANEL_STAGE_WIDTH, panel_height))?;
    position_island_window(window, PANEL_STAGE_WIDTH, false)?;
    apply_island_hit_region(window, IslandRegion::expanded(panel_height))
}

pub fn show_main_window<R: tauri::Runtime>(
    app: &AppHandle<R>,
    main_window_label: &str,
) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(main_window_label) {
        apply_overlay_window_flags(&window)?;
        window.show()?;
        window.unminimize()?;
        refresh_overlay_topmost(&window)?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn reposition_main_window_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
    main_window_label: &str,
) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(main_window_label) else {
        return Ok(());
    };
    let size = window.inner_size()?;
    let scale = window.scale_factor()?;
    let width = size.width as f64 / scale;
    let height = size.height as f64 / scale;
    let expanded = width > COMPACT_WIDTH + 1.0 || height > COMPACT_HEIGHT + 1.0;
    position_island_window(&window, width, expanded)
}

fn apply_overlay_window_flags<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.set_decorations(false)?;
    window.set_shadow(false)?;
    window.set_resizable(false)?;
    window.set_skip_taskbar(true)?;
    refresh_overlay_topmost(window)?;
    Ok(())
}

fn refresh_overlay_topmost<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    #[cfg(windows)]
    {
        apply_native_topmost(window)?;
    }

    #[cfg(not(windows))]
    {
        window.set_always_on_top(false)?;
        window.set_always_on_top(true)?;
        apply_native_topmost(window)?;
    }

    Ok(())
}

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Clone, Copy)]
struct IslandRegion {
    hit_width: f64,
    hit_height: f64,
    compact_shoulders: bool,
}

impl IslandRegion {
    fn compact() -> Self {
        Self {
            hit_width: COMPACT_HIT_WIDTH,
            hit_height: COMPACT_HIT_HEIGHT,
            compact_shoulders: true,
        }
    }

    fn bar_stage() -> Self {
        Self {
            hit_width: BAR_STAGE_WIDTH,
            hit_height: BAR_STAGE_HEIGHT,
            compact_shoulders: false,
        }
    }

    fn expanded(height: f64) -> Self {
        Self {
            hit_width: EXPANDED_HIT_WIDTH,
            hit_height: height,
            compact_shoulders: false,
        }
    }

    fn from_mode(expanded: bool, height: f64) -> Self {
        if expanded {
            Self::expanded(height)
        } else {
            Self::compact()
        }
    }
}

#[cfg(windows)]
fn apply_island_hit_region<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    region: IslandRegion,
) -> tauri::Result<()> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, DeleteObject, RGN_OR, SetWindowRgn,
    };

    let hwnd = window.hwnd()?.0 as isize;
    let scale = window.scale_factor()?;
    let window_size = window.inner_size()?;
    let client_width = window_size.width as f64 / scale;
    let offset_x = ((client_width - region.hit_width) / 2.0).max(0.0);

    let left = logical_to_physical(offset_x, scale);
    let top = 0;
    let right = logical_to_physical(offset_x + region.hit_width, scale);
    let bottom = logical_to_physical(region.hit_height, scale);

    let hrgn = if region.compact_shoulders {
        let shoulder = logical_to_physical(6.0, scale).max(1);
        let main = unsafe { CreateRectRgn(left + shoulder, top, right - shoulder, bottom) };
        if main == null_mut() {
            return Ok(());
        }

        for row in 0..shoulder {
            let dy = shoulder - row;
            let threshold = ((shoulder * shoulder - dy * dy) as f64).sqrt().round() as i32;
            let shoulder_bottom = top + row + 1;

            let left_rgn = unsafe {
                CreateRectRgn(
                    left + threshold,
                    top + row,
                    left + shoulder,
                    shoulder_bottom,
                )
            };
            if left_rgn != null_mut() {
                unsafe {
                    let _ = CombineRgn(main, main, left_rgn, RGN_OR);
                    let _ = DeleteObject(left_rgn);
                }
            }

            let right_rgn = unsafe {
                CreateRectRgn(
                    right - shoulder,
                    top + row,
                    right - threshold,
                    shoulder_bottom,
                )
            };
            if right_rgn != null_mut() {
                unsafe {
                    let _ = CombineRgn(main, main, right_rgn, RGN_OR);
                    let _ = DeleteObject(right_rgn);
                }
            }
        }

        main
    } else {
        unsafe { CreateRectRgn(left, top, right, bottom) }
    };
    if hrgn == null_mut() {
        return Ok(());
    }

    let result = unsafe { SetWindowRgn(hwnd as _, hrgn, 1) };
    if result == 0 {
        unsafe {
            let _ = DeleteObject(hrgn);
        }
        return Ok(());
    }

    Ok(())
}

#[cfg(not(windows))]
fn apply_island_hit_region<R: tauri::Runtime>(
    _window: &WebviewWindow<R>,
    _region: IslandRegion,
) -> tauri::Result<()> {
    Ok(())
}

#[cfg(windows)]
fn logical_to_physical(value: f64, scale: f64) -> i32 {
    (value * scale).round() as i32
}

fn apply_transparent_background<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.set_background_color(Some(Color(0, 0, 0, 0)))
}

#[cfg(windows)]
fn apply_native_topmost<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    use std::io;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowPos,
    };

    let hwnd = window.hwnd()?.0 as isize;
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
    let ok = unsafe { SetWindowPos(hwnd as _, HWND_TOPMOST, 0, 0, 0, 0, flags) };
    if ok == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_native_topmost<R: tauri::Runtime>(_window: &WebviewWindow<R>) -> tauri::Result<()> {
    Ok(())
}

fn position_island_window<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
    expanded: bool,
) -> tauri::Result<()> {
    let settings = current_app_settings();
    let monitors = window.available_monitors()?;
    let displays = monitors
        .iter()
        .enumerate()
        .map(|(index, monitor)| DisplayOption {
            index,
            key: display_key_for_monitor(monitor),
            name: monitor
                .name()
                .cloned()
                .unwrap_or_else(|| format!("Display {}", index + 1)),
            width: monitor.size().width,
            height: monitor.size().height,
        })
        .collect::<Vec<_>>();
    let preferred_index =
        resolve_preferred_display_index(&displays, settings.preferred_display_key.as_deref());
    let monitor = monitors
        .into_iter()
        .nth(preferred_index)
        .or_else(|| window.primary_monitor().ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let scale = monitor.scale_factor();
        let monitor_x = monitor_position.x as f64 / scale;
        let monitor_y = monitor_position.y as f64 / scale;
        let monitor_width = monitor_size.width as f64 / scale;
        let x = monitor_x + ((monitor_width - width) / 2.0).max(0.0);
        let y = monitor_y + if expanded { 8.0 } else { TOP_MARGIN };
        window.set_position(LogicalPosition::new(x, y))?;
    } else {
        window.center()?;
    }

    Ok(())
}
