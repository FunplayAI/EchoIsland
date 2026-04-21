use super::PanelTransitionFrame;

pub(crate) const COLLAPSED_PANEL_HEIGHT: f64 = 80.0;
pub(crate) const PANEL_MORPH_DELAY_MS: u64 = 120;
pub(crate) const PANEL_MORPH_MS: u64 = 270;
pub(crate) const PANEL_SHOULDER_HIDE_MS: u64 = 120;
pub(crate) const PANEL_HEIGHT_MS: u64 = 270;
pub(crate) const PANEL_OPEN_TOTAL_MS: u64 = 660;
pub(crate) const PANEL_CLOSE_MORPH_DELAY_MS: u64 = 270;
pub(crate) const PANEL_CLOSE_SHOULDER_DELAY_MS: u64 = 540;
pub(crate) const PANEL_CLOSE_SHOULDER_MS: u64 = 120;
pub(crate) const PANEL_SURFACE_SWITCH_HEIGHT_MS: u64 = 220;
pub(crate) const PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS: f64 = 0.24;

pub(crate) fn surface_switch_card_progress(elapsed_ms: u64, card_total_ms: u64) -> f64 {
    if card_total_ms == 0 {
        return 1.0;
    }
    lerp(
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        1.0,
        animation_phase(elapsed_ms, 0, card_total_ms),
    )
}

pub(crate) fn resolve_open_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> PanelTransitionFrame {
    let morph_phase = animation_phase(elapsed_ms, PANEL_MORPH_DELAY_MS, PANEL_MORPH_MS);
    let height_phase = animation_phase(
        elapsed_ms,
        PANEL_MORPH_DELAY_MS + PANEL_MORPH_MS,
        PANEL_HEIGHT_MS,
    );
    let morph_progress = morph_phase.clamp(0.0, 1.0);
    let height_progress = height_phase.clamp(0.0, 1.0);
    PanelTransitionFrame {
        canvas_height,
        visible_height: lerp(COLLAPSED_PANEL_HEIGHT, target_height, height_progress),
        bar_progress: morph_progress,
        height_progress,
        shoulder_progress: ease_in_cubic(animation_phase(elapsed_ms, 0, PANEL_SHOULDER_HIDE_MS)),
        drop_progress: ease_out_cubic(height_phase),
        cards_progress: animation_phase(elapsed_ms, PANEL_OPEN_TOTAL_MS, card_total_ms),
    }
}

pub(crate) fn resolve_surface_switch_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> PanelTransitionFrame {
    let height_progress = ease_out_cubic(animation_phase(
        elapsed_ms,
        0,
        PANEL_SURFACE_SWITCH_HEIGHT_MS,
    ));
    PanelTransitionFrame {
        canvas_height,
        visible_height: lerp(start_height, target_height, height_progress),
        bar_progress: 1.0,
        height_progress: 1.0,
        shoulder_progress: 1.0,
        drop_progress: 1.0,
        cards_progress: surface_switch_card_progress(elapsed_ms, card_total_ms),
    }
}

pub(crate) fn resolve_close_transition_frame(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    close_delay_ms: u64,
    card_total_ms: u64,
) -> PanelTransitionFrame {
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
    PanelTransitionFrame {
        canvas_height,
        visible_height: lerp(COLLAPSED_PANEL_HEIGHT, start_height, height_progress),
        bar_progress: 1.0 - ease_in_cubic(morph_phase),
        height_progress,
        shoulder_progress: 1.0 - ease_out_cubic(shoulder_phase),
        drop_progress: 1.0 - ease_out_cubic(morph_phase),
        cards_progress: animation_phase(elapsed_ms, 0, card_total_ms),
    }
}

pub(crate) fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}

pub(crate) fn animation_phase(elapsed_ms: u64, delay_ms: u64, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        return 1.0;
    }

    elapsed_ms.saturating_sub(delay_ms) as f64 / duration_ms as f64
}

pub(crate) fn ease_in_cubic(progress: f64) -> f64 {
    progress.clamp(0.0, 1.0).powi(3)
}

pub(crate) fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}
