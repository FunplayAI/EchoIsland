use std::path::PathBuf;

use echoisland_paths::{
    ipc_token_path, openclaw_config_path, openclaw_config_path_from_home, openclaw_dir,
    openclaw_dir_from_home, openclaw_hooks_dir, openclaw_hooks_dir_from_home, user_home_dir,
};
use serde::Serialize;

use crate::{AdapterPath, AdapterStatus, InstallableAdapter};

mod install;

pub use install::{DEFAULT_OPENCLAW_RECEIVER_URL, get_openclaw_status, install_openclaw_adapter};

const OPENCLAW_HOOK_ID: &str = "echoisland";

#[derive(Debug, Clone)]
pub struct OpenClawAdapter {
    paths: OpenClawPaths,
}

#[derive(Debug, Clone)]
pub struct OpenClawPaths {
    pub home_dir: PathBuf,
    pub openclaw_dir: PathBuf,
    pub config_path: PathBuf,
    pub hooks_dir: PathBuf,
    pub hook_dir: PathBuf,
    pub hook_manifest_path: PathBuf,
    pub hook_handler_path: PathBuf,
    pub token_path: PathBuf,
    pub receiver_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenClawStatus {
    pub openclaw_dir_exists: bool,
    pub config_path_exists: bool,
    pub hooks_dir_exists: bool,
    pub hook_installed: bool,
    pub hook_enabled: bool,
    pub token_exists: bool,
    pub live_capture_supported: bool,
    pub live_capture_ready: bool,
    pub status_note: Option<String>,
    pub openclaw_dir: String,
    pub config_path: String,
    pub hooks_dir: String,
    pub hook_dir: String,
    pub hook_manifest_path: String,
    pub hook_handler_path: String,
    pub token_path: String,
    pub receiver_url: String,
}

impl OpenClawPaths {
    pub fn from_home(home_dir: impl Into<PathBuf>) -> Self {
        Self::from_home_with_runtime(
            home_dir,
            DEFAULT_OPENCLAW_RECEIVER_URL.to_string(),
            ipc_token_path(),
        )
    }

    pub fn from_home_with_runtime(
        home_dir: impl Into<PathBuf>,
        receiver_url: impl Into<String>,
        token_path: impl Into<PathBuf>,
    ) -> Self {
        let home_dir = home_dir.into();
        let openclaw_dir = openclaw_dir_from_home(&home_dir);
        let config_path = openclaw_config_path_from_home(&home_dir);
        let hooks_dir = openclaw_hooks_dir_from_home(&home_dir);
        let hook_dir = hooks_dir.join(OPENCLAW_HOOK_ID);

        Self {
            home_dir,
            openclaw_dir,
            config_path,
            hooks_dir,
            hook_manifest_path: hook_dir.join("HOOK.md"),
            hook_handler_path: hook_dir.join("handler.ts"),
            hook_dir,
            token_path: token_path.into(),
            receiver_url: receiver_url.into(),
        }
    }
}

impl OpenClawAdapter {
    pub fn new(paths: OpenClawPaths) -> Self {
        Self { paths }
    }

    pub fn with_default_paths() -> Self {
        Self::new(default_paths())
    }
}

pub fn default_paths() -> OpenClawPaths {
    OpenClawPaths {
        home_dir: user_home_dir(),
        openclaw_dir: openclaw_dir(),
        config_path: openclaw_config_path(),
        hooks_dir: openclaw_hooks_dir(),
        hook_dir: openclaw_hooks_dir().join(OPENCLAW_HOOK_ID),
        hook_manifest_path: openclaw_hooks_dir().join(OPENCLAW_HOOK_ID).join("HOOK.md"),
        hook_handler_path: openclaw_hooks_dir()
            .join(OPENCLAW_HOOK_ID)
            .join("handler.ts"),
        token_path: ipc_token_path(),
        receiver_url: DEFAULT_OPENCLAW_RECEIVER_URL.to_string(),
    }
}

impl From<OpenClawStatus> for AdapterStatus {
    fn from(value: OpenClawStatus) -> Self {
        Self {
            adapter: "openclaw".to_string(),
            config_dir_exists: value.openclaw_dir_exists,
            bridge_exists: value.hook_installed,
            hooks_installed: value.hook_installed,
            hooks_enabled: value.hook_enabled,
            live_capture_supported: value.live_capture_supported,
            live_capture_ready: value.live_capture_ready,
            status_note: value.status_note,
            paths: vec![
                AdapterPath {
                    label: "openclaw_dir".to_string(),
                    path: value.openclaw_dir,
                },
                AdapterPath {
                    label: "config_path".to_string(),
                    path: value.config_path,
                },
                AdapterPath {
                    label: "hook_dir".to_string(),
                    path: value.hook_dir,
                },
                AdapterPath {
                    label: "hook_manifest_path".to_string(),
                    path: value.hook_manifest_path,
                },
                AdapterPath {
                    label: "hook_handler_path".to_string(),
                    path: value.hook_handler_path,
                },
                AdapterPath {
                    label: "token_path".to_string(),
                    path: value.token_path,
                },
            ],
        }
    }
}

impl InstallableAdapter for OpenClawAdapter {
    fn adapter_id(&self) -> &'static str {
        "openclaw"
    }

    fn status(&self) -> anyhow::Result<AdapterStatus> {
        Ok(get_openclaw_status(&self.paths)?.into())
    }

    fn install(&self, _source_bridge: &std::path::Path) -> anyhow::Result<AdapterStatus> {
        Ok(install_openclaw_adapter(&self.paths)?.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::InstallableAdapter;

    use super::{OpenClawAdapter, OpenClawPaths, get_openclaw_status, install_openclaw_adapter};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("echoisland-openclaw-{suffix}-{counter}"))
    }

    #[test]
    fn installs_openclaw_hook_pack() {
        let root = temp_root();
        let token_path = root.join("EchoIsland").join("ipc-token");
        fs::create_dir_all(token_path.parent().unwrap()).unwrap();
        fs::write(&token_path, b"secret").unwrap();

        let paths = OpenClawPaths::from_home_with_runtime(
            &root,
            "http://127.0.0.1:37892/event",
            &token_path,
        );
        let status = install_openclaw_adapter(&paths).unwrap();

        assert!(status.openclaw_dir_exists);
        assert!(status.hooks_dir_exists);
        assert!(status.hook_installed);
        assert!(status.hook_enabled);
        assert!(status.live_capture_ready);

        let hook_md = fs::read_to_string(&paths.hook_manifest_path).unwrap();
        let handler = fs::read_to_string(&paths.hook_handler_path).unwrap();
        assert!(hook_md.contains("message:received"));
        assert!(handler.contains("echoisland-openclaw-hook"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn openclaw_adapter_exposes_generic_status() {
        let root = temp_root();
        let token_path = root.join("EchoIsland").join("ipc-token");
        fs::create_dir_all(token_path.parent().unwrap()).unwrap();
        fs::write(&token_path, b"secret").unwrap();

        let adapter = OpenClawAdapter::new(OpenClawPaths::from_home_with_runtime(
            &root,
            "http://127.0.0.1:37892/event",
            &token_path,
        ));
        let status = adapter.install(std::path::Path::new("ignored")).unwrap();

        assert_eq!(InstallableAdapter::adapter_id(&adapter), "openclaw");
        assert!(status.config_dir_exists);
        assert!(status.hooks_installed);

        let raw_status = get_openclaw_status(&OpenClawPaths::from_home_with_runtime(
            &root,
            "http://127.0.0.1:37892/event",
            &token_path,
        ))
        .unwrap();
        assert!(raw_status.hook_installed);

        let _ = fs::remove_dir_all(root);
    }
}
