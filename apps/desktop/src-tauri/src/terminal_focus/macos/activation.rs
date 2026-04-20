use tracing::info;

use super::{
    SessionFocusTarget,
    native_apps::{activate_app_bundle, activate_app_name},
    osascript::{escape_applescript, run_osascript_raw},
    title_match::{
        applescript_title_candidate_records, ide_window_title_candidates,
        terminal_window_title_candidates,
    },
    tmux::{effective_tty, tmux_session_name, tmux_window_key},
    util::{
        find_binary, last_path_component, normalize_tty, resolve_symlinks, run_process, tilde_path,
    },
};

pub(super) fn activate_iterm2(target: &SessionFocusTarget) -> bool {
    if let Some(iterm_session_id) = target
        .iterm_session_id
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return activate_iterm2_session_id(iterm_session_id);
    }

    if let Some(tty) = effective_tty(target) {
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

pub(super) fn activate_ghostty(target: &SessionFocusTarget) -> bool {
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

pub(super) fn activate_terminal_app(target: &SessionFocusTarget) -> bool {
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

pub(super) fn activate_wezterm(target: &SessionFocusTarget) -> bool {
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

pub(super) fn activate_kitty(target: &SessionFocusTarget) -> bool {
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

pub(super) fn activate_terminal_window(
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

pub(super) fn activate_ide_window(
    target: &SessionFocusTarget,
    bundle_id: &str,
    fallback_name: &str,
) -> bool {
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
