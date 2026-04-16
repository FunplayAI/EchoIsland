use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocol::{EventEnvelope, EventMetadata, QuestionPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Processing,
    Running,
    WaitingApproval,
    WaitingQuestion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolHistoryEntry {
    pub tool: String,
    pub description: Option<String>,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub source: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub project_name: Option<String>,
    #[serde(default)]
    pub terminal_app: Option<String>,
    #[serde(default)]
    pub terminal_bundle: Option<String>,
    #[serde(default)]
    pub host_app: Option<String>,
    #[serde(default)]
    pub window_title: Option<String>,
    #[serde(default)]
    pub tty: Option<String>,
    #[serde(default)]
    pub terminal_pid: Option<u32>,
    #[serde(default)]
    pub cli_pid: Option<u32>,
    #[serde(default)]
    pub iterm_session_id: Option<String>,
    #[serde(default)]
    pub kitty_window_id: Option<String>,
    #[serde(default)]
    pub tmux_env: Option<String>,
    #[serde(default)]
    pub tmux_pane: Option<String>,
    #[serde(default)]
    pub tmux_client_tty: Option<String>,
    pub status: AgentStatus,
    pub current_tool: Option<String>,
    pub tool_description: Option<String>,
    pub last_user_prompt: Option<String>,
    pub last_assistant_message: Option<String>,
    pub tool_history: Vec<ToolHistoryEntry>,
    pub last_activity: DateTime<Utc>,
}

impl SessionRecord {
    pub fn new(event: &EventEnvelope) -> Self {
        Self {
            session_id: event.session_id.clone(),
            source: event.source.clone(),
            cwd: event.cwd.clone(),
            model: event.model.clone(),
            project_name: event.project_name(),
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
            status: AgentStatus::Idle,
            current_tool: None,
            tool_description: None,
            last_user_prompt: None,
            last_assistant_message: None,
            tool_history: Vec::new(),
            last_activity: event.timestamp,
        }
    }

    fn refresh_metadata(&mut self, event: &EventEnvelope) {
        if let Some(cwd) = &event.cwd {
            self.cwd = Some(cwd.clone());
            self.project_name = event.project_name();
        }
        if let Some(model) = &event.model {
            self.model = Some(model.clone());
        }
        if let Some(metadata) = &event.metadata {
            if let Some(terminal_app) = &metadata.terminal_app {
                self.terminal_app = Some(terminal_app.clone());
            }
            if let Some(terminal_bundle) = &metadata.terminal_bundle {
                self.terminal_bundle = Some(terminal_bundle.clone());
            }
            if let Some(host_app) = &metadata.host_app {
                self.host_app = Some(host_app.clone());
            }
            if let Some(window_title) = &metadata.window_title {
                self.window_title = Some(window_title.clone());
            }
            if let Some(tty) = &metadata.tty {
                self.tty = Some(tty.clone());
            }
            if let Some(pid) = metadata.pid {
                self.terminal_pid = Some(pid);
            }
            if let Some(cli_pid) = metadata.cli_pid {
                self.cli_pid = Some(cli_pid);
            }
            if let Some(iterm_session_id) = &metadata.iterm_session_id {
                self.iterm_session_id = Some(iterm_session_id.clone());
            }
            if let Some(kitty_window_id) = &metadata.kitty_window_id {
                self.kitty_window_id = Some(kitty_window_id.clone());
            }
            if let Some(tmux_env) = &metadata.tmux_env {
                self.tmux_env = Some(tmux_env.clone());
            }
            if let Some(tmux_pane) = &metadata.tmux_pane {
                self.tmux_pane = Some(tmux_pane.clone());
            }
            if let Some(tmux_client_tty) = &metadata.tmux_client_tty {
                self.tmux_client_tty = Some(tmux_client_tty.clone());
            }
        }
        self.source = event.source.clone();
        self.last_activity = event.timestamp;
    }

    fn merge_terminal_metadata(&mut self, metadata: &EventMetadata) -> bool {
        let mut changed = false;

        if let Some(terminal_app) = &metadata.terminal_app
            && self.terminal_app.as_ref() != Some(terminal_app)
        {
            self.terminal_app = Some(terminal_app.clone());
            changed = true;
        }
        if let Some(terminal_bundle) = &metadata.terminal_bundle
            && self.terminal_bundle.as_ref() != Some(terminal_bundle)
        {
            self.terminal_bundle = Some(terminal_bundle.clone());
            changed = true;
        }
        if let Some(host_app) = &metadata.host_app
            && self.host_app.as_ref() != Some(host_app)
        {
            self.host_app = Some(host_app.clone());
            changed = true;
        }
        if let Some(window_title) = &metadata.window_title
            && self.window_title.as_ref() != Some(window_title)
        {
            self.window_title = Some(window_title.clone());
            changed = true;
        }
        if let Some(tty) = &metadata.tty
            && self.tty.as_ref() != Some(tty)
        {
            self.tty = Some(tty.clone());
            changed = true;
        }
        if let Some(pid) = metadata.pid
            && self.terminal_pid != Some(pid)
        {
            self.terminal_pid = Some(pid);
            changed = true;
        }
        if let Some(cli_pid) = metadata.cli_pid
            && self.cli_pid != Some(cli_pid)
        {
            self.cli_pid = Some(cli_pid);
            changed = true;
        }
        if let Some(iterm_session_id) = &metadata.iterm_session_id
            && self.iterm_session_id.as_ref() != Some(iterm_session_id)
        {
            self.iterm_session_id = Some(iterm_session_id.clone());
            changed = true;
        }
        if let Some(kitty_window_id) = &metadata.kitty_window_id
            && self.kitty_window_id.as_ref() != Some(kitty_window_id)
        {
            self.kitty_window_id = Some(kitty_window_id.clone());
            changed = true;
        }
        if let Some(tmux_env) = &metadata.tmux_env
            && self.tmux_env.as_ref() != Some(tmux_env)
        {
            self.tmux_env = Some(tmux_env.clone());
            changed = true;
        }
        if let Some(tmux_pane) = &metadata.tmux_pane
            && self.tmux_pane.as_ref() != Some(tmux_pane)
        {
            self.tmux_pane = Some(tmux_pane.clone());
            changed = true;
        }
        if let Some(tmux_client_tty) = &metadata.tmux_client_tty
            && self.tmux_client_tty.as_ref() != Some(tmux_client_tty)
        {
            self.tmux_client_tty = Some(tmux_client_tty.clone());
            changed = true;
        }

        changed
    }
}

fn should_drop_legacy_session(session: &SessionRecord) -> bool {
    if session.source != "opencode" {
        return false;
    }

    session.session_id.starts_with("open-")
        && session.cwd.is_none()
        && session.project_name.is_none()
        && session.model.is_none()
        && session.current_tool.is_none()
        && session.tool_description.is_none()
        && session.last_user_prompt.is_none()
        && session.last_assistant_message.is_none()
}

#[derive(Debug, Clone)]
pub struct PendingPermission {
    pub request_id: String,
    pub session_id: String,
    pub source: String,
    pub tool_name: Option<String>,
    pub tool_description: Option<String>,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PendingQuestion {
    pub request_id: String,
    pub session_id: String,
    pub source: String,
    pub payload: QuestionPayload,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DerivedSummary {
    pub status: AgentStatus,
    pub primary_source: String,
    pub active_session_count: usize,
    pub total_session_count: usize,
}

impl Default for DerivedSummary {
    fn default() -> Self {
        Self {
            status: AgentStatus::Idle,
            primary_source: "claude".to_string(),
            active_session_count: 0,
            total_session_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IngestKind {
    Ack,
    PermissionRequest(PendingPermission),
    QuestionRequest(PendingQuestion),
}

#[derive(Debug, Clone)]
pub struct IngestOutcome {
    pub kind: IngestKind,
    pub summary: DerivedSummary,
    pub cleared_permission_request_ids: Vec<String>,
    pub cleared_question_request_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PendingCleanup {
    pub cleared_permission_request_ids: Vec<String>,
    pub cleared_question_request_ids: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AppState {
    sessions: HashMap<String, SessionRecord>,
    permission_queue: VecDeque<PendingPermission>,
    question_queue: VecDeque<PendingQuestion>,
    summary: DerivedSummary,
    max_tool_history: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            max_tool_history: 20,
            ..Self::default()
        }
    }

    pub fn sessions(&self) -> &HashMap<String, SessionRecord> {
        &self.sessions
    }

    pub fn merge_session_terminal_metadata(
        &mut self,
        session_id: &str,
        metadata: &EventMetadata,
    ) -> bool {
        let Some(session) = self.sessions.get_mut(session_id) else {
            return false;
        };
        session.merge_terminal_metadata(metadata)
    }

    pub fn summary(&self) -> &DerivedSummary {
        &self.summary
    }

    pub fn pending_permission_count(&self) -> usize {
        self.permission_queue.len()
    }

    pub fn pending_permission(&self) -> Option<&PendingPermission> {
        self.permission_queue.front()
    }

    pub fn pending_permissions(&self) -> &VecDeque<PendingPermission> {
        &self.permission_queue
    }

    pub fn pending_question_count(&self) -> usize {
        self.question_queue.len()
    }

    pub fn pending_question(&self) -> Option<&PendingQuestion> {
        self.question_queue.front()
    }

    pub fn pending_questions(&self) -> &VecDeque<PendingQuestion> {
        &self.question_queue
    }

    pub fn export_sessions(&self) -> Vec<SessionRecord> {
        self.sessions.values().cloned().collect()
    }

    pub fn restore_sessions(&mut self, sessions: Vec<SessionRecord>) {
        self.sessions.clear();
        self.permission_queue.clear();
        self.question_queue.clear();

        for mut session in sessions {
            if should_drop_legacy_session(&session) {
                continue;
            }
            if matches!(
                session.status,
                AgentStatus::WaitingApproval | AgentStatus::WaitingQuestion
            ) {
                session.status = AgentStatus::Idle;
                session.current_tool = None;
                session.tool_description = None;
            }
            self.sessions.insert(session.session_id.clone(), session);
        }

        self.summary = derive_summary(self.sessions.values());
    }

    pub fn replace_source_sessions(&mut self, source: &str, sessions: Vec<SessionRecord>) -> bool {
        let pending_permission_by_session = self
            .permission_queue
            .iter()
            .filter(|pending| pending.source == source)
            .map(|pending| (pending.session_id.clone(), pending))
            .collect::<HashMap<_, _>>();
        let pending_question_by_session = self
            .question_queue
            .iter()
            .filter(|pending| pending.source == source)
            .map(|pending| (pending.session_id.clone(), pending))
            .collect::<HashMap<_, _>>();

        let existing_source_sessions = self
            .sessions
            .values()
            .filter(|session| session.source == source)
            .cloned()
            .collect::<Vec<_>>();
        let existing_by_id = existing_source_sessions
            .iter()
            .cloned()
            .map(|session| (session.session_id.clone(), session))
            .collect::<HashMap<_, _>>();

        let mut incoming = sessions
            .into_iter()
            .map(|mut session| {
                let session_id = session.session_id.clone();
                preserve_session_terminal_metadata(&mut session, existing_by_id.get(&session_id));
                preserve_pending_session_state(
                    &mut session,
                    existing_by_id.get(&session_id),
                    pending_permission_by_session.get(&session_id).copied(),
                    pending_question_by_session.get(&session_id).copied(),
                );
                session
            })
            .collect::<Vec<_>>();

        for session_id in pending_permission_by_session
            .keys()
            .chain(pending_question_by_session.keys())
        {
            if incoming
                .iter()
                .any(|session| &session.session_id == session_id)
            {
                continue;
            }
            if let Some(existing_session) = existing_by_id.get(session_id).cloned() {
                incoming.push(existing_session);
            }
        }

        incoming.sort_by(|a, b| a.session_id.cmp(&b.session_id));

        let mut existing = existing_source_sessions;
        existing.sort_by(|a, b| a.session_id.cmp(&b.session_id));

        if existing == incoming {
            return false;
        }

        self.sessions.retain(|_, session| session.source != source);

        for session in incoming {
            self.sessions.insert(session.session_id.clone(), session);
        }

        self.summary = derive_summary(self.sessions.values());
        true
    }

    pub fn remove_sessions(&mut self, session_ids: &[&str]) -> bool {
        let before_len = self.sessions.len();
        self.sessions
            .retain(|session_id, _| !session_ids.iter().any(|value| value == session_id));
        self.permission_queue
            .retain(|pending| !session_ids.iter().any(|value| *value == pending.session_id));
        self.question_queue
            .retain(|pending| !session_ids.iter().any(|value| *value == pending.session_id));

        let changed = self.sessions.len() != before_len;
        if changed {
            self.summary = derive_summary(self.sessions.values());
        }
        changed
    }

    pub fn prune_sessions_before(&mut self, cutoff: DateTime<Utc>) -> bool {
        let expired_ids = self
            .sessions
            .iter()
            .filter(|(_, session)| session.last_activity < cutoff)
            .map(|(session_id, _)| session_id.clone())
            .collect::<Vec<_>>();

        if expired_ids.is_empty() {
            return false;
        }

        let expired_lookup = expired_ids.iter().collect::<std::collections::HashSet<_>>();
        self.sessions
            .retain(|session_id, _| !expired_lookup.contains(session_id));
        self.permission_queue
            .retain(|pending| !expired_lookup.contains(&pending.session_id));
        self.question_queue
            .retain(|pending| !expired_lookup.contains(&pending.session_id));
        self.summary = derive_summary(self.sessions.values());
        true
    }

    pub fn ingest_event(&mut self, event: EventEnvelope) -> IngestOutcome {
        let normalized = event.normalized_event_name();
        self.sessions
            .entry(event.session_id.clone())
            .or_insert_with(|| SessionRecord::new(&event));
        let was_waiting = self
            .sessions
            .get(&event.session_id)
            .is_some_and(|session| is_waiting_status(session.status));
        if let Some(session) = self.sessions.get_mut(&event.session_id) {
            session.refresh_metadata(&event);
        }

        let mut cleared_permission_request_ids = Vec::new();
        let mut cleared_question_request_ids = Vec::new();

        let kind = match normalized.as_str() {
            "SessionStart" => {
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::Processing;
                IngestKind::Ack
            }
            "UserPromptSubmit" => {
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::Processing;
                if let Some(message) = event.message.as_ref().filter(|msg| !msg.trim().is_empty()) {
                    session.last_user_prompt = Some(message.clone());
                }
                IngestKind::Ack
            }
            "PreToolUse" => {
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::Running;
                session.current_tool = event.tool_name.clone();
                session.tool_description = event.tool_description();
                IngestKind::Ack
            }
            "PostToolUse" => {
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                let description = event
                    .tool_description()
                    .or_else(|| session.tool_description.clone());
                if let Some(tool) = event
                    .tool_name
                    .clone()
                    .or_else(|| session.current_tool.clone())
                {
                    session.tool_history.push(ToolHistoryEntry {
                        tool,
                        description,
                        success: true,
                        timestamp: event.timestamp,
                    });
                    trim_history(&mut session.tool_history, self.max_tool_history);
                }
                session.status = AgentStatus::Processing;
                session.current_tool = None;
                session.tool_description = None;
                IngestKind::Ack
            }
            "Notification" | "AfterAgentResponse" => {
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::Processing;
                if let Some(message) = event.message.as_ref().filter(|msg| !msg.trim().is_empty()) {
                    session.last_assistant_message = Some(message.clone());
                }
                IngestKind::Ack
            }
            "Stop" | "SessionEnd" => {
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::Idle;
                session.current_tool = None;
                session.tool_description = None;
                if let Some(message) = event.message.as_ref().filter(|msg| !msg.trim().is_empty()) {
                    session.last_assistant_message = Some(message.clone());
                }
                IngestKind::Ack
            }
            "PermissionRequest" => {
                cleared_question_request_ids.extend(
                    self.drain_question_requests_for_session(&event.session_id)
                        .into_iter()
                        .map(|pending| pending.request_id),
                );
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::WaitingApproval;
                session.current_tool = event.tool_name.clone();
                session.tool_description = event.tool_description();
                let request = PendingPermission {
                    request_id: Uuid::new_v4().to_string(),
                    session_id: event.session_id.clone(),
                    source: event.source.clone(),
                    tool_name: event.tool_name.clone(),
                    tool_description: event.tool_description(),
                    requested_at: event.timestamp,
                };
                self.permission_queue.push_back(request.clone());
                IngestKind::PermissionRequest(request)
            }
            "AskUserQuestion" => {
                cleared_permission_request_ids.extend(
                    self.drain_permission_requests_for_session(&event.session_id)
                        .into_iter()
                        .map(|pending| pending.request_id),
                );
                cleared_question_request_ids.extend(
                    self.drain_question_requests_for_session(&event.session_id)
                        .into_iter()
                        .map(|pending| pending.request_id),
                );
                let session = self
                    .sessions
                    .get_mut(&event.session_id)
                    .expect("session exists");
                session.status = AgentStatus::WaitingQuestion;
                let payload = event.question.clone().unwrap_or(QuestionPayload {
                    header: None,
                    text: event
                        .message
                        .clone()
                        .unwrap_or_else(|| "Question".to_string()),
                    options: Vec::new(),
                });
                let request = PendingQuestion {
                    request_id: Uuid::new_v4().to_string(),
                    session_id: event.session_id.clone(),
                    source: event.source.clone(),
                    payload,
                    requested_at: event.timestamp,
                };
                self.question_queue.push_back(request.clone());
                IngestKind::QuestionRequest(request)
            }
            _ => IngestKind::Ack,
        };

        if was_waiting && !event_keeps_waiting_state(&normalized) {
            cleared_permission_request_ids.extend(
                self.drain_permission_requests_for_session(&event.session_id)
                    .into_iter()
                    .map(|pending| pending.request_id),
            );
            cleared_question_request_ids.extend(
                self.drain_question_requests_for_session(&event.session_id)
                    .into_iter()
                    .map(|pending| pending.request_id),
            );

            if let Some(session) = self.sessions.get_mut(&event.session_id) {
                session.current_tool = None;
                session.tool_description = None;
                if is_waiting_status(session.status) {
                    session.status = if normalized == "Stop" || normalized == "SessionEnd" {
                        AgentStatus::Idle
                    } else {
                        AgentStatus::Processing
                    };
                }
            }
        }

        self.summary = derive_summary(self.sessions.values());
        IngestOutcome {
            kind,
            summary: self.summary.clone(),
            cleared_permission_request_ids,
            cleared_question_request_ids,
        }
    }

    pub fn deny_permission(&mut self, request_id: &str) -> Option<PendingPermission> {
        let index = self
            .permission_queue
            .iter()
            .position(|item| item.request_id == request_id)?;
        let pending = self.permission_queue.remove(index)?;
        if let Some(session) = self.sessions.get_mut(&pending.session_id) {
            session.status = AgentStatus::Idle;
            session.current_tool = None;
            session.tool_description = None;
            session.last_activity = Utc::now();
        }
        self.summary = derive_summary(self.sessions.values());
        Some(pending)
    }

    pub fn approve_permission(&mut self, request_id: &str) -> Option<PendingPermission> {
        let index = self
            .permission_queue
            .iter()
            .position(|item| item.request_id == request_id)?;
        let pending = self.permission_queue.remove(index)?;
        if let Some(session) = self.sessions.get_mut(&pending.session_id) {
            session.status = AgentStatus::Running;
            session.last_activity = Utc::now();
        }
        self.summary = derive_summary(self.sessions.values());
        Some(pending)
    }

    pub fn answer_question(&mut self, request_id: &str, answer: &str) -> Option<PendingQuestion> {
        let index = self
            .question_queue
            .iter()
            .position(|item| item.request_id == request_id)?;
        let pending = self.question_queue.remove(index)?;
        if let Some(session) = self.sessions.get_mut(&pending.session_id) {
            session.status = AgentStatus::Processing;
            session.last_assistant_message = Some(answer.to_string());
            session.last_activity = Utc::now();
        }
        self.summary = derive_summary(self.sessions.values());
        Some(pending)
    }

    pub fn skip_question(&mut self, request_id: &str) -> Option<PendingQuestion> {
        let index = self
            .question_queue
            .iter()
            .position(|item| item.request_id == request_id)?;
        let pending = self.question_queue.remove(index)?;
        if let Some(session) = self.sessions.get_mut(&pending.session_id) {
            session.status = AgentStatus::Processing;
            session.last_activity = Utc::now();
        }
        self.summary = derive_summary(self.sessions.values());
        Some(pending)
    }

    pub fn handle_peer_disconnect(&mut self, session_id: &str) -> PendingCleanup {
        let cleared_permission_request_ids = self
            .drain_permission_requests_for_session(session_id)
            .into_iter()
            .map(|pending| pending.request_id)
            .collect::<Vec<_>>();
        let cleared_question_request_ids = self
            .drain_question_requests_for_session(session_id)
            .into_iter()
            .map(|pending| pending.request_id)
            .collect::<Vec<_>>();

        if let Some(session) = self.sessions.get_mut(session_id)
            && (!cleared_permission_request_ids.is_empty()
                || !cleared_question_request_ids.is_empty())
        {
            if is_waiting_status(session.status) {
                session.status = AgentStatus::Processing;
            }
            session.current_tool = None;
            session.tool_description = None;
            session.last_activity = Utc::now();
        }

        if !cleared_permission_request_ids.is_empty() || !cleared_question_request_ids.is_empty() {
            self.summary = derive_summary(self.sessions.values());
        }

        PendingCleanup {
            cleared_permission_request_ids,
            cleared_question_request_ids,
        }
    }

    fn drain_permission_requests_for_session(
        &mut self,
        session_id: &str,
    ) -> Vec<PendingPermission> {
        let mut drained = Vec::new();
        self.permission_queue.retain(|pending| {
            if pending.session_id == session_id {
                drained.push(pending.clone());
                false
            } else {
                true
            }
        });
        drained
    }

    fn drain_question_requests_for_session(&mut self, session_id: &str) -> Vec<PendingQuestion> {
        let mut drained = Vec::new();
        self.question_queue.retain(|pending| {
            if pending.session_id == session_id {
                drained.push(pending.clone());
                false
            } else {
                true
            }
        });
        drained
    }
}

fn is_waiting_status(status: AgentStatus) -> bool {
    matches!(
        status,
        AgentStatus::WaitingApproval | AgentStatus::WaitingQuestion
    )
}

fn event_keeps_waiting_state(normalized: &str) -> bool {
    matches!(
        normalized,
        "Notification" | "SessionStart" | "PermissionRequest" | "AskUserQuestion"
    )
}

fn preserve_pending_session_state(
    session: &mut SessionRecord,
    existing_session: Option<&SessionRecord>,
    pending_permission: Option<&PendingPermission>,
    pending_question: Option<&PendingQuestion>,
) {
    if let Some(pending) = pending_permission {
        session.status = AgentStatus::WaitingApproval;
        session.current_tool = pending
            .tool_name
            .clone()
            .or_else(|| existing_session.and_then(|existing| existing.current_tool.clone()));
        session.tool_description = pending
            .tool_description
            .clone()
            .or_else(|| existing_session.and_then(|existing| existing.tool_description.clone()));
        session.last_activity = session.last_activity.max(pending.requested_at);
        return;
    }

    if let Some(pending) = pending_question {
        session.status = AgentStatus::WaitingQuestion;
        session.current_tool = None;
        session.tool_description = None;
        session.last_activity = session.last_activity.max(pending.requested_at);
        return;
    }

    preserve_newer_local_idle_state(session, existing_session);
}

fn preserve_session_terminal_metadata(
    session: &mut SessionRecord,
    existing_session: Option<&SessionRecord>,
) {
    let Some(existing) = existing_session else {
        return;
    };

    if session.terminal_app.is_none() {
        session.terminal_app = existing.terminal_app.clone();
    }
    if session.terminal_bundle.is_none() {
        session.terminal_bundle = existing.terminal_bundle.clone();
    }
    if session.host_app.is_none() {
        session.host_app = existing.host_app.clone();
    }
    if session.window_title.is_none() {
        session.window_title = existing.window_title.clone();
    }
    if session.tty.is_none() {
        session.tty = existing.tty.clone();
    }
    if session.terminal_pid.is_none() {
        session.terminal_pid = existing.terminal_pid;
    }
    if session.cli_pid.is_none() {
        session.cli_pid = existing.cli_pid;
    }
    if session.iterm_session_id.is_none() {
        session.iterm_session_id = existing.iterm_session_id.clone();
    }
    if session.kitty_window_id.is_none() {
        session.kitty_window_id = existing.kitty_window_id.clone();
    }
    if session.tmux_env.is_none() {
        session.tmux_env = existing.tmux_env.clone();
    }
    if session.tmux_pane.is_none() {
        session.tmux_pane = existing.tmux_pane.clone();
    }
    if session.tmux_client_tty.is_none() {
        session.tmux_client_tty = existing.tmux_client_tty.clone();
    }
}

fn preserve_newer_local_idle_state(
    session: &mut SessionRecord,
    existing_session: Option<&SessionRecord>,
) {
    let Some(existing) = existing_session else {
        return;
    };

    if existing.status != AgentStatus::Idle {
        return;
    }

    if existing.last_activity <= session.last_activity {
        return;
    }

    if !matches!(
        session.status,
        AgentStatus::Running | AgentStatus::Processing
    ) {
        return;
    }

    session.status = AgentStatus::Idle;
    session.current_tool = None;
    session.tool_description = None;
    session.last_activity = existing.last_activity;
}

fn trim_history(entries: &mut Vec<ToolHistoryEntry>, max_len: usize) {
    if entries.len() > max_len {
        let extra = entries.len() - max_len;
        entries.drain(0..extra);
    }
}

fn status_priority(status: AgentStatus) -> u8 {
    match status {
        AgentStatus::WaitingApproval => 5,
        AgentStatus::WaitingQuestion => 4,
        AgentStatus::Running => 3,
        AgentStatus::Processing => 2,
        AgentStatus::Idle => 0,
    }
}

fn derive_summary<'a>(sessions: impl Iterator<Item = &'a SessionRecord>) -> DerivedSummary {
    let mut summary = DerivedSummary::default();
    let mut highest_priority = 0;
    let mut most_recent_idle: Option<(&str, DateTime<Utc>)> = None;

    let collected: Vec<&SessionRecord> = sessions.collect();
    summary.total_session_count = collected.len();

    for session in collected {
        let priority = status_priority(session.status);
        if session.status != AgentStatus::Idle {
            summary.active_session_count += 1;
        } else if most_recent_idle
            .map(|(_, ts)| session.last_activity > ts)
            .unwrap_or(true)
        {
            most_recent_idle = Some((&session.source, session.last_activity));
        }

        if priority > highest_priority {
            highest_priority = priority;
            summary.status = session.status;
            summary.primary_source = session.source.clone();
        }
    }

    if highest_priority == 0 {
        if let Some((source, _)) = most_recent_idle {
            summary.primary_source = source.to_string();
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use serde_json::json;

    use crate::protocol::{
        EventEnvelope, EventMetadata, PROTOCOL_VERSION, QuestionChoice, QuestionPayload,
    };

    use super::{AgentStatus, AppState, IngestKind, SessionRecord};

    fn base_event(name: &str) -> EventEnvelope {
        EventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            hook_event_name: name.to_string(),
            session_id: "session-1".to_string(),
            source: "codex".to_string(),
            timestamp: Utc::now(),
            tool_name: None,
            tool_input: None,
            cwd: Some("D:/AI Island/project".to_string()),
            model: Some("gpt-5.4".to_string()),
            message: None,
            agent_id: None,
            metadata: None,
            question: None,
        }
    }

    #[test]
    fn transitions_tool_usage() {
        let mut state = AppState::new();
        state.ingest_event(base_event("SessionStart"));

        let mut pre = base_event("PreToolUse");
        pre.tool_name = Some("Bash".to_string());
        pre.tool_input = Some(json!({"command":"cargo test"}));
        state.ingest_event(pre);

        assert_eq!(
            state.sessions().get("session-1").map(|s| s.status),
            Some(AgentStatus::Running)
        );

        let mut post = base_event("PostToolUse");
        post.tool_name = Some("Bash".to_string());
        state.ingest_event(post);

        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::Processing);
        assert_eq!(session.tool_history.len(), 1);
    }

    #[test]
    fn queues_permission_requests() {
        let mut state = AppState::new();
        let mut event = base_event("PermissionRequest");
        event.source = "claude".to_string();
        event.tool_name = Some("Bash".to_string());
        event.tool_input = Some(json!({"description":"Run formatter"}));

        let outcome = state.ingest_event(event);
        assert!(matches!(outcome.kind, IngestKind::PermissionRequest(_)));
        assert_eq!(state.pending_permission_count(), 1);

        let request_id = state
            .pending_permission()
            .map(|request| request.request_id.clone())
            .unwrap();
        state.deny_permission(&request_id);
        assert_eq!(state.pending_permission_count(), 0);
    }

    #[test]
    fn queues_questions() {
        let mut state = AppState::new();
        let mut event = base_event("AskUserQuestion");
        event.question = Some(QuestionPayload {
            header: Some("environment".to_string()),
            text: "Where should I deploy?".to_string(),
            options: vec![QuestionChoice {
                label: "staging".to_string(),
                description: Some("Safer".to_string()),
            }],
        });

        let outcome = state.ingest_event(event);
        assert!(matches!(outcome.kind, IngestKind::QuestionRequest(_)));
        assert_eq!(state.pending_question_count(), 1);

        let request_id = state
            .pending_question()
            .map(|request| request.request_id.clone())
            .unwrap();
        state.answer_question(&request_id, "staging");
        assert_eq!(state.pending_question_count(), 0);
    }

    #[test]
    fn activity_event_clears_pending_permission_for_same_session() {
        let mut state = AppState::new();
        let mut permission = base_event("PermissionRequest");
        permission.source = "claude".to_string();
        permission.tool_name = Some("Bash".to_string());
        let outcome = state.ingest_event(permission);
        let request_id = match outcome.kind {
            IngestKind::PermissionRequest(request) => request.request_id,
            other => panic!("expected permission request, got {other:?}"),
        };

        let mut prompt = base_event("UserPromptSubmit");
        prompt.source = "claude".to_string();
        prompt.message = Some("continue".to_string());
        let outcome = state.ingest_event(prompt);

        assert_eq!(outcome.cleared_permission_request_ids, vec![request_id]);
        assert!(outcome.cleared_question_request_ids.is_empty());
        assert_eq!(state.pending_permission_count(), 0);
        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::Processing);
        assert_eq!(session.current_tool, None);
        assert_eq!(session.tool_description, None);
    }

    #[test]
    fn question_request_replaces_existing_permission_for_same_session() {
        let mut state = AppState::new();
        let mut permission = base_event("PermissionRequest");
        permission.source = "claude".to_string();
        permission.tool_name = Some("Edit".to_string());
        let outcome = state.ingest_event(permission);
        let permission_request_id = match outcome.kind {
            IngestKind::PermissionRequest(request) => request.request_id,
            other => panic!("expected permission request, got {other:?}"),
        };

        let mut question = base_event("AskUserQuestion");
        question.source = "claude".to_string();
        question.question = Some(QuestionPayload {
            header: Some("environment".to_string()),
            text: "Where should I deploy?".to_string(),
            options: vec![QuestionChoice {
                label: "staging".to_string(),
                description: Some("Safer".to_string()),
            }],
        });
        let outcome = state.ingest_event(question);

        assert_eq!(
            outcome.cleared_permission_request_ids,
            vec![permission_request_id]
        );
        assert_eq!(state.pending_permission_count(), 0);
        assert_eq!(state.pending_question_count(), 1);
        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::WaitingQuestion);
    }

    #[test]
    fn replace_source_sessions_preserves_pending_permission_queue_and_status() {
        let mut state = AppState::new();
        let mut event = base_event("PermissionRequest");
        event.source = "claude".to_string();
        event.tool_name = Some("Bash".to_string());
        event.tool_input = Some(json!({"description":"Run formatter"}));
        state.ingest_event(event);

        let scanned = SessionRecord {
            session_id: "session-1".to_string(),
            source: "claude".to_string(),
            cwd: Some("D:/AI Island/project".to_string()),
            model: Some("sonnet".to_string()),
            project_name: Some("project".to_string()),
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
            status: AgentStatus::Processing,
            current_tool: None,
            tool_description: None,
            last_user_prompt: Some("format files".to_string()),
            last_assistant_message: None,
            tool_history: Vec::new(),
            last_activity: Utc::now(),
        };

        assert!(state.replace_source_sessions("claude", vec![scanned]));
        assert_eq!(state.pending_permission_count(), 1);

        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::WaitingApproval);
        assert_eq!(session.current_tool.as_deref(), Some("Bash"));
        assert!(state.summary().status == AgentStatus::WaitingApproval);
    }

    #[test]
    fn replace_source_sessions_keeps_pending_session_when_scan_temporarily_misses_it() {
        let mut state = AppState::new();
        let mut event = base_event("PermissionRequest");
        event.source = "claude".to_string();
        event.tool_name = Some("Edit".to_string());
        event.tool_input = Some(json!({"description":"Update config"}));
        state.ingest_event(event);

        assert!(!state.replace_source_sessions("claude", Vec::new()));
        assert_eq!(state.pending_permission_count(), 1);

        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::WaitingApproval);
        assert_eq!(session.current_tool.as_deref(), Some("Edit"));
    }

    #[test]
    fn replace_source_sessions_keeps_denied_session_idle_when_scan_is_stale() {
        let mut state = AppState::new();
        let mut event = base_event("PermissionRequest");
        event.source = "claude".to_string();
        event.tool_name = Some("Bash".to_string());
        event.tool_input = Some(json!({"description":"cargo test"}));
        state.ingest_event(event);

        let request_id = state
            .pending_permission()
            .map(|request| request.request_id.clone())
            .unwrap();
        state.deny_permission(&request_id);

        let denied_at = state
            .sessions()
            .get("session-1")
            .map(|session| session.last_activity)
            .unwrap();

        let scanned = SessionRecord {
            session_id: "session-1".to_string(),
            source: "claude".to_string(),
            cwd: Some("D:/AI Island/project".to_string()),
            model: Some("sonnet".to_string()),
            project_name: Some("project".to_string()),
            terminal_app: None,
            terminal_bundle: None,
            host_app: Some("claude".to_string()),
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
            status: AgentStatus::Running,
            current_tool: Some("Bash".to_string()),
            tool_description: Some("cargo test".to_string()),
            last_user_prompt: Some("run tests".to_string()),
            last_assistant_message: None,
            tool_history: Vec::new(),
            last_activity: denied_at - Duration::seconds(2),
        };

        assert!(state.replace_source_sessions("claude", vec![scanned]));

        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::Idle);
        assert_eq!(session.current_tool, None);
        assert_eq!(session.tool_description, None);
        assert_eq!(session.last_activity, denied_at);
    }

    #[test]
    fn replace_source_sessions_preserves_terminal_metadata_from_existing_session() {
        let mut state = AppState::new();
        let mut event = base_event("SessionStart");
        event.metadata = Some(EventMetadata {
            terminal_app: Some("Apple_Terminal".to_string()),
            terminal_bundle: Some("com.apple.Terminal".to_string()),
            host_app: Some("terminal".to_string()),
            window_title: Some("codeisland".to_string()),
            tty: Some("/dev/ttys001".to_string()),
            pid: Some(1234),
            cli_pid: Some(5678),
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: Some("/tmp/tmux-1000/default,111,0".to_string()),
            tmux_pane: Some("%1".to_string()),
            tmux_client_tty: Some("/dev/ttys002".to_string()),
            workspace_roots: None,
        });
        state.ingest_event(event);

        let scanned = SessionRecord {
            session_id: "session-1".to_string(),
            source: "codex".to_string(),
            cwd: Some("/Users/wenuts/Documents/codeisland".to_string()),
            model: Some("gpt-5.4".to_string()),
            project_name: Some("codeisland".to_string()),
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
            status: AgentStatus::Processing,
            current_tool: None,
            tool_description: None,
            last_user_prompt: Some("latest prompt".to_string()),
            last_assistant_message: Some("latest reply".to_string()),
            tool_history: Vec::new(),
            last_activity: Utc::now() + Duration::seconds(1),
        };

        assert!(state.replace_source_sessions("codex", vec![scanned]));

        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.terminal_app.as_deref(), Some("Apple_Terminal"));
        assert_eq!(
            session.terminal_bundle.as_deref(),
            Some("com.apple.Terminal")
        );
        assert_eq!(session.host_app.as_deref(), Some("terminal"));
        assert_eq!(session.window_title.as_deref(), Some("codeisland"));
        assert_eq!(session.tty.as_deref(), Some("/dev/ttys001"));
        assert_eq!(session.terminal_pid, Some(1234));
        assert_eq!(session.cli_pid, Some(5678));
        assert_eq!(
            session.tmux_env.as_deref(),
            Some("/tmp/tmux-1000/default,111,0")
        );
        assert_eq!(session.tmux_pane.as_deref(), Some("%1"));
        assert_eq!(session.tmux_client_tty.as_deref(), Some("/dev/ttys002"));
        assert_eq!(session.last_user_prompt.as_deref(), Some("latest prompt"));
        assert_eq!(
            session.last_assistant_message.as_deref(),
            Some("latest reply")
        );
    }

    #[test]
    fn replace_source_sessions_allows_newer_scan_after_denied_session() {
        let mut state = AppState::new();
        let mut event = base_event("PermissionRequest");
        event.source = "claude".to_string();
        event.tool_name = Some("Bash".to_string());
        event.tool_input = Some(json!({"description":"cargo test"}));
        state.ingest_event(event);

        let request_id = state
            .pending_permission()
            .map(|request| request.request_id.clone())
            .unwrap();
        state.deny_permission(&request_id);

        let denied_at = state
            .sessions()
            .get("session-1")
            .map(|session| session.last_activity)
            .unwrap();

        let scanned = SessionRecord {
            session_id: "session-1".to_string(),
            source: "claude".to_string(),
            cwd: Some("D:/AI Island/project".to_string()),
            model: Some("sonnet".to_string()),
            project_name: Some("project".to_string()),
            terminal_app: None,
            terminal_bundle: None,
            host_app: Some("claude".to_string()),
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
            status: AgentStatus::Running,
            current_tool: Some("Bash".to_string()),
            tool_description: Some("cargo test".to_string()),
            last_user_prompt: Some("run tests".to_string()),
            last_assistant_message: None,
            tool_history: Vec::new(),
            last_activity: denied_at + Duration::seconds(2),
        };

        assert!(state.replace_source_sessions("claude", vec![scanned]));

        let session = state.sessions().get("session-1").unwrap();
        assert_eq!(session.status, AgentStatus::Running);
        assert_eq!(session.current_tool.as_deref(), Some("Bash"));
        assert_eq!(session.tool_description.as_deref(), Some("cargo test"));
    }

    #[test]
    fn prunes_sessions_older_than_cutoff() {
        let mut state = AppState::new();
        state.ingest_event(base_event("SessionStart"));

        state.sessions.get_mut("session-1").unwrap().last_activity =
            Utc::now() - Duration::minutes(31);

        let changed = state.prune_sessions_before(Utc::now() - Duration::minutes(30));

        assert!(changed);
        assert!(state.sessions().is_empty());
        assert_eq!(state.summary().total_session_count, 0);
    }
}
