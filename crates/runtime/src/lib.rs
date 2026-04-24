use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    time::Duration as StdDuration,
};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use echoisland_core::{
    AppState, EventEnvelope, EventMetadata, IngestKind, PendingCleanup, ResponseEnvelope,
    SessionRecord,
};
use echoisland_persistence::{default_state_path, load_sessions, save_sessions};
use serde::Serialize;
use tokio::{
    sync::{Mutex, oneshot},
    time::{Duration, timeout},
};
use tracing::{info, warn};

const SESSION_EXPIRY_MINUTES: i64 = 30;
const PERSIST_DEBOUNCE_MS: u64 = 350;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ToolHistoryEntryView {
    pub tool: String,
    pub description: Option<String>,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct SessionSnapshotView {
    pub session_id: String,
    pub source: String,
    pub project_name: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub terminal_app: Option<String>,
    pub terminal_bundle: Option<String>,
    pub host_app: Option<String>,
    pub window_title: Option<String>,
    pub tty: Option<String>,
    pub terminal_pid: Option<u32>,
    pub cli_pid: Option<u32>,
    pub iterm_session_id: Option<String>,
    pub kitty_window_id: Option<String>,
    pub tmux_env: Option<String>,
    pub tmux_pane: Option<String>,
    pub tmux_client_tty: Option<String>,
    pub status: String,
    pub current_tool: Option<String>,
    pub tool_description: Option<String>,
    pub last_user_prompt: Option<String>,
    pub last_assistant_message: Option<String>,
    pub tool_history_count: usize,
    pub tool_history: Vec<ToolHistoryEntryView>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct RuntimeSnapshot {
    pub status: String,
    pub primary_source: String,
    pub active_session_count: usize,
    pub total_session_count: usize,
    pub pending_permission_count: usize,
    pub pending_question_count: usize,
    pub pending_permission: Option<PendingPermissionView>,
    pub pending_question: Option<PendingQuestionView>,
    pub pending_permissions: Vec<PendingPermissionView>,
    pub pending_questions: Vec<PendingQuestionView>,
    pub sessions: Vec<SessionSnapshotView>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct PendingPermissionView {
    pub request_id: String,
    pub session_id: String,
    pub source: String,
    pub tool_name: Option<String>,
    pub tool_description: Option<String>,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct PendingQuestionView {
    pub request_id: String,
    pub session_id: String,
    pub source: String,
    pub header: Option<String>,
    pub text: String,
    pub options: Vec<String>,
    pub requested_at: DateTime<Utc>,
}

fn pending_permission_view(pending: &echoisland_core::PendingPermission) -> PendingPermissionView {
    PendingPermissionView {
        request_id: pending.request_id.clone(),
        session_id: pending.session_id.clone(),
        source: pending.source.clone(),
        tool_name: pending.tool_name.clone(),
        tool_description: pending.tool_description.clone(),
        requested_at: pending.requested_at,
    }
}

fn pending_question_view(pending: &echoisland_core::PendingQuestion) -> PendingQuestionView {
    PendingQuestionView {
        request_id: pending.request_id.clone(),
        session_id: pending.session_id.clone(),
        source: pending.source.clone(),
        header: pending.payload.header.clone(),
        text: pending.payload.text.clone(),
        options: pending
            .payload
            .options
            .iter()
            .map(|option| option.label.clone())
            .collect(),
        requested_at: pending.requested_at,
    }
}

#[derive(Debug)]
pub struct SharedRuntime {
    state: Mutex<AppState>,
    permission_waiters: Mutex<std::collections::HashMap<String, oneshot::Sender<ResponseEnvelope>>>,
    question_waiters: Mutex<std::collections::HashMap<String, oneshot::Sender<ResponseEnvelope>>>,
    persist_tx: Sender<PersistCommand>,
}

#[derive(Debug)]
enum PersistCommand {
    Save(Vec<SessionRecord>),
    Flush(Sender<()>),
}

impl SharedRuntime {
    pub fn new() -> Self {
        Self::with_storage_path(default_state_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        let mut state = AppState::new();
        match load_sessions(&storage_path) {
            Ok(mut sessions) if !sessions.is_empty() => {
                let now = Utc::now();
                sessions.retain(|session| !is_session_expired(session, now));
                info!(
                    session_count = sessions.len(),
                    path = %storage_path.display(),
                    "restored persisted sessions"
                );
                state.restore_sessions(sessions);
            }
            Ok(_) => {}
            Err(error) => {
                warn!(
                    path = %storage_path.display(),
                    error = %error,
                    "failed to load persisted sessions"
                );
            }
        }

        Self {
            state: Mutex::new(state),
            permission_waiters: Mutex::new(std::collections::HashMap::new()),
            question_waiters: Mutex::new(std::collections::HashMap::new()),
            persist_tx: spawn_persistence_worker(storage_path.clone()),
        }
    }

    pub async fn handle_event(&self, event: EventEnvelope) -> ResponseEnvelope {
        let outcome = {
            let mut state = self.state.lock().await;
            state.ingest_event(event)
        };
        self.queue_persist_state().await;
        self.resolve_cleared_waiters(&outcome).await;

        match outcome.kind {
            IngestKind::Ack => {
                info!(
                    active_sessions = outcome.summary.active_session_count,
                    total_sessions = outcome.summary.total_session_count,
                    "event accepted"
                );
                ResponseEnvelope::ok()
            }
            IngestKind::PermissionRequest(request) => {
                let request_id = request.request_id.clone();
                let (tx, rx) = oneshot::channel();
                self.permission_waiters
                    .lock()
                    .await
                    .insert(request_id.clone(), tx);
                info!(
                    session_id = request.session_id,
                    request_id, "permission request queued"
                );
                match timeout(Duration::from_secs(1800), rx).await {
                    Ok(Ok(response)) => response,
                    _ => {
                        warn!("permission request timed out or sender dropped");
                        self.permission_waiters
                            .lock()
                            .await
                            .remove(&request.request_id);
                        let mut state = self.state.lock().await;
                        let _ = state.deny_permission(&request.request_id);
                        drop(state);
                        self.queue_persist_state().await;
                        ResponseEnvelope::deny()
                    }
                }
            }
            IngestKind::QuestionRequest(request) => {
                let request_id = request.request_id.clone();
                let (tx, rx) = oneshot::channel();
                self.question_waiters
                    .lock()
                    .await
                    .insert(request_id.clone(), tx);
                info!(
                    session_id = request.session_id,
                    request_id, "question request queued"
                );
                match timeout(Duration::from_secs(1800), rx).await {
                    Ok(Ok(response)) => response,
                    _ => {
                        warn!("question request timed out or sender dropped");
                        self.question_waiters
                            .lock()
                            .await
                            .remove(&request.request_id);
                        let mut state = self.state.lock().await;
                        let _ = state.skip_question(&request.request_id);
                        drop(state);
                        self.queue_persist_state().await;
                        ResponseEnvelope::skipped()
                    }
                }
            }
        }
    }

    async fn resolve_cleared_waiters(&self, outcome: &echoisland_core::IngestOutcome) {
        if !outcome.cleared_permission_request_ids.is_empty() {
            let mut waiters = self.permission_waiters.lock().await;
            for request_id in &outcome.cleared_permission_request_ids {
                if let Some(sender) = waiters.remove(request_id) {
                    let _ = sender.send(ResponseEnvelope::deny());
                    info!(
                        request_id,
                        "permission waiter cleared after external session activity"
                    );
                }
            }
        }

        if !outcome.cleared_question_request_ids.is_empty() {
            let mut waiters = self.question_waiters.lock().await;
            for request_id in &outcome.cleared_question_request_ids {
                if let Some(sender) = waiters.remove(request_id) {
                    let _ = sender.send(ResponseEnvelope::skipped());
                    info!(
                        request_id,
                        "question waiter cleared after external session activity"
                    );
                }
            }
        }
    }

    pub async fn handle_peer_disconnect(&self, session_id: &str) -> bool {
        let cleanup = {
            let mut state = self.state.lock().await;
            state.handle_peer_disconnect(session_id)
        };
        let changed = self.resolve_pending_cleanup(&cleanup).await;
        if changed {
            self.queue_persist_state().await;
            warn!(session_id, "cleared pending requests after peer disconnect");
        }
        changed
    }

    async fn resolve_pending_cleanup(&self, cleanup: &PendingCleanup) -> bool {
        let mut changed = false;

        if !cleanup.cleared_permission_request_ids.is_empty() {
            let mut waiters = self.permission_waiters.lock().await;
            for request_id in &cleanup.cleared_permission_request_ids {
                if let Some(sender) = waiters.remove(request_id) {
                    let _ = sender.send(ResponseEnvelope::deny());
                    changed = true;
                    info!(request_id, "permission waiter cleared");
                }
            }
        }

        if !cleanup.cleared_question_request_ids.is_empty() {
            let mut waiters = self.question_waiters.lock().await;
            for request_id in &cleanup.cleared_question_request_ids {
                if let Some(sender) = waiters.remove(request_id) {
                    let _ = sender.send(ResponseEnvelope::skipped());
                    changed = true;
                    info!(request_id, "question waiter cleared");
                }
            }
        }

        changed
            || !cleanup.cleared_permission_request_ids.is_empty()
            || !cleanup.cleared_question_request_ids.is_empty()
    }

    pub async fn approve_permission(&self, request_id: &str) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let pending = state
            .approve_permission(request_id)
            .ok_or_else(|| format!("permission request not found: {request_id}"))?;
        drop(state);

        let sender = self
            .permission_waiters
            .lock()
            .await
            .remove(&pending.request_id)
            .ok_or_else(|| format!("permission waiter not found: {request_id}"))?;
        self.queue_persist_state().await;
        sender
            .send(ResponseEnvelope::allow())
            .map_err(|_| format!("failed to deliver approval response for {request_id}"))?;
        Ok(())
    }

    pub async fn deny_permission(&self, request_id: &str) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let pending = state
            .deny_permission(request_id)
            .ok_or_else(|| format!("permission request not found: {request_id}"))?;
        drop(state);

        let sender = self
            .permission_waiters
            .lock()
            .await
            .remove(&pending.request_id)
            .ok_or_else(|| format!("permission waiter not found: {request_id}"))?;
        self.queue_persist_state().await;
        sender
            .send(ResponseEnvelope::deny())
            .map_err(|_| format!("failed to deliver deny response for {request_id}"))?;
        Ok(())
    }

    pub async fn answer_question(&self, request_id: &str, answer: &str) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let pending = state
            .answer_question(request_id, answer)
            .ok_or_else(|| format!("question request not found: {request_id}"))?;
        drop(state);

        let sender = self
            .question_waiters
            .lock()
            .await
            .remove(&pending.request_id)
            .ok_or_else(|| format!("question waiter not found: {request_id}"))?;
        self.queue_persist_state().await;
        sender
            .send(ResponseEnvelope::answer(answer))
            .map_err(|_| format!("failed to deliver answer response for {request_id}"))?;
        Ok(())
    }

    pub async fn skip_question(&self, request_id: &str) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let pending = state
            .skip_question(request_id)
            .ok_or_else(|| format!("question request not found: {request_id}"))?;
        drop(state);

        let sender = self
            .question_waiters
            .lock()
            .await
            .remove(&pending.request_id)
            .ok_or_else(|| format!("question waiter not found: {request_id}"))?;
        self.queue_persist_state().await;
        sender
            .send(ResponseEnvelope::skipped())
            .map_err(|_| format!("failed to deliver skip response for {request_id}"))?;
        Ok(())
    }

    pub async fn snapshot(&self) -> RuntimeSnapshot {
        let (snapshot, pruned) = {
            let mut state = self.state.lock().await;
            let pruned = prune_expired_sessions(&mut state);
            let summary = state.summary().clone();
            let mut sessions = state
                .sessions()
                .values()
                .map(|session| SessionSnapshotView {
                    session_id: session.session_id.clone(),
                    source: session.source.clone(),
                    project_name: session.project_name.clone(),
                    cwd: session.cwd.clone(),
                    model: session.model.clone(),
                    terminal_app: session.terminal_app.clone(),
                    terminal_bundle: session.terminal_bundle.clone(),
                    host_app: session.host_app.clone(),
                    window_title: session.window_title.clone(),
                    tty: session.tty.clone(),
                    terminal_pid: session.terminal_pid,
                    cli_pid: session.cli_pid,
                    iterm_session_id: session.iterm_session_id.clone(),
                    kitty_window_id: session.kitty_window_id.clone(),
                    tmux_env: session.tmux_env.clone(),
                    tmux_pane: session.tmux_pane.clone(),
                    tmux_client_tty: session.tmux_client_tty.clone(),
                    status: format!("{:?}", session.status),
                    current_tool: session.current_tool.clone(),
                    tool_description: session.tool_description.clone(),
                    last_user_prompt: session.last_user_prompt.clone(),
                    last_assistant_message: session.last_assistant_message.clone(),
                    tool_history_count: session.tool_history.len(),
                    tool_history: session
                        .tool_history
                        .iter()
                        .rev()
                        .take(3)
                        .map(|entry| ToolHistoryEntryView {
                            tool: entry.tool.clone(),
                            description: entry.description.clone(),
                            success: entry.success,
                            timestamp: entry.timestamp,
                        })
                        .collect(),
                    last_activity: session.last_activity,
                })
                .collect::<Vec<_>>();
            sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

            (
                RuntimeSnapshot {
                    status: format!("{:?}", summary.status),
                    primary_source: summary.primary_source,
                    active_session_count: summary.active_session_count,
                    total_session_count: summary.total_session_count,
                    pending_permission_count: state.pending_permission_count(),
                    pending_question_count: state.pending_question_count(),
                    pending_permission: state.pending_permission().map(pending_permission_view),
                    pending_question: state.pending_question().map(pending_question_view),
                    pending_permissions: state
                        .pending_permissions()
                        .iter()
                        .map(pending_permission_view)
                        .collect(),
                    pending_questions: state
                        .pending_questions()
                        .iter()
                        .map(pending_question_view)
                        .collect(),
                    sessions,
                },
                pruned,
            )
        };

        if pruned {
            self.queue_persist_state().await;
        }

        snapshot
    }

    pub async fn session(&self, session_id: &str) -> Option<SessionRecord> {
        let state = self.state.lock().await;
        state.sessions().get(session_id).cloned()
    }

    pub async fn merge_session_terminal_metadata(
        &self,
        session_id: &str,
        metadata: EventMetadata,
    ) -> bool {
        let changed = {
            let mut state = self.state.lock().await;
            state.merge_session_terminal_metadata(session_id, &metadata)
        };

        if changed {
            self.queue_persist_state().await;
        }

        changed
    }

    pub async fn sync_source_sessions(
        &self,
        source: &str,
        sessions: Vec<echoisland_core::SessionRecord>,
    ) {
        let now = Utc::now();
        let sessions = sessions
            .into_iter()
            .filter(|session| !is_session_expired(session, now))
            .collect::<Vec<_>>();
        let changed = {
            let mut state = self.state.lock().await;
            state.replace_source_sessions(source, sessions)
        };

        if changed {
            self.queue_persist_state().await;
        }
    }

    pub async fn remove_sessions(&self, session_ids: &[&str]) {
        let changed = {
            let mut state = self.state.lock().await;
            state.remove_sessions(session_ids)
        };

        if changed {
            self.queue_persist_state().await;
        }
    }

    pub async fn flush_persistence(&self) {
        self.queue_persist_state().await;
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.persist_tx.send(PersistCommand::Flush(ack_tx)).is_err() {
            return;
        }
        let _ = tokio::task::spawn_blocking(move || ack_rx.recv()).await;
    }

    async fn queue_persist_state(&self) {
        let sessions = {
            let state = self.state.lock().await;
            state.export_sessions()
        };

        let _ = self.persist_tx.send(PersistCommand::Save(sessions));
    }
}

fn spawn_persistence_worker(storage_path: PathBuf) -> Sender<PersistCommand> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || run_persistence_worker(storage_path, rx));
    tx
}

fn run_persistence_worker(storage_path: PathBuf, rx: Receiver<PersistCommand>) {
    let mut latest_sessions: Option<Vec<SessionRecord>> = None;

    loop {
        let command = if latest_sessions.is_some() {
            match rx.recv_timeout(StdDuration::from_millis(PERSIST_DEBOUNCE_MS)) {
                Ok(command) => command,
                Err(RecvTimeoutError::Timeout) => {
                    flush_latest_sessions(&storage_path, &mut latest_sessions);
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    flush_latest_sessions(&storage_path, &mut latest_sessions);
                    break;
                }
            }
        } else {
            match rx.recv() {
                Ok(command) => command,
                Err(_) => break,
            }
        };

        match command {
            PersistCommand::Save(sessions) => {
                latest_sessions = Some(sessions);
            }
            PersistCommand::Flush(ack_tx) => {
                flush_latest_sessions(&storage_path, &mut latest_sessions);
                let _ = ack_tx.send(());
            }
        }
    }
}

fn flush_latest_sessions(storage_path: &PathBuf, latest_sessions: &mut Option<Vec<SessionRecord>>) {
    let Some(sessions) = latest_sessions.take() else {
        return;
    };

    if let Err(error) = save_sessions(storage_path, &sessions) {
        warn!(
            path = %storage_path.display(),
            error = %error,
            "failed to persist sessions"
        );
    }
}

fn session_expiry_cutoff(now: DateTime<Utc>) -> DateTime<Utc> {
    now - ChronoDuration::minutes(SESSION_EXPIRY_MINUTES)
}

fn is_session_expired(session: &SessionRecord, now: DateTime<Utc>) -> bool {
    session.last_activity < session_expiry_cutoff(now)
}

fn prune_expired_sessions(state: &mut AppState) -> bool {
    state.prune_sessions_before(session_expiry_cutoff(Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::SharedRuntime;
    use chrono::{Duration as ChronoDuration, Utc};
    use echoisland_core::protocol::{
        EventEnvelope, PROTOCOL_VERSION, QuestionChoice, QuestionPayload,
    };
    use echoisland_core::{AgentStatus, SessionRecord};
    use tokio::time::Duration;

    fn temp_state_path() -> std::path::PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("echoisland-runtime-state-{suffix}.json"))
    }

    fn session(session_id: &str, minutes_ago: i64) -> SessionRecord {
        SessionRecord {
            session_id: session_id.to_string(),
            source: "codex".to_string(),
            cwd: Some("D:/repo".to_string()),
            model: Some("gpt-5.4".to_string()),
            project_name: Some("repo".to_string()),
            terminal_app: None,
            terminal_bundle: None,
            host_app: None,
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
            status: AgentStatus::Idle,
            current_tool: None,
            tool_description: None,
            last_user_prompt: Some("hello".to_string()),
            last_assistant_message: Some("world".to_string()),
            tool_history: Vec::new(),
            last_activity: Utc::now() - ChronoDuration::minutes(minutes_ago),
        }
    }

    #[tokio::test]
    async fn snapshot_prunes_expired_sessions() {
        let path = temp_state_path();
        let runtime = SharedRuntime::with_storage_path(path.clone());
        runtime
            .sync_source_sessions("codex", vec![session("expired", 31), session("fresh", 5)])
            .await;

        let snapshot = runtime.snapshot().await;

        assert_eq!(snapshot.total_session_count, 1);
        assert_eq!(snapshot.sessions[0].session_id, "fresh");

        runtime.flush_persistence().await;
        let _ = std::fs::remove_file(path);
    }

    fn base_event(name: &str, session_id: &str, source: &str) -> EventEnvelope {
        EventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            hook_event_name: name.to_string(),
            session_id: session_id.to_string(),
            source: source.to_string(),
            timestamp: Utc::now(),
            tool_name: None,
            tool_input: None,
            cwd: Some("D:/repo".to_string()),
            model: Some("gpt-5.4".to_string()),
            message: None,
            agent_id: None,
            metadata: None,
            question: None,
        }
    }

    #[tokio::test]
    async fn snapshot_exposes_full_pending_queues() {
        let path = temp_state_path();
        let runtime = SharedRuntime::with_storage_path(path.clone());

        let mut permission_event_a = base_event("PermissionRequest", "permission-a", "claude");
        permission_event_a.tool_name = Some("Bash".to_string());

        let mut permission_event_b = base_event("PermissionRequest", "permission-b", "claude");
        permission_event_b.tool_name = Some("Edit".to_string());

        let mut question_event = base_event("AskUserQuestion", "question-a", "claude");
        question_event.question = Some(QuestionPayload {
            header: Some("environment".to_string()),
            text: "Where should I deploy?".to_string(),
            options: vec![QuestionChoice {
                label: "staging".to_string(),
                description: Some("Safer".to_string()),
            }],
        });

        {
            let mut state = runtime.state.lock().await;
            state.ingest_event(permission_event_a);
            state.ingest_event(permission_event_b);
            state.ingest_event(question_event);
        }

        let snapshot = runtime.snapshot().await;

        assert_eq!(snapshot.pending_permission_count, 2);
        assert_eq!(snapshot.pending_question_count, 1);
        assert_eq!(snapshot.pending_permissions.len(), 2);
        assert_eq!(snapshot.pending_questions.len(), 1);
        assert_eq!(snapshot.pending_permissions[0].session_id, "permission-a");
        assert_eq!(snapshot.pending_permissions[1].session_id, "permission-b");
        assert_eq!(snapshot.pending_questions[0].session_id, "question-a");

        runtime.flush_persistence().await;
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn activity_event_resolves_pending_permission_waiter() {
        let path = temp_state_path();
        let runtime = SharedRuntime::with_storage_path(path.clone());

        let mut permission_event = base_event("PermissionRequest", "permission-a", "claude");
        permission_event.tool_name = Some("Bash".to_string());

        let mut activity_event = base_event("UserPromptSubmit", "permission-a", "claude");
        activity_event.message = Some("continue".to_string());

        let (response, _) = tokio::join!(runtime.handle_event(permission_event), async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            runtime.handle_event(activity_event).await
        });

        assert_eq!(
            response
                .decision
                .as_ref()
                .map(|decision| decision.behavior.as_str()),
            Some("deny")
        );

        let snapshot = runtime.snapshot().await;
        assert_eq!(snapshot.pending_permission_count, 0);
        assert_eq!(snapshot.sessions[0].status, "Processing");

        runtime.flush_persistence().await;
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn peer_disconnect_clears_pending_permission() {
        let path = temp_state_path();
        let runtime = SharedRuntime::with_storage_path(path.clone());

        let mut permission_event = base_event("PermissionRequest", "permission-a", "claude");
        permission_event.tool_name = Some("Bash".to_string());

        let (response, _) = tokio::join!(runtime.handle_event(permission_event), async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            assert!(runtime.handle_peer_disconnect("permission-a").await);
        });
        assert_eq!(
            response
                .decision
                .as_ref()
                .map(|decision| decision.behavior.as_str()),
            Some("deny")
        );

        let snapshot = runtime.snapshot().await;
        assert_eq!(snapshot.pending_permission_count, 0);
        assert_eq!(snapshot.sessions[0].status, "Processing");

        runtime.flush_persistence().await;
        let _ = std::fs::remove_file(path);
    }
}
