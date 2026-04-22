use crate::native_panel_core::{
    LastFocusClick, PanelClickInput, PanelHitTarget, PanelInteractionCommand,
    resolve_panel_click_action,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use std::time::Instant;

use super::panel_constants::CARD_FOCUS_CLICK_DEBOUNCE_MS;
use super::panel_geometry::{absolute_rect, compose_local_rect, point_in_rect};
use super::panel_types::{NativeCardHitTarget, NativePanelState};

pub(super) fn resolve_native_click_command(
    state: &NativePanelState,
    primary_click_started: bool,
    settings_clicked: bool,
    quit_clicked: bool,
    cards_visible: bool,
    panel_frame: NSRect,
    expanded_frame: NSRect,
    cards_frame: NSRect,
    mouse: NSPoint,
    now: Instant,
) -> (PanelInteractionCommand, Option<String>) {
    let clicked_action = if primary_click_started
        && state.expanded
        && !state.transitioning
        && !settings_clicked
        && !quit_clicked
        && cards_visible
    {
        find_clicked_card_target(
            &state.card_hit_targets,
            panel_frame,
            expanded_frame,
            cards_frame,
            mouse,
        )
        .map(native_card_hit_target_action)
    } else {
        None
    };

    let click_resolution = resolve_panel_click_action(PanelClickInput {
        primary_click_started,
        expanded: state.expanded,
        transitioning: state.transitioning,
        settings_button_hit: settings_clicked,
        quit_button_hit: quit_clicked,
        cards_visible,
        card_target: clicked_action,
        last_focus_click: state
            .last_focus_click
            .as_ref()
            .map(|(session_id, clicked_at)| LastFocusClick {
                session_id,
                clicked_at: *clicked_at,
            }),
        now,
        focus_debounce_ms: CARD_FOCUS_CLICK_DEBOUNCE_MS,
    });

    (
        click_resolution.command,
        click_resolution.focus_click_to_record,
    )
}

pub(super) fn find_clicked_card_target(
    targets: &[NativeCardHitTarget],
    panel_frame: NSRect,
    expanded_frame: NSRect,
    cards_frame: NSRect,
    mouse: NSPoint,
) -> Option<NativeCardHitTarget> {
    targets
        .iter()
        .find(|target| {
            point_in_rect(
                mouse,
                absolute_rect(
                    panel_frame,
                    compose_local_rect(
                        expanded_frame,
                        compose_local_rect(cards_frame, target.frame),
                    ),
                ),
            )
        })
        .cloned()
}

pub(super) fn native_card_hit_target_action(target: NativeCardHitTarget) -> PanelHitTarget {
    PanelHitTarget {
        action: target.action,
        value: target.value,
    }
}

pub(super) fn native_edge_action_button_rect(
    panel_frame: NSRect,
    pill_frame: NSRect,
    button_frame: NSRect,
) -> NSRect {
    absolute_rect(panel_frame, compose_local_rect(pill_frame, button_frame))
}

pub(super) fn native_hover_pill_rect(panel_frame: NSRect, pill_frame: NSRect) -> NSRect {
    let top_gap =
        (panel_frame.size.height - (pill_frame.origin.y + pill_frame.size.height)).max(0.0);
    absolute_rect(
        panel_frame,
        NSRect::new(
            pill_frame.origin,
            NSSize::new(pill_frame.size.width, pill_frame.size.height + top_gap),
        ),
    )
}
