#[cfg(target_os = "windows")]
use chrono::Utc;
use echoisland_runtime::RuntimeSnapshot;
use tracing::{info, warn};

use crate::{
    app_runtime::AppRuntime,
    terminal_focus::{
        SessionFocusTarget, focus_session_terminal as focus_session_terminal_impl,
        learn_newly_active_session_tabs, observe_foreground_terminal_tab,
    },
};

#[cfg(not(target_os = "windows"))]
use crate::platform_stub;
#[cfg(target_os = "windows")]
use crate::terminal_focus::{ObservedTab, foreground_session_terminal_tab};

pub struct TerminalFocusService<'a> {
    app_runtime: &'a AppRuntime,
}

impl<'a> TerminalFocusService<'a> {
    pub fn new(app_runtime: &'a AppRuntime) -> Self {
        Self { app_runtime }
    }

    pub async fn sync_snapshot_focus_bindings(
        &self,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        observe_foreground_terminal_tab(&self.app_runtime.recent_foreground_tab).await;
        if let Some((session_id, tab)) = learn_newly_active_session_tabs(
            snapshot,
            &self.app_runtime.session_observations,
            &self.app_runtime.recent_foreground_tab,
        )
        .await
        {
            self.app_runtime.upsert_focus_binding(session_id, tab).await;
        }
        Ok(())
    }

    pub async fn focus_session(&self, session_id: &str) -> Result<bool, String> {
        let session = self
            .app_runtime
            .runtime
            .session(session_id)
            .await
            .ok_or_else(|| format!("session not found: {session_id}"))?;
        let target = SessionFocusTarget {
            session_id: session_id.to_string(),
            source: session.source,
            project_name: session.project_name,
            cwd: session.cwd,
            terminal_app: session.terminal_app,
            terminal_bundle: session.terminal_bundle,
            host_app: session.host_app,
            window_title: session.window_title,
            tty: session.tty,
            terminal_pid: session.terminal_pid,
            cli_pid: session.cli_pid,
            iterm_session_id: session.iterm_session_id,
            kitty_window_id: session.kitty_window_id,
            tmux_env: session.tmux_env,
            tmux_pane: session.tmux_pane,
            tmux_client_tty: session.tmux_client_tty,
        };
        #[cfg(target_os = "macos")]
        let target = {
            let mut target = target;
            if let Some(inferred_metadata) = crate::terminal_focus::infer_terminal_metadata(&target)
            {
                if target.terminal_app.is_none() {
                    target.terminal_app = inferred_metadata.terminal_app.clone();
                }
                if target.terminal_bundle.is_none() {
                    target.terminal_bundle = inferred_metadata.terminal_bundle.clone();
                }
                if target
                    .tty
                    .as_deref()
                    .is_none_or(|value| !is_precise_terminal_tty(value))
                {
                    target.tty = inferred_metadata.tty.clone();
                }
                if target.cli_pid.is_none() {
                    target.cli_pid = inferred_metadata.cli_pid;
                }
                if inferred_metadata.pid.is_some() {
                    target.terminal_pid = inferred_metadata.pid;
                }
                if self
                    .app_runtime
                    .runtime
                    .merge_session_terminal_metadata(session_id, inferred_metadata)
                    .await
                {
                    info!(
                        session_id = %session_id,
                        terminal_app = ?target.terminal_app,
                        terminal_bundle = ?target.terminal_bundle,
                        tty = ?target.tty,
                        cli_pid = ?target.cli_pid,
                        "persisted inferred macOS terminal binding"
                    );
                }
            }
            target
        };
        let cached_tab = {
            let cache = self.app_runtime.focus_cache.lock().await;
            cache.get(session_id).cloned()
        };
        info!(
            session_id = %session_id,
            has_cached_tab = cached_tab.is_some(),
            project_name = ?target.project_name,
            cwd = ?target.cwd,
            terminal_app = ?target.terminal_app,
            terminal_bundle = ?target.terminal_bundle,
            host_app = ?target.host_app,
            tty = ?target.tty,
            terminal_pid = ?target.terminal_pid,
            cli_pid = ?target.cli_pid,
            "focus requested"
        );
        let outcome = focus_session_terminal_impl(&target, cached_tab.as_ref())
            .map_err(|error| error.to_string())?;
        info!(
            session_id = %session_id,
            focused = outcome.focused,
            selected_tab = outcome.selected_tab.is_some(),
            "focus backend completed"
        );

        if let Some(selected_tab) = outcome.selected_tab {
            info!(
                session_id = %session_id,
                terminal_pid = selected_tab.terminal_pid,
                runtime_id = %selected_tab.runtime_id,
                title = %selected_tab.title,
                "focus selected tab and updated cache"
            );
            self.app_runtime
                .upsert_focus_binding(session_id.to_string(), selected_tab)
                .await;
        } else if cached_tab.is_some() {
            info!(
                session_id = %session_id,
                "focus did not return selected tab; keeping existing cache for future retries"
            );
        }
        Ok(outcome.focused)
    }

    pub async fn bind_session(&self, session_id: &str) -> Result<String, String> {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = session_id;
            return Err(platform_stub::unsupported_terminal_binding_message());
        }

        #[cfg(target_os = "windows")]
        {
            self.app_runtime
                .runtime
                .session(session_id)
                .await
                .ok_or_else(|| format!("session not found: {session_id}"))?;

            let tab = foreground_session_terminal_tab()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "当前前台不是可绑定的 Windows Terminal 标签页".to_string())?;

            let cache_entry = tab.cache.clone();
            let title = cache_entry.title.clone();
            self.app_runtime
                .upsert_focus_binding(session_id.to_string(), cache_entry.clone())
                .await;
            {
                let mut recent = self.app_runtime.recent_foreground_tab.lock().await;
                *recent = Some(ObservedTab {
                    cache: cache_entry.clone(),
                    observed_at: Utc::now(),
                });
            }

            info!(
                session_id = %session_id,
                terminal_pid = cache_entry.terminal_pid,
                runtime_id = %cache_entry.runtime_id,
                title = %title,
                "manually bound foreground terminal tab"
            );

            Ok(title)
        }
    }
}

pub(crate) async fn focus_runtime_session(
    app_runtime: &AppRuntime,
    session_id: &str,
) -> Result<bool, String> {
    TerminalFocusService::new(app_runtime)
        .focus_session(session_id)
        .await
}

pub(crate) fn spawn_runtime_focus_session(app_runtime: AppRuntime, session_id: String) {
    tauri::async_runtime::spawn(async move {
        match focus_runtime_session(&app_runtime, &session_id).await {
            Ok(true) => {
                info!(session_id = %session_id, "native panel focused terminal session");
            }
            Ok(false) => {
                warn!(
                    session_id = %session_id,
                    "native panel did not find a focusable terminal target"
                );
            }
            Err(error) => {
                warn!(
                    session_id = %session_id,
                    error = %error,
                    "native panel failed to focus terminal session"
                );
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn is_precise_terminal_tty(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "/dev/tty" && trimmed != "/dev/console"
}
