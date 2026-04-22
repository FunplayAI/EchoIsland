use std::sync::atomic::Ordering;
use std::time::Instant;

use tauri::AppHandle;
use tokio::time::Duration;

use super::card_animation::card_transition_total_ms;
use super::panel_constants::{
    COLLAPSED_PANEL_HEIGHT, PANEL_ANIMATION_FRAME_MS, PANEL_CARD_EXIT_MS,
    PANEL_CARD_EXIT_SETTLE_MS, PANEL_CARD_EXIT_STAGGER_MS, PANEL_CARD_REVEAL_MS,
    PANEL_CARD_REVEAL_STAGGER_MS, PANEL_CLOSE_TOTAL_MS, PANEL_OPEN_TOTAL_MS,
    PANEL_SURFACE_SWITCH_CARD_REVEAL_MS, PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS,
    PANEL_SURFACE_SWITCH_HEIGHT_MS,
};
use super::panel_geometry::panel_transition_canvas_height;
use super::panel_globals::NATIVE_TEST_PANEL_ANIMATION_ID;
use super::panel_types::{NativePanelHandles, NativePanelTransitionFrame};
use super::panel_view_updates::with_disabled_layer_actions;
use super::transition_logic::{
    resolve_close_transition_frame, resolve_open_transition_frame,
    resolve_surface_switch_transition_frame, update_timeline_transition_state,
};
use super::transition_ui::apply_transition_timeline_frame;

pub(super) async fn animate_open_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) {
    let card_total_ms = card_transition_total_ms(
        card_count,
        PANEL_CARD_REVEAL_MS,
        PANEL_CARD_REVEAL_STAGGER_MS,
    );
    let total_ms = PANEL_OPEN_TOTAL_MS + card_total_ms;
    let canvas_height = panel_transition_canvas_height(start_height, target_height);
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        total_ms,
        move |elapsed_ms| {
            resolve_open_transition_frame(elapsed_ms, canvas_height, target_height, card_total_ms)
        },
        true,
    )
    .await;
}

pub(super) async fn animate_surface_switch_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) {
    let card_total_ms = card_transition_total_ms(
        card_count,
        PANEL_SURFACE_SWITCH_CARD_REVEAL_MS,
        PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS,
    );
    let total_ms = PANEL_SURFACE_SWITCH_HEIGHT_MS.max(card_total_ms);
    let canvas_height = panel_transition_canvas_height(start_height, target_height);
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        total_ms,
        move |elapsed_ms| {
            resolve_surface_switch_transition_frame(
                elapsed_ms,
                canvas_height,
                start_height,
                target_height,
                card_total_ms,
            )
        },
        true,
    )
    .await;
}

pub(super) async fn animate_close_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    start_height: f64,
    card_count: usize,
) {
    let card_total_ms =
        card_transition_total_ms(card_count, PANEL_CARD_EXIT_MS, PANEL_CARD_EXIT_STAGGER_MS);
    let card_exit_settle_ms = if card_count > 0 {
        PANEL_CARD_EXIT_SETTLE_MS
    } else {
        0
    };
    let close_delay_ms = card_total_ms + card_exit_settle_ms;
    let total_ms = close_delay_ms + PANEL_CLOSE_TOTAL_MS;
    let canvas_height = panel_transition_canvas_height(start_height, COLLAPSED_PANEL_HEIGHT);
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        total_ms,
        move |elapsed_ms| {
            resolve_close_transition_frame(
                elapsed_ms,
                canvas_height,
                start_height,
                close_delay_ms,
                card_total_ms,
            )
        },
        false,
    )
    .await;
}

async fn animate_panel_timeline<R, F>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    total_ms: u64,
    mut sample: F,
    cards_entering: bool,
) where
    R: tauri::Runtime + 'static,
    F: FnMut(u64) -> NativePanelTransitionFrame + Send + 'static,
{
    let started = Instant::now();
    loop {
        let elapsed_ms = started.elapsed().as_millis().min(total_ms as u128) as u64;
        let frame = sample(elapsed_ms);
        let _ = app.run_on_main_thread(move || unsafe {
            if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                return;
            }
            with_disabled_layer_actions(|| {
                update_timeline_transition_state(frame, cards_entering);
                apply_transition_timeline_frame(handles, frame, cards_entering);
            });
        });

        if elapsed_ms >= total_ms {
            break;
        }

        tokio::time::sleep(Duration::from_millis(PANEL_ANIMATION_FRAME_MS)).await;
    }
}
