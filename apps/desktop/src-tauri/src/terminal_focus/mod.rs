use anyhow::Result;
use tracing::warn;

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
use crate::platform_stub;

mod learning;
#[cfg(target_os = "macos")]
mod macos;
mod model;
mod policy;
#[cfg(target_os = "windows")]
mod windows;

pub use learning::{learn_newly_active_session_tabs, observe_foreground_terminal_tab};
#[cfg(target_os = "macos")]
pub use macos::infer_terminal_metadata;
pub use model::{
    FocusOutcome, ForegroundTabInfo, ObservedTab, SessionFocusTarget, SessionObservation,
    SessionTabCache,
};
pub use policy::is_active_status;
#[cfg(target_os = "windows")]
pub use policy::{cwd_leaf, focus_tokens, host_app_aliases, normalized_token, tab_focus_tokens};

pub fn focus_session_terminal(
    target: &SessionFocusTarget,
    cached_tab: Option<&SessionTabCache>,
) -> Result<FocusOutcome> {
    #[cfg(target_os = "macos")]
    {
        warn!(
            source = %target.source,
            cwd = ?target.cwd,
            terminal_app = ?target.terminal_app,
            host_app = ?target.host_app,
            "using macOS terminal focus backend"
        );
        return macos::focus_session_terminal(target, cached_tab);
    }

    #[cfg(target_os = "windows")]
    {
        warn!("using Windows terminal focus backend");
        return windows::focus_session_terminal(target, cached_tab);
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        warn!("using stub terminal focus backend");
        platform_stub::focus_session_terminal(target, cached_tab)
    }
}

#[cfg(target_os = "windows")]
pub fn foreground_session_terminal_tab() -> Result<Option<ForegroundTabInfo>> {
    #[cfg(target_os = "windows")]
    {
        return windows::foreground_windows_terminal_tab();
    }
}

#[cfg(target_os = "windows")]
pub fn foreground_session_terminal_tab_if_helper_running() -> Result<Option<ForegroundTabInfo>> {
    #[cfg(target_os = "windows")]
    {
        return windows::foreground_windows_terminal_tab_if_helper_running();
    }
}
