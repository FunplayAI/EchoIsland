use std::sync::{Arc, Mutex as StdMutex};

use echoisland_adapters::{CodexPaths, CodexSessionScanner, default_paths as codex_default_paths};
use echoisland_runtime::SharedRuntime;
use notify::RecursiveMode;
use tokio::time::Duration;

use crate::session_scan_runner::{SessionScanner, create_session_watcher, spawn_session_scan_loop};

impl SessionScanner for CodexSessionScanner {
    fn scan(&mut self) -> anyhow::Result<Option<Vec<echoisland_core::SessionRecord>>> {
        CodexSessionScanner::scan(self)
    }

    fn recommended_poll_interval(&self) -> Duration {
        CodexSessionScanner::recommended_poll_interval(self)
    }
}

pub fn spawn_codex_scan_loop(runtime: Arc<SharedRuntime>) {
    let paths = codex_default_paths();
    let scanner = Arc::new(StdMutex::new(CodexSessionScanner::new(paths.clone())));

    spawn_session_scan_loop(
        runtime,
        "codex",
        scanner,
        move |watch_tx| create_codex_watcher(&paths, watch_tx),
        "failed to scan Codex fallback sessions",
    );
}

fn create_codex_watcher(
    paths: &CodexPaths,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut targets = Vec::new();
    targets.push((paths.codex_dir.clone(), RecursiveMode::NonRecursive));
    targets.push((paths.codex_dir.join("sessions"), RecursiveMode::Recursive));
    create_session_watcher("codex", tx, targets)
}
