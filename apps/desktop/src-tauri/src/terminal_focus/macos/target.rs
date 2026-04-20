use std::{
    env,
    process::{Command, Stdio},
};

use echoisland_core::EventMetadata;
use tracing::{info, warn};

use super::{
    SessionFocusTarget,
    process_inference::resolve_focus_target_from_process_tree,
    util::{has_text, is_precise_terminal_tty, normalize_token},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MacFocusTarget {
    ITerm2,
    Ghostty,
    TerminalApp,
    Warp,
    WezTerm,
    Kitty,
    Alacritty,
    Hyper,
    Tabby,
    Rio,
    Cursor,
    VSCode,
    CodexApp,
    TraeApp,
    QoderApp,
    FactoryApp,
    CodeBuddyApp,
    CodyBuddyCnApp,
    StepFunApp,
    OpenCodeApp,
}

#[derive(Clone, Copy, Debug)]
struct RunningTerminalCandidate {
    target: MacFocusTarget,
    display_name: &'static str,
    process_names: &'static [&'static str],
}

const RUNNING_TERMINAL_CANDIDATES: &[RunningTerminalCandidate] = &[
    RunningTerminalCandidate {
        target: MacFocusTarget::Warp,
        display_name: "Warp",
        process_names: &["Warp", "stable"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::Ghostty,
        display_name: "Ghostty",
        process_names: &["Ghostty", "ghostty"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::ITerm2,
        display_name: "iTerm2",
        process_names: &["iTerm2"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::WezTerm,
        display_name: "WezTerm",
        process_names: &["WezTerm", "wezterm-gui"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::Kitty,
        display_name: "kitty",
        process_names: &["kitty"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::Alacritty,
        display_name: "Alacritty",
        process_names: &["Alacritty", "alacritty"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::Hyper,
        display_name: "Hyper",
        process_names: &["Hyper"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::Tabby,
        display_name: "Tabby",
        process_names: &["Tabby"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::Rio,
        display_name: "Rio",
        process_names: &["Rio", "rio"],
    },
    RunningTerminalCandidate {
        target: MacFocusTarget::TerminalApp,
        display_name: "Terminal",
        process_names: &["Terminal"],
    },
];

pub(super) fn resolve_focus_target(target: &SessionFocusTarget) -> Option<MacFocusTarget> {
    if let Some(bundle_target) = target
        .terminal_bundle
        .as_deref()
        .and_then(resolve_focus_target_bundle)
    {
        return Some(bundle_target);
    }

    let terminal_app = target.terminal_app.as_deref().unwrap_or_default();
    let host_app = target.host_app.as_deref().unwrap_or_default();

    if let Some(resolved) = resolve_focus_target_bundle(terminal_app) {
        return Some(resolved);
    }

    if let Some(resolved) = resolve_focus_target_bundle(host_app) {
        return Some(resolved);
    }

    let terminal_app = normalize_token(terminal_app);
    let host_app = normalize_token(host_app);

    let allow_terminal_app_token =
        target.terminal_bundle.is_some() || !is_unreliable_terminal_app_token(&terminal_app);

    if allow_terminal_app_token {
        if let Some(resolved) = resolve_focus_target_token(&terminal_app) {
            return Some(resolved);
        }
    } else {
        info!(
            source = %target.source,
            session_id = %target.session_id,
            terminal_app = ?target.terminal_app,
            terminal_bundle = ?target.terminal_bundle,
            "skipping direct macOS terminal token resolution because TERM_PROGRAM may be inherited"
        );
    }

    if let Some(resolved) = resolve_focus_target_token(&host_app) {
        return Some(resolved);
    }

    if let Some(process_target) = resolve_focus_target_from_process_tree(target) {
        info!(
            source = %target.source,
            session_id = %target.session_id,
            cli_pid = ?target.cli_pid,
            resolved_target = ?process_target,
            "resolved macOS terminal focus from CLI process tree"
        );
        return Some(process_target);
    }

    detect_running_terminal_target(target)
}

fn resolve_focus_target_bundle(bundle_id: &str) -> Option<MacFocusTarget> {
    match bundle_id {
        "com.googlecode.iterm2" => Some(MacFocusTarget::ITerm2),
        "com.mitchellh.ghostty" => Some(MacFocusTarget::Ghostty),
        "com.apple.Terminal" => Some(MacFocusTarget::TerminalApp),
        "dev.warp.Warp-Stable" => Some(MacFocusTarget::Warp),
        "com.github.wez.wezterm" => Some(MacFocusTarget::WezTerm),
        "net.kovidgoyal.kitty" => Some(MacFocusTarget::Kitty),
        "org.alacritty" => Some(MacFocusTarget::Alacritty),
        "co.zeit.hyper" => Some(MacFocusTarget::Hyper),
        "org.tabby" => Some(MacFocusTarget::Tabby),
        "com.raphaelamorim.rio" => Some(MacFocusTarget::Rio),
        "com.todesktop.230313mzl4w4u92" => Some(MacFocusTarget::Cursor),
        "com.microsoft.VSCode" => Some(MacFocusTarget::VSCode),
        "com.openai.codex" => Some(MacFocusTarget::CodexApp),
        "com.trae.app" => Some(MacFocusTarget::TraeApp),
        "com.qoder.ide" => Some(MacFocusTarget::QoderApp),
        "com.factory.app" => Some(MacFocusTarget::FactoryApp),
        "com.tencent.codebuddy" => Some(MacFocusTarget::CodeBuddyApp),
        "com.tencent.codebuddy.cn" => Some(MacFocusTarget::CodyBuddyCnApp),
        "com.stepfun.app" => Some(MacFocusTarget::StepFunApp),
        "ai.opencode.desktop" => Some(MacFocusTarget::OpenCodeApp),
        _ => None,
    }
}

fn resolve_focus_target_token(value: &str) -> Option<MacFocusTarget> {
    if value.is_empty() {
        return None;
    }
    if value.contains("iterm") {
        return Some(MacFocusTarget::ITerm2);
    }
    if value.contains("ghostty") {
        return Some(MacFocusTarget::Ghostty);
    }
    if value == "terminal" || value.contains("terminalapp") {
        return Some(MacFocusTarget::TerminalApp);
    }
    if value.contains("warp") {
        return Some(MacFocusTarget::Warp);
    }
    if value.contains("wezterm") || value == "wez" {
        return Some(MacFocusTarget::WezTerm);
    }
    if value.contains("kitty") {
        return Some(MacFocusTarget::Kitty);
    }
    if value.contains("alacritty") {
        return Some(MacFocusTarget::Alacritty);
    }
    if value.contains("hyper") {
        return Some(MacFocusTarget::Hyper);
    }
    if value.contains("tabby") {
        return Some(MacFocusTarget::Tabby);
    }
    if value.contains("rio") {
        return Some(MacFocusTarget::Rio);
    }
    if value.contains("cursor") {
        return Some(MacFocusTarget::Cursor);
    }
    if value.contains("vscode") || value == "code" || value.contains("claude_vscode") {
        return Some(MacFocusTarget::VSCode);
    }
    None
}

fn is_unreliable_terminal_app_token(value: &str) -> bool {
    matches!(value, "apple_terminal" | "terminal" | "terminalapp")
}

fn detect_running_terminal_target(target: &SessionFocusTarget) -> Option<MacFocusTarget> {
    if !should_detect_running_terminal(target) {
        return None;
    }

    if let Some(configured) = configured_terminal_target() {
        info!(
            source = %target.source,
            configured_target = ?configured,
            "resolved macOS terminal focus from configured default"
        );
        return Some(configured);
    }

    let mut detected = Vec::new();
    for candidate in RUNNING_TERMINAL_CANDIDATES {
        if is_application_running(candidate) {
            detected.push(*candidate);
        }
    }

    if should_skip_detected_running_terminal_fallback(target, detected.len()) {
        warn!(
            source = %target.source,
            session_id = %target.session_id,
            cwd = ?target.cwd,
            detected_count = detected.len(),
            "skipping ambiguous macOS running-terminal fallback because session has no reliable terminal metadata"
        );
        return None;
    }

    let candidate = detected.first()?;
    info!(
        source = %target.source,
        cwd = ?target.cwd,
        detected_terminal = candidate.display_name,
        detected_count = detected.len(),
        "resolved macOS terminal focus from running terminal fallback"
    );
    Some(candidate.target)
}

fn is_application_running(candidate: &RunningTerminalCandidate) -> bool {
    candidate
        .process_names
        .iter()
        .any(|process_name| is_process_running(process_name))
}

pub(super) fn is_bundle_running(bundle_id: &str) -> bool {
    Command::new("/usr/bin/pgrep")
        .arg("-f")
        .arg(bundle_id)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn should_detect_running_terminal(target: &SessionFocusTarget) -> bool {
    let terminal_app = target
        .terminal_app
        .as_deref()
        .map(normalize_token)
        .unwrap_or_default();
    terminal_app.is_empty()
        || terminal_app == "tmux"
        || terminal_app == "screen"
        || terminal_app == "cli"
}

pub(super) fn should_skip_detected_running_terminal_fallback(
    target: &SessionFocusTarget,
    detected_count: usize,
) -> bool {
    detected_count > 1 && !has_reliable_terminal_focus_metadata(target)
}

pub(super) fn should_use_source_native_app_fallback(target: &SessionFocusTarget) -> bool {
    !has_terminal_focus_metadata(target)
}

fn has_terminal_focus_metadata(target: &SessionFocusTarget) -> bool {
    target.terminal_app.as_deref().is_some_and(has_text)
        || target.terminal_bundle.as_deref().is_some_and(has_text)
        || target.tty.as_deref().is_some_and(is_precise_terminal_tty)
        || target.terminal_pid.is_some()
        || target.cli_pid.is_some()
        || target.iterm_session_id.as_deref().is_some_and(has_text)
        || target.kitty_window_id.as_deref().is_some_and(has_text)
        || target.tmux_env.as_deref().is_some_and(has_text)
        || target.tmux_pane.as_deref().is_some_and(has_text)
        || target
            .tmux_client_tty
            .as_deref()
            .is_some_and(is_precise_terminal_tty)
}

fn has_reliable_terminal_focus_metadata(target: &SessionFocusTarget) -> bool {
    target
        .terminal_bundle
        .as_deref()
        .and_then(resolve_focus_target_bundle)
        .is_some()
        || target.tty.as_deref().is_some_and(is_precise_terminal_tty)
        || target.terminal_pid.is_some()
        || target.cli_pid.is_some()
        || target.iterm_session_id.as_deref().is_some_and(has_text)
        || target.kitty_window_id.as_deref().is_some_and(has_text)
        || target.tmux_pane.as_deref().is_some_and(has_text)
        || target
            .tmux_client_tty
            .as_deref()
            .is_some_and(is_precise_terminal_tty)
        || target
            .terminal_app
            .as_deref()
            .map(normalize_token)
            .is_some_and(|value| {
                resolve_focus_target_token(&value).is_some()
                    && !is_unreliable_terminal_app_token(&value)
                    && !matches!(value.as_str(), "tmux" | "screen" | "cli")
            })
}

fn configured_terminal_target() -> Option<MacFocusTarget> {
    for name in [
        "CODEISLAND_MACOS_FOCUS_TERMINAL",
        "CODEISLAND_DEFAULT_TERMINAL",
    ] {
        let Ok(value) = env::var(name) else {
            continue;
        };
        let normalized = normalize_token(&value);
        if let Some(target) = resolve_focus_target_token(&normalized) {
            return Some(target);
        }
    }

    if let Ok(value) = env::var("TERM_PROGRAM") {
        let normalized = normalize_token(&value);
        if let Some(target) = resolve_focus_target_token(&normalized) {
            info!(
                term_program = value,
                resolved_target = ?target,
                "resolved macOS terminal focus from launcher TERM_PROGRAM"
            );
            return Some(target);
        }
    }

    None
}

fn is_process_running(process_name: &str) -> bool {
    Command::new("/usr/bin/pgrep")
        .arg("-qx")
        .arg(process_name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub(super) fn terminal_metadata_for_target(
    target: MacFocusTarget,
) -> Option<(Option<String>, Option<String>)> {
    match target {
        MacFocusTarget::ITerm2 => Some((
            Some("iTerm.app".to_string()),
            Some("com.googlecode.iterm2".to_string()),
        )),
        MacFocusTarget::Ghostty => Some((
            Some("ghostty".to_string()),
            Some("com.mitchellh.ghostty".to_string()),
        )),
        MacFocusTarget::TerminalApp => Some((
            Some("Apple_Terminal".to_string()),
            Some("com.apple.Terminal".to_string()),
        )),
        MacFocusTarget::Warp => Some((
            Some("Warp".to_string()),
            Some("dev.warp.Warp-Stable".to_string()),
        )),
        MacFocusTarget::WezTerm => Some((
            Some("WezTerm".to_string()),
            Some("com.github.wez.wezterm".to_string()),
        )),
        MacFocusTarget::Kitty => Some((
            Some("kitty".to_string()),
            Some("net.kovidgoyal.kitty".to_string()),
        )),
        MacFocusTarget::Alacritty => Some((
            Some("Alacritty".to_string()),
            Some("org.alacritty".to_string()),
        )),
        _ => None,
    }
}

pub(super) fn has_terminal_metadata(metadata: &EventMetadata) -> bool {
    metadata.terminal_app.is_some()
        || metadata.terminal_bundle.is_some()
        || metadata.tty.is_some()
        || metadata.pid.is_some()
        || metadata.cli_pid.is_some()
        || metadata.iterm_session_id.is_some()
        || metadata.kitty_window_id.is_some()
        || metadata.tmux_env.is_some()
        || metadata.tmux_pane.is_some()
        || metadata.tmux_client_tty.is_some()
}
