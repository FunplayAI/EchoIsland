use super::{FocusOutcome, SessionFocusTarget, SessionTabCache, tab_focus_tokens};
use anyhow::Result;
use tracing::{debug, info, warn};

mod focus_ops;
mod matcher;
mod tab_helper;
mod window_enum;

use focus_ops::activate_window;
use matcher::select_window_candidate;
use tab_helper::select_windows_terminal_tab;
pub(crate) use tab_helper::{
    foreground_windows_terminal_tab, foreground_windows_terminal_tab_if_helper_running,
};
use window_enum::collect_windows;

pub fn focus_session_terminal(
    target: &SessionFocusTarget,
    cached_tab: Option<&SessionTabCache>,
) -> Result<FocusOutcome> {
    let windows = collect_windows();
    if windows.is_empty() {
        return Ok(FocusOutcome {
            focused: false,
            selected_tab: None,
        });
    }

    let tab_tokens = tab_focus_tokens(target);

    if let Some(cached_tab) = cached_tab {
        info!(
            terminal_pid = cached_tab.terminal_pid,
            window_hwnd = cached_tab.window_hwnd,
            runtime_id = %cached_tab.runtime_id,
            title = %cached_tab.title,
            "trying cached Windows Terminal tab first"
        );
        if let Some(selected_tab) = select_windows_terminal_tab(
            cached_tab.window_hwnd,
            cached_tab.terminal_pid,
            Some(cached_tab),
            &tab_tokens,
        )? {
            if let Some(window) = windows
                .iter()
                .find(|window| window.hwnd as isize as i64 == selected_tab.window_hwnd)
            {
                activate_window(window.hwnd)?;
                return Ok(FocusOutcome {
                    focused: true,
                    selected_tab: Some(selected_tab),
                });
            }
        }
        warn!(
            terminal_pid = cached_tab.terminal_pid,
            window_hwnd = cached_tab.window_hwnd,
            runtime_id = %cached_tab.runtime_id,
            title = %cached_tab.title,
            "cached Windows Terminal tab lookup failed; falling back to window matching"
        );
    }

    let (best, candidate_logs, host_aliases, token_count) =
        select_window_candidate(&windows, target);

    if !candidate_logs.is_empty() {
        debug!(
            candidates = ?candidate_logs,
            token_count,
            host_alias_count = host_aliases.len(),
            "window focus candidate scores"
        );
    }

    let Some(candidate) = best else {
        debug!(
            target_source = %target.source,
            host_alias_count = host_aliases.len(),
            "no window focus candidate matched"
        );
        return Ok(FocusOutcome {
            focused: false,
            selected_tab: None,
        });
    };

    if candidate.process_name == "windowsterminal" {
        info!(
            terminal_pid = candidate.pid,
            window_hwnd = candidate.hwnd as isize as i64,
            has_cached_tab = cached_tab.is_some(),
            token_count,
            tab_token_count = tab_tokens.len(),
            "trying Windows Terminal tab selection"
        );
        if let Some(selected_tab) = select_windows_terminal_tab(
            candidate.hwnd as isize as i64,
            candidate.pid,
            cached_tab,
            &tab_tokens,
        )? {
            if let Some(window) = windows
                .iter()
                .find(|window| window.hwnd as isize as i64 == selected_tab.window_hwnd)
            {
                activate_window(window.hwnd)?;
                return Ok(FocusOutcome {
                    focused: true,
                    selected_tab: Some(selected_tab),
                });
            }
        }
        warn!(
            terminal_pid = candidate.pid,
            "Windows Terminal tab selection returned no tab; falling back to window focus"
        );
    }

    activate_window(candidate.hwnd)?;
    Ok(FocusOutcome {
        focused: true,
        selected_tab: None,
    })
}
