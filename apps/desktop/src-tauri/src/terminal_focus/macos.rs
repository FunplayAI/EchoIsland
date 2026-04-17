use std::{
    collections::BTreeSet,
    env,
    io::Write,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Result;
use chrono::{Local, NaiveDateTime, TimeZone};
use echoisland_core::EventMetadata;
use tracing::{debug, info, warn};

use super::{FocusOutcome, ForegroundTabInfo, SessionFocusTarget, SessionTabCache};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MacFocusTarget {
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

struct ProcessInfo {
    ppid: u32,
    command: String,
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

static DID_WARN_ACCESSIBILITY_PERMISSION: AtomicBool = AtomicBool::new(false);
static DID_WARN_AUTOMATION_PERMISSION: AtomicBool = AtomicBool::new(false);

pub fn focus_session_terminal(
    target: &SessionFocusTarget,
    cached_tab: Option<&SessionTabCache>,
) -> Result<FocusOutcome> {
    let _ = cached_tab;

    if let Some((bundle_id, display_name)) = native_app_bundle_for_terminal(target) {
        info!(
            source = %target.source,
            bundle_id,
            display_name,
            "focusing macOS native desktop app by terminal bundle"
        );
        return Ok(FocusOutcome {
            focused: activate_app_bundle(bundle_id, display_name),
            selected_tab: None,
        });
    }

    if let Some((bundle_id, display_name)) = source_native_app_bundle(&target.source)
        && target.terminal_bundle.is_none()
        && is_bundle_running(bundle_id)
    {
        info!(
            source = %target.source,
            bundle_id,
            display_name,
            "focusing macOS native desktop app from source fallback"
        );
        return Ok(FocusOutcome {
            focused: activate_app_bundle(bundle_id, display_name),
            selected_tab: None,
        });
    }

    let resolved_target = resolve_focus_target(target);
    info!(
        source = %target.source,
        terminal_app = ?target.terminal_app,
        host_app = ?target.host_app,
        cwd = ?target.cwd,
        resolved_target = ?resolved_target,
        "trying macOS terminal focus"
    );

    let focused = match resolved_target {
        Some(MacFocusTarget::ITerm2) => activate_iterm2(target),
        Some(MacFocusTarget::Ghostty) => activate_ghostty(target),
        Some(MacFocusTarget::TerminalApp) => activate_terminal_app(target),
        Some(MacFocusTarget::Warp) => {
            activate_terminal_window("dev.warp.Warp-Stable", target.cwd.as_deref(), "Warp")
        }
        Some(MacFocusTarget::WezTerm) => activate_wezterm(target),
        Some(MacFocusTarget::Kitty) => activate_kitty(target),
        Some(MacFocusTarget::Alacritty) => {
            activate_terminal_window("org.alacritty", target.cwd.as_deref(), "Alacritty")
        }
        Some(MacFocusTarget::Hyper) => activate_terminal_window("", target.cwd.as_deref(), "Hyper"),
        Some(MacFocusTarget::Tabby) => activate_terminal_window("", target.cwd.as_deref(), "Tabby"),
        Some(MacFocusTarget::Rio) => activate_terminal_window("", target.cwd.as_deref(), "Rio"),
        Some(MacFocusTarget::Cursor) => activate_ide_window(
            "com.todesktop.230313mzl4w4u92",
            target.cwd.as_deref(),
            "Cursor",
        ),
        Some(MacFocusTarget::VSCode) => activate_ide_window(
            "com.microsoft.VSCode",
            target.cwd.as_deref(),
            "Visual Studio Code",
        ),
        Some(MacFocusTarget::CodexApp) => activate_app_bundle("com.openai.codex", "Codex"),
        Some(MacFocusTarget::TraeApp) => activate_app_bundle("com.trae.app", "Trae"),
        Some(MacFocusTarget::QoderApp) => activate_app_bundle("com.qoder.ide", "Qoder"),
        Some(MacFocusTarget::FactoryApp) => activate_app_bundle("com.factory.app", "Factory"),
        Some(MacFocusTarget::CodeBuddyApp) => {
            activate_app_bundle("com.tencent.codebuddy", "CodeBuddy")
        }
        Some(MacFocusTarget::CodyBuddyCnApp) => {
            activate_app_bundle("com.tencent.codebuddy.cn", "CodyBuddyCN")
        }
        Some(MacFocusTarget::StepFunApp) => activate_app_bundle("com.stepfun.app", "StepFun"),
        Some(MacFocusTarget::OpenCodeApp) => activate_app_bundle("ai.opencode.desktop", "OpenCode"),
        None => false,
    };

    Ok(FocusOutcome {
        focused,
        selected_tab: None,
    })
}

#[allow(dead_code)]
pub fn foreground_session_terminal_tab() -> Result<Option<ForegroundTabInfo>> {
    Ok(None)
}

pub fn infer_terminal_metadata(target: &SessionFocusTarget) -> Option<EventMetadata> {
    let cli_pid = target
        .cli_pid
        .or_else(|| infer_cli_pid_from_running_processes(target));
    let inferred_terminal = target
        .terminal_pid
        .and_then(|pid| known_terminal_target_for_pid(pid).map(|target| (pid, target)))
        .or_else(|| cli_pid.and_then(terminal_process_from_process_tree));
    let inferred_terminal_pid = inferred_terminal.map(|(pid, _)| pid);
    let inferred_tty = effective_tty(target)
        .map(str::to_string)
        .or_else(|| cli_pid.and_then(detect_tty_path_from_pid))
        .or_else(|| inferred_terminal_pid.and_then(detect_tty_path_from_pid));
    let inferred_target = inferred_terminal.map(|(_, target)| target);
    let (inferred_app, inferred_bundle) = inferred_target
        .and_then(terminal_metadata_for_target)
        .unwrap_or((None, None));

    let metadata = EventMetadata {
        terminal_app: target.terminal_app.clone().or(inferred_app),
        terminal_bundle: target.terminal_bundle.clone().or(inferred_bundle),
        host_app: target.host_app.clone(),
        window_title: target.window_title.clone(),
        tty: inferred_tty,
        pid: inferred_terminal_pid,
        cli_pid: target.cli_pid.or(cli_pid),
        iterm_session_id: target.iterm_session_id.clone(),
        kitty_window_id: target.kitty_window_id.clone(),
        tmux_env: target.tmux_env.clone(),
        tmux_pane: target.tmux_pane.clone(),
        tmux_client_tty: target.tmux_client_tty.clone(),
        workspace_roots: None,
    };

    has_terminal_metadata(&metadata).then_some(metadata)
}

fn resolve_focus_target(target: &SessionFocusTarget) -> Option<MacFocusTarget> {
    if let Some(bundle_target) = target
        .terminal_bundle
        .as_deref()
        .and_then(resolve_focus_target_bundle)
    {
        return Some(bundle_target);
    }

    let terminal_app = target
        .terminal_app
        .as_deref()
        .map(normalize_token)
        .unwrap_or_default();
    let host_app = target
        .host_app
        .as_deref()
        .map(normalize_token)
        .unwrap_or_default();

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

    if should_skip_ambiguous_running_terminal_fallback(target) {
        warn!(
            source = %target.source,
            session_id = %target.session_id,
            cwd = ?target.cwd,
            "skipping ambiguous macOS running-terminal fallback because session has no terminal metadata"
        );
        return None;
    }

    let mut detected = Vec::new();
    for candidate in RUNNING_TERMINAL_CANDIDATES {
        if is_application_running(candidate) {
            detected.push(*candidate);
        }
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

fn is_bundle_running(bundle_id: &str) -> bool {
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

fn should_skip_ambiguous_running_terminal_fallback(target: &SessionFocusTarget) -> bool {
    target.source == "codex"
        && target.terminal_app.is_none()
        && target.terminal_bundle.is_none()
        && target.tty.is_none()
        && target.cli_pid.is_none()
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

fn resolve_focus_target_from_process_tree(target: &SessionFocusTarget) -> Option<MacFocusTarget> {
    let pid = match target
        .cli_pid
        .or_else(|| infer_cli_pid_from_running_processes(target))
        .or_else(|| {
            target
                .terminal_pid
                .filter(|pid| known_terminal_target_for_pid(*pid).is_some())
        }) {
        Some(pid) => pid,
        None => {
            info!(
                source = %target.source,
                session_id = %target.session_id,
                cwd = ?target.cwd,
                terminal_app = ?target.terminal_app,
                terminal_bundle = ?target.terminal_bundle,
                tty = ?target.tty,
                cli_pid = ?target.cli_pid,
                "macOS terminal process-tree resolution skipped because no CLI pid was available"
            );
            return None;
        }
    };

    let resolved = terminal_process_from_process_tree(pid).map(|(_, target)| target);
    if resolved.is_none() {
        warn!(
            source = %target.source,
            session_id = %target.session_id,
            cwd = ?target.cwd,
            cli_pid = pid,
            "macOS terminal process-tree resolution did not map to a known terminal"
        );
    }
    resolved
}

fn terminal_process_from_process_tree(mut pid: u32) -> Option<(u32, MacFocusTarget)> {
    for _ in 0..16 {
        if pid <= 1 {
            return None;
        }
        let info = process_info(pid)?;
        if let Some(target) = terminal_target_from_command(&info.command) {
            return Some((pid, target));
        }
        pid = info.ppid;
    }
    None
}

fn known_terminal_target_for_pid(pid: u32) -> Option<MacFocusTarget> {
    let info = process_info(pid)?;
    terminal_target_from_command(&info.command)
}

fn terminal_target_from_command(command: &str) -> Option<MacFocusTarget> {
    let value = command.to_ascii_lowercase();
    if value.contains("/terminal.app/") || value.ends_with("/terminal") {
        return Some(MacFocusTarget::TerminalApp);
    }
    if value.contains("/warp.app/") {
        return Some(MacFocusTarget::Warp);
    }
    if value.contains("/iterm.app/") || value.contains("/iterm2.app/") {
        return Some(MacFocusTarget::ITerm2);
    }
    if value.contains("/ghostty.app/") {
        return Some(MacFocusTarget::Ghostty);
    }
    if value.contains("/wezterm.app/") {
        return Some(MacFocusTarget::WezTerm);
    }
    if value.contains("/kitty.app/") {
        return Some(MacFocusTarget::Kitty);
    }
    if value.contains("/alacritty.app/") {
        return Some(MacFocusTarget::Alacritty);
    }
    None
}

fn terminal_metadata_for_target(
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

fn has_terminal_metadata(metadata: &EventMetadata) -> bool {
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

fn process_info(pid: u32) -> Option<ProcessInfo> {
    let pid_text = pid.to_string();
    let output = run_process(
        "/bin/ps",
        &["-p", &pid_text, "-o", "ppid=", "-o", "command="],
        None,
    )?;
    let mut parts = output.trim().splitn(2, char::is_whitespace);
    let ppid = parts.next()?.trim().parse::<u32>().ok()?;
    let command = parts.next().unwrap_or_default().trim().to_string();
    Some(ProcessInfo { ppid, command })
}

fn detect_tty_path_from_pid(pid: u32) -> Option<String> {
    let pid_text = pid.to_string();
    let tty = run_process("/bin/ps", &["-p", &pid_text, "-o", "tty="], None)?
        .trim()
        .to_string();
    if tty.is_empty() || tty == "??" {
        return None;
    }
    let tty_path = if tty.starts_with("/dev/") {
        tty
    } else {
        format!("/dev/{tty}")
    };
    is_precise_terminal_tty(&tty_path).then_some(tty_path)
}

fn infer_cli_pid_from_running_processes(target: &SessionFocusTarget) -> Option<u32> {
    let cwd = target.cwd.as_deref().filter(|value| !value.is_empty())?;
    let matching_pids = candidate_cli_pids_for_source(&target.source)?
        .into_iter()
        .filter(|pid| process_cwd(*pid).is_some_and(|process_cwd| same_path(&process_cwd, cwd)))
        .collect::<Vec<_>>();

    if matching_pids.is_empty() {
        warn!(
            source = %target.source,
            session_id = %target.session_id,
            cwd,
            "failed to infer CLI pid from running processes because no cwd-matched candidates were found"
        );
        return None;
    }

    if target.source == "codex" {
        let session_started_at = uuid_v7_millis(&target.session_id);
        info!(
            source = %target.source,
            session_id = %target.session_id,
            cwd,
            session_started_at = ?session_started_at,
            matching_pids = ?matching_pids,
            "evaluating running CLI processes for macOS terminal inference"
        );

        if let Some(session_started_at) = session_started_at {
            if let Some((pid, distance)) = matching_pids
                .iter()
                .filter_map(|pid| {
                    let started_at = process_start_millis(*pid)?;
                    let distance = (started_at - session_started_at).abs();
                    (distance <= 5 * 60 * 1000).then_some((*pid, distance))
                })
                .min_by_key(|(_, distance)| *distance)
            {
                info!(
                    source = %target.source,
                    session_id = %target.session_id,
                    cwd,
                    selected_pid = pid,
                    distance_ms = distance,
                    "matched CLI pid from running processes within 5-minute window"
                );
                return Some(pid);
            }
        }

        if let Some(session_started_at) = session_started_at {
            if let Some((pid, distance)) = matching_pids
                .iter()
                .filter_map(|pid| {
                    let started_at = process_start_millis(*pid)?;
                    let distance = (started_at - session_started_at).abs();
                    Some((*pid, distance))
                })
                .min_by_key(|(_, distance)| *distance)
            {
                info!(
                    source = %target.source,
                    session_id = %target.session_id,
                    cwd,
                    selected_pid = pid,
                    distance_ms = distance,
                    "matched CLI pid from nearest running process by start time"
                );
                return Some(pid);
            }
        }
    } else {
        info!(
            source = %target.source,
            session_id = %target.session_id,
            cwd,
            matching_pids = ?matching_pids,
            "evaluating running CLI processes for macOS terminal inference"
        );
    }

    if matching_pids.len() == 1 {
        let pid = matching_pids[0];
        info!(
            source = %target.source,
            session_id = %target.session_id,
            cwd,
            selected_pid = pid,
            "matched CLI pid from single cwd candidate"
        );
        return Some(pid);
    }

    if target.source == "claude" {
        if let Some((pid, started_at)) = matching_pids
            .iter()
            .filter_map(|pid| process_start_millis(*pid).map(|started_at| (*pid, started_at)))
            .max_by_key(|(_, started_at)| *started_at)
        {
            info!(
                source = %target.source,
                session_id = %target.session_id,
                cwd,
                selected_pid = pid,
                selected_started_at = started_at,
                matching_pids = ?matching_pids,
                "matched Claude CLI pid from most recently started cwd candidate"
            );
            return Some(pid);
        }
    }

    warn!(
        source = %target.source,
        session_id = %target.session_id,
        cwd,
        matching_pids = ?matching_pids,
        "failed to infer CLI pid from running processes"
    );
    None
}

fn candidate_cli_pids_for_source(source: &str) -> Option<Vec<u32>> {
    let patterns = cli_process_patterns(source)?;
    let mut matched = BTreeSet::new();

    for pattern in patterns {
        let Some(output) = run_process("/usr/bin/pgrep", &["-f", &pattern], None) else {
            continue;
        };
        for pid in output
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
        {
            matched.insert(pid);
        }
    }

    (!matched.is_empty()).then_some(matched.into_iter().collect())
}

fn cli_process_patterns(source: &str) -> Option<Vec<String>> {
    match source {
        "codex" => Some(vec![
            "/codex/codex".to_string(),
            "@openai/codex".to_string(),
            "openai-codex".to_string(),
        ]),
        "claude" => env::var("HOME").ok().map(|home| {
            vec![
                format!("{home}/.local/share/claude/versions/"),
                "/.local/share/claude/versions/".to_string(),
            ]
        }),
        _ => None,
    }
}

fn process_cwd(pid: u32) -> Option<String> {
    let pid_text = pid.to_string();
    let output = run_process(
        "/usr/sbin/lsof",
        &["-a", "-p", &pid_text, "-d", "cwd", "-Fn"],
        None,
    )?;
    output
        .lines()
        .find_map(|line| line.strip_prefix('n').map(|value| value.trim().to_string()))
        .filter(|value| !value.is_empty())
}

fn process_start_millis(pid: u32) -> Option<i64> {
    let pid_text = pid.to_string();
    let output = run_process("/bin/ps", &["-p", &pid_text, "-o", "lstart="], None)?;
    let naive = NaiveDateTime::parse_from_str(output.trim(), "%a %b %e %H:%M:%S %Y").ok()?;
    Local
        .from_local_datetime(&naive)
        .single()
        .map(|value| value.timestamp_millis())
}

fn uuid_v7_millis(session_id: &str) -> Option<i64> {
    let prefix = session_id
        .chars()
        .filter(|char| *char != '-')
        .take(12)
        .collect::<String>();
    i64::from_str_radix(&prefix, 16).ok()
}

fn same_path(left: &str, right: &str) -> bool {
    let left_trimmed = left.trim_end_matches('/');
    let right_trimmed = right.trim_end_matches('/');
    if left_trimmed == right_trimmed {
        return true;
    }
    let left = std::fs::canonicalize(left).ok();
    let right = std::fs::canonicalize(right).ok();
    left.zip(right).is_some_and(|(left, right)| left == right)
}

fn activate_iterm2(target: &SessionFocusTarget) -> bool {
    if let Some(iterm_session_id) = target
        .iterm_session_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return activate_iterm2_session_id(iterm_session_id);
    }

    let tty = effective_tty(target);
    if let Some(tty) = tty {
        let full_tty = if tty.starts_with("/dev/") {
            tty.to_string()
        } else {
            format!("/dev/{tty}")
        };
        let script = format!(
            r#"
try
    tell application "iTerm2"
        activate
        repeat with w in windows
            repeat with t in tabs of w
                repeat with s in sessions of t
                    try
                        if tty of s is "{tty}" then
                            if miniaturized of w then set miniaturized of w to false
                            select t
                            select s
                            set index of w to 1
                            return true
                        end if
                    end try
                end repeat
            end repeat
        end repeat
    end tell
end try
return false
"#,
            tty = escape_applescript(&full_tty)
        );
        if run_osascript_raw(&script, "iTerm2 tty focus")
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
        {
            info!(tty = %full_tty, "matched iTerm2 session by tty");
            return true;
        }
    }

    let cwd = target.cwd.as_deref().unwrap_or_default();
    if cwd.is_empty() {
        return activate_app_bundle("com.googlecode.iterm2", "iTerm2");
    }

    let dir_name = last_path_component(cwd);
    let script = format!(
        r#"
try
    tell application "iTerm2"
        activate
        repeat with aWindow in windows
            repeat with aTab in tabs of aWindow
                repeat with aSession in sessions of aTab
                    try
                        if name of aSession contains "{dir_name}" or path of aSession contains "{dir_name}" then
                            if miniaturized of aWindow then set miniaturized of aWindow to false
                            select aTab
                            select aSession
                            return true
                        end if
                    end try
                end repeat
            end repeat
        end repeat
    end tell
end try
return false
"#,
        dir_name = escape_applescript(&dir_name)
    );

    if run_osascript_raw(&script, "iTerm2 focus")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
    {
        info!(
            cwd,
            dir_name, "matched iTerm2 session by cwd/title fallback"
        );
        return true;
    }

    info!(cwd, "falling back to iTerm2 app activation");
    activate_app_bundle("com.googlecode.iterm2", "iTerm2")
}

fn activate_ghostty(target: &SessionFocusTarget) -> bool {
    let cwd = target.cwd.as_deref().unwrap_or_default();
    if cwd.is_empty() {
        return activate_app_bundle("com.mitchellh.ghostty", "Ghostty");
    }

    let dir_name = last_path_component(cwd);
    let source = escape_applescript(&target.source);
    let escaped_cwd = escape_applescript(cwd);
    let escaped_cwd_resolved = escape_applescript(&resolve_symlinks(cwd));
    let escaped_dir_name = escape_applescript(&dir_name);
    let escaped_tilde_cwd = escape_applescript(&tilde_path(cwd));
    let tmux_key = escape_applescript(&tmux_window_key(target));
    let tmux_session = escape_applescript(&tmux_session_name(target));
    let session_id_snippet = if let Some(session_id) = target
        .iterm_session_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        let sid = escape_applescript(session_id);
        format!(
            r#"
    repeat with t in matches
        try
            if name of t contains "{sid}" then
                focus t
                activate
                return "session_id"
            end if
        end try
    end repeat
"#
        )
    } else {
        String::new()
    };
    let script = format!(
        r#"
tell application "Ghostty"
    set allTerms to terminals
    set tmuxKey to "{tmux_key}"
    set tmuxSession to "{tmux_session}"
    if tmuxKey is not "" then
        repeat with t in allTerms
            try
                if name of t contains tmuxKey then
                    focus t
                    activate
                    return "tmux_key"
                end if
            end try
        end repeat
    end if
    if tmuxSession is not "" then
        repeat with t in allTerms
            try
                set tname to (name of t as text)
                if tname starts with (tmuxSession & ":") then
                    focus t
                    activate
                    return "tmux_session"
                end if
            end try
        end repeat
    end if
    set matches to {{}}
    try
        set matches to (every terminal whose working directory is "{cwd}")
    end try
    if (count of matches) = 0 and "{cwd_resolved}" is not "" and "{cwd_resolved}" is not "{cwd}" then
        try
            set matches to (every terminal whose working directory is "{cwd_resolved}")
        end try
    end if
    if (count of matches) = 0 then
        repeat with t in allTerms
            try
                set tname to (name of t as text)
                if ("{tilde_cwd}" is not "" and tname contains "{tilde_cwd}") or tname contains "{cwd}" or tname contains "{cwd_resolved}" or tname contains "{dir_name}" then
                    set end of matches to t
                end if
            end try
        end repeat
    end if

{session_id_snippet}

    repeat with t in matches
        try
            if name of t contains "{source}" then
                focus t
                activate
                return "source"
            end if
        end try
    end repeat

    if (count of matches) > 0 then
        focus (item 1 of matches)
        activate
        return "cwd_or_title"
    end if
    activate
    return "app_activate"
end tell
"#,
        cwd = escaped_cwd,
        cwd_resolved = escaped_cwd_resolved,
        dir_name = escaped_dir_name,
        tilde_cwd = escaped_tilde_cwd,
        source = source,
        tmux_key = tmux_key,
        tmux_session = tmux_session,
        session_id_snippet = session_id_snippet,
    );

    match run_osascript_raw(&script, "Ghostty focus") {
        Some(reason) => {
            info!(
                cwd,
                tmux_key = %tmux_key,
                tmux_session = %tmux_session,
                reason = %reason,
                "matched Ghostty terminal focus"
            );
            true
        }
        None => false,
    }
}

fn activate_terminal_app(target: &SessionFocusTarget) -> bool {
    if let Some(tty) = effective_tty(target) {
        let tty = normalize_tty(tty);
        let script = format!(
            r#"
tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            try
                if tty of t is "{tty}" then
                    if miniaturized of w then set miniaturized of w to false
                    set selected tab of w to t
                    set index of w to 1
                    activate
                    return true
                end if
            end try
        end repeat
    end repeat
end tell
return false
"#,
            tty = escape_applescript(&tty)
        );
        if run_osascript_raw(&script, "Terminal.app tty focus")
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
        {
            info!(tty = %tty, "matched Terminal.app tab by tty");
            return true;
        }
    }

    let cwd = target.cwd.as_deref().unwrap_or_default();
    if cwd.is_empty() {
        return activate_app_bundle("com.apple.Terminal", "Terminal");
    }

    let dir_name = last_path_component(cwd);
    let script = format!(
        r#"
tell application "Terminal"
    repeat with w in windows
        repeat with t in tabs of w
            try
                if custom title of t contains "{dir_name}" then
                    if miniaturized of w then set miniaturized of w to false
                    set selected tab of w to t
                    set index of w to 1
                    activate
                    return true
                end if
            end try
        end repeat
    end repeat
end tell
return false
"#,
        dir_name = escape_applescript(&dir_name)
    );

    if run_osascript_raw(&script, "Terminal.app cwd focus")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
    {
        info!(
            cwd,
            dir_name, "matched Terminal.app tab by cwd/title fallback"
        );
        return true;
    }

    info!(cwd, "falling back to Terminal.app activation");
    activate_app_bundle("com.apple.Terminal", "Terminal")
}

fn activate_wezterm(target: &SessionFocusTarget) -> bool {
    let focused = activate_app_bundle("com.github.wez.wezterm", "WezTerm");
    let Some(bin) = find_binary("wezterm") else {
        return focused;
    };
    let mut tab_id = None;
    if let Some(tty) = effective_tty(target) {
        tab_id = wezterm_tab_id_for_tty(&bin, tty);
    }
    if tab_id.is_none() {
        tab_id = target
            .cwd
            .as_deref()
            .and_then(|cwd| wezterm_tab_id_for_cwd(&bin, cwd));
    }
    if let Some(tab_id) = tab_id {
        info!(tab_id, tty = ?effective_tty(target), cwd = ?target.cwd, "matched WezTerm tab");
        let _ = run_process(&bin, &["cli", "activate-tab", "--tab-id", &tab_id], None);
        return true;
    }
    info!(tty = ?effective_tty(target), cwd = ?target.cwd, "falling back to WezTerm app activation");
    focused
}

fn activate_kitty(target: &SessionFocusTarget) -> bool {
    let focused = activate_app_bundle("net.kovidgoyal.kitty", "kitty");
    let Some(bin) = find_binary("kitten") else {
        return focused;
    };
    if let Some(window_id) = target
        .kitty_window_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        info!(window_id, "matched kitty window by window id");
        let _ = run_process(
            &bin,
            &["@", "focus-window", "--match", &format!("id:{window_id}")],
            None,
        );
        return true;
    }
    if let Some(cwd) = target.cwd.as_deref().filter(|value| !value.is_empty()) {
        if run_process(
            &bin,
            &["@", "focus-tab", "--match", &format!("cwd:{cwd}")],
            None,
        )
        .is_some()
        {
            info!(cwd, "matched kitty tab by cwd");
            return true;
        }
        if run_process(
            &bin,
            &[
                "@",
                "focus-tab",
                "--match",
                &format!("title:{}", target.source),
            ],
            None,
        )
        .is_some()
        {
            info!(source = %target.source, "matched kitty tab by source title");
            return true;
        }
    }
    info!(cwd = ?target.cwd, "falling back to kitty app activation");
    focused
}

fn activate_iterm2_session_id(session_id: &str) -> bool {
    let script = format!(
        r#"
try
    tell application "iTerm2"
        activate
        repeat with aWindow in windows
            if miniaturized of aWindow then set miniaturized of aWindow to false
            repeat with aTab in tabs of aWindow
                repeat with aSession in sessions of aTab
                    try
                        if unique ID of aSession is "{session_id}" then
                            select aTab
                            select aSession
                            return true
                        end if
                    end try
                end repeat
            end repeat
        end repeat
    end tell
end try
return false
"#,
        session_id = escape_applescript(session_id),
    );
    if run_osascript_raw(&script, "iTerm2 session focus")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
    {
        info!(session_id, "matched iTerm2 session by session id");
        return true;
    }

    info!(
        session_id,
        "falling back to iTerm2 app activation after session id miss"
    );
    activate_app_bundle("com.googlecode.iterm2", "iTerm2")
}

fn activate_terminal_window(bundle_id: &str, cwd: Option<&str>, fallback_name: &str) -> bool {
    let activated = if bundle_id.is_empty() {
        activate_app_name(fallback_name)
    } else {
        activate_app_bundle(bundle_id, fallback_name)
    };
    let Some(cwd) = cwd.filter(|value| !value.is_empty()) else {
        return activated;
    };
    let resolved_cwd = resolve_symlinks(cwd);
    let tilde_cwd = tilde_path(cwd);
    let folder_name = last_path_component(cwd);
    if folder_name.is_empty() {
        return activated;
    }
    let script = format!(
        r#"
tell application "System Events"
    tell process "{app_name}"
        set frontmost to true
        set bestWindow to missing value
        set bestReason to ""
        set bestScore to -1
        set bestLen to 999999
        repeat with w in windows
            try
                set wName to name of w as text
                set score to 0
                set reason to ""
                if "{cwd}" is not "" and wName contains "{cwd}" then
                    set score to 4
                    set reason to "cwd"
                else if "{resolved_cwd}" is not "" and "{resolved_cwd}" is not "{cwd}" and wName contains "{resolved_cwd}" then
                    set score to 3
                    set reason to "resolved_cwd"
                else if "{tilde_cwd}" is not "" and wName contains "{tilde_cwd}" then
                    set score to 2
                    set reason to "tilde_cwd"
                else if "{folder_name}" is not "" and wName contains "{folder_name}" then
                    set score to 1
                    set reason to "folder_name"
                end if

                if score > 0 then
                    set wLen to count of wName
                    if score > bestScore or (score = bestScore and wLen < bestLen) then
                        set bestWindow to w
                        set bestReason to reason
                        set bestScore to score
                        set bestLen to wLen
                    end if
                end if
            end try
        end repeat
        if bestWindow is not missing value then
            perform action "AXRaise" of bestWindow
            return bestReason
        end if
    end tell
end tell
return false
"#,
        app_name = escape_applescript(fallback_name),
        cwd = escape_applescript(cwd),
        resolved_cwd = escape_applescript(&resolved_cwd),
        tilde_cwd = escape_applescript(&tilde_cwd),
        folder_name = escape_applescript(&folder_name),
    );
    match run_osascript_raw(&script, fallback_name) {
        Some(reason) if !reason.trim().eq_ignore_ascii_case("false") => {
            info!(
                bundle_id,
                fallback_name,
                cwd,
                resolved_cwd = %resolved_cwd,
                tilde_cwd = %tilde_cwd,
                folder_name,
                reason = %reason,
                "matched generic terminal window"
            );
            true
        }
        _ => {
            info!(
                bundle_id,
                fallback_name,
                cwd,
                resolved_cwd = %resolved_cwd,
                tilde_cwd = %tilde_cwd,
                folder_name,
                activated,
                "generic terminal window title match missed; using app activation result"
            );
            activated
        }
    }
}

fn activate_ide_window(bundle_id: &str, cwd: Option<&str>, fallback_name: &str) -> bool {
    let activated = activate_app_bundle(bundle_id, fallback_name);
    let Some(cwd) = cwd.filter(|value| !value.is_empty()) else {
        return activated;
    };
    let folder_name = last_path_component(cwd);
    if folder_name.is_empty() {
        return activated;
    }
    let script = format!(
        r#"
tell application "System Events"
    tell process "{app_name}"
        set frontmost to true
        set bestWindow to missing value
        set bestLen to 999999
        repeat with w in windows
            try
                set wName to name of w as text
                if wName contains "{folder_name}" then
                    set wLen to count of wName
                    if wLen < bestLen then
                        set bestWindow to w
                        set bestLen to wLen
                    end if
                end if
            end try
        end repeat
        if bestWindow is not missing value then
            perform action "AXRaise" of bestWindow
            return true
        end if
    end tell
end tell
return false
"#,
        app_name = escape_applescript(fallback_name),
        folder_name = escape_applescript(&folder_name),
    );
    run_osascript_bool(&script, fallback_name) || activated
}

fn native_app_bundle_for_terminal(
    target: &SessionFocusTarget,
) -> Option<(&'static str, &'static str)> {
    target
        .terminal_bundle
        .as_deref()
        .and_then(native_app_bundle_display_name)
}

fn native_app_bundle_display_name(bundle_id: &str) -> Option<(&'static str, &'static str)> {
    match bundle_id {
        "com.openai.codex" => Some(("com.openai.codex", "Codex")),
        "com.trae.app" => Some(("com.trae.app", "Trae")),
        "com.qoder.ide" => Some(("com.qoder.ide", "Qoder")),
        "com.factory.app" => Some(("com.factory.app", "Factory")),
        "com.tencent.codebuddy" => Some(("com.tencent.codebuddy", "CodeBuddy")),
        "com.tencent.codebuddy.cn" => Some(("com.tencent.codebuddy.cn", "CodyBuddyCN")),
        "com.stepfun.app" => Some(("com.stepfun.app", "StepFun")),
        "ai.opencode.desktop" => Some(("ai.opencode.desktop", "OpenCode")),
        _ => None,
    }
}

fn source_native_app_bundle(source: &str) -> Option<(&'static str, &'static str)> {
    match source {
        "codex" => Some(("com.openai.codex", "Codex")),
        "trae" | "traecn" => Some(("com.trae.app", "Trae")),
        "qoder" => Some(("com.qoder.ide", "Qoder")),
        "droid" => Some(("com.factory.app", "Factory")),
        "codebuddy" => Some(("com.tencent.codebuddy", "CodeBuddy")),
        "codybuddycn" => Some(("com.tencent.codebuddy.cn", "CodyBuddyCN")),
        "stepfun" => Some(("com.stepfun.app", "StepFun")),
        "opencode" => Some(("ai.opencode.desktop", "OpenCode")),
        _ => None,
    }
}

fn activate_app_bundle(bundle_id: &str, display_name: &str) -> bool {
    let opened = run_command_success("/usr/bin/open", &["-b", bundle_id], None);
    let script = format!(
        r#"try
tell application id "{bundle_id}" to activate
on error
    tell application "{display_name}" to activate
end try
"#,
        bundle_id = escape_applescript(bundle_id),
        display_name = escape_applescript(display_name)
    );
    run_osascript(&script, display_name) || opened
}

fn activate_app_name(display_name: &str) -> bool {
    let opened = run_command_success("/usr/bin/open", &["-a", display_name], None);
    let script = format!(
        r#"tell application "{display_name}" to activate
"#,
        display_name = escape_applescript(display_name)
    );
    run_osascript(&script, display_name) || opened
}

fn effective_tty(target: &SessionFocusTarget) -> Option<&str> {
    if target
        .tmux_pane
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return target
            .tmux_client_tty
            .as_deref()
            .filter(|value| is_precise_terminal_tty(value));
    }
    target
        .tty
        .as_deref()
        .filter(|value| is_precise_terminal_tty(value))
}

fn tmux_window_key(target: &SessionFocusTarget) -> String {
    let Some(tmux_pane) = target
        .tmux_pane
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let Some(bin) = find_binary("tmux") else {
        return String::new();
    };
    let env_pairs = target
        .tmux_env
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| [("TMUX", value)]);
    run_process(
        &bin,
        &[
            "display-message",
            "-p",
            "-t",
            tmux_pane,
            "-F",
            "#{session_name}:#{window_index}:#{window_name}",
        ],
        env_pairs.as_ref().map(|pairs| &pairs[..]),
    )
    .unwrap_or_default()
}

fn tmux_session_name(target: &SessionFocusTarget) -> String {
    let Some(tmux_pane) = target
        .tmux_pane
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let Some(bin) = find_binary("tmux") else {
        return String::new();
    };
    let env_pairs = target
        .tmux_env
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| [("TMUX", value)]);
    run_process(
        &bin,
        &[
            "display-message",
            "-p",
            "-t",
            tmux_pane,
            "-F",
            "#{session_name}",
        ],
        env_pairs.as_ref().map(|pairs| &pairs[..]),
    )
    .unwrap_or_default()
}

fn find_binary(name: &str) -> Option<String> {
    [
        format!("/opt/homebrew/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
    ]
    .into_iter()
    .find(|path| std::fs::metadata(path).is_ok())
}

fn run_process(path: &str, args: &[&str], extra_env: Option<&[(&str, &str)]>) -> Option<String> {
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

fn run_command_success(path: &str, args: &[&str], extra_env: Option<&[(&str, &str)]>) -> bool {
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

fn wezterm_tab_id_for_tty(bin: &str, tty: &str) -> Option<String> {
    let json = run_process(bin, &["cli", "list", "--format", "json"], None)?;
    parse_wezterm_tab_id(&json, |pane| {
        pane.get("tty_name").and_then(|value| value.as_str()) == Some(tty)
    })
}

fn wezterm_tab_id_for_cwd(bin: &str, cwd: &str) -> Option<String> {
    let json = run_process(bin, &["cli", "list", "--format", "json"], None)?;
    let cwd_url = format!("file://{cwd}");
    parse_wezterm_tab_id(&json, |pane| {
        pane.get("cwd")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == cwd || value == cwd_url)
    })
}

fn parse_wezterm_tab_id(
    raw_json: &str,
    predicate: impl Fn(&serde_json::Value) -> bool,
) -> Option<String> {
    let panes = serde_json::from_str::<Vec<serde_json::Value>>(raw_json).ok()?;
    panes.into_iter().find_map(|pane| {
        if !predicate(&pane) {
            return None;
        }
        pane.get("tab_id")
            .map(|value| match value {
                serde_json::Value::String(value) => value.clone(),
                serde_json::Value::Number(value) => value.to_string(),
                _ => String::new(),
            })
            .filter(|value| !value.is_empty())
    })
}

fn resolve_symlinks(path: &str) -> String {
    std::fs::canonicalize(path)
        .ok()
        .and_then(|value| value.to_str().map(|value| value.to_string()))
        .unwrap_or_else(|| path.to_string())
}

fn tilde_path(path: &str) -> String {
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

fn run_osascript(source: &str, label: &str) -> bool {
    let Some(output) = run_osascript_raw(source, label) else {
        return false;
    };
    debug!(label, output = %output, "osascript command succeeded");
    true
}

fn run_osascript_bool(source: &str, label: &str) -> bool {
    run_osascript_raw(source, label).is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
}

fn run_osascript_raw(source: &str, label: &str) -> Option<String> {
    let mut child = match Command::new("/usr/bin/osascript")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            warn!(label, error = %error, "failed to spawn osascript");
            return None;
        }
    };

    if let Some(stdin) = child.stdin.as_mut()
        && let Err(error) = stdin.write_all(source.as_bytes())
    {
        warn!(label, error = %error, "failed to write osascript source");
        let _ = child.kill();
        return None;
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            warn_macos_permission_hint(&stderr);
            warn!(label, stderr, "osascript command failed");
            None
        }
        Err(error) => {
            warn!(label, error = %error, "failed waiting for osascript");
            None
        }
    }
}

fn warn_macos_permission_hint(stderr: &str) {
    if stderr.is_empty() {
        return;
    }

    if is_accessibility_permission_error(stderr) {
        if !DID_WARN_ACCESSIBILITY_PERMISSION.swap(true, Ordering::Relaxed) {
            warn!(
                "macOS terminal focus requires Accessibility permission. Enable the terminal app that launched EchoIsland in System Settings > Privacy & Security > Accessibility."
            );
        }
        return;
    }

    if is_automation_permission_error(stderr)
        && !DID_WARN_AUTOMATION_PERMISSION.swap(true, Ordering::Relaxed)
    {
        warn!(
            "macOS terminal focus requires Automation permission. Allow the launcher app to control System Events and your terminal app in System Settings > Privacy & Security > Automation."
        );
    }
}

fn is_accessibility_permission_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("(-25211)") || lower.contains("assistive access") || lower.contains("辅助访问")
}

fn is_automation_permission_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("(-1743)")
        || lower.contains("not authorized to send apple events")
        || lower.contains("apple events")
        || lower.contains("不允许发送 apple event")
        || lower.contains("不允许控制")
}

fn last_path_component(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    trimmed
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

fn normalize_token(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '.'], "_")
}

fn normalize_tty(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("/dev/") {
        trimmed.to_string()
    } else {
        format!("/dev/{trimmed}")
    }
}

fn is_precise_terminal_tty(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "/dev/tty" && trimmed != "/dev/console"
}

fn escape_applescript(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::{MacFocusTarget, cli_process_patterns, resolve_focus_target};
    use crate::terminal_focus::SessionFocusTarget;

    fn target() -> SessionFocusTarget {
        SessionFocusTarget {
            session_id: "019d915a-ea19-7180-8c0a-8f40316d0526".to_string(),
            source: "codex".to_string(),
            project_name: None,
            cwd: Some("/Users/wenuts".to_string()),
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
        }
    }

    #[test]
    fn terminal_bundle_beats_apple_terminal_token() {
        let mut target = target();
        target.terminal_app = Some("Apple_Terminal".to_string());
        target.terminal_bundle = Some("dev.warp.Warp-Stable".to_string());

        assert_eq!(resolve_focus_target(&target), Some(MacFocusTarget::Warp));
    }

    #[test]
    fn inherited_apple_terminal_token_does_not_force_terminal_app() {
        let mut target = target();
        target.terminal_app = Some("Apple_Terminal".to_string());

        assert_ne!(
            resolve_focus_target(&target),
            Some(MacFocusTarget::TerminalApp)
        );
    }

    #[test]
    fn codex_cli_patterns_include_primary_launch_modes() {
        let patterns = cli_process_patterns("codex").expect("codex patterns");

        assert!(patterns.iter().any(|value| value == "/codex/codex"));
        assert!(patterns.iter().any(|value| value == "@openai/codex"));
        assert!(patterns.iter().any(|value| value == "openai-codex"));
    }

    #[test]
    fn claude_cli_patterns_include_versioned_install_dir() {
        let patterns = cli_process_patterns("claude").expect("claude patterns");

        assert!(
            patterns
                .iter()
                .any(|value| value.contains("/.local/share/claude/versions/"))
        );
    }
}
