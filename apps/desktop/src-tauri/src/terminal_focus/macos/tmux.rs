use super::{
    SessionFocusTarget,
    util::{find_binary, is_precise_terminal_tty, run_process},
};

pub(super) fn effective_tty(target: &SessionFocusTarget) -> Option<&str> {
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

pub(super) fn tmux_window_key(target: &SessionFocusTarget) -> String {
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

pub(super) fn tmux_session_name(target: &SessionFocusTarget) -> String {
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
