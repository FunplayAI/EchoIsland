use std::{env, path::PathBuf};

const ECHOISLAND_APP_DIR: &str = "CodeIsland";
const IPC_TOKEN_FILE_NAME: &str = "ipc-token";
const STATE_FILE_NAME: &str = "state.json";
const FOCUS_BINDINGS_FILE_NAME: &str = "focus-bindings.json";
const BRIDGE_LOG_FILE_NAME: &str = "bridge.log";

#[derive(Debug, Clone)]
pub struct PlatformPaths {
    pub home_dir: PathBuf,
    pub app_data_dir: PathBuf,
    pub echoisland_app_dir: PathBuf,
    pub state_path: PathBuf,
    pub ipc_token_path: PathBuf,
    pub focus_bindings_path: PathBuf,
    pub bridge_log_path: PathBuf,
    pub echoisland_home_dir: PathBuf,
    pub echoisland_bin_dir: PathBuf,
    pub bridge_binary_path: PathBuf,
    pub codex_dir: PathBuf,
    pub codex_hooks_path: PathBuf,
    pub codex_config_path: PathBuf,
    pub claude_config_dir: PathBuf,
    pub claude_settings_path: PathBuf,
    pub claude_projects_dir: PathBuf,
    pub openclaw_dir: PathBuf,
    pub openclaw_config_path: PathBuf,
    pub openclaw_hooks_dir: PathBuf,
}

pub fn user_home_dir() -> PathBuf {
    if cfg!(windows) {
        env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

pub fn app_data_dir() -> PathBuf {
    if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .or_else(|| env::var_os("USERPROFILE"))
            .or_else(|| env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    } else if cfg!(target_os = "macos") {
        user_home_dir().join("Library").join("Application Support")
    } else {
        env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| user_home_dir().join(".local").join("state"))
    }
}

pub fn echoisland_app_dir() -> PathBuf {
    app_data_dir().join(ECHOISLAND_APP_DIR)
}

pub fn state_path() -> PathBuf {
    echoisland_app_dir().join(STATE_FILE_NAME)
}

pub fn ipc_token_path() -> PathBuf {
    echoisland_app_dir().join(IPC_TOKEN_FILE_NAME)
}

pub fn focus_bindings_path() -> PathBuf {
    echoisland_app_dir().join(FOCUS_BINDINGS_FILE_NAME)
}

pub fn bridge_log_path() -> PathBuf {
    echoisland_app_dir().join(BRIDGE_LOG_FILE_NAME)
}

pub fn codex_dir() -> PathBuf {
    user_home_dir().join(".codex")
}

pub fn codex_hooks_path() -> PathBuf {
    codex_dir().join("hooks.json")
}

pub fn codex_config_path() -> PathBuf {
    codex_dir().join("config.toml")
}

pub fn claude_config_dir() -> PathBuf {
    env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_home_dir().join(".claude"))
}

pub fn claude_settings_path() -> PathBuf {
    claude_config_dir().join("settings.json")
}

pub fn claude_projects_dir() -> PathBuf {
    claude_config_dir().join("projects")
}

pub fn openclaw_dir() -> PathBuf {
    env::var_os("OPENCLAW_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| user_home_dir().join(".openclaw"))
}

pub fn openclaw_config_path() -> PathBuf {
    env::var_os("OPENCLAW_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| openclaw_dir().join("openclaw.json"))
}

pub fn openclaw_hooks_dir() -> PathBuf {
    openclaw_dir().join("hooks")
}

pub fn echoisland_home_dir() -> PathBuf {
    user_home_dir().join(".codeisland")
}

pub fn echoisland_home_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    home_dir.into().join(".codeisland")
}

pub fn echoisland_bin_dir() -> PathBuf {
    echoisland_home_dir().join("bin")
}

pub fn echoisland_bin_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    echoisland_home_dir_from_home(home_dir).join("bin")
}

pub fn bridge_binary_name() -> &'static str {
    if cfg!(windows) {
        "codeisland-hook-bridge.exe"
    } else {
        "codeisland-hook-bridge"
    }
}

pub fn bridge_binary_path() -> PathBuf {
    echoisland_bin_dir().join(bridge_binary_name())
}

pub fn bridge_binary_path_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    echoisland_bin_dir_from_home(home_dir).join(bridge_binary_name())
}

pub fn codex_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    home_dir.into().join(".codex")
}

pub fn codex_hooks_path_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    codex_dir_from_home(home_dir).join("hooks.json")
}

pub fn codex_config_path_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    codex_dir_from_home(home_dir).join("config.toml")
}

pub fn claude_config_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    home_dir.into().join(".claude")
}

pub fn claude_settings_path_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    claude_config_dir_from_home(home_dir).join("settings.json")
}

pub fn claude_projects_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    claude_config_dir_from_home(home_dir).join("projects")
}

pub fn openclaw_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    home_dir.into().join(".openclaw")
}

pub fn openclaw_config_path_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    openclaw_dir_from_home(home_dir).join("openclaw.json")
}

pub fn openclaw_hooks_dir_from_home(home_dir: impl Into<PathBuf>) -> PathBuf {
    openclaw_dir_from_home(home_dir).join("hooks")
}

pub fn current_platform_paths() -> PlatformPaths {
    let home_dir = user_home_dir();
    let app_data_dir = app_data_dir();
    let echoisland_app_dir = app_data_dir.join(ECHOISLAND_APP_DIR);
    let state_path = echoisland_app_dir.join(STATE_FILE_NAME);
    let ipc_token_path = echoisland_app_dir.join(IPC_TOKEN_FILE_NAME);
    let focus_bindings_path = echoisland_app_dir.join(FOCUS_BINDINGS_FILE_NAME);
    let bridge_log_path = echoisland_app_dir.join(BRIDGE_LOG_FILE_NAME);
    let echoisland_home_dir = home_dir.join(".codeisland");
    let echoisland_bin_dir = echoisland_home_dir.join("bin");
    let bridge_binary_path = echoisland_bin_dir.join(bridge_binary_name());
    let codex_dir = home_dir.join(".codex");
    let codex_hooks_path = codex_dir.join("hooks.json");
    let codex_config_path = codex_dir.join("config.toml");
    let claude_config_dir = env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".claude"));
    let claude_settings_path = claude_config_dir.join("settings.json");
    let claude_projects_dir = claude_config_dir.join("projects");
    let openclaw_dir = env::var_os("OPENCLAW_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".openclaw"));
    let openclaw_config_path = env::var_os("OPENCLAW_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| openclaw_dir.join("openclaw.json"));
    let openclaw_hooks_dir = openclaw_dir.join("hooks");

    PlatformPaths {
        home_dir,
        app_data_dir,
        echoisland_app_dir,
        state_path,
        ipc_token_path,
        focus_bindings_path,
        bridge_log_path,
        echoisland_home_dir,
        echoisland_bin_dir,
        bridge_binary_path,
        codex_dir,
        codex_hooks_path,
        codex_config_path,
        claude_config_dir,
        claude_settings_path,
        claude_projects_dir,
        openclaw_dir,
        openclaw_config_path,
        openclaw_hooks_dir,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        app_data_dir, bridge_binary_name, bridge_binary_path, bridge_binary_path_from_home,
        bridge_log_path, claude_config_dir, claude_config_dir_from_home, claude_projects_dir,
        claude_projects_dir_from_home, claude_settings_path, claude_settings_path_from_home,
        codex_config_path, codex_config_path_from_home, codex_dir, codex_dir_from_home,
        codex_hooks_path, codex_hooks_path_from_home, current_platform_paths, echoisland_app_dir,
        echoisland_bin_dir, echoisland_bin_dir_from_home, echoisland_home_dir,
        echoisland_home_dir_from_home, focus_bindings_path, ipc_token_path, openclaw_config_path,
        openclaw_config_path_from_home, openclaw_dir, openclaw_dir_from_home, openclaw_hooks_dir,
        openclaw_hooks_dir_from_home, state_path, user_home_dir,
    };

    #[test]
    fn echoisland_paths_share_common_roots() {
        assert_eq!(state_path(), echoisland_app_dir().join("state.json"));
        assert_eq!(ipc_token_path(), echoisland_app_dir().join("ipc-token"));
        assert_eq!(
            focus_bindings_path(),
            echoisland_app_dir().join("focus-bindings.json")
        );
        assert_eq!(bridge_log_path(), echoisland_app_dir().join("bridge.log"));
        assert_eq!(
            bridge_binary_path(),
            echoisland_bin_dir().join(bridge_binary_name())
        );
        assert_eq!(codex_hooks_path(), codex_dir().join("hooks.json"));
        assert_eq!(codex_config_path(), codex_dir().join("config.toml"));
        assert_eq!(
            claude_settings_path(),
            claude_config_dir().join("settings.json")
        );
        assert_eq!(claude_projects_dir(), claude_config_dir().join("projects"));
        assert_eq!(openclaw_config_path(), openclaw_dir().join("openclaw.json"));
        assert_eq!(openclaw_hooks_dir(), openclaw_dir().join("hooks"));
    }

    #[test]
    fn roots_fall_back_to_some_directory() {
        assert!(!user_home_dir().as_os_str().is_empty());
        assert!(!app_data_dir().as_os_str().is_empty());
        assert_eq!(
            echoisland_home_dir().file_name().and_then(|v| v.to_str()),
            Some(".codeisland")
        );
        assert_eq!(
            echoisland_app_dir().file_name().and_then(|v| v.to_str()),
            Some("CodeIsland")
        );
        assert_eq!(
            claude_config_dir().file_name().and_then(|v| v.to_str()),
            Some(".claude")
        );
    }

    #[test]
    fn aggregates_current_platform_paths() {
        let paths = current_platform_paths();

        assert_eq!(paths.state_path, state_path());
        assert_eq!(paths.ipc_token_path, ipc_token_path());
        assert_eq!(paths.focus_bindings_path, focus_bindings_path());
        assert_eq!(paths.bridge_log_path, bridge_log_path());
        assert_eq!(paths.bridge_binary_path, bridge_binary_path());
        assert_eq!(paths.codex_dir, codex_dir());
        assert_eq!(paths.codex_hooks_path, codex_hooks_path());
        assert_eq!(paths.codex_config_path, codex_config_path());
        assert_eq!(paths.claude_config_dir, claude_config_dir());
        assert_eq!(paths.claude_settings_path, claude_settings_path());
        assert_eq!(paths.claude_projects_dir, claude_projects_dir());
        assert_eq!(paths.openclaw_dir, openclaw_dir());
        assert_eq!(paths.openclaw_config_path, openclaw_config_path());
        assert_eq!(paths.openclaw_hooks_dir, openclaw_hooks_dir());
    }

    #[test]
    fn builds_deterministic_paths_from_custom_home() {
        let home = std::path::PathBuf::from("/tmp/codeisland-home");

        assert_eq!(
            echoisland_home_dir_from_home(&home),
            home.join(".codeisland")
        );
        assert_eq!(
            echoisland_bin_dir_from_home(&home),
            home.join(".codeisland").join("bin")
        );
        assert_eq!(
            bridge_binary_path_from_home(&home),
            home.join(".codeisland")
                .join("bin")
                .join(bridge_binary_name())
        );
        assert_eq!(codex_dir_from_home(&home), home.join(".codex"));
        assert_eq!(
            codex_hooks_path_from_home(&home),
            home.join(".codex").join("hooks.json")
        );
        assert_eq!(
            codex_config_path_from_home(&home),
            home.join(".codex").join("config.toml")
        );
        assert_eq!(claude_config_dir_from_home(&home), home.join(".claude"));
        assert_eq!(
            claude_settings_path_from_home(&home),
            home.join(".claude").join("settings.json")
        );
        assert_eq!(
            claude_projects_dir_from_home(&home),
            home.join(".claude").join("projects")
        );
        assert_eq!(openclaw_dir_from_home(&home), home.join(".openclaw"));
        assert_eq!(
            openclaw_config_path_from_home(&home),
            home.join(".openclaw").join("openclaw.json")
        );
        assert_eq!(
            openclaw_hooks_dir_from_home(&home),
            home.join(".openclaw").join("hooks")
        );
    }
}
