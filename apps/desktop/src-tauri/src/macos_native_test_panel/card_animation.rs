use std::collections::HashMap;
use std::sync::Mutex;

use objc2_app_kit::NSView;
use objc2_core_graphics::{CGAffineTransformMakeScale, CGAffineTransformTranslate};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::panel_constants::{
    PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS, PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS,
    PANEL_CARD_EXIT_MS, PANEL_CARD_EXIT_STAGGER_MS, PANEL_CARD_REVEAL_MS,
    PANEL_CARD_REVEAL_STAGGER_MS, PANEL_CARD_REVEAL_Y,
};
use super::panel_globals::CARD_ANIMATION_LAYOUTS;
use super::panel_helpers::{ease_in_cubic, ease_out_cubic, lerp};
use super::panel_types::CardAnimationLayout;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_card_stack_transition(
    cards_container: &NSView,
    cards_progress: f64,
    entering: bool,
) {
    let subviews = cards_container.subviews();
    for index in 0..subviews.len() {
        let card = subviews.objectAtIndex(index);
        let phase = staggered_card_phase(cards_progress, index, subviews.len(), entering);
        let base_layout = card_animation_layout(&card).unwrap_or(CardAnimationLayout {
            frame: card.frame(),
            collapsed_height: (card.frame().size.height * 0.58).round().max(34.0),
        });
        let (shell_opacity, current_height, scale_x, scale_y, translate_y, content_progress) =
            if entering {
                let shell_progress = ease_out_cubic(phase);
                (
                    shell_progress,
                    lerp(
                        base_layout.collapsed_height,
                        base_layout.frame.size.height,
                        shell_progress,
                    ),
                    lerp(0.96, 1.0, shell_progress),
                    lerp(0.82, 1.0, shell_progress),
                    lerp(PANEL_CARD_REVEAL_Y, 0.0, shell_progress),
                    card_content_visibility_phase(phase, true),
                )
            } else {
                let reverse_phase = 1.0 - phase.clamp(0.0, 1.0);
                let shell_progress = ease_out_cubic(reverse_phase);
                (
                    shell_progress,
                    lerp(
                        base_layout.collapsed_height,
                        base_layout.frame.size.height,
                        shell_progress,
                    ),
                    lerp(0.96, 1.0, shell_progress),
                    lerp(0.82, 1.0, shell_progress),
                    lerp(PANEL_CARD_REVEAL_Y, 0.0, shell_progress),
                    card_content_visibility_phase(reverse_phase, true),
                )
            };

        let frame = NSRect::new(
            NSPoint::new(
                base_layout.frame.origin.x,
                base_layout.frame.origin.y + (base_layout.frame.size.height - current_height),
            ),
            NSSize::new(base_layout.frame.size.width, current_height),
        );
        card.setFrame(frame);
        card.setHidden(shell_opacity <= 0.001);
        card.setAlphaValue(shell_opacity);
        if let Some(layer) = card.layer() {
            let transform = CGAffineTransformTranslate(
                CGAffineTransformMakeScale(scale_x, scale_y),
                0.0,
                translate_y,
            );
            layer.setAffineTransform(transform);
            layer.setShadowOpacity((shell_opacity * 0.08).clamp(0.0, 0.08) as f32);
            layer.setShadowRadius(lerp(0.0, 8.0, shell_opacity));
            layer.setShadowOffset(NSSize::new(0.0, lerp(0.0, -2.0, shell_opacity)));
        }

        apply_card_content_phase(&card, phase, entering, content_progress);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_card_exit_phase(card: &NSView, phase: f64) {
    let phase = phase.clamp(0.0, 1.0);
    let base_layout = card_animation_layout(card).unwrap_or(CardAnimationLayout {
        frame: card.frame(),
        collapsed_height: (card.frame().size.height * 0.58).round().max(34.0),
    });

    let reverse_phase = 1.0 - phase;
    let shell_progress = ease_out_cubic(reverse_phase);
    let shell_opacity = shell_progress;
    let content_progress = card_content_visibility_phase(reverse_phase, true);
    let current_height = lerp(
        base_layout.collapsed_height,
        base_layout.frame.size.height,
        shell_progress,
    );
    let scale_x = lerp(0.96, 1.0, shell_progress);
    let scale_y = lerp(0.82, 1.0, shell_progress);
    let translate_y = lerp(PANEL_CARD_REVEAL_Y, 0.0, shell_progress);

    let frame = NSRect::new(
        NSPoint::new(
            base_layout.frame.origin.x,
            base_layout.frame.origin.y + (base_layout.frame.size.height - current_height),
        ),
        NSSize::new(base_layout.frame.size.width, current_height),
    );
    card.setFrame(frame);
    card.setHidden(shell_opacity <= 0.001);
    card.setAlphaValue(shell_opacity);

    if let Some(layer) = card.layer() {
        let transform = CGAffineTransformTranslate(
            CGAffineTransformMakeScale(scale_x, scale_y),
            0.0,
            translate_y,
        );
        layer.setAffineTransform(transform);
        layer.setShadowOpacity((shell_opacity * 0.08).clamp(0.0, 0.08) as f32);
        layer.setShadowRadius(lerp(0.0, 8.0, shell_opacity));
        layer.setShadowOffset(NSSize::new(0.0, lerp(0.0, -2.0, shell_opacity)));
    }

    apply_card_content_phase(card, phase, false, content_progress);
}

pub(super) fn card_content_visibility_phase(phase: f64, entering: bool) -> f64 {
    let phase = phase.clamp(0.0, 1.0);
    if entering {
        ease_out_cubic(
            ((phase - PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS)
                / (1.0 - PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS))
                .clamp(0.0, 1.0),
        )
    } else if phase <= PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS {
        1.0 - (0.06 * (phase / PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS).clamp(0.0, 1.0))
    } else {
        0.94 * (1.0
            - ease_in_cubic(
                ((phase - PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS)
                    / (1.0 - PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS))
                    .clamp(0.0, 1.0),
            ))
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_card_content_phase(
    card: &NSView,
    phase: f64,
    entering: bool,
    content_progress: f64,
) {
    let children = card.subviews();
    for child_index in 0..children.len() {
        let child = children.objectAtIndex(child_index);
        child.setHidden(content_progress <= 0.001);
        child.setAlphaValue(content_progress);
        child.setWantsLayer(true);

        if let Some(layer) = child.layer() {
            let transform = if entering {
                let reveal_progress = phase.clamp(0.0, 1.0);
                CGAffineTransformTranslate(
                    CGAffineTransformMakeScale(1.0, lerp(0.92, 1.0, reveal_progress)),
                    0.0,
                    lerp(-5.0, 0.0, reveal_progress),
                )
            } else if phase <= 0.30 {
                let early_phase = (phase / 0.30).clamp(0.0, 1.0);
                CGAffineTransformTranslate(
                    CGAffineTransformMakeScale(1.0, lerp(1.0, 0.92, early_phase)),
                    0.0,
                    0.0,
                )
            } else {
                let late_phase = ((phase - 0.30) / 0.70).clamp(0.0, 1.0);
                CGAffineTransformTranslate(
                    CGAffineTransformMakeScale(1.0, lerp(0.92, 0.82, late_phase)),
                    0.0,
                    0.0,
                )
            };
            layer.setAffineTransform(transform);
        }
    }
}

pub(super) fn staggered_card_phase(
    progress: f64,
    index: usize,
    total: usize,
    entering: bool,
) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    let duration_ms = if entering {
        PANEL_CARD_REVEAL_MS
    } else {
        PANEL_CARD_EXIT_MS
    };
    let stagger_ms = if entering {
        PANEL_CARD_REVEAL_STAGGER_MS
    } else {
        PANEL_CARD_EXIT_STAGGER_MS
    };
    let total_ms = card_transition_total_ms(total, duration_ms, stagger_ms) as f64;
    let order_index = if entering {
        index
    } else {
        total.saturating_sub(index + 1)
    };
    let elapsed_ms = progress * total_ms;
    let delay_ms = order_index as f64 * stagger_ms as f64;

    ((elapsed_ms - delay_ms) / duration_ms as f64).clamp(0.0, 1.0)
}

pub(super) fn card_transition_total_ms(
    card_count: usize,
    duration_ms: u64,
    stagger_ms: u64,
) -> u64 {
    crate::native_panel_core::card_transition_total_ms(card_count, duration_ms, stagger_ms)
}

pub(super) fn clear_card_animation_layouts() {
    if let Some(layouts) = CARD_ANIMATION_LAYOUTS.get() {
        if let Ok(mut layouts) = layouts.lock() {
            layouts.clear();
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn register_card_animation_layout(
    view: &NSView,
    frame: NSRect,
    collapsed_height: f64,
) {
    let layouts = CARD_ANIMATION_LAYOUTS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut layouts) = layouts.lock() {
        layouts.insert(
            (view as *const NSView) as usize,
            CardAnimationLayout {
                frame,
                collapsed_height: collapsed_height.clamp(1.0, frame.size.height.max(1.0)),
            },
        );
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn card_animation_layout(view: &NSView) -> Option<CardAnimationLayout> {
    CARD_ANIMATION_LAYOUTS
        .get()
        .and_then(|layouts| layouts.lock().ok())
        .and_then(|layouts| layouts.get(&((view as *const NSView) as usize)).copied())
}
