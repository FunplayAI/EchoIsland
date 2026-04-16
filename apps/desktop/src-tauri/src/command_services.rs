use std::path::PathBuf;

use echoisland_core::{EventEnvelope, ResponseEnvelope};
use echoisland_runtime::RuntimeSnapshot;
use tokio::fs;

use crate::{app_runtime::AppRuntime, terminal_focus_service::TerminalFocusService};

const SAMPLE_SESSION_IDS: [&str; 8] = [
    "codex-session-001",
    "claude-session-001",
    "claude-session-002",
    "claude-session-003",
    "claude-session-004",
    "claude-session-005",
    "claude-session-006",
    "claude-session-007",
];

pub struct SnapshotCommandService<'a> {
    runtime: &'a AppRuntime,
}

impl<'a> SnapshotCommandService<'a> {
    pub fn new(runtime: &'a AppRuntime) -> Self {
        Self { runtime }
    }

    pub async fn get_snapshot(&self) -> Result<RuntimeSnapshot, String> {
        self.runtime
            .runtime
            .remove_sessions(&SAMPLE_SESSION_IDS)
            .await;
        let snapshot = self.runtime.runtime.snapshot().await;
        if snapshot.active_session_count > 0 {
            TerminalFocusService::new(self.runtime)
                .sync_snapshot_focus_bindings(&snapshot)
                .await?;
        }
        Ok(snapshot)
    }
}

pub struct SampleIngestService<'a> {
    runtime: &'a AppRuntime,
}

impl<'a> SampleIngestService<'a> {
    pub fn new(runtime: &'a AppRuntime) -> Self {
        Self { runtime }
    }

    pub async fn ingest_sample(&self, file_name: String) -> Result<ResponseEnvelope, String> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..");
        let path = root.join("samples").join("events").join(file_name);
        let raw = fs::read(&path)
            .await
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        let event = serde_json::from_slice::<EventEnvelope>(&raw)
            .map_err(|error| format!("failed to parse sample event: {error}"))?;
        event
            .validate()
            .map_err(|error| format!("invalid sample event: {error}"))?;
        let normalized = event.normalized_event_name();
        if normalized == "PermissionRequest" || normalized == "AskUserQuestion" {
            let shared_runtime = self.runtime.runtime.clone();
            tauri::async_runtime::spawn(async move {
                let _ = shared_runtime.handle_event(event).await;
            });
            Ok(ResponseEnvelope::ok())
        } else {
            Ok(self.runtime.runtime.handle_event(event).await)
        }
    }
}
