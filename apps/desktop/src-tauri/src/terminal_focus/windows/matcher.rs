use super::window_enum::WindowCandidate;
use crate::terminal_focus::{SessionFocusTarget, focus_tokens, host_app_aliases, normalized_token};

pub(super) fn select_window_candidate<'a>(
    windows: &'a [WindowCandidate],
    target: &SessionFocusTarget,
) -> (Option<&'a WindowCandidate>, Vec<String>, Vec<String>, usize) {
    let tokens = focus_tokens(target);
    let terminal_window_count = windows
        .iter()
        .filter(|window| window.is_terminal_like)
        .count();
    let target_window_title = target.window_title.as_deref().and_then(normalized_token);
    let terminal_app = target.terminal_app.as_deref().and_then(normalized_token);
    let host_app = target.host_app.as_deref().and_then(normalized_token);
    let host_aliases = host_app_aliases(&target.source, target.host_app.as_deref())
        .into_iter()
        .filter_map(|value| normalized_token(&value))
        .collect::<Vec<_>>();
    let host_alias_window_count = windows
        .iter()
        .filter(|candidate| candidate_matches_host_aliases(candidate, &host_aliases))
        .count();
    let source = normalized_token(&target.source);

    let mut candidate_logs = Vec::new();
    let mut best: Option<(i32, &WindowCandidate)> = None;
    for candidate in windows {
        if !candidate.is_terminal_like && target.terminal_pid != Some(candidate.pid) {
            continue;
        }

        let mut score = 0i32;
        let title = candidate.title.to_ascii_lowercase();
        let process_name = candidate.process_name.to_ascii_lowercase();
        let mut matched = false;

        if target.terminal_pid == Some(candidate.pid) {
            score += 500;
            matched = true;
        }

        if let Some(window_title) = &target_window_title {
            if title == *window_title {
                score += 240;
                matched = true;
            } else if title.contains(window_title) {
                score += 180;
                matched = true;
            }
        }

        for token in &tokens {
            if title.contains(token) {
                score += 95;
                matched = true;
            }
        }

        if let Some(app) = &terminal_app {
            if process_name.contains(app) || title.contains(app) {
                score += 70;
                matched = true;
            }
        }

        if let Some(app) = &host_app {
            if process_name.contains(app) || title.contains(app) {
                score += 45;
                matched = true;
            }
        }

        for alias in &host_aliases {
            if process_name == *alias {
                score += 120;
                matched = true;
            } else if process_name.contains(alias) || title.contains(alias) {
                score += 80;
                matched = true;
            }
        }

        if host_alias_window_count == 1 && candidate_matches_host_aliases(candidate, &host_aliases)
        {
            score += 90;
            matched = true;
        }

        if let Some(source) = &source
            && title.contains(source)
        {
            score += 20;
            matched = true;
        }

        if candidate.is_terminal_like {
            score += 10;
        }

        if !matched && candidate.is_terminal_like && terminal_window_count == 1 {
            score += 25;
            matched = true;
        }

        if matched {
            candidate_logs.push(format!(
                "score={score} pid={} proc={} title={}",
                candidate.pid, candidate.process_name, candidate.title
            ));
            if best
                .as_ref()
                .is_none_or(|(best_score, _)| score > *best_score)
            {
                best = Some((score, candidate));
            }
        }
    }

    (
        best.map(|(_, candidate)| candidate),
        candidate_logs,
        host_aliases,
        tokens.len(),
    )
}

fn candidate_matches_host_aliases(candidate: &WindowCandidate, aliases: &[String]) -> bool {
    let process_name = candidate.process_name.to_ascii_lowercase();
    let title = candidate.title.to_ascii_lowercase();
    aliases.iter().any(|alias| {
        process_name == *alias || process_name.contains(alias) || title.contains(alias)
    })
}
