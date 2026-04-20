use anyhow::Result;
use echoisland_core::EventMetadata;
use tracing::info;

use super::{FocusOutcome, ForegroundTabInfo, SessionFocusTarget, SessionTabCache};

mod activation;
mod native_apps;
mod osascript;
mod process_inference;
mod target;
mod title_match;
mod tmux;
mod util;

use activation::{
    activate_ghostty, activate_ide_window, activate_iterm2, activate_kitty, activate_terminal_app,
    activate_terminal_window, activate_wezterm,
};
use native_apps::{activate_app_bundle, native_app_bundle_for_terminal, source_native_app_bundle};
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
#[cfg(test)]
use title_match::{
    applescript_title_candidate_records, ide_window_title_candidates,
    terminal_window_title_candidates,
};
use tmux::effective_tty;

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
