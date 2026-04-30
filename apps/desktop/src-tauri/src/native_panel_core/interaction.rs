use std::time::Instant;

use super::{ExpandedSurface, HoverTransition, PanelHitAction, PanelState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PanelHitTarget {
    pub(crate) action: PanelHitAction,
    pub(crate) value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PanelInteractionCommand {
    None,
    ToggleSettingsSurface,
    QuitApplication,
    HitTarget(PanelHitTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LastFocusClick<'a> {
    pub(crate) session_id: &'a str,
    pub(crate) clicked_at: Instant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PanelClickInput<'a> {
    pub(crate) primary_click_started: bool,
    pub(crate) expanded: bool,
    pub(crate) transitioning: bool,
    pub(crate) settings_button_hit: bool,
    pub(crate) quit_button_hit: bool,
    pub(crate) cards_visible: bool,
    pub(crate) card_target: Option<PanelHitTarget>,
    pub(crate) last_focus_click: Option<LastFocusClick<'a>>,
    pub(crate) now: Instant,
    pub(crate) focus_debounce_ms: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PanelClickResolution {
    pub(crate) command: PanelInteractionCommand,
    pub(crate) focus_click_to_record: Option<String>,
}

pub(crate) fn resolve_panel_click_action(input: PanelClickInput<'_>) -> PanelClickResolution {
    if !input.primary_click_started || !input.expanded {
        return PanelClickResolution::none();
    }

    if input.settings_button_hit {
        return PanelClickResolution {
            command: PanelInteractionCommand::ToggleSettingsSurface,
            focus_click_to_record: None,
        };
    }

    if input.quit_button_hit {
        return PanelClickResolution {
            command: PanelInteractionCommand::QuitApplication,
            focus_click_to_record: None,
        };
    }

    if input.transitioning {
        return PanelClickResolution::none();
    }

    if !input.cards_visible {
        return PanelClickResolution::none();
    }

    let Some(target) = input.card_target else {
        return PanelClickResolution::none();
    };

    if target.action != PanelHitAction::FocusSession {
        return PanelClickResolution {
            command: PanelInteractionCommand::HitTarget(target),
            focus_click_to_record: None,
        };
    }

    if focus_click_suppressed(
        &target.value,
        input.last_focus_click,
        input.now,
        input.focus_debounce_ms,
    ) {
        return PanelClickResolution::none();
    }

    PanelClickResolution {
        focus_click_to_record: Some(target.value.clone()),
        command: PanelInteractionCommand::HitTarget(target),
    }
}

pub(crate) fn sync_hover_expansion_state(
    state: &mut PanelState,
    inside: bool,
    now: Instant,
    hover_delay_ms: u64,
) -> Option<HoverTransition> {
    if inside {
        state.pointer_outside_since = None;
        state.pointer_inside_since.get_or_insert(now);
        if !state.expanded
            && state.pointer_inside_since.is_some_and(|entered_at| {
                now.duration_since(entered_at).as_millis() >= hover_delay_ms as u128
            })
        {
            state.expanded = true;
            state.completion_badge_items.clear();
            state.status_auto_expanded = false;
            state.surface_mode = ExpandedSurface::Default;
            return Some(HoverTransition::Expand);
        }
    } else {
        state.pointer_inside_since = None;
        state.pointer_outside_since.get_or_insert(now);
        let keep_open_for_status = state.status_auto_expanded
            && state.surface_mode == ExpandedSurface::Status
            && !state.status_queue.is_empty();
        if state.expanded
            && !keep_open_for_status
            && state.pointer_outside_since.is_some_and(|left_at| {
                now.duration_since(left_at).as_millis() >= hover_delay_ms as u128
            })
        {
            state.expanded = false;
            state.status_auto_expanded = false;
            state.surface_mode = ExpandedSurface::Default;
            return Some(HoverTransition::Collapse);
        }
    }

    None
}

impl PanelClickResolution {
    fn none() -> Self {
        Self {
            command: PanelInteractionCommand::None,
            focus_click_to_record: None,
        }
    }
}

fn focus_click_suppressed(
    session_id: &str,
    last_focus_click: Option<LastFocusClick<'_>>,
    now: Instant,
    focus_debounce_ms: u128,
) -> bool {
    last_focus_click.is_some_and(|last| {
        last.session_id == session_id
            && now.duration_since(last.clicked_at).as_millis() < focus_debounce_ms
    })
}
