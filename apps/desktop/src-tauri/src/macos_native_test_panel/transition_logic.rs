use super::*;

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

pub(super) fn surface_switch_card_progress(elapsed_ms: u64, card_total_ms: u64) -> f64 {
    if card_total_ms == 0 {
        return 1.0;
    }
    lerp(
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        1.0,
        animation_phase(elapsed_ms, 0, card_total_ms),
    )
}

pub(super) fn resolve_open_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> NativePanelTransitionFrame {
    let morph_phase = animation_phase(elapsed_ms, PANEL_MORPH_DELAY_MS, PANEL_MORPH_MS);
    let height_phase = animation_phase(
        elapsed_ms,
        PANEL_MORPH_DELAY_MS + PANEL_MORPH_MS,
        PANEL_HEIGHT_MS,
    );
    let morph_progress = morph_phase.clamp(0.0, 1.0);
    let height_progress = height_phase.clamp(0.0, 1.0);
    NativePanelTransitionFrame {
        canvas_height,
        visible_height: lerp(COLLAPSED_PANEL_HEIGHT, target_height, height_progress),
        bar_progress: morph_progress,
        height_progress,
        shoulder_progress: ease_in_cubic(animation_phase(elapsed_ms, 0, PANEL_SHOULDER_HIDE_MS)),
        drop_progress: ease_out_cubic(morph_phase),
        cards_progress: animation_phase(elapsed_ms, PANEL_OPEN_TOTAL_MS, card_total_ms),
    }
}

pub(super) fn resolve_surface_switch_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> NativePanelTransitionFrame {
    let height_progress = ease_out_cubic(animation_phase(
        elapsed_ms,
        0,
        PANEL_SURFACE_SWITCH_HEIGHT_MS,
    ));
    NativePanelTransitionFrame {
        canvas_height,
        visible_height: lerp(start_height, target_height, height_progress),
        bar_progress: 1.0,
        height_progress: 1.0,
        shoulder_progress: 1.0,
        drop_progress: 1.0,
        cards_progress: surface_switch_card_progress(elapsed_ms, card_total_ms),
    }
}

pub(super) fn resolve_close_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    close_delay_ms: u64,
    card_total_ms: u64,
) -> NativePanelTransitionFrame {
    let height_phase = animation_phase(elapsed_ms, close_delay_ms, PANEL_HEIGHT_MS);
    let morph_phase = animation_phase(
        elapsed_ms,
        close_delay_ms + PANEL_CLOSE_MORPH_DELAY_MS,
        PANEL_MORPH_MS,
    );
    let shoulder_phase = animation_phase(
        elapsed_ms,
        close_delay_ms + PANEL_CLOSE_SHOULDER_DELAY_MS,
        PANEL_CLOSE_SHOULDER_MS,
    );
    let height_progress = 1.0 - height_phase.clamp(0.0, 1.0);
    NativePanelTransitionFrame {
        canvas_height,
        visible_height: lerp(COLLAPSED_PANEL_HEIGHT, start_height, height_progress),
        bar_progress: 1.0 - ease_in_cubic(morph_phase),
        height_progress,
        shoulder_progress: 1.0 - ease_out_cubic(shoulder_phase),
        drop_progress: 1.0 - ease_out_cubic(morph_phase),
        cards_progress: animation_phase(elapsed_ms, 0, card_total_ms),
    }
}
