use std::path::PathBuf;

use echoisland_paths::{
    bridge_binary_path, bridge_binary_path_from_home, claude_config_dir,
    claude_config_dir_from_home, claude_projects_dir, claude_projects_dir_from_home,
    claude_settings_path, claude_settings_path_from_home, echoisland_bin_dir,
    echoisland_bin_dir_from_home, user_home_dir,
};
use serde::Serialize;

use crate::{AdapterPath, AdapterStatus, InstallableAdapter, SessionScanningAdapter};

mod install;
mod scan;

pub use install::{get_claude_status, install_claude_adapter};
pub use scan::{ClaudeSessionScanner, scan_claude_sessions};

#[derive(Debug, Clone)]
pub struct ClaudeAdapter {
    paths: ClaudePaths,
}

#[derive(Debug, Clone)]
pub struct ClaudePaths {
    pub home_dir: PathBuf,
    pub claude_dir: PathBuf,
    pub settings_path: PathBuf,
    pub projects_dir: PathBuf,
    pub hook_script_path: PathBuf,
    pub bridge_install_dir: PathBuf,
    pub bridge_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaudeStatus {
    pub claude_dir_exists: bool,
    pub bridge_exists: bool,
    pub hooks_installed: bool,
    pub live_capture_supported: bool,
    pub live_capture_ready: bool,
    pub status_note: Option<String>,
    pub claude_dir: String,
    pub settings_path: String,
    pub projects_dir: String,
    pub bridge_path: String,
}

impl ClaudePaths {
    pub fn from_home(home_dir: impl Into<PathBuf>) -> Self {
        let home_dir = home_dir.into();
        let claude_dir = claude_config_dir_from_home(&home_dir);
        let settings_path = claude_settings_path_from_home(&home_dir);
        let projects_dir = claude_projects_dir_from_home(&home_dir);
        let hook_script_path = claude_dir.join("hooks").join("echoisland-hook.sh");
        let bridge_install_dir = echoisland_bin_dir_from_home(&home_dir);
        let bridge_path = bridge_binary_path_from_home(&home_dir);
        Self {
            home_dir,
            claude_dir,
            settings_path,
            projects_dir,
            hook_script_path,
            bridge_install_dir,
            bridge_path,
        }
    }
}

impl ClaudeAdapter {
    pub fn new(paths: ClaudePaths) -> Self {
        Self { paths }
    }

    pub fn with_default_paths() -> Self {
        Self::new(default_paths())
    }
}

pub fn default_paths() -> ClaudePaths {
    ClaudePaths {
        home_dir: user_home_dir(),
        claude_dir: claude_config_dir(),
        settings_path: claude_settings_path(),
        projects_dir: claude_projects_dir(),
        hook_script_path: claude_config_dir().join("hooks").join("echoisland-hook.sh"),
        bridge_install_dir: echoisland_bin_dir(),
        bridge_path: bridge_binary_path(),
    }
}

impl From<ClaudeStatus> for AdapterStatus {
    fn from(value: ClaudeStatus) -> Self {
        Self {
            adapter: "claude".to_string(),
            config_dir_exists: value.claude_dir_exists,
            bridge_exists: value.bridge_exists,
            hooks_installed: value.hooks_installed,
            hooks_enabled: value.hooks_installed,
            live_capture_supported: value.live_capture_supported,
            live_capture_ready: value.live_capture_ready,
            status_note: value.status_note,
            paths: vec![
                AdapterPath {
                    label: "claude_dir".to_string(),
                    path: value.claude_dir,
                },
                AdapterPath {
                    label: "settings_path".to_string(),
                    path: value.settings_path,
                },
                AdapterPath {
                    label: "projects_dir".to_string(),
                    path: value.projects_dir,
                },
                AdapterPath {
                    label: "bridge_path".to_string(),
                    path: value.bridge_path,
                },
            ],
        }
    }
}

impl InstallableAdapter for ClaudeAdapter {
    fn adapter_id(&self) -> &'static str {
        "claude"
    }

    fn status(&self) -> anyhow::Result<AdapterStatus> {
        Ok(get_claude_status(&self.paths)?.into())
    }

    fn install(&self, source_bridge: &std::path::Path) -> anyhow::Result<AdapterStatus> {
        Ok(install_claude_adapter(&self.paths, source_bridge)?.into())
    }
}

impl SessionScanningAdapter for ClaudeAdapter {
    fn adapter_id(&self) -> &'static str {
        "claude"
    }

    fn scan_sessions(&self) -> anyhow::Result<Vec<echoisland_core::SessionRecord>> {
        scan_claude_sessions(&self.paths)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use chrono::{Duration, Utc};
    use echoisland_core::AgentStatus;
    use echoisland_paths::bridge_binary_name;

    use crate::InstallableAdapter;

    use super::{
        ClaudeAdapter, ClaudePaths, get_claude_status, install_claude_adapter, scan_claude_sessions,
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("echoisland-claude-{suffix}-{counter}"))
    }

    #[test]
    fn installs_claude_hooks_into_settings() {
        let root = temp_root();
        let bridge_source = root.join(bridge_binary_name());
        fs::create_dir_all(root.join(".claude")).unwrap();
        fs::write(&bridge_source, b"bridge-binary").unwrap();

        let paths = ClaudePaths::from_home(&root);
        let status = install_claude_adapter(&paths, &bridge_source).unwrap();

        assert!(status.claude_dir_exists);
        assert!(status.bridge_exists);
        assert!(status.hooks_installed);
        assert!(status.live_capture_ready);

        let status2 = get_claude_status(&paths).unwrap();
        assert!(status2.hooks_installed);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn claude_adapter_exposes_generic_status() {
        let root = temp_root();
        let bridge_source = root.join(bridge_binary_name());
        fs::create_dir_all(root.join(".claude")).unwrap();
        fs::write(&bridge_source, b"bridge-binary").unwrap();

        let adapter = ClaudeAdapter::new(ClaudePaths::from_home(&root));
        let status = adapter.install(&bridge_source).unwrap();

        assert_eq!(InstallableAdapter::adapter_id(&adapter), "claude");
        assert!(status.config_dir_exists);
        assert!(status.hooks_installed);
        assert!(
            status
                .paths
                .iter()
                .any(|entry| entry.label == "settings_path")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scans_claude_transcripts() {
        let root = temp_root();
        let projects_dir = root.join(".claude").join("projects").join("C--Users-Adim");
        fs::create_dir_all(&projects_dir).unwrap();
        let session_path = projects_dir.join("session-001.jsonl");
        let recent_time = (Utc::now() - Duration::seconds(20)).to_rfc3339();
        fs::write(
            &session_path,
            format!(
                "{{\"timestamp\":\"{recent_time}\",\"cwd\":\"C:/Users/Adim\",\"type\":\"user\",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"text\",\"text\":\"Add login page\"}}]}},\"entrypoint\":\"claude-vscode\",\"sessionId\":\"session-001\"}}\n{{\"timestamp\":\"{recent_time}\",\"type\":\"ai-title\",\"aiTitle\":\"Login implementation\"}}\n{{\"timestamp\":\"{recent_time}\",\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"text\",\"text\":\"I am wiring the login form now.\"}}]}}}}\n"
            ),
        )
        .unwrap();

        let paths = ClaudePaths::from_home(&root);
        let sessions = scan_claude_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.session_id, "session-001");
        assert_eq!(session.source, "claude");
        assert_eq!(
            session.project_name.as_deref(),
            Some("Login implementation")
        );
        assert_eq!(session.last_user_prompt.as_deref(), Some("Add login page"));
        assert_eq!(
            session.last_assistant_message.as_deref(),
            Some("I am wiring the login form now.")
        );
        assert_eq!(session.host_app.as_deref(), Some("claude-vscode"));
        assert_eq!(session.status, AgentStatus::Idle);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scans_completed_claude_transcript_with_last_prompt_metadata_as_idle() {
        let root = temp_root();
        let projects_dir = root.join(".claude").join("projects").join("C--Users-Adim");
        fs::create_dir_all(&projects_dir).unwrap();
        let session_path = projects_dir.join("session-last-prompt.jsonl");
        let recent_time = (Utc::now() - Duration::seconds(15)).to_rfc3339();
        fs::write(
            &session_path,
            format!(
                "{{\"timestamp\":\"{recent_time}\",\"cwd\":\"C:/Users/Adim/project\",\"type\":\"user\",\"message\":{{\"role\":\"user\",\"content\":\"Summarize changes\"}},\"sessionId\":\"session-last-prompt\"}}\n{{\"timestamp\":\"{recent_time}\",\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"thinking\",\"thinking\":\"...\"}}]}}}}\n{{\"timestamp\":\"{recent_time}\",\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"text\",\"text\":\"Done.\"}}]}}}}\n{{\"type\":\"last-prompt\",\"lastPrompt\":\"Summarize changes\"}}\n"
            ),
        )
        .unwrap();

        let paths = ClaudePaths::from_home(&root);
        let sessions = scan_claude_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.status, AgentStatus::Idle);
        assert_eq!(
            session.last_user_prompt.as_deref(),
            Some("Summarize changes")
        );
        assert_eq!(session.last_assistant_message.as_deref(), Some("Done."));
        assert_eq!(session.current_tool, None);
        assert_eq!(session.tool_description, None);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scans_running_claude_tool_use() {
        let root = temp_root();
        let projects_dir = root.join(".claude").join("projects").join("C--Users-Adim");
        fs::create_dir_all(&projects_dir).unwrap();
        let session_path = projects_dir.join("session-002.jsonl");
        let recent_time = (Utc::now() - Duration::seconds(15)).to_rfc3339();
        fs::write(
            &session_path,
            format!(
                "{{\"timestamp\":\"{recent_time}\",\"cwd\":\"C:/Users/Adim/project\",\"type\":\"user\",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"text\",\"text\":\"Run tests\"}}]}},\"sessionId\":\"session-002\"}}\n{{\"timestamp\":\"{recent_time}\",\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"tool_use\",\"name\":\"Bash\",\"input\":{{\"description\":\"cargo test\",\"command\":\"cargo test\"}}}}]}}}}\n"
            ),
        )
        .unwrap();

        let paths = ClaudePaths::from_home(&root);
        let sessions = scan_claude_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.status, AgentStatus::Running);
        assert_eq!(session.current_tool.as_deref(), Some("Bash"));
        assert_eq!(session.tool_description.as_deref(), Some("cargo test"));

        let _ = fs::remove_dir_all(root);
    }
}
