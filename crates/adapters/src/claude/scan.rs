use std::{
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
use tracing::debug;

use super::ClaudePaths;

const SESSION_SCAN_LIMIT: usize = 16;
const SESSION_HEAD_LINES: usize = 96;
const SESSION_TAIL_BYTES: u64 = 64 * 1024;
const ACTIVE_WINDOW_SECS: i64 = 300;
const ACTIVE_SCAN_INTERVAL_SECS: u64 = 3;
const IDLE_SCAN_INTERVAL_SECS: u64 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSessionFile {
    session_id: String,
    cwd: Option<String>,
    model: Option<String>,
    last_activity: DateTime<Utc>,
    last_user_prompt: Option<String>,
    last_assistant_message: Option<String>,
    project_name: Option<String>,
    host_app: Option<String>,
    current_tool: Option<String>,
    tool_description: Option<String>,
    last_event_kind: ClaudeEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaudeEventKind {
    Unknown,
    UserPrompt,
    AssistantMessage,
    ToolUse,
    ToolResult,
}

#[derive(Debug, Clone)]
pub struct ClaudeSessionScanner {
    paths: ClaudePaths,
    last_sessions: Vec<SessionRecord>,
}

impl ClaudeSessionScanner {
    pub fn new(paths: ClaudePaths) -> Self {
        Self {
            paths,
            last_sessions: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> Result<Option<Vec<SessionRecord>>> {
        let session_paths = recent_session_files(&self.paths.projects_dir, SESSION_SCAN_LIMIT)?;
        let mut sessions = session_paths
            .iter()
            .filter_map(|path| match parse_session_file(path) {
                Ok(Some(session)) => Some(session),
                Ok(None) => None,
                Err(error) => {
                    debug!(
                        path = %path.display(),
                        error = %error,
                        "skipping unreadable Claude session file"
                    );
                    None
                }
            })
            .into_iter()
            .map(build_session_record)
            .collect::<Vec<_>>();

        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

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

pub fn scan_claude_sessions(paths: &ClaudePaths) -> Result<Vec<SessionRecord>> {
    let mut scanner = ClaudeSessionScanner::new(paths.clone());
    Ok(scanner.scan()?.unwrap_or_default())
}

fn parse_session_file(path: &Path) -> Result<Option<ParsedSessionFile>> {
    let head_lines = read_head_lines(path, SESSION_HEAD_LINES)?;
    let tail_lines = read_tail_lines(path, SESSION_TAIL_BYTES)?;

    let mut session_id = path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned);
    let mut cwd = None;
    let mut model = None;
    let mut last_activity = file_modified_utc(path)?;
    let mut last_user_prompt = None;
    let mut last_assistant_message = None;
    let mut project_name = None;
    let mut host_app = None;
    let mut current_tool = None;
    let mut tool_description = None;
    let mut last_event_kind = ClaudeEventKind::Unknown;

    for line in head_lines.iter().chain(tail_lines.iter()) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if let Some(timestamp) = parse_timestamp(&value) {
            last_activity = last_activity.max(timestamp);
        }

        if session_id.is_none() {
            session_id = extract_session_id(&value);
        }
        if cwd.is_none() {
            cwd = extract_cwd(&value);
        }
        if model.is_none() {
            model = extract_model(&value);
        }
        if project_name.is_none() {
            project_name = extract_project_name(&value);
        }
        if host_app.is_none() {
            host_app = extract_host_app(&value);
        }
    }

    for line in &tail_lines {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if let Some(prompt) = extract_last_prompt(&value) {
            last_user_prompt = Some(prompt);
        }

        if let Some(prompt) = extract_user_prompt(&value) {
            last_user_prompt = Some(prompt);
            last_event_kind = ClaudeEventKind::UserPrompt;
        }

        if let Some(message) = extract_assistant_message(&value) {
            last_assistant_message = Some(message);
            last_event_kind = ClaudeEventKind::AssistantMessage;
        }

        if let Some(tool_use) = extract_assistant_tool_use(&value) {
            current_tool = Some(tool_use.name);
            tool_description = tool_use.description;
            last_event_kind = ClaudeEventKind::ToolUse;
        }

        if extract_tool_result(&value).is_some() {
            last_event_kind = ClaudeEventKind::ToolResult;
        }
    }

    let Some(session_id) = session_id else {
        return Ok(None);
    };

    let project_name = project_name
        .or_else(|| cwd.as_deref().and_then(project_name_from_cwd))
        .or_else(|| decode_project_name_from_path(path))
        .or_else(|| {
            path.parent()
                .and_then(|value| value.file_name())
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        });

    Ok(Some(ParsedSessionFile {
        session_id,
        cwd,
        model,
        last_activity,
        last_user_prompt,
        last_assistant_message,
        project_name,
        host_app,
        current_tool,
        tool_description,
        last_event_kind,
    }))
}

fn build_session_record(parsed: ParsedSessionFile) -> SessionRecord {
    let now = Utc::now();
    let recent = (now - parsed.last_activity).num_seconds() <= ACTIVE_WINDOW_SECS;
    let status = if !recent {
        AgentStatus::Idle
    } else {
        match parsed.last_event_kind {
            ClaudeEventKind::ToolUse => AgentStatus::Running,
            ClaudeEventKind::UserPrompt | ClaudeEventKind::ToolResult => AgentStatus::Processing,
            ClaudeEventKind::AssistantMessage | ClaudeEventKind::Unknown => AgentStatus::Idle,
        }
    };

    SessionRecord {
        session_id: parsed.session_id,
        source: "claude".to_string(),
        project_name: parsed.project_name,
        cwd: parsed.cwd,
        model: parsed.model,
        terminal_app: None,
        terminal_bundle: None,
        host_app: parsed.host_app.or(Some("claude".to_string())),
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
        current_tool: if status == AgentStatus::Running {
            parsed.current_tool
        } else {
            None
        },
        tool_description: if status == AgentStatus::Running {
            parsed.tool_description
        } else {
            None
        },
        last_user_prompt: parsed.last_user_prompt,
        last_assistant_message: parsed.last_assistant_message,
        tool_history: Vec::new(),
        last_activity: parsed.last_activity,
    }
}

fn parse_timestamp(value: &Value) -> Option<DateTime<Utc>> {
    value
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            value
                .get("created_at")
                .and_then(Value::as_str)
                .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
                .map(|value| value.with_timezone(&Utc))
        })
}

fn extract_session_id(value: &Value) -> Option<String> {
    value
        .get("session_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("sessionId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn extract_cwd(value: &Value) -> Option<String> {
    value
        .get("cwd")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("project_path")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .or_else(|| {
            value
                .get("workspace")
                .and_then(Value::as_object)
                .and_then(|workspace| workspace.get("cwd"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn extract_model(value: &Value) -> Option<String> {
    value
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("model")
                .and_then(Value::as_object)
                .and_then(|model| {
                    model
                        .get("display_name")
                        .or_else(|| model.get("id"))
                        .or_else(|| model.get("name"))
                })
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn extract_host_app(value: &Value) -> Option<String> {
    value
        .get("entrypoint")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn extract_project_name(value: &Value) -> Option<String> {
    value
        .get("aiTitle")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("ai_title")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn extract_last_prompt(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("last-prompt") {
        return None;
    }
    value
        .get("lastPrompt")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn extract_user_prompt(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("user") {
        return None;
    }
    let message = value.get("message")?.as_object()?;
    if message.get("role").and_then(Value::as_str) != Some("user") {
        return None;
    }
    extract_message_content_text(message.get("content")?)
}

fn extract_assistant_message(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("assistant") {
        return None;
    }
    let message = value.get("message")?.as_object()?;
    if message.get("role").and_then(Value::as_str) != Some("assistant") {
        return None;
    }
    extract_message_content_text(message.get("content")?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolUseEvent {
    name: String,
    description: Option<String>,
}

fn extract_assistant_tool_use(value: &Value) -> Option<ToolUseEvent> {
    if value.get("type").and_then(Value::as_str) != Some("assistant") {
        return None;
    }
    let message = value.get("message")?.as_object()?;
    let content = message.get("content")?.as_array()?;
    content.iter().rev().find_map(|item| {
        let object = item.as_object()?;
        if object.get("type").and_then(Value::as_str) != Some("tool_use") {
            return None;
        }
        let name = object
            .get("name")
            .or_else(|| object.get("tool_name"))
            .and_then(Value::as_str)?;
        let description = object
            .get("input")
            .and_then(Value::as_object)
            .and_then(|input| {
                input
                    .get("description")
                    .or_else(|| input.get("command"))
                    .and_then(Value::as_str)
            })
            .map(ToOwned::to_owned);
        Some(ToolUseEvent {
            name: name.to_string(),
            description,
        })
    })
}

fn extract_tool_result(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("user") {
        return None;
    }
    let message = value.get("message")?.as_object()?;
    let content = message.get("content")?.as_array()?;
    content.iter().find_map(|item| {
        let object = item.as_object()?;
        if object.get("type").and_then(Value::as_str) != Some("tool_result") {
            return None;
        }
        object
            .get("content")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    })
}

fn extract_message_content_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Value::Array(items) => {
            let joined = items
                .iter()
                .filter_map(|item| {
                    let object = item.as_object()?;
                    match object.get("type").and_then(Value::as_str) {
                        Some("text") => object
                            .get("text")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|text| !text.is_empty())
                            .map(ToOwned::to_owned),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            if joined.trim().is_empty() {
                None
            } else {
                Some(joined)
            }
        }
        Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            object.values().find_map(extract_message_content_text)
        }
        _ => None,
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

fn decode_project_name_from_path(path: &Path) -> Option<String> {
    let project_dir = path.parent()?.file_name()?.to_str()?;
    if project_dir.eq_ignore_ascii_case("subagents") {
        return path
            .parent()?
            .parent()?
            .file_name()?
            .to_str()
            .and_then(decode_claude_project_dir);
    }
    decode_claude_project_dir(project_dir)
}

fn decode_claude_project_dir(raw: &str) -> Option<String> {
    let decoded = raw.replace("--", "://").replace('-', "\\");
    project_name_from_cwd(&decoded)
}

fn recent_session_files(root: &Path, limit: usize) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_session_files(root, &mut files)?;
    files.sort_by(|left, right| {
        let left_time = file_modified_utc(left).unwrap_or_else(|_| Utc::now());
        let right_time = file_modified_utc(right).unwrap_or_else(|_| Utc::now());
        right_time.cmp(&left_time)
    });
    files.truncate(limit);
    Ok(files)
}

fn collect_session_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry =
            entry.with_context(|| format!("failed to read entry under {}", root.display()))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        if metadata.is_dir() {
            collect_session_files(&path, files)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("jsonl"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn file_modified_utc(path: &Path) -> Result<DateTime<Utc>> {
    let metadata =
        fs::metadata(path).with_context(|| format!("failed to stat {}", path.display()))?;
    let modified = metadata
        .modified()
        .with_context(|| format!("failed to read modified time for {}", path.display()))?;
    Ok(DateTime::<Utc>::from(modified))
}

fn read_head_lines(path: &Path, limit: usize) -> Result<Vec<String>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .take(limit)
        .filter_map(|line| line.ok())
        .collect())
}

fn read_tail_lines(path: &Path, max_bytes: u64) -> Result<Vec<String>> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let len = file
        .metadata()
        .with_context(|| format!("failed to stat {}", path.display()))?
        .len();
    let seek_to = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(seek_to))
        .with_context(|| format!("failed to seek {}", path.display()))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let buffer = String::from_utf8_lossy(&buffer);

    let mut lines = buffer.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    if seek_to > 0 && !lines.is_empty() {
        lines.remove(0);
    }
    Ok(lines)
}
