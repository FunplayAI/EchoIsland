use std::{
    env,
    process::{Command, Stdio},
};

pub(super) fn find_binary(name: &str) -> Option<String> {
    [
        format!("/opt/homebrew/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
    ]
    .into_iter()
    .find(|path| std::fs::metadata(path).is_ok())
}

pub(super) fn run_process(
    path: &str,
    args: &[&str],
    extra_env: Option<&[(&str, &str)]>,
) -> Option<String> {
    let mut command = Command::new(path);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(extra_env) = extra_env {
        command.envs(extra_env.iter().copied());
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!stdout.is_empty()).then_some(stdout)
}

pub(super) fn run_command_success(
    path: &str,
    args: &[&str],
    extra_env: Option<&[(&str, &str)]>,
) -> bool {
    let mut command = Command::new(path);
    command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(extra_env) = extra_env {
        command.envs(extra_env.iter().copied());
    }
    command
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub(super) fn resolve_symlinks(path: &str) -> String {
    std::fs::canonicalize(path)
        .ok()
        .and_then(|value| value.to_str().map(|value| value.to_string()))
        .unwrap_or_else(|| path.to_string())
}

pub(super) fn tilde_path(path: &str) -> String {
    let Ok(home) = env::var("HOME") else {
        return String::new();
    };
    if path == home {
        "~".to_string()
    } else if let Some(suffix) = path.strip_prefix(&(home.clone() + "/")) {
        format!("~/{suffix}")
    } else {
        String::new()
    }
}

pub(super) fn last_path_component(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    trimmed
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

pub(super) fn normalize_token(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '.'], "_")
}

pub(super) fn normalize_tty(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("/dev/") {
        trimmed.to_string()
    } else {
        format!("/dev/{trimmed}")
    }
}

pub(super) fn is_precise_terminal_tty(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "/dev/tty" && trimmed != "/dev/console"
}

pub(super) fn has_text(value: &str) -> bool {
    !value.trim().is_empty()
}
