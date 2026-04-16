use std::{
    path::PathBuf,
    sync::{Arc, Mutex as StdMutex},
};

use echoisland_core::SessionRecord;
use echoisland_runtime::SharedRuntime;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{
    sync::mpsc,
    time::{Duration, sleep},
};

const WATCHER_DEBOUNCE_MS: u64 = 250;

pub trait SessionScanner: Send + 'static {
    fn scan(&mut self) -> anyhow::Result<Option<Vec<SessionRecord>>>;
    fn recommended_poll_interval(&self) -> Duration;
}

pub fn spawn_session_scan_loop<S>(
    runtime: Arc<SharedRuntime>,
    source: &'static str,
    scanner: Arc<StdMutex<S>>,
    watcher_factory: impl FnOnce(mpsc::UnboundedSender<()>) -> notify::Result<RecommendedWatcher>
    + Send
    + 'static,
    scan_failure_label: &'static str,
) where
    S: SessionScanner,
{
    tauri::async_runtime::spawn(async move {
        run_session_scan_loop(
            runtime,
            source,
            scanner,
            watcher_factory,
            scan_failure_label,
        )
        .await;
    });
}

async fn run_session_scan_loop<S, F>(
    runtime: Arc<SharedRuntime>,
    source: &'static str,
    scanner: Arc<StdMutex<S>>,
    watcher_factory: F,
    scan_failure_label: &'static str,
) where
    S: SessionScanner,
    F: FnOnce(mpsc::UnboundedSender<()>) -> notify::Result<RecommendedWatcher>,
{
    let (watch_tx, watch_rx) = mpsc::unbounded_channel();
    let _watcher = match watcher_factory(watch_tx) {
        Ok(watcher) => Some(watcher),
        Err(error) => {
            tracing::warn!(error = %error, source, "failed to start session watcher");
            None
        }
    };
    let mut watch_rx = watch_rx;

    loop {
        let scanner = scanner.clone();
        let scan_result = tokio::task::spawn_blocking(move || {
            let mut scanner = scanner.lock().expect("session scanner mutex poisoned");
            let result = scanner.scan();
            let interval = scanner.recommended_poll_interval();
            (result, interval)
        })
        .await;

        let mut next_interval = Duration::from_secs(15);
        match scan_result {
            Ok((Ok(Some(sessions)), interval)) => {
                runtime.sync_source_sessions(source, sessions).await;
                next_interval = interval;
            }
            Ok((Ok(None), interval)) => {
                next_interval = interval;
            }
            Ok((Err(error), _)) => {
                tracing::warn!(error = %error, source, scan_failure_label);
            }
            Err(error) => {
                tracing::warn!(error = %error, source, "session scan task failed");
            }
        }

        if next_interval > Duration::from_secs(0) {
            tokio::select! {
                _ = sleep(next_interval) => {}
                changed = watch_rx.recv() => {
                    if changed.is_some() {
                        debounce_watcher_events(&mut watch_rx).await;
                    }
                }
            }
        }
    }
}

pub async fn debounce_watcher_events(watch_rx: &mut mpsc::UnboundedReceiver<()>) {
    let debounce = sleep(Duration::from_millis(WATCHER_DEBOUNCE_MS));
    tokio::pin!(debounce);

    loop {
        tokio::select! {
            _ = &mut debounce => break,
            changed = watch_rx.recv() => {
                if changed.is_none() {
                    break;
                }
                debounce
                    .as_mut()
                    .reset(tokio::time::Instant::now() + Duration::from_millis(WATCHER_DEBOUNCE_MS));
            }
        }
    }

    while watch_rx.try_recv().is_ok() {}
}

pub fn create_session_watcher(
    source: &'static str,
    tx: mpsc::UnboundedSender<()>,
    targets: Vec<(PathBuf, RecursiveMode)>,
) -> notify::Result<RecommendedWatcher> {
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| match result {
            Ok(_event) => {
                let _ = tx.send(());
            }
            Err(error) => {
                tracing::warn!(error = %error, source, "session watcher event failed");
            }
        },
        Config::default(),
    )?;

    for (path, mode) in targets {
        if path.exists() {
            watcher.watch(&path, mode)?;
        }
    }

    Ok(watcher)
}
