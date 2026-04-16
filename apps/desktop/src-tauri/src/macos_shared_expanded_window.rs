#[cfg(target_os = "macos")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "macos")]
use echoisland_runtime::RuntimeSnapshot;

#[cfg(target_os = "macos")]
use objc2_app_kit::NSColor;

#[cfg(target_os = "macos")]
use objc2_app_kit::NSWindow;

#[cfg(target_os = "macos")]
use objc2_foundation::NSRect;

#[cfg(target_os = "macos")]
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use crate::constants::SHARED_EXPANDED_WINDOW_LABEL;

#[cfg(target_os = "macos")]
static SHARED_EXPANDED_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
#[cfg(target_os = "macos")]
static SHARED_EXPANDED_WINDOW_STATE: OnceLock<Mutex<SharedExpandedWindowState>> = OnceLock::new();

#[cfg(target_os = "macos")]
const SHARED_EXPANDED_WINDOW_LEVEL: isize = 27;

#[cfg(target_os = "macos")]
const SHARED_EXPANDED_BOOTSTRAP_SCRIPT: &str = r#"
window.__CODEISLAND_SHELL__ = "native";
window.__CODEISLAND_PLATFORM_BACKEND__ = "tauri-native";
"#;

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Default)]
struct SharedExpandedWindowState {
    visible: bool,
    interactive: bool,
    frame_x: f64,
    frame_y: f64,
    frame_width: f64,
    frame_height: f64,
}

#[cfg(target_os = "macos")]
pub fn shared_expanded_enabled() -> bool {
    matches!(
        std::env::var("CODEISLAND_MACOS_SHARED_EXPANDED").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

#[cfg(not(target_os = "macos"))]
pub fn shared_expanded_enabled() -> bool {
    false
}

#[cfg(target_os = "macos")]
pub fn create_shared_expanded_window(app: &AppHandle) -> Result<(), String> {
    if !shared_expanded_enabled() {
        return Ok(());
    }
    let _ = SHARED_EXPANDED_APP_HANDLE.set(app.clone());
    let _ = SHARED_EXPANDED_WINDOW_STATE.set(Mutex::new(SharedExpandedWindowState::default()));

    if app
        .get_webview_window(SHARED_EXPANDED_WINDOW_LABEL)
        .is_some()
    {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        SHARED_EXPANDED_WINDOW_LABEL,
        WebviewUrl::App("shared-expanded.html".into()),
    )
    .initialization_script(SHARED_EXPANDED_BOOTSTRAP_SCRIPT)
    .decorations(false)
    .transparent(true)
    .resizable(false)
    .focused(false)
    .visible(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .build()
    .map_err(|error| error.to_string())?;

    window
        .set_ignore_cursor_events(true)
        .map_err(|error| error.to_string())?;

    let ns_window_ptr = window.ns_window().map_err(|error| error.to_string())?;
    if !ns_window_ptr.is_null() {
        let ns_window = unsafe { &*(ns_window_ptr.cast::<NSWindow>()) };
        ns_window.setLevel(SHARED_EXPANDED_WINDOW_LEVEL);
        ns_window.setOpaque(false);
        ns_window.setHasShadow(false);
        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn hide_shared_expanded_window<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if !shared_expanded_enabled() {
        return Ok(());
    }
    if let Some(window) = app.get_webview_window(SHARED_EXPANDED_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|error| error.to_string())?;
        }
    }

    if let Some(state) = SHARED_EXPANDED_WINDOW_STATE.get() {
        if let Ok(mut state) = state.lock() {
            state.visible = false;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn sync_shared_expanded_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    if !shared_expanded_enabled() {
        return Ok(());
    }
    app.emit_to(
        SHARED_EXPANDED_WINDOW_LABEL,
        "shared-expanded-snapshot",
        snapshot,
    )
    .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn sync_shared_expanded_frame(
    frame: NSRect,
    visible: bool,
    interactive: bool,
) -> Result<(), String> {
    if !shared_expanded_enabled() {
        return Ok(());
    }
    let Some(app) = SHARED_EXPANDED_APP_HANDLE.get() else {
        return Ok(());
    };
    let Some(window) = app.get_webview_window(SHARED_EXPANDED_WINDOW_LABEL) else {
        return Ok(());
    };
    let state_mutex = SHARED_EXPANDED_WINDOW_STATE.get();

    if !visible || frame.size.width <= 1.0 || frame.size.height <= 1.0 {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|error| error.to_string())?;
        }
        if let Some(state_mutex) = state_mutex {
            if let Ok(mut state) = state_mutex.lock() {
                state.visible = false;
                state.interactive = false;
            }
        }
        return Ok(());
    }

    let ns_window_ptr = window.ns_window().map_err(|error| error.to_string())?;
    if ns_window_ptr.is_null() {
        return Ok(());
    }

    let ns_window = &*(ns_window_ptr.cast::<NSWindow>());
    let should_update_frame = state_mutex
        .and_then(|mutex| mutex.lock().ok())
        .map(|state| {
            !state.visible
                || (state.frame_x - frame.origin.x).abs() > 0.5
                || (state.frame_y - frame.origin.y).abs() > 0.5
                || (state.frame_width - frame.size.width).abs() > 0.5
                || (state.frame_height - frame.size.height).abs() > 0.5
        })
        .unwrap_or(true);
    let should_update_interactivity = state_mutex
        .and_then(|mutex| mutex.lock().ok())
        .map(|state| state.interactive != interactive)
        .unwrap_or(true);

    if should_update_frame {
        ns_window.setFrame_display(frame, true);
    }
    if should_update_interactivity {
        window
            .set_ignore_cursor_events(!interactive)
            .map_err(|error| error.to_string())?;
    }

    if !window.is_visible().unwrap_or(false) {
        window.show().map_err(|error| error.to_string())?;
        ns_window.orderFrontRegardless();
    }

    if let Some(state_mutex) = state_mutex {
        if let Ok(mut state) = state_mutex.lock() {
            state.visible = true;
            state.interactive = interactive;
            state.frame_x = frame.origin.x;
            state.frame_y = frame.origin.y;
            state.frame_width = frame.size.width;
            state.frame_height = frame.size.height;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn create_shared_expanded_window(_: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn hide_shared_expanded_window<R: tauri::Runtime>(
    _: &tauri::AppHandle<R>,
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn sync_shared_expanded_snapshot<R: tauri::Runtime>(
    _: &tauri::AppHandle<R>,
    _: &echoisland_runtime::RuntimeSnapshot,
) -> Result<(), String> {
    Ok(())
}
