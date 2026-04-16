#![cfg_attr(any(target_os = "windows", target_os = "macos"), allow(dead_code))]

use anyhow::Result;
use tracing::debug;

use crate::{
    platform::{CaptureMode, PlatformBackend, PlatformCapabilities},
    terminal_focus::{FocusOutcome, ForegroundTabInfo, SessionFocusTarget, SessionTabCache},
};

pub fn stub_platform_capabilities(platform: &str) -> PlatformCapabilities {
    PlatformCapabilities {
        platform: platform.to_string(),
        platform_backend: PlatformBackend::Stub,
        can_focus_terminal: false,
        can_bind_terminal_tab: false,
        can_live_capture: true,
        can_shape_window_region: false,
        supports_tray: false,
        capture_mode: CaptureMode::Fallback,
    }
}

pub fn focus_session_terminal(
    target: &SessionFocusTarget,
    cached_tab: Option<&SessionTabCache>,
) -> Result<FocusOutcome> {
    let _ = cached_tab;
    debug!(
        source = %target.source,
        project_name = ?target.project_name,
        cwd = ?target.cwd,
        "stub terminal focus backend returned unsupported result"
    );
    Ok(FocusOutcome {
        focused: false,
        selected_tab: None,
    })
}

pub fn foreground_session_terminal_tab() -> Result<Option<ForegroundTabInfo>> {
    debug!("stub terminal focus backend has no foreground terminal tab support");
    Ok(None)
}

pub fn unsupported_terminal_binding_message() -> String {
    "当前平台暂不支持终端标签页绑定。".to_string()
}
