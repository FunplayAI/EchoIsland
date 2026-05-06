use std::path::PathBuf;

use echoisland_paths::{
    bridge_binary_path, bridge_binary_path_from_home, codex_config_path,
    codex_config_path_from_home, codex_dir, codex_dir_from_home, codex_hooks_path,
    codex_hooks_path_from_home, echoisland_bin_dir, echoisland_bin_dir_from_home, user_home_dir,
};
use serde::Serialize;

use crate::{AdapterPath, AdapterStatus, InstallableAdapter, SessionScanningAdapter};

mod install;
mod scan;

pub use install::{get_codex_status, install_codex_adapter};
pub use scan::{CodexSessionScanner, scan_codex_sessions};

#[derive(Debug, Clone)]
pub struct CodexAdapter {
    paths: CodexPaths,
}

#[derive(Debug, Clone)]
pub struct CodexPaths {
    pub home_dir: PathBuf,
    pub codex_dir: PathBuf,
    pub hooks_path: PathBuf,
    pub config_path: PathBuf,
    pub bridge_install_dir: PathBuf,
    pub bridge_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodexStatus {
    pub codex_dir_exists: bool,
    pub bridge_exists: bool,
    pub hooks_installed: bool,
    pub codex_hooks_enabled: bool,
    pub live_capture_supported: bool,
    pub live_capture_ready: bool,
    pub status_note: Option<String>,
    pub codex_dir: String,
    pub hooks_path: String,
    pub config_path: String,
    pub bridge_path: String,
}

impl CodexPaths {
    pub fn from_home(home_dir: impl Into<PathBuf>) -> Self {
        let home_dir = home_dir.into();
        let codex_dir = codex_dir_from_home(&home_dir);
        let hooks_path = codex_hooks_path_from_home(&home_dir);
        let config_path = codex_config_path_from_home(&home_dir);
        let bridge_install_dir = echoisland_bin_dir_from_home(&home_dir);
        let bridge_path = bridge_binary_path_from_home(&home_dir);
        Self {
            home_dir,
            codex_dir,
            hooks_path,
            config_path,
            bridge_install_dir,
            bridge_path,
        }
    }
}

impl CodexAdapter {
    pub fn new(paths: CodexPaths) -> Self {
        Self { paths }
    }

    pub fn with_default_paths() -> Self {
        Self::new(default_paths())
    }

    pub fn paths(&self) -> &CodexPaths {
        &self.paths
    }
}

pub fn default_paths() -> CodexPaths {
    CodexPaths {
        home_dir: user_home_dir(),
        codex_dir: codex_dir(),
        hooks_path: codex_hooks_path(),
        config_path: codex_config_path(),
        bridge_install_dir: echoisland_bin_dir(),
        bridge_path: bridge_binary_path(),
    }
}

impl From<CodexStatus> for AdapterStatus {
    fn from(value: CodexStatus) -> Self {
        Self {
            adapter: "codex".to_string(),
            config_dir_exists: value.codex_dir_exists,
            bridge_exists: value.bridge_exists,
            hooks_installed: value.hooks_installed,
            hooks_enabled: value.codex_hooks_enabled,
            live_capture_supported: value.live_capture_supported,
            live_capture_ready: value.live_capture_ready,
            status_note: value.status_note,
            paths: vec![
                AdapterPath {
                    label: "codex_dir".to_string(),
                    path: value.codex_dir,
                },
                AdapterPath {
                    label: "hooks_path".to_string(),
                    path: value.hooks_path,
                },
                AdapterPath {
                    label: "config_path".to_string(),
                    path: value.config_path,
                },
                AdapterPath {
                    label: "bridge_path".to_string(),
                    path: value.bridge_path,
                },
            ],
        }
    }
}

impl InstallableAdapter for CodexAdapter {
    fn adapter_id(&self) -> &'static str {
        "codex"
    }

    fn status(&self) -> anyhow::Result<AdapterStatus> {
        Ok(get_codex_status(&self.paths)?.into())
    }

    fn install(&self, source_bridge: &std::path::Path) -> anyhow::Result<AdapterStatus> {
        Ok(install_codex_adapter(&self.paths, source_bridge)?.into())
    }
}

impl SessionScanningAdapter for CodexAdapter {
    fn adapter_id(&self) -> &'static str {
        "codex"
    }

    fn scan_sessions(&self) -> anyhow::Result<Vec<echoisland_core::SessionRecord>> {
        scan_codex_sessions(&self.paths)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use echoisland_core::AgentStatus;
    use echoisland_paths::bridge_binary_name;

    use crate::{InstallableAdapter, SessionScanningAdapter};

    use super::{
        CodexAdapter, CodexPaths, get_codex_status, install_codex_adapter, scan_codex_sessions,
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("echoisland-codex-{suffix}-{counter}"))
    }

    #[test]
    fn installs_codex_hooks_and_config() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        fs::create_dir_all(&codex_dir).unwrap();
        fs::write(
            codex_dir.join("config.toml"),
            "[features]\ncodex_hooks = true\n",
        )
        .unwrap();
        fs::write(
            codex_dir.join("hooks.json"),
            r#"{
              "hooks": {
                "UserPromptSubmit": [
                  { "hooks": [{ "command": "\"C:\\Users\\Adim\\.echoisland\\bin\\echoisland-hook-bridge.exe\" --source codex" }] },
                  { "hooks": [{ "command": "echo keep" }] }
                ]
              }
            }"#,
        )
        .unwrap();
        let bridge_source = root.join(bridge_binary_name());
        fs::write(&bridge_source, b"bridge-binary").unwrap();

        let paths = CodexPaths::from_home(&root);
        let status = install_codex_adapter(&paths, &bridge_source).unwrap();

        assert!(status.codex_dir_exists);
        assert!(status.bridge_exists);
        assert!(status.hooks_installed);
        assert!(status.codex_hooks_enabled);
        assert!(status.live_capture_ready);
        let hooks_raw = fs::read_to_string(paths.hooks_path.clone()).unwrap_or_default();
        assert!(hooks_raw.contains("--source codex"));
        assert!(!hooks_raw.contains("powershell.exe"));
        assert!(hooks_raw.contains("echo keep"));
        let config_raw = fs::read_to_string(paths.config_path.clone()).unwrap();
        assert!(config_raw.contains("codex_hooks = true"));

        let status2 = get_codex_status(&paths).unwrap();
        assert!(status2.hooks_installed);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn codex_adapter_exposes_generic_status() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        fs::create_dir_all(&codex_dir).unwrap();
        let bridge_source = root.join(bridge_binary_name());
        fs::write(&bridge_source, b"bridge-binary").unwrap();

        let adapter = CodexAdapter::new(CodexPaths::from_home(&root));
        let status = adapter.install(&bridge_source).unwrap();

        assert_eq!(InstallableAdapter::adapter_id(&adapter), "codex");
        assert!(status.config_dir_exists);
        assert!(status.hooks_installed);
        assert_eq!(
            status
                .paths
                .iter()
                .find(|entry| entry.label == "codex_dir")
                .is_some(),
            true
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scans_codex_session_files() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        let sessions_dir = codex_dir
            .join("sessions")
            .join("2026")
            .join("04")
            .join("09");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(
            codex_dir.join("history.jsonl"),
            concat!(
                "{\"session_id\":\"019-test-session\",\"ts\":1775738408,\"text\":\"latest prompt\"}\n"
            ),
        )
        .unwrap();

        let session_path = sessions_dir.join("rollout-2026-04-09T14-57-22-019-test-session.jsonl");
        fs::write(
            &session_path,
            concat!(
                "{\"timestamp\":\"2026-04-09T06:57:34.427Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"019-test-session\",\"cwd\":\"D:\\\\AI Island\\\\Repo\",\"originator\":\"codex-tui\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:57:34.432Z\",\"type\":\"turn_context\",\"payload\":{\"cwd\":\"D:\\\\AI Island\\\\Repo\",\"model\":\"gpt-5.4\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:58:00.000Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_message\",\"message\":\"assistant summary\"}}\n"
            ),
        )
        .unwrap();

        let paths = CodexPaths::from_home(&root);
        let sessions = scan_codex_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.session_id, "019-test-session");
        assert_eq!(session.source, "codex");
        assert_eq!(session.model.as_deref(), Some("gpt-5.4"));
        assert_eq!(session.project_name.as_deref(), Some("Repo"));
        assert_eq!(session.last_user_prompt.as_deref(), Some("latest prompt"));
        assert_eq!(
            session.last_assistant_message.as_deref(),
            Some("assistant summary")
        );
        assert_eq!(session.status, AgentStatus::Processing);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marks_session_idle_after_task_complete() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        let sessions_dir = codex_dir
            .join("sessions")
            .join("2026")
            .join("04")
            .join("09");
        fs::create_dir_all(&sessions_dir).unwrap();

        let session_path = sessions_dir.join("rollout-2026-04-09T14-57-22-019-complete.jsonl");
        fs::write(
            &session_path,
            concat!(
                "{\"timestamp\":\"2026-04-09T06:57:34.427Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"019-complete\",\"cwd\":\"D:\\\\AI Island\\\\Repo\",\"originator\":\"codex-tui\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:57:34.432Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:58:00.000Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_complete\",\"turn_id\":\"turn-1\",\"last_agent_message\":\"done\"}}\n"
            ),
        )
        .unwrap();

        let paths = CodexPaths::from_home(&root);
        let sessions = scan_codex_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.status, AgentStatus::Idle);
        assert_eq!(session.last_assistant_message.as_deref(), Some("done"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marks_session_idle_after_turn_aborted() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        let sessions_dir = codex_dir
            .join("sessions")
            .join("2026")
            .join("04")
            .join("09");
        fs::create_dir_all(&sessions_dir).unwrap();

        let session_path = sessions_dir.join("rollout-2026-04-09T14-57-22-019-aborted.jsonl");
        fs::write(
            &session_path,
            concat!(
                "{\"timestamp\":\"2026-04-09T06:57:34.427Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"019-aborted\",\"cwd\":\"D:\\\\AI Island\\\\Repo\",\"originator\":\"codex-tui\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:57:34.432Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:58:00.000Z\",\"type\":\"turn_aborted\",\"reason\":\"interrupted\"}\n"
            ),
        )
        .unwrap();

        let paths = CodexPaths::from_home(&root);
        let sessions = scan_codex_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.status, AgentStatus::Idle);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn marks_stale_started_session_idle_without_task_complete() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        let sessions_dir = codex_dir
            .join("sessions")
            .join("2026")
            .join("04")
            .join("09");
        fs::create_dir_all(&sessions_dir).unwrap();

        let session_path = sessions_dir.join("rollout-2026-04-09T14-57-22-019-started.jsonl");
        fs::write(
            &session_path,
            concat!(
                "{\"timestamp\":\"2026-04-09T06:57:34.427Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"019-started\",\"cwd\":\"D:\\\\AI Island\\\\Repo\",\"originator\":\"codex-tui\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:57:34.432Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:58:00.000Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_message\",\"message\":\"working\"}}\n"
            ),
        )
        .unwrap();

        let paths = CodexPaths::from_home(&root);
        let sessions = scan_codex_sessions(&paths).unwrap();

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.status, AgentStatus::Idle);
        assert_eq!(session.last_assistant_message.as_deref(), Some("working"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn codex_adapter_scans_sessions_via_generic_trait() {
        let root = temp_root();
        let codex_dir = root.join(".codex");
        let sessions_dir = codex_dir
            .join("sessions")
            .join("2026")
            .join("04")
            .join("09");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(
            codex_dir.join("history.jsonl"),
            "{\"session_id\":\"019-generic-session\",\"ts\":1775738408,\"text\":\"latest prompt\"}\n",
        )
        .unwrap();
        fs::write(
            sessions_dir.join("rollout-2026-04-09T14-57-22-019-generic-session.jsonl"),
            concat!(
                "{\"timestamp\":\"2026-04-09T06:57:34.427Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"019-generic-session\",\"cwd\":\"D:\\\\AI Island\\\\Repo\",\"originator\":\"codex-tui\"}}\n",
                "{\"timestamp\":\"2026-04-09T06:57:34.432Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_message\",\"message\":\"assistant summary\"}}\n"
            ),
        )
        .unwrap();

        let adapter = CodexAdapter::new(CodexPaths::from_home(&root));
        let sessions = adapter.scan_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "019-generic-session");

        let _ = fs::remove_dir_all(root);
    }
}
