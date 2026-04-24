use super::{
    PanelAnimationDescriptor, PanelAnimationKind, PanelTransitionFrame,
    constants::{
        COLLAPSED_PANEL_HEIGHT, PANEL_CARD_EXIT_MS, PANEL_CARD_EXIT_SETTLE_MS,
        PANEL_CARD_EXIT_STAGGER_MS, PANEL_CARD_REVEAL_MS, PANEL_CARD_REVEAL_STAGGER_MS,
        PANEL_CLOSE_MORPH_DELAY_MS, PANEL_CLOSE_SHOULDER_DELAY_MS, PANEL_CLOSE_SHOULDER_MS,
        PANEL_CLOSE_TOTAL_MS, PANEL_HEIGHT_MS, PANEL_MORPH_DELAY_MS, PANEL_MORPH_MS,
        PANEL_OPEN_TOTAL_MS, PANEL_SHOULDER_HIDE_MS, PANEL_SURFACE_SWITCH_CARD_REVEAL_MS,
        PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS, PANEL_SURFACE_SWITCH_HEIGHT_MS,
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
    },
    render::resolve_panel_transition_canvas_height,
    resolve_panel_animation_descriptor,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PanelAnimationTimeline {
    Open {
        total_ms: u64,
        canvas_height: f64,
        target_height: f64,
        card_total_ms: u64,
    },
    SurfaceSwitch {
        total_ms: u64,
        canvas_height: f64,
        start_height: f64,
        target_height: f64,
        card_total_ms: u64,
    },
    Close {
        total_ms: u64,
        canvas_height: f64,
        start_height: f64,
        close_delay_ms: u64,
        card_total_ms: u64,
    },
}

impl PanelAnimationTimeline {
    pub(crate) fn open(start_height: f64, target_height: f64, card_count: usize) -> Self {
        let card_total_ms = card_transition_total_ms(
            card_count,
            PANEL_CARD_REVEAL_MS,
            PANEL_CARD_REVEAL_STAGGER_MS,
        );
        Self::Open {
            total_ms: PANEL_OPEN_TOTAL_MS + card_total_ms,
            canvas_height: panel_transition_canvas_height(start_height, target_height),
            target_height,
            card_total_ms,
        }
    }

    pub(crate) fn surface_switch(start_height: f64, target_height: f64, card_count: usize) -> Self {
        let card_total_ms = card_transition_total_ms(
            card_count,
            PANEL_SURFACE_SWITCH_CARD_REVEAL_MS,
            PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS,
        );
        Self::SurfaceSwitch {
            total_ms: PANEL_SURFACE_SWITCH_HEIGHT_MS.max(card_total_ms),
            canvas_height: panel_transition_canvas_height(start_height, target_height),
            start_height,
            target_height,
            card_total_ms,
        }
    }

    pub(crate) fn close(start_height: f64, card_count: usize) -> Self {
        let card_total_ms =
            card_transition_total_ms(card_count, PANEL_CARD_EXIT_MS, PANEL_CARD_EXIT_STAGGER_MS);
        let close_delay_ms = card_total_ms
            + if card_count > 0 {
                PANEL_CARD_EXIT_SETTLE_MS
            } else {
                0
            };
        Self::Close {
            total_ms: close_delay_ms + PANEL_CLOSE_TOTAL_MS,
            canvas_height: panel_transition_canvas_height(start_height, COLLAPSED_PANEL_HEIGHT),
            start_height,
            close_delay_ms,
            card_total_ms,
        }
    }

    pub(crate) fn total_ms(self) -> u64 {
        match self {
            Self::Open { total_ms, .. }
            | Self::SurfaceSwitch { total_ms, .. }
            | Self::Close { total_ms, .. } => total_ms,
        }
    }

    pub(crate) fn sample(self, elapsed_ms: u64) -> PanelAnimationDescriptor {
        match self {
            Self::Open {
                canvas_height,
                target_height,
                card_total_ms,
                ..
            } => resolve_open_transition_descriptor(
                elapsed_ms,
                canvas_height,
                target_height,
                card_total_ms,
            ),
            Self::SurfaceSwitch {
                canvas_height,
                start_height,
                target_height,
                card_total_ms,
                ..
            } => resolve_surface_switch_transition_descriptor(
                elapsed_ms,
                canvas_height,
                start_height,
                target_height,
                card_total_ms,
            ),
            Self::Close {
                canvas_height,
                start_height,
                close_delay_ms,
                card_total_ms,
                ..
            } => resolve_close_transition_descriptor(
                elapsed_ms,
                canvas_height,
                start_height,
                close_delay_ms,
                card_total_ms,
            ),
        }
    }
}

pub(crate) fn card_transition_total_ms(
    card_count: usize,
    duration_ms: u64,
    stagger_ms: u64,
) -> u64 {
    if card_count == 0 {
        return 0;
    }
    duration_ms + card_count.saturating_sub(1) as u64 * stagger_ms
}

pub(crate) fn panel_transition_canvas_height(start_height: f64, target_height: f64) -> f64 {
    resolve_panel_transition_canvas_height(start_height, target_height, COLLAPSED_PANEL_HEIGHT)
}

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

pub(crate) fn resolve_open_transition_descriptor(
    elapsed_ms: u64,
    canvas_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> PanelAnimationDescriptor {
    resolve_panel_animation_descriptor(
        PanelAnimationKind::Open,
        resolve_open_transition_frame(elapsed_ms, canvas_height, target_height, card_total_ms),
    )
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

pub(crate) fn resolve_surface_switch_transition_descriptor(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    target_height: f64,
    card_total_ms: u64,
) -> PanelAnimationDescriptor {
    resolve_panel_animation_descriptor(
        PanelAnimationKind::SurfaceSwitch,
        resolve_surface_switch_transition_frame(
            elapsed_ms,
            canvas_height,
            start_height,
            target_height,
            card_total_ms,
        ),
    )
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

pub(crate) fn resolve_close_transition_descriptor(
    elapsed_ms: u64,
    canvas_height: f64,
    start_height: f64,
    close_delay_ms: u64,
    card_total_ms: u64,
) -> PanelAnimationDescriptor {
    resolve_panel_animation_descriptor(
        PanelAnimationKind::Close,
        resolve_close_transition_frame(
            elapsed_ms,
            canvas_height,
            start_height,
            close_delay_ms,
            card_total_ms,
        ),
    )
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
