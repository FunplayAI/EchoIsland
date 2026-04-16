use std::sync::{Arc, Mutex as StdMutex};

use echoisland_adapters::{ClaudePaths, ClaudeSessionScanner, claude_default_paths};
use echoisland_runtime::SharedRuntime;
use notify::RecursiveMode;
use tokio::time::Duration;

use crate::session_scan_runner::{SessionScanner, create_session_watcher, spawn_session_scan_loop};

impl SessionScanner for ClaudeSessionScanner {
    fn scan(&mut self) -> anyhow::Result<Option<Vec<echoisland_core::SessionRecord>>> {
        ClaudeSessionScanner::scan(self)
    }

    fn recommended_poll_interval(&self) -> Duration {
        ClaudeSessionScanner::recommended_poll_interval(self)
    }
}

pub fn spawn_claude_scan_loop(runtime: Arc<SharedRuntime>) {
    let paths = claude_default_paths();
    let scanner = Arc::new(StdMutex::new(ClaudeSessionScanner::new(paths.clone())));

    spawn_session_scan_loop(
        runtime,
        "claude",
        scanner,
        move |watch_tx| create_claude_watcher(&paths, watch_tx),
        "failed to scan Claude fallback sessions",
    );
}

fn create_claude_watcher(
    paths: &ClaudePaths,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut targets = Vec::new();
    targets.push((paths.claude_dir.clone(), RecursiveMode::NonRecursive));
    targets.push((paths.projects_dir.clone(), RecursiveMode::Recursive));
    create_session_watcher("claude", tx, targets)
}
