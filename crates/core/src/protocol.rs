use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::normalize_event_name;

pub const PROTOCOL_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventMetadata {
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
    pub pid: Option<u32>,
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
    #[serde(default)]
    pub workspace_roots: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionChoice {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionPayload {
    #[serde(default)]
    pub header: Option<String>,
    pub text: String,
    #[serde(default)]
    pub options: Vec<QuestionChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub protocol_version: String,
    pub hook_event_name: String,
    pub session_id: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<Value>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<EventMetadata>,
    #[serde(default)]
    pub question: Option<QuestionPayload>,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("empty session_id")]
    EmptySessionId,
    #[error("empty source")]
    EmptySource,
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(String),
}

impl EventEnvelope {
    pub fn normalized_event_name(&self) -> String {
        normalize_event_name(&self.hook_event_name)
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(
                self.protocol_version.clone(),
            ));
        }
        if self.session_id.trim().is_empty() {
            return Err(ProtocolError::EmptySessionId);
        }
        if self.source.trim().is_empty() {
            return Err(ProtocolError::EmptySource);
        }
        Ok(())
    }

    pub fn project_name(&self) -> Option<String> {
        self.cwd.as_ref().and_then(|cwd| {
            let trimmed = cwd.trim_end_matches(['\\', '/']);
            trimmed
                .rsplit(['\\', '/'])
                .next()
                .map(|part| part.to_string())
                .filter(|part| !part.is_empty())
        })
    }

    pub fn tool_description(&self) -> Option<String> {
        let input = self.tool_input.as_ref()?;
        if let Some(description) = input.get("description").and_then(Value::as_str) {
            if !description.is_empty() {
                return Some(description.to_string());
            }
        }
        if let Some(command) = input.get("command").and_then(Value::as_str) {
            let first_line = command.lines().next().unwrap_or(command).trim();
            if !first_line.is_empty() {
                return Some(first_line.chars().take(80).collect());
            }
        }
        if let Some(file_path) = input.get("file_path").and_then(Value::as_str) {
            let name = file_path
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(file_path)
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
        if let Some(pattern) = input.get("pattern").and_then(Value::as_str) {
            return Some(pattern.to_string());
        }
        if let Some(prompt) = input.get("prompt").and_then(Value::as_str) {
            let trimmed = prompt.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.chars().take(80).collect());
            }
        }
        self.message.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecisionPayload {
    pub behavior: String,
    #[serde(default)]
    pub updated_permissions: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnswerPayload {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecisionResponse {
    pub behavior: String,
    #[serde(default)]
    pub updated_permissions: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnswerResponse {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub decision: Option<DecisionResponse>,
    #[serde(default)]
    pub answer: Option<AnswerResponse>,
}

impl ResponseEnvelope {
    pub fn ok() -> Self {
        Self {
            ok: true,
            error: None,
            decision: None,
            answer: None,
        }
    }

    pub fn parse_failed() -> Self {
        Self::error("parse_failed")
    }

    pub fn invalid_payload() -> Self {
        Self::error("invalid_payload")
    }

    pub fn error(code: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(code.into()),
            decision: None,
            answer: None,
        }
    }

    pub fn allow() -> Self {
        Self {
            ok: true,
            error: None,
            decision: Some(DecisionResponse {
                behavior: "allow".to_string(),
                updated_permissions: None,
            }),
            answer: None,
        }
    }

    pub fn deny() -> Self {
        Self {
            ok: true,
            error: None,
            decision: Some(DecisionResponse {
                behavior: "deny".to_string(),
                updated_permissions: None,
            }),
            answer: None,
        }
    }

    pub fn answer(value: impl Into<String>) -> Self {
        Self {
            ok: true,
            error: None,
            decision: None,
            answer: Some(AnswerResponse {
                value: Some(value.into()),
                skipped: false,
            }),
        }
    }

    pub fn skipped() -> Self {
        Self {
            ok: true,
            error: None,
            decision: None,
            answer: Some(AnswerResponse {
                value: None,
                skipped: true,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{EventEnvelope, PROTOCOL_VERSION};

    #[test]
    fn event_validation_requires_non_empty_fields() {
        let event = EventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            hook_event_name: "SessionStart".to_string(),
            session_id: "abc".to_string(),
            source: "codex".to_string(),
            timestamp: Utc::now(),
            tool_name: None,
            tool_input: None,
            cwd: None,
            model: None,
            message: None,
            agent_id: None,
            metadata: None,
            question: None,
        };
        assert!(event.validate().is_ok());
    }

    #[test]
    fn tool_description_prefers_explicit_description() {
        let event = EventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            hook_event_name: "PreToolUse".to_string(),
            session_id: "abc".to_string(),
            source: "claude".to_string(),
            timestamp: Utc::now(),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(json!({
                "description": "Run tests",
                "command": "cargo test"
            })),
            cwd: None,
            model: None,
            message: None,
            agent_id: None,
            metadata: None,
            question: None,
        };
        assert_eq!(event.tool_description().as_deref(), Some("Run tests"));
    }
}
