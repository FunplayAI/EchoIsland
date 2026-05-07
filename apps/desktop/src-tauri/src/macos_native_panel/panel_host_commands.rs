use objc2_app_kit::NSPanel;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use tauri::AppHandle;

use super::panel_geometry::centered_top_frame;
use super::panel_refs::{native_panel_handles, panel_from_ptr};
use super::panel_types::NativePanelHandles;
use crate::native_panel_core::PanelRect;
use crate::native_panel_renderer::facade::host::{
    NativePanelHostDisplayReposition, sync_runtime_host_display_reposition_in_state,
    sync_runtime_host_visibility_in_state,
};

pub(super) unsafe fn with_native_panel_window<T>(
    f: impl FnOnce(NativePanelHandles, &'static NSPanel) -> T,
) -> Option<T> {
    let handles = native_panel_handles()?;
    Some(f(handles, unsafe { panel_from_ptr(handles.panel) }))
}

pub(super) unsafe fn order_out_native_panel_window(
    sync_state: impl FnOnce() -> Option<()>,
) -> Option<()> {
    unsafe {
        with_native_panel_window(|_, panel| {
            let _ = sync_state();
            panel.orderOut(None);
        })
    }
}

pub(super) unsafe fn reposition_native_panel_window(
    reposition: NativePanelHostDisplayReposition,
    centered_frame: NSRect,
    sync_state: impl FnOnce(NativePanelHostDisplayReposition) -> Option<()>,
) -> Option<()> {
    unsafe {
        with_native_panel_window(|_, panel| {
            panel.setFrame_display(centered_frame, true);
            let _ = sync_state(reposition);
        })
    }
}

pub(super) unsafe fn current_native_panel_window_frame() -> Option<NSRect> {
    unsafe { with_native_panel_window(|_, panel| panel.frame()) }
}

pub(super) fn order_out_native_panel_window_with_app<R: tauri::Runtime>(
    app: &AppHandle<R>,
    sync_state: impl FnOnce() -> Option<()> + Send + 'static,
) -> Result<(), String> {
    app.run_on_main_thread(move || unsafe {
        let _ = order_out_native_panel_window(sync_state);
    })
    .map_err(|error| error.to_string())
}

pub(super) fn reposition_native_panel_window_with_app<R: tauri::Runtime>(
    app: &AppHandle<R>,
    reposition: NativePanelHostDisplayReposition,
    current_window_frame: impl FnOnce() -> Option<NSRect> + Send + 'static,
    sync_state: impl FnOnce(NativePanelHostDisplayReposition) -> Option<()> + Send + 'static,
) -> Result<(), String> {
    app.run_on_main_thread(move || unsafe {
        let Some(panel_frame) = current_window_frame() else {
            return;
        };
        let Some(screen_frame) = reposition.screen_frame else {
            return;
        };
        let frame = centered_top_frame(ns_rect_from_panel_rect(screen_frame), panel_frame.size);
        let _ = reposition_native_panel_window(reposition, frame, sync_state);
    })
    .map_err(|error| error.to_string())
}

pub(super) fn reposition_native_panel_window_resolving_on_main_with_app<R: tauri::Runtime>(
    app: &AppHandle<R>,
    resolve_reposition: impl FnOnce() -> Option<NativePanelHostDisplayReposition> + Send + 'static,
    current_window_frame: impl FnOnce() -> Option<NSRect> + Send + 'static,
    sync_state: impl FnOnce(NativePanelHostDisplayReposition) -> Option<()> + Send + 'static,
) -> Result<(), String> {
    app.run_on_main_thread(move || unsafe {
        let Some(reposition) = resolve_reposition() else {
            return;
        };
        let Some(panel_frame) = current_window_frame() else {
            return;
        };
        let Some(screen_frame) = reposition.screen_frame else {
            return;
        };
        let frame = centered_top_frame(ns_rect_from_panel_rect(screen_frame), panel_frame.size);
        let _ = reposition_native_panel_window(reposition, frame, sync_state);
    })
    .map_err(|error| error.to_string())
}

pub(super) fn sync_order_out_in_runtime_state(
    mut_state: impl FnOnce(&mut super::panel_types::NativePanelState) -> (),
) -> Option<()> {
    super::panel_host_runtime::with_native_runtime_panel_state_mut(|state| {
        mut_state(state);
        sync_runtime_host_visibility_in_state(state, false);
    })
}

pub(super) fn sync_reposition_in_runtime_state(
    reposition: NativePanelHostDisplayReposition,
    mut_state: impl FnOnce(&mut super::panel_types::NativePanelState) -> (),
) -> Option<()> {
    super::panel_host_runtime::with_native_runtime_panel_state_mut(|state| {
        mut_state(state);
        sync_runtime_host_visibility_in_state(state, true);
        sync_runtime_host_display_reposition_in_state(state, reposition);
    })
}

fn ns_rect_from_panel_rect(rect: PanelRect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, rect.y),
        NSSize::new(rect.width, rect.height),
    )
}
