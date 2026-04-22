use super::panel_refs::native_panel_state;
use super::panel_types::{NativePanelState, NativePanelTransitionFrame};

fn with_native_panel_state_mut<T>(f: impl FnOnce(&mut NativePanelState) -> T) -> Option<T> {
    native_panel_state()
        .and_then(|state_mutex| state_mutex.lock().ok().map(|mut state| f(&mut state)))
}

pub(super) fn take_skip_close_card_exit_and_begin_transition(expanded: bool) -> bool {
    with_native_panel_state_mut(|state| {
        state.transitioning = true;
        if expanded {
            false
        } else {
            let skip_close_card_exit = state.skip_next_close_card_exit;
            state.skip_next_close_card_exit = false;
            skip_close_card_exit
        }
    })
    .unwrap_or(false)
}

pub(super) fn set_transition_cards_state(progress: f64, entering: bool) {
    let _ = with_native_panel_state_mut(|state| {
        state.transition_cards_progress = progress;
        state.transition_cards_entering = entering;
    });
}

pub(super) fn finish_transition_state(progress: f64, entering: bool) {
    let _ = with_native_panel_state_mut(|state| {
        state.transitioning = false;
        state.transition_cards_progress = progress;
        state.transition_cards_entering = entering;
    });
}

pub(super) fn update_timeline_transition_state(
    frame: NativePanelTransitionFrame,
    cards_entering: bool,
) {
    set_transition_cards_state(frame.cards_progress.clamp(0.0, 1.0), cards_entering);
}

#[cfg(test)]
pub(super) fn surface_switch_card_progress(elapsed_ms: u64, card_total_ms: u64) -> f64 {
    crate::native_panel_core::surface_switch_card_progress(elapsed_ms, card_total_ms)
}

pub(super) fn resolve_open_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> NativePanelTransitionFrame {
    crate::native_panel_core::resolve_open_transition_frame(
        elapsed_ms,
        canvas_height,
        target_height,
        card_total_ms,
    )
}

pub(super) fn resolve_surface_switch_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> NativePanelTransitionFrame {
    crate::native_panel_core::resolve_surface_switch_transition_frame(
        elapsed_ms,
        canvas_height,
        start_height,
        target_height,
        card_total_ms,
    )
}

pub(super) fn resolve_close_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    close_delay_ms: u64,
    card_total_ms: u64,
) -> NativePanelTransitionFrame {
    crate::native_panel_core::resolve_close_transition_frame(
        elapsed_ms,
        canvas_height,
        start_height,
        close_delay_ms,
        card_total_ms,
    )
}
