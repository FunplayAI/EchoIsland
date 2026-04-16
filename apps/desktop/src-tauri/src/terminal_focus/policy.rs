use std::collections::HashSet;

use super::SessionFocusTarget;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn normalized_token(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn cwd_leaf(cwd: &str) -> Option<String> {
    cwd.trim_end_matches(['\\', '/'])
        .rsplit(['\\', '/'])
        .next()
        .and_then(normalized_token)
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn focus_tokens(target: &SessionFocusTarget) -> Vec<String> {
    let mut values = HashSet::new();
    if let Some(window_title) = &target.window_title
        && let Some(token) = normalized_token(window_title)
    {
        values.insert(token);
    }
    if let Some(project_name) = &target.project_name
        && let Some(token) = normalized_token(project_name)
    {
        values.insert(token);
    }
    if let Some(cwd) = &target.cwd {
        if let Some(token) = normalized_token(cwd) {
            values.insert(token);
        }
        if let Some(token) = cwd_leaf(cwd) {
            values.insert(token);
        }
    }
    values.into_iter().collect()
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn tab_focus_tokens(target: &SessionFocusTarget) -> Vec<String> {
    let mut values = focus_tokens(target).into_iter().collect::<HashSet<_>>();
    for alias in tab_title_aliases(&target.source, target.host_app.as_deref()) {
        if let Some(token) = normalized_token(&alias) {
            values.insert(token);
        }
    }
    values.into_iter().collect()
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn host_app_aliases(source: &str, host_app: Option<&str>) -> Vec<String> {
    let source = source.to_ascii_lowercase();
    let host_app = host_app.unwrap_or_default().to_ascii_lowercase();
    match host_app.as_str() {
        "claude-vscode" | "vscode" => vec![
            "code".to_string(),
            "visual studio code".to_string(),
            "claude".to_string(),
        ],
        "cursor" => vec!["cursor".to_string()],
        "windsurf" => vec!["windsurf".to_string()],
        "cli" if source == "claude" => vec![
            "claude".to_string(),
            "windowsterminal".to_string(),
            "powershell".to_string(),
            "pwsh".to_string(),
            "cmd".to_string(),
        ],
        "cli" if source == "opencode" => vec![
            "windowsterminal".to_string(),
            "powershell".to_string(),
            "pwsh".to_string(),
            "cmd".to_string(),
        ],
        "cli" if source == "openclaw" => vec![
            "openclaw".to_string(),
            "windowsterminal".to_string(),
            "powershell".to_string(),
            "pwsh".to_string(),
            "cmd".to_string(),
        ],
        other if !other.is_empty() => vec![other.to_string()],
        _ => Vec::new(),
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn tab_title_aliases(source: &str, host_app: Option<&str>) -> Vec<String> {
    let source = source.to_ascii_lowercase();
    let host_app = host_app.unwrap_or_default().to_ascii_lowercase();
    match (source.as_str(), host_app.as_str()) {
        ("codex", _) => vec!["codex".to_string()],
        ("claude", "cli") => vec!["initial conversation setup".to_string()],
        ("claude", "claude-vscode") => vec!["claude".to_string()],
        _ => Vec::new(),
    }
}

pub fn is_active_status(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "processing" | "running" | "waitingapproval" | "waitingquestion"
    )
}

#[cfg(test)]
mod tests {
    use super::{SessionFocusTarget, focus_tokens, host_app_aliases, tab_focus_tokens};

    #[test]
    fn claude_vscode_adds_editor_aliases() {
        let aliases = host_app_aliases("claude", Some("claude-vscode"));
        assert!(aliases.iter().any(|value| value == "code"));
        assert!(aliases.iter().any(|value| value == "visual studio code"));
    }

    #[test]
    fn focus_tokens_exclude_broad_host_aliases() {
        let target = SessionFocusTarget {
            session_id: "session-1".to_string(),
            source: "claude".to_string(),
            project_name: None,
            cwd: None,
            terminal_app: None,
            terminal_bundle: None,
            host_app: Some("claude-vscode".to_string()),
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
        };
        let tokens = focus_tokens(&target);
        assert!(!tokens.iter().any(|value| value == "code"));
    }

    #[test]
    fn tab_focus_tokens_include_claude_cli_default_title() {
        let target = SessionFocusTarget {
            session_id: "session-1".to_string(),
            source: "claude".to_string(),
            project_name: None,
            cwd: None,
            terminal_app: None,
            terminal_bundle: None,
            host_app: Some("cli".to_string()),
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
        };
        let tokens = tab_focus_tokens(&target);
        assert!(
            tokens
                .iter()
                .any(|value| value == "initial conversation setup")
        );
    }
}
