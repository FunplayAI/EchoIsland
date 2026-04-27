use std::sync::atomic::Ordering;
use std::time::Instant;

use tauri::AppHandle;
use tokio::time::Duration;

use crate::{
    native_panel_core::PANEL_ANIMATION_FRAME_MS,
    native_panel_renderer::facade::{
        descriptor::native_panel_timeline_descriptor_for_animation,
        transition::{NativePanelTransitionRequest, resolve_native_panel_animation_timeline},
    },
};

use super::panel_globals::NATIVE_TEST_PANEL_ANIMATION_ID;
use super::panel_types::NativePanelHandles;
use super::panel_view_updates::with_disabled_layer_actions;
use super::transition_logic::update_timeline_transition_state_from_descriptor;
use super::transition_ui::apply_transition_timeline_descriptor;

pub(super) async fn animate_transition_request<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    request: NativePanelTransitionRequest,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) {
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        resolve_native_panel_animation_timeline(request, start_height, target_height, card_count),
    )
    .await;
}

async fn animate_panel_timeline<R>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    timeline: crate::native_panel_core::PanelAnimationTimeline,
) where
    R: tauri::Runtime + 'static,
{
    let started = Instant::now();
    loop {
        let total_ms = timeline.total_ms();
        let elapsed_ms = started.elapsed().as_millis().min(total_ms as u128) as u64;
        let frame = timeline.sample(elapsed_ms);
        let timeline_descriptor = native_panel_timeline_descriptor_for_animation(frame);
        let _ = app.run_on_main_thread(move || unsafe {
            if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                return;
            }
            with_disabled_layer_actions(|| {
                update_timeline_transition_state_from_descriptor(timeline_descriptor);
                apply_transition_timeline_descriptor(handles, timeline_descriptor);
            });
        });

        if elapsed_ms >= total_ms {
            break;
        }

        tokio::time::sleep(Duration::from_millis(PANEL_ANIMATION_FRAME_MS)).await;
    }
}
