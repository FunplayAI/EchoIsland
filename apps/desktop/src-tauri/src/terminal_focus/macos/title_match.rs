use super::{
    SessionFocusTarget,
    osascript::escape_applescript,
    util::{has_text, last_path_component, resolve_symlinks, tilde_path},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TitleMatchCandidate {
    pub(super) reason: &'static str,
    pub(super) value: String,
    pub(super) score: u8,
}

pub(super) fn terminal_window_title_candidates(
    target: &SessionFocusTarget,
) -> Vec<TitleMatchCandidate> {
    let mut candidates = Vec::new();
    push_title_candidate(
        &mut candidates,
        "window_title",
        target.window_title.as_deref(),
        6,
    );
    if let Some(cwd) = target.cwd.as_deref().filter(|value| has_text(value)) {
        push_title_candidate(&mut candidates, "cwd", Some(cwd), 5);
        let resolved_cwd = resolve_symlinks(cwd);
        if resolved_cwd != cwd {
            push_title_candidate(&mut candidates, "resolved_cwd", Some(&resolved_cwd), 4);
        }
        let tilde_cwd = tilde_path(cwd);
        push_title_candidate(&mut candidates, "tilde_cwd", Some(&tilde_cwd), 3);
        let folder_name = last_path_component(cwd);
        push_title_candidate(&mut candidates, "folder_name", Some(&folder_name), 1);
    }
    push_title_candidate(
        &mut candidates,
        "project_name",
        target.project_name.as_deref(),
        2,
    );
    candidates
}

pub(super) fn ide_window_title_candidates(target: &SessionFocusTarget) -> Vec<TitleMatchCandidate> {
    let mut candidates = Vec::new();
    push_title_candidate(
        &mut candidates,
        "window_title",
        target.window_title.as_deref(),
        5,
    );
    push_title_candidate(
        &mut candidates,
        "project_name",
        target.project_name.as_deref(),
        4,
    );
    if let Some(cwd) = target.cwd.as_deref().filter(|value| has_text(value)) {
        let folder_name = last_path_component(cwd);
        push_title_candidate(&mut candidates, "folder_name", Some(&folder_name), 3);
        push_title_candidate(&mut candidates, "cwd", Some(cwd), 2);
    }
    candidates
}

fn push_title_candidate(
    candidates: &mut Vec<TitleMatchCandidate>,
    reason: &'static str,
    value: Option<&str>,
    score: u8,
) {
    let Some(value) = value.map(str::trim).filter(|value| value.len() >= 2) else {
        return;
    };
    if candidates.iter().any(|candidate| candidate.value == value) {
        return;
    }
    candidates.push(TitleMatchCandidate {
        reason,
        value: value.to_string(),
        score,
    });
}

pub(super) fn applescript_title_candidate_records(candidates: &[TitleMatchCandidate]) -> String {
    if candidates.is_empty() {
        return "{}".to_string();
    }
    let records = candidates
        .iter()
        .map(|candidate| {
            format!(
                "{{\"{}\", \"{}\", {}}}",
                escape_applescript(&candidate.value),
                escape_applescript(candidate.reason),
                candidate.score
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{records}}}")
}
