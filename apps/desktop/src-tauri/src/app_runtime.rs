use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use echoisland_core::{EventEnvelope, EventMetadata, ResponseEnvelope};
use echoisland_ipc::{DEFAULT_ADDR, EventHandler, serve_tcp};
use echoisland_runtime::SharedRuntime;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::focus_store::{default_focus_bindings_path, load_focus_bindings, save_focus_bindings};
use crate::native_ui_refresh::maybe_refresh_native_ui_for_event;
use crate::terminal_focus::{ObservedTab, SessionFocusTarget, SessionObservation, SessionTabCache};

pub struct RuntimeEventHandler<R: tauri::Runtime> {
    app_handle: AppHandle<R>,
    app_runtime: AppRuntime,
}

#[async_trait]
impl<R: tauri::Runtime + 'static> EventHandler for RuntimeEventHandler<R> {
    async fn handle_event(&self, mut event: EventEnvelope) -> ResponseEnvelope {
        let normalized = event.normalized_event_name();
        if matches!(normalized.as_str(), "PermissionRequest" | "AskUserQuestion") {
            warn!(event_name = %normalized, "native pending event received");
        }
        maybe_enrich_event_terminal_metadata(&mut event);
        maybe_refresh_native_ui_for_event(
            self.app_handle.clone(),
            self.app_runtime.runtime.clone(),
            &normalized,
        );
        let response = self.app_runtime.runtime.handle_event(event).await;
        maybe_refresh_native_ui_for_event(
            self.app_handle.clone(),
            self.app_runtime.runtime.clone(),
            &normalized,
        );
        response
    }

    async fn handle_disconnect(&self, session_id: &str) {
        if self
            .app_runtime
            .runtime
            .handle_peer_disconnect(session_id)
            .await
        {
            maybe_refresh_native_ui_for_event(
                self.app_handle.clone(),
                self.app_runtime.runtime.clone(),
                "PeerDisconnect",
            );
        }
    }
}

#[derive(Clone)]
pub struct AppRuntime {
    pub(crate) runtime: Arc<SharedRuntime>,
    pub(crate) focus_cache: Arc<Mutex<HashMap<String, SessionTabCache>>>,
    pub(crate) session_observations: Arc<Mutex<HashMap<String, SessionObservation>>>,
    pub(crate) recent_foreground_tab: Arc<Mutex<Option<ObservedTab>>>,
    focus_cache_path: Arc<PathBuf>,
}

impl AppRuntime {
    pub fn new(runtime: Arc<SharedRuntime>) -> Self {
        Self::with_focus_cache_path(runtime, default_focus_bindings_path())
    }

    pub fn with_focus_cache_path(runtime: Arc<SharedRuntime>, focus_cache_path: PathBuf) -> Self {
        let focus_cache = match load_focus_bindings(&focus_cache_path) {
            Ok(bindings) => {
                if !bindings.is_empty() {
                    info!(
                        binding_count = bindings.len(),
                        path = %focus_cache_path.display(),
                        "restored persisted focus bindings"
                    );
                }
                bindings
            }
            Err(error) => {
                warn!(
                    path = %focus_cache_path.display(),
                    error = %error,
                    "failed to load persisted focus bindings"
                );
                HashMap::new()
            }
        };

        Self {
            runtime,
            focus_cache: Arc::new(Mutex::new(focus_cache)),
            session_observations: Arc::new(Mutex::new(HashMap::new())),
            recent_foreground_tab: Arc::new(Mutex::new(None)),
            focus_cache_path: Arc::new(focus_cache_path),
        }
    }

    pub async fn upsert_focus_binding(&self, session_id: String, tab: SessionTabCache) {
        let snapshot = {
            let mut focus_cache = self.focus_cache.lock().await;
            focus_cache.insert(session_id, tab);
            focus_cache.clone()
        };

        if let Err(error) = save_focus_bindings(self.focus_cache_path.as_ref(), &snapshot) {
            warn!(
                path = %self.focus_cache_path.display(),
                error = %error,
                "failed to persist focus bindings"
            );
        }
    }
}

pub fn spawn_ipc_server<R: tauri::Runtime + 'static>(
    app_handle: AppHandle<R>,
    app_runtime: AppRuntime,
) {
    let handler = Arc::new(RuntimeEventHandler {
        app_handle: app_handle.clone(),
        app_runtime,
    });
    tauri::async_runtime::spawn(async move {
        if let Err(error) = serve_tcp(DEFAULT_ADDR, handler).await {
            let _ = app_handle.emit("ipc-error", error.to_string());
        }
    });
}

#[cfg(target_os = "macos")]
fn maybe_enrich_event_terminal_metadata(event: &mut EventEnvelope) {
    let target = SessionFocusTarget {
        session_id: event.session_id.clone(),
        source: event.source.clone(),
        project_name: event.project_name(),
        cwd: event.cwd.clone(),
        terminal_app: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.terminal_app.clone()),
        terminal_bundle: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.terminal_bundle.clone()),
        host_app: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.host_app.clone()),
        window_title: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.window_title.clone()),
        tty: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.tty.clone()),
        terminal_pid: event.metadata.as_ref().and_then(|metadata| metadata.pid),
        cli_pid: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.cli_pid),
        iterm_session_id: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.iterm_session_id.clone()),
        kitty_window_id: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.kitty_window_id.clone()),
        tmux_env: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.tmux_env.clone()),
        tmux_pane: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.tmux_pane.clone()),
        tmux_client_tty: event
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.tmux_client_tty.clone()),
    };
    let Some(inferred) = crate::terminal_focus::infer_terminal_metadata(&target) else {
        return;
    };
    let metadata = event.metadata.get_or_insert_with(EventMetadata::default);
    merge_missing_event_metadata(metadata, inferred);
}

#[cfg(not(target_os = "macos"))]
fn maybe_enrich_event_terminal_metadata(_event: &mut EventEnvelope) {}

fn merge_missing_event_metadata(metadata: &mut EventMetadata, inferred: EventMetadata) {
    if metadata.terminal_app.is_none() {
        metadata.terminal_app = inferred.terminal_app;
    }
    if metadata.terminal_bundle.is_none() {
        metadata.terminal_bundle = inferred.terminal_bundle;
    }
    if metadata.host_app.is_none() {
        metadata.host_app = inferred.host_app;
    }
    if metadata.window_title.is_none() {
        metadata.window_title = inferred.window_title;
    }
    if metadata
        .tty
        .as_deref()
        .is_none_or(|value| !is_precise_terminal_tty(value))
    {
        metadata.tty = inferred.tty;
    }
    if metadata.pid.is_none() {
        metadata.pid = inferred.pid;
    }
    if metadata.cli_pid.is_none() {
        metadata.cli_pid = inferred.cli_pid;
    }
    if metadata.iterm_session_id.is_none() {
        metadata.iterm_session_id = inferred.iterm_session_id;
    }
    if metadata.kitty_window_id.is_none() {
        metadata.kitty_window_id = inferred.kitty_window_id;
    }
    if metadata.tmux_env.is_none() {
        metadata.tmux_env = inferred.tmux_env;
    }
    if metadata.tmux_pane.is_none() {
        metadata.tmux_pane = inferred.tmux_pane;
    }
    if metadata
        .tmux_client_tty
        .as_deref()
        .is_none_or(|value| !is_precise_terminal_tty(value))
    {
        metadata.tmux_client_tty = inferred.tmux_client_tty;
    }
}

fn is_precise_terminal_tty(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "/dev/tty" && trimmed != "/dev/console"
}
