use std::{collections::BTreeSet, env};

use chrono::{Local, NaiveDateTime, TimeZone};
use tracing::{info, warn};

use super::{
    MacFocusTarget, SessionFocusTarget,
    util::{is_precise_terminal_tty, run_process},
};

struct ProcessInfo {
    ppid: u32,
    command: String,
}

pub(super) fn resolve_focus_target_from_process_tree(
    target: &SessionFocusTarget,
) -> Option<MacFocusTarget> {
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

pub(super) fn terminal_process_from_process_tree(mut pid: u32) -> Option<(u32, MacFocusTarget)> {
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

pub(super) fn known_terminal_target_for_pid(pid: u32) -> Option<MacFocusTarget> {
    let info = process_info(pid)?;
    terminal_target_from_command(&info.command)
}

pub(super) fn terminal_target_from_command(command: &str) -> Option<MacFocusTarget> {
    let value = command.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
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
    command_executable_name(&value).and_then(terminal_target_from_executable_name)
}

fn command_executable_name(command: &str) -> Option<&str> {
    let executable = if let Some(rest) = command.strip_prefix('"') {
        rest.split_once('"').map(|(value, _)| value).unwrap_or(rest)
    } else {
        command.split_whitespace().next()?
    };
    let executable = executable.trim().trim_matches('"').trim_end_matches('/');
    let name = executable
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or(executable);
    (!name.is_empty()).then_some(name)
}

fn terminal_target_from_executable_name(name: &str) -> Option<MacFocusTarget> {
    match name {
        "iterm" | "iterm2" => Some(MacFocusTarget::ITerm2),
        "ghostty" => Some(MacFocusTarget::Ghostty),
        "terminal" => Some(MacFocusTarget::TerminalApp),
        "wezterm" | "wezterm-gui" => Some(MacFocusTarget::WezTerm),
        "kitty" => Some(MacFocusTarget::Kitty),
        "alacritty" => Some(MacFocusTarget::Alacritty),
        "hyper" => Some(MacFocusTarget::Hyper),
        "tabby" => Some(MacFocusTarget::Tabby),
        "rio" => Some(MacFocusTarget::Rio),
        _ => None,
    }
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

pub(super) fn detect_tty_path_from_pid(pid: u32) -> Option<String> {
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

pub(super) fn infer_cli_pid_from_running_processes(target: &SessionFocusTarget) -> Option<u32> {
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

pub(super) fn cli_process_patterns(source: &str) -> Option<Vec<String>> {
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
