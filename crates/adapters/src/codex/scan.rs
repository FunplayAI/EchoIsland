use std::{
    collections::{HashMap, HashSet},
    fs,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use echoisland_core::{AgentStatus, SessionRecord};
use serde_json::Value;

use crate::platform_support::codex_running_process_limit;

use super::CodexPaths;

const SESSION_SCAN_LIMIT: usize = 16;
const SESSION_HEAD_LINES: usize = 96;
const SESSION_TAIL_BYTES: u64 = 64 * 1024;
const ACTIVE_WINDOW_SECS: i64 = 300;
const ACTIVE_SCAN_INTERVAL_SECS: u64 = 3;
const IDLE_SCAN_INTERVAL_SECS: u64 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryEntry {
    timestamp: DateTime<Utc>,
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskMarker {
    Started,
    Complete,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaskSignal {
    marker: TaskMarker,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct HistoryScanState {
    size: u64,
    modified_at: Option<DateTime<Utc>>,
    offset: u64,
    latest_prompt_by_session: HashMap<String, HistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSessionFile {
    session_id: String,
    cwd: Option<String>,
    model: Option<String>,
    last_activity: DateTime<Utc>,
    last_assistant_message: Option<String>,
    latest_task_signal: Option<TaskSignal>,
}

#[derive(Debug, Clone)]
struct SessionFileState {
    size: u64,
    modified_at: DateTime<Utc>,
    parsed: Option<ParsedSessionFile>,
}

#[derive(Debug, Clone)]
pub struct CodexSessionScanner {
    paths: CodexPaths,
    history_state: HistoryScanState,
    session_files: HashMap<PathBuf, SessionFileState>,
    last_sessions: Vec<SessionRecord>,
}

impl Default for HistoryScanState {
    fn default() -> Self {
        Self {
            size: 0,
            modified_at: None,
            offset: 0,
            latest_prompt_by_session: HashMap::new(),
        }
    }
}

impl CodexSessionScanner {
    pub fn new(paths: CodexPaths) -> Self {
        Self {
            paths,
            history_state: HistoryScanState::default(),
            session_files: HashMap::new(),
            last_sessions: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> Result<Option<Vec<SessionRecord>>> {
        let history_path = self.paths.codex_dir.join("history.jsonl");
        refresh_history_state(&history_path, &mut self.history_state)?;

        let session_root = self.paths.codex_dir.join("sessions");
        let session_paths = recent_session_files(&session_root, SESSION_SCAN_LIMIT)?;
        let interesting_paths = session_paths.iter().cloned().collect::<HashSet<_>>();

        self.session_files
            .retain(|path, _| interesting_paths.contains(path));

        for path in &session_paths {
            let modified_at = file_modified_utc(path)?;
            let size = file_size(path)?;
            let needs_refresh = self
                .session_files
                .get(path)
                .map(|state| state.size != size || state.modified_at != modified_at)
                .unwrap_or(true);

            if needs_refresh {
                let parsed = parse_session_file(path)?;
                self.session_files.insert(
                    path.clone(),
                    SessionFileState {
                        size,
                        modified_at,
                        parsed,
                    },
                );
            }
        }

        let mut sessions = self
            .session_files
            .values()
            .filter_map(|state| {
                state.parsed.as_ref().map(|parsed| {
                    build_session_record(parsed, &self.history_state.latest_prompt_by_session)
                })
            })
            .collect::<Vec<_>>();

        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        if let Some(process_count) = codex_running_process_limit(&self.paths.home_dir) {
            sessions.truncate(process_count);
        }

        if sessions == self.last_sessions {
            return Ok(None);
        }

        self.last_sessions = sessions.clone();
        Ok(Some(sessions))
    }

    pub fn recommended_poll_interval(&self) -> Duration {
        let now = Utc::now();
        let recently_active = self.last_sessions.iter().any(|session| {
            session.status != AgentStatus::Idle
                || (now - session.last_activity).num_seconds() <= ACTIVE_WINDOW_SECS
        });

        if recently_active {
            Duration::from_secs(ACTIVE_SCAN_INTERVAL_SECS)
        } else {
            Duration::from_secs(IDLE_SCAN_INTERVAL_SECS)
        }
    }
}

pub fn scan_codex_sessions(paths: &CodexPaths) -> Result<Vec<SessionRecord>> {
    let mut scanner = CodexSessionScanner::new(paths.clone());
    Ok(scanner.scan()?.unwrap_or_default())
}

fn parse_session_file(path: &Path) -> Result<Option<ParsedSessionFile>> {
    let head_lines = read_head_lines(path, SESSION_HEAD_LINES)?;
    let tail_lines = read_tail_lines(path, SESSION_TAIL_BYTES)?;

    let mut session_id = None;
    let mut cwd = None;
    let mut model = None;
    let mut last_activity = file_modified_utc(path)?;

    for line in &head_lines {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        match value.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                let payload = value.get("payload").and_then(Value::as_object);
                if let Some(payload) = payload {
                    session_id = payload
                        .get("id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(session_id);
                    cwd = payload
                        .get("cwd")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(cwd);
                }
                if let Some(timestamp) = parse_timestamp(&value) {
                    last_activity = last_activity.max(timestamp);
                }
            }
            Some("turn_context") => {
                let payload = value.get("payload").and_then(Value::as_object);
                if let Some(payload) = payload {
                    cwd = payload
                        .get("cwd")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(cwd);
                    model = payload
                        .get("model")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(model);
                }
                if let Some(timestamp) = parse_timestamp(&value) {
                    last_activity = last_activity.max(timestamp);
                }
            }
            _ => {
                if let Some(timestamp) = parse_timestamp(&value) {
                    last_activity = last_activity.max(timestamp);
                }
            }
        }
    }

    let Some(session_id) = session_id else {
        return Ok(None);
    };

    let mut last_assistant_message = None;
    let mut latest_task_signal = None;
    for line in tail_lines.iter().rev() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if let Some(timestamp) = parse_timestamp(&value) {
            last_activity = last_activity.max(timestamp);
        }

        if last_assistant_message.is_none() {
            last_assistant_message = extract_task_complete_message(&value)
                .or_else(|| extract_agent_message(&value))
                .or_else(|| extract_assistant_output(&value));
        }

        if latest_task_signal.is_none() {
            latest_task_signal = extract_task_signal(&value);
        }
    }

    Ok(Some(ParsedSessionFile {
        session_id,
        cwd,
        model,
        last_activity,
        last_assistant_message,
        latest_task_signal,
    }))
}

fn build_session_record(
    parsed: &ParsedSessionFile,
    history: &HashMap<String, HistoryEntry>,
) -> SessionRecord {
    let mut last_activity = parsed.last_activity;
    let last_user_prompt = history.get(&parsed.session_id).map(|entry| {
        last_activity = last_activity.max(entry.timestamp);
        entry.text.clone()
    });

    let now = Utc::now();
    let status = match parsed.latest_task_signal {
        Some(TaskSignal {
            marker: TaskMarker::Started,
            timestamp,
        }) if (now - timestamp).num_seconds() <= ACTIVE_WINDOW_SECS => AgentStatus::Processing,
        Some(TaskSignal {
            marker: TaskMarker::Started,
            ..
        }) => AgentStatus::Idle,
        Some(TaskSignal {
            marker: TaskMarker::Complete,
            ..
        }) => AgentStatus::Idle,
        Some(TaskSignal {
            marker: TaskMarker::Aborted,
            ..
        }) => AgentStatus::Idle,
        None if (now - last_activity).num_seconds() <= ACTIVE_WINDOW_SECS => {
            AgentStatus::Processing
        }
        None => AgentStatus::Idle,
    };

    SessionRecord {
        session_id: parsed.session_id.clone(),
        source: "codex".to_string(),
        project_name: parsed
            .cwd
            .as_ref()
            .and_then(|value| project_name_from_cwd(value)),
        cwd: parsed.cwd.clone(),
        model: parsed.model.clone(),
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
        status,
        current_tool: None,
        tool_description: None,
        last_user_prompt,
        last_assistant_message: parsed.last_assistant_message.clone(),
        tool_history: Vec::new(),
        last_activity,
    }
}

fn project_name_from_cwd(cwd: &str) -> Option<String> {
    let trimmed = cwd.trim_end_matches(['\\', '/']);
    trimmed
        .rsplit(['\\', '/'])
        .next()
        .map(ToOwned::to_owned)
        .filter(|value| !value.is_empty())
}

fn recent_session_files(root: &Path, limit: usize) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_session_files(root, &mut files)?;
    files.sort_by(|a, b| modified_key(b).cmp(&modified_key(a)));
    files.truncate(limit);
    Ok(files)
}

fn collect_session_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry =
            entry.with_context(|| format!("failed to read entry under {}", root.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", path.display()))?;

        if file_type.is_dir() {
            collect_session_files(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
    Ok(())
}

fn modified_key(path: &Path) -> DateTime<Utc> {
    file_modified_utc(path).unwrap_or_else(|_| DateTime::<Utc>::from(std::time::UNIX_EPOCH))
}

fn file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .len())
}

fn load_history(path: &Path) -> Result<std::collections::HashMap<String, HistoryEntry>> {
    if !path.exists() {
        return Ok(std::collections::HashMap::new());
    }

    let reader = BufReader::new(
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?,
    );
    let mut history = std::collections::HashMap::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read {}", path.display()))?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        let Some(session_id) = value.get("session_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(text) = value.get("text").and_then(Value::as_str) else {
            continue;
        };
        let Some(timestamp) = value
            .get("ts")
            .and_then(Value::as_i64)
            .and_then(|value| DateTime::<Utc>::from_timestamp(value, 0))
        else {
            continue;
        };

        history.insert(
            session_id.to_string(),
            HistoryEntry {
                timestamp,
                text: text.to_string(),
            },
        );
    }

    Ok(history)
}

fn refresh_history_state(path: &Path, state: &mut HistoryScanState) -> Result<()> {
    if !path.exists() {
        *state = HistoryScanState::default();
        return Ok(());
    }

    let modified_at = Some(file_modified_utc(path)?);
    let size = file_size(path)?;
    let truncated = size < state.offset;
    let unchanged = !truncated && state.size == size && state.modified_at == modified_at;

    if unchanged {
        return Ok(());
    }

    if truncated || state.offset == 0 {
        state.latest_prompt_by_session = load_history(path)?;
        state.offset = size;
        state.size = size;
        state.modified_at = modified_at;
        return Ok(());
    }

    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    file.seek(SeekFrom::Start(state.offset))
        .with_context(|| format!("failed to seek {}", path.display()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read {}", path.display()))?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        let Some(session_id) = value.get("session_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(text) = value.get("text").and_then(Value::as_str) else {
            continue;
        };
        let Some(timestamp) = value
            .get("ts")
            .and_then(Value::as_i64)
            .and_then(|value| DateTime::<Utc>::from_timestamp(value, 0))
        else {
            continue;
        };

        state.latest_prompt_by_session.insert(
            session_id.to_string(),
            HistoryEntry {
                timestamp,
                text: text.to_string(),
            },
        );
    }

    state.offset = size;
    state.size = size;
    state.modified_at = modified_at;
    Ok(())
}

fn read_head_lines(path: &Path, limit: usize) -> Result<Vec<String>> {
    let reader = BufReader::new(
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?,
    );
    reader
        .lines()
        .take(limit)
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("failed to read {}", path.display()))
}

fn read_tail_lines(path: &Path, max_bytes: u64) -> Result<Vec<String>> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let file_len = file
        .metadata()
        .with_context(|| format!("failed to stat {}", path.display()))?
        .len();
    let start = file_len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))
        .with_context(|| format!("failed to seek {}", path.display()))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let text = String::from_utf8_lossy(&buffer);
    let mut lines = text.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    if start > 0 && !lines.is_empty() {
        lines.remove(0);
    }
    Ok(lines)
}

fn file_modified_utc(path: &Path) -> Result<DateTime<Utc>> {
    let modified = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .modified()
        .with_context(|| format!("failed to read modified time for {}", path.display()))?;
    Ok(DateTime::<Utc>::from(modified))
}

fn parse_timestamp(value: &Value) -> Option<DateTime<Utc>> {
    value
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
}

fn extract_agent_message(value: &Value) -> Option<String> {
    let payload = value.get("payload")?;
    if value.get("type").and_then(Value::as_str) != Some("event_msg") {
        return None;
    }
    if payload.get("type").and_then(Value::as_str) != Some("agent_message") {
        return None;
    }
    payload
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn extract_task_signal(value: &Value) -> Option<TaskSignal> {
    if value.get("type").and_then(Value::as_str) == Some("turn_aborted") {
        let timestamp = parse_timestamp(value)?;
        return Some(TaskSignal {
            marker: TaskMarker::Aborted,
            timestamp,
        });
    }

    let payload = value.get("payload")?;
    if value.get("type").and_then(Value::as_str) != Some("event_msg") {
        return None;
    }

    let timestamp = parse_timestamp(value)?;

    match payload.get("type").and_then(Value::as_str) {
        Some("task_started") => Some(TaskSignal {
            marker: TaskMarker::Started,
            timestamp,
        }),
        Some("task_complete") => Some(TaskSignal {
            marker: TaskMarker::Complete,
            timestamp,
        }),
        _ => None,
    }
}

fn extract_task_complete_message(value: &Value) -> Option<String> {
    let payload = value.get("payload")?;
    if value.get("type").and_then(Value::as_str) != Some("event_msg") {
        return None;
    }
    if payload.get("type").and_then(Value::as_str) != Some("task_complete") {
        return None;
    }

    payload
        .get("last_agent_message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn extract_assistant_output(value: &Value) -> Option<String> {
    let payload = value.get("payload")?;
    if value.get("type").and_then(Value::as_str) != Some("response_item") {
        return None;
    }
    if payload.get("type").and_then(Value::as_str) != Some("message") {
        return None;
    }
    if payload.get("role").and_then(Value::as_str) != Some("assistant") {
        return None;
    }

    payload
        .get("content")
        .and_then(Value::as_array)
        .and_then(|content| {
            content.iter().find_map(|item| {
                item.get("text")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
}
