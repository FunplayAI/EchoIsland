use crate::native_panel_core::{LastFocusClick, PanelClickInput, resolve_panel_click_action};
use crate::native_panel_renderer::{
    NativePanelPlatformEvent, NativePanelPointerInput, native_panel_hit_target_at_point,
    native_panel_platform_event_for_pointer_input, native_panel_pointer_inside_for_input,
};
use echoisland_runtime::RuntimeSnapshot;
use objc2_app_kit::NSEvent;
use std::time::Instant;
use tauri::AppHandle;

use super::compact_bar_layout::sync_active_count_marquee;
use super::panel_constants::{
    CARD_FOCUS_CLICK_DEBOUNCE_MS, COLLAPSED_PANEL_HEIGHT, HOVER_DELAY_MS,
};
use super::panel_geometry::{absolute_rect, point_in_rect};
use super::panel_hit_testing::native_hover_pill_rect;
use super::panel_interaction_effects::handle_native_click_command;
use super::panel_refs::{
    native_panel_handles, native_panel_state, panel_from_ptr, resolve_native_panel_refs,
    view_from_ptr,
};
use super::panel_transition_entry::{
    begin_native_panel_surface_transition, begin_native_panel_transition,
};
use super::panel_types::{NativeExpandedSurface, NativeHoverTransition, NativePanelState};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn sync_hover_state_on_main_thread<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
) {
    let Some(handles) = native_panel_handles() else {
        return;
    };
    let Some(state_mutex) = native_panel_state() else {
        return;
    };

    let refs = resolve_native_panel_refs(handles);
    sync_active_count_marquee(&refs);

    let panel = panel_from_ptr(handles.panel);
    let mouse = NSEvent::mouseLocation();
    let primary_mouse_down = (NSEvent::pressedMouseButtons() & 1) != 0;
    let panel_frame = panel.frame();
    let pill_frame = refs.pill_view.frame();
    let hover_pill_frame = native_hover_pill_rect(panel_frame, pill_frame);
    let expanded_container = view_from_ptr(handles.expanded_container);
    let cards_container = view_from_ptr(handles.cards_container);
    let fallback_inside = if panel_frame.size.height > COLLAPSED_PANEL_HEIGHT + 0.5 {
        point_in_rect(
            mouse,
            absolute_rect(panel_frame, expanded_container.frame()),
        ) || point_in_rect(mouse, hover_pill_frame)
    } else {
        point_in_rect(mouse, hover_pill_frame)
    };

    let now = Instant::now();
    let mut transition_snapshot: Option<(RuntimeSnapshot, bool)> = None;
    let mut surface_transition_snapshot: Option<RuntimeSnapshot> = None;
    let click_command;
    let inside;

    {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let pointer = crate::native_panel_core::PanelPoint {
            x: mouse.x,
            y: mouse.y,
        };
        inside = native_panel_pointer_inside_for_input(
            &state.pointer_regions,
            NativePanelPointerInput::Move(pointer),
        )
        .unwrap_or(fallback_inside);
        let primary_click_started = primary_mouse_down && !state.primary_mouse_down && inside;
        let click_event = primary_click_started
            .then(|| {
                native_panel_platform_event_for_pointer_input(
                    &state.pointer_regions,
                    NativePanelPointerInput::Click(pointer),
                )
            })
            .flatten();
        let settings_clicked = matches!(
            click_event,
            Some(NativePanelPlatformEvent::ToggleSettingsSurface)
        );
        let quit_clicked = matches!(click_event, Some(NativePanelPlatformEvent::QuitApplication));
        if primary_click_started
            && state.expanded
            && !state.transitioning
            && !settings_clicked
            && !quit_clicked
        {
            let click_resolution = resolve_panel_click_action(PanelClickInput {
                primary_click_started,
                expanded: state.expanded,
                transitioning: state.transitioning,
                settings_button_hit: settings_clicked,
                quit_button_hit: quit_clicked,
                cards_visible: !cards_container.isHidden(),
                card_target: native_panel_hit_target_at_point(&state.pointer_regions, pointer),
                last_focus_click: state.last_focus_click.as_ref().map(
                    |(session_id, clicked_at)| LastFocusClick {
                        session_id,
                        clicked_at: *clicked_at,
                    },
                ),
                now,
                focus_debounce_ms: CARD_FOCUS_CLICK_DEBOUNCE_MS,
            });
            let resolved_command = click_resolution.command;
            let focus_click_to_record = click_resolution.focus_click_to_record;
            if let Some(session_id) = focus_click_to_record {
                state.last_focus_click = Some((session_id, now));
            }
            click_command = resolved_command;
        } else {
            click_command = resolve_panel_click_action(PanelClickInput {
                primary_click_started,
                expanded: state.expanded,
                transitioning: state.transitioning,
                settings_button_hit: settings_clicked,
                quit_button_hit: quit_clicked,
                cards_visible: !cards_container.isHidden(),
                card_target: None,
                last_focus_click: state.last_focus_click.as_ref().map(
                    |(session_id, clicked_at)| LastFocusClick {
                        session_id,
                        clicked_at: *clicked_at,
                    },
                ),
                now,
                focus_debounce_ms: CARD_FOCUS_CLICK_DEBOUNCE_MS,
            })
            .command;
        }
        state.primary_mouse_down = primary_mouse_down;
        let was_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        let was_surface_mode = state.surface_mode;

        if let Some(hover_transition) = sync_native_hover_expansion_state(&mut state, inside, now) {
            if let Some(snapshot) = state.last_snapshot.clone() {
                transition_snapshot =
                    Some((snapshot, hover_transition == NativeHoverTransition::Expand));
            }
        }

        if primary_click_started && state.expanded && !state.transitioning && settings_clicked {
            if toggle_native_settings_surface(&mut state) {
                surface_transition_snapshot = state.last_snapshot.clone();
            }
        }

        let is_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        if surface_transition_snapshot.is_none()
            && was_status_surface != is_status_surface
            && state.expanded
            && !state.transitioning
        {
            if let Some(snapshot) = state.last_snapshot.clone() {
                surface_transition_snapshot = Some(snapshot);
            }
        } else if surface_transition_snapshot.is_none()
            && was_surface_mode != state.surface_mode
            && state.expanded
            && !state.transitioning
        {
            surface_transition_snapshot = state.last_snapshot.clone();
        }
    }

    panel.setIgnoresMouseEvents(!inside);

    if let Some((snapshot, expanded)) = transition_snapshot {
        begin_native_panel_transition(app.clone(), handles, snapshot, expanded);
    } else if let Some(snapshot) = surface_transition_snapshot {
        begin_native_panel_surface_transition(app.clone(), handles, snapshot);
    }

    handle_native_click_command(app, click_command);
}

pub(super) fn toggle_native_settings_surface(state: &mut NativePanelState) -> bool {
    let mut core = state.to_core_panel_state();
    let changed = crate::native_panel_core::toggle_settings_surface(&mut core);
    state.apply_core_panel_state(core);
    changed
}

pub(super) fn sync_native_hover_expansion_state(
    state: &mut NativePanelState,
    inside: bool,
    now: Instant,
) -> Option<NativeHoverTransition> {
    let mut core = state.to_core_panel_state();
    let transition = crate::native_panel_core::sync_hover_expansion_state(
        &mut core,
        inside,
        now,
        HOVER_DELAY_MS,
    );
    state.apply_core_panel_state(core);
    transition
}

pub(super) fn native_status_surface_active() -> bool {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                guard.surface_mode == NativeExpandedSurface::Status
                    && !guard.status_queue.is_empty()
            })
        })
        .unwrap_or(false)
}

pub(super) fn native_settings_surface_active() -> bool {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                guard.surface_mode == NativeExpandedSurface::Settings && guard.expanded
            })
        })
        .unwrap_or(false)
}
