use anyhow::Result;
use echoisland_core::EventMetadata;
use tracing::info;

use super::{FocusOutcome, ForegroundTabInfo, SessionFocusTarget, SessionTabCache};

mod native_apps;
mod osascript;
mod process_inference;
mod target;
mod title_match;
mod tmux;
mod util;

use native_apps::{
    activate_app_bundle, activate_app_name, native_app_bundle_for_terminal,
    source_native_app_bundle,
};
use osascript::{escape_applescript, run_osascript_raw};
#[cfg(test)]
use process_inference::{cli_process_patterns, terminal_target_from_command};
use process_inference::{
    detect_tty_path_from_pid, infer_cli_pid_from_running_processes, known_terminal_target_for_pid,
    terminal_process_from_process_tree,
};
#[cfg(test)]
use target::should_skip_detected_running_terminal_fallback;
use target::{
    MacFocusTarget, has_terminal_metadata, is_bundle_running, resolve_focus_target,
    should_use_source_native_app_fallback, terminal_metadata_for_target,
};
use title_match::{
    applescript_title_candidate_records, ide_window_title_candidates,
    terminal_window_title_candidates,
};
use tmux::{effective_tty, tmux_session_name, tmux_window_key};
use util::{
    find_binary, last_path_component, normalize_tty, resolve_symlinks, run_process, tilde_path,
};

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

    let resolved_target = resolve_focus_target(target);
    info!(
        source = %target.source,
        terminal_app = ?target.terminal_app,
        host_app = ?target.host_app,
        cwd = ?target.cwd,
        resolved_target = ?resolved_target,
        "trying macOS terminal focus"
    );

    if resolved_target.is_none()
        && should_use_source_native_app_fallback(target)
        && let Some((bundle_id, display_name)) = source_native_app_bundle(&target.source)
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

    let focused = match resolved_target {
        Some(MacFocusTarget::ITerm2) => activate_iterm2(target),
        Some(MacFocusTarget::Ghostty) => activate_ghostty(target),
        Some(MacFocusTarget::TerminalApp) => activate_terminal_app(target),
        Some(MacFocusTarget::Warp) => {
            activate_terminal_window(target, "dev.warp.Warp-Stable", "Warp")
        }
        Some(MacFocusTarget::WezTerm) => activate_wezterm(target),
        Some(MacFocusTarget::Kitty) => activate_kitty(target),
        Some(MacFocusTarget::Alacritty) => {
            activate_terminal_window(target, "org.alacritty", "Alacritty")
        }
        Some(MacFocusTarget::Hyper) => activate_terminal_window(target, "", "Hyper"),
        Some(MacFocusTarget::Tabby) => activate_terminal_window(target, "", "Tabby"),
        Some(MacFocusTarget::Rio) => activate_terminal_window(target, "", "Rio"),
        Some(MacFocusTarget::Cursor) => {
            activate_ide_window(target, "com.todesktop.230313mzl4w4u92", "Cursor")
        }
        Some(MacFocusTarget::VSCode) => {
            activate_ide_window(target, "com.microsoft.VSCode", "Visual Studio Code")
        }
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

fn activate_terminal_window(
    target: &SessionFocusTarget,
    bundle_id: &str,
    fallback_name: &str,
) -> bool {
    let activated = if bundle_id.is_empty() {
        activate_app_name(fallback_name)
    } else {
        activate_app_bundle(bundle_id, fallback_name)
    };
    let candidates = terminal_window_title_candidates(target);
    if candidates.is_empty() {
        return activated;
    }
    let candidate_records = applescript_title_candidate_records(&candidates);
    let script = format!(
        r#"
tell application "System Events"
    tell process "{app_name}"
        set frontmost to true
        set titleCandidates to {title_candidates}
        set bestWindow to missing value
        set bestReason to ""
        set bestScore to -1
        set bestMatchLen to -1
        set bestLen to 999999
        repeat with w in windows
            try
                set wName to name of w as text
                set score to 0
                set reason to ""
                set matchLen to 0
                repeat with candidate in titleCandidates
                    set matchText to item 1 of candidate
                    set matchReason to item 2 of candidate
                    set matchScore to item 3 of candidate
                    if matchText is not "" and wName contains matchText and matchScore > score then
                        set score to matchScore
                        set reason to matchReason
                        set matchLen to count of matchText
                    end if
                end repeat

                if score > 0 then
                    set wLen to count of wName
                    if score > bestScore or (score = bestScore and matchLen > bestMatchLen) or (score = bestScore and matchLen = bestMatchLen and wLen < bestLen) then
                        set bestWindow to w
                        set bestReason to reason
                        set bestScore to score
                        set bestMatchLen to matchLen
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
        title_candidates = candidate_records,
    );
    match run_osascript_raw(&script, fallback_name) {
        Some(reason) if !reason.trim().eq_ignore_ascii_case("false") => {
            info!(
                bundle_id,
                fallback_name,
                cwd = ?target.cwd,
                project_name = ?target.project_name,
                window_title = ?target.window_title,
                reason = %reason,
                "matched generic terminal window"
            );
            true
        }
        _ => {
            info!(
                bundle_id,
                fallback_name,
                cwd = ?target.cwd,
                project_name = ?target.project_name,
                window_title = ?target.window_title,
                activated,
                "generic terminal window title match missed; using app activation result"
            );
            activated
        }
    }
}

fn activate_ide_window(target: &SessionFocusTarget, bundle_id: &str, fallback_name: &str) -> bool {
    let activated = activate_app_bundle(bundle_id, fallback_name);
    let candidates = ide_window_title_candidates(target);
    if candidates.is_empty() {
        return activated;
    }
    let candidate_records = applescript_title_candidate_records(&candidates);
    let script = format!(
        r#"
tell application "System Events"
    tell process "{app_name}"
        set frontmost to true
        set titleCandidates to {title_candidates}
        set bestWindow to missing value
        set bestReason to ""
        set bestScore to -1
        set bestMatchLen to -1
        set bestLen to 999999
        repeat with w in windows
            try
                set wName to name of w as text
                set score to 0
                set reason to ""
                set matchLen to 0
                repeat with candidate in titleCandidates
                    set matchText to item 1 of candidate
                    set matchReason to item 2 of candidate
                    set matchScore to item 3 of candidate
                    if matchText is not "" and wName contains matchText and matchScore > score then
                        set score to matchScore
                        set reason to matchReason
                        set matchLen to count of matchText
                    end if
                end repeat
                if score > 0 then
                    set wLen to count of wName
                    if score > bestScore or (score = bestScore and matchLen > bestMatchLen) or (score = bestScore and matchLen = bestMatchLen and wLen < bestLen) then
                        set bestWindow to w
                        set bestReason to reason
                        set bestScore to score
                        set bestMatchLen to matchLen
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
        title_candidates = candidate_records,
    );
    let matched = run_osascript_raw(&script, fallback_name)
        .filter(|reason| !reason.trim().eq_ignore_ascii_case("false"));
    if let Some(reason) = matched {
        info!(
            bundle_id,
            fallback_name,
            cwd = ?target.cwd,
            project_name = ?target.project_name,
            window_title = ?target.window_title,
            reason = %reason,
            "matched IDE window"
        );
        true
    } else {
        activated
    }
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

#[cfg(test)]
mod tests {
    use super::{
        MacFocusTarget, applescript_title_candidate_records, cli_process_patterns,
        ide_window_title_candidates, resolve_focus_target,
        should_skip_detected_running_terminal_fallback, should_use_source_native_app_fallback,
        terminal_target_from_command, terminal_window_title_candidates,
    };
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
    fn host_app_bundle_resolves_native_desktop_target() {
        let mut target = target();
        target.host_app = Some("com.openai.codex".to_string());

        assert_eq!(
            resolve_focus_target(&target),
            Some(MacFocusTarget::CodexApp)
        );
    }

    #[test]
    fn terminal_app_bundle_resolves_before_token_normalization() {
        let mut target = target();
        target.terminal_app = Some("com.github.wez.wezterm".to_string());

        assert_eq!(resolve_focus_target(&target), Some(MacFocusTarget::WezTerm));
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

    #[test]
    fn command_basename_detection_matches_wezterm_gui() {
        assert_eq!(
            terminal_target_from_command("/opt/homebrew/bin/wezterm-gui start --cwd /tmp"),
            Some(MacFocusTarget::WezTerm)
        );
    }

    #[test]
    fn quoted_command_path_detection_matches_kitty_binary() {
        assert_eq!(
            terminal_target_from_command("\"/Applications/kitty.app/Contents/MacOS/kitty\" @ ls"),
            Some(MacFocusTarget::Kitty)
        );
    }

    #[test]
    fn command_basename_detection_matches_ghostty_binary() {
        assert_eq!(
            terminal_target_from_command("/usr/local/bin/ghostty +list-fonts"),
            Some(MacFocusTarget::Ghostty)
        );
    }

    #[test]
    fn multiple_running_terminals_are_not_guessed_without_reliable_metadata() {
        let target = target();

        assert!(should_skip_detected_running_terminal_fallback(&target, 2));
    }

    #[test]
    fn single_running_terminal_can_be_used_for_legacy_metadata_miss() {
        let target = target();

        assert!(!should_skip_detected_running_terminal_fallback(&target, 1));
    }

    #[test]
    fn explicit_terminal_app_allows_running_terminal_fallback_even_when_multiple() {
        let mut target = target();
        target.terminal_app = Some("Warp".to_string());

        assert!(!should_skip_detected_running_terminal_fallback(&target, 2));
    }

    #[test]
    fn source_native_app_fallback_is_skipped_when_terminal_metadata_exists() {
        let mut target = target();
        target.terminal_app = Some("Warp".to_string());

        assert!(!should_use_source_native_app_fallback(&target));
    }

    #[test]
    fn terminal_title_candidates_prioritize_precise_window_title_and_cwd() {
        let mut target = target();
        target.cwd = Some("/Users/wenuts/Documents/EchoIsland".to_string());
        target.project_name = Some("EchoIsland".to_string());
        target.window_title = Some("EchoIsland — codex".to_string());

        let candidates = terminal_window_title_candidates(&target);

        assert_eq!(candidates[0].reason, "window_title");
        assert_eq!(candidates[0].score, 6);
        assert_eq!(candidates[1].reason, "cwd");
        assert_eq!(candidates[1].score, 5);
        assert!(candidates.iter().any(|candidate| {
            candidate.reason == "folder_name" && candidate.value == "EchoIsland"
        }));
    }

    #[test]
    fn ide_title_candidates_prefer_project_name_over_folder_and_cwd() {
        let mut target = target();
        target.cwd = Some("/Users/wenuts/Documents/EchoIsland".to_string());
        target.project_name = Some("EchoIsland Workspace".to_string());

        let candidates = ide_window_title_candidates(&target);

        assert_eq!(candidates[0].reason, "project_name");
        assert_eq!(candidates[0].score, 4);
        assert_eq!(candidates[1].reason, "folder_name");
        assert_eq!(candidates[1].score, 3);
        assert_eq!(candidates[2].reason, "cwd");
        assert_eq!(candidates[2].score, 2);
    }

    #[test]
    fn applescript_title_candidate_records_escape_values() {
        let mut target = target();
        target.window_title = Some("Echo \"Island\"".to_string());

        let records =
            applescript_title_candidate_records(&terminal_window_title_candidates(&target));

        assert!(records.contains("Echo \\\"Island\\\""));
        assert!(records.starts_with("{{"));
        assert!(records.ends_with("}"));
    }
}
