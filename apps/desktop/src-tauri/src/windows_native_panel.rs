#![allow(dead_code)]

// Windows owns only the host boundary: Win32 window/message/pump/paint details.
// Scene, layout, interaction, transition requests, and runtime commands must flow
// through native_panel_renderer::facade so macOS and Windows consume the same model.

use crate::native_panel_core::PanelRect;

mod d2d_painter;
mod direct2d;
mod directwrite;
mod dpi;
mod draw_presenter;
mod facade;
mod hit_region;
mod host_runtime;
mod host_window;
mod layered_window;
mod message_dispatch;
mod paint_backend;
mod paint_bridge;
mod platform_loop;
mod renderer;
pub(crate) mod runtime_backend;
mod runtime_entry;
mod runtime_input;
mod runtime_traits;
mod window_shell;

#[allow(unused_imports)]
pub(crate) use facade::{
    apply_native_panel_animation_descriptor, create_native_panel,
    current_windows_native_panel_runtime_backend, dispatch_queued_native_panel_platform_events,
    handle_native_panel_pointer_click, handle_native_panel_pointer_leave,
    handle_native_panel_pointer_move, handle_native_panel_window_message, hide_main_webview_window,
    hide_native_panel, native_ui_enabled, refresh_native_panel_from_last_snapshot,
    reposition_native_panel_to_selected_display, set_shared_expanded_body_height,
    spawn_platform_loops, update_native_panel_snapshot,
};
#[cfg(test)]
pub(crate) use host_runtime::WindowsNativePanelHost;
pub(crate) use host_runtime::WindowsNativePanelRuntime;
#[cfg(test)]
use host_window::WindowsNativePanelDrawFrame;
pub(crate) use renderer::WindowsNativePanelRenderer;
#[cfg(target_os = "windows")]
pub(crate) use runtime_backend::WindowsNativePanelRuntimeBackendFacade;

#[cfg(test)]
use platform_loop::{
    clear_windows_native_panel_window_messages, queue_windows_native_panel_window_message,
    wait_windows_native_platform_loop_processed_at_least, windows_native_platform_loop_generations,
};
#[cfg(test)]
use window_shell::WINDOWS_WM_PAINT;

const WINDOWS_FALLBACK_PANEL_SCREEN_FRAME: PanelRect = PanelRect {
    x: 0.0,
    y: 0.0,
    width: 1440.0,
    height: 900.0,
};

#[cfg(test)]
mod tests;
