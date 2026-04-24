use std::sync::atomic::Ordering;

use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use super::panel_constants::{COLLAPSED_PANEL_HEIGHT, PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS};
use super::panel_globals::NATIVE_TEST_PANEL_ANIMATION_ID;
use super::panel_scene_adapter::resolve_snapshot_render_plan;
use super::panel_types::NativePanelHandles;
use super::panel_view_updates::apply_snapshot_values_to_panel;
use super::transition_logic::{
    finish_transition_state, set_transition_cards_state,
    take_skip_close_card_exit_and_begin_transition,
};
use super::transition_runner::{
    animate_close_transition, animate_open_transition, animate_surface_switch_transition,
};
use super::transition_ui::{
    finalize_close_transition, finalize_open_transition, finalize_surface_switch_transition,
    prepare_close_transition, prepare_open_transition, prepare_surface_switch_transition,
    resolve_native_transition_context, resolved_expanded_target_height_for_plan,
};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn begin_native_panel_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    snapshot: RuntimeSnapshot,
    expanded: bool,
) {
    let animation_id = NATIVE_TEST_PANEL_ANIMATION_ID.fetch_add(1, Ordering::SeqCst) + 1;
    apply_snapshot_values_to_panel(handles, &snapshot);
    let skip_close_card_exit = take_skip_close_card_exit_and_begin_transition(expanded);

    let context = resolve_native_transition_context(handles);
    let panel = context.refs.panel;
    let render_plan = resolve_snapshot_render_plan(&snapshot);

    let start_height = panel.frame().size.height;
    let target_height = if expanded {
        resolved_expanded_target_height_for_plan(context, &render_plan)
    } else {
        COLLAPSED_PANEL_HEIGHT
    };

    if expanded {
        let card_count = prepare_open_transition(context, &render_plan);
        set_transition_cards_state(0.0, true);
        tauri::async_runtime::spawn(async move {
            animate_open_transition(
                app.clone(),
                handles,
                animation_id,
                start_height,
                target_height,
                card_count,
            )
            .await;

            let _ = app.run_on_main_thread(move || unsafe {
                if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                    return;
                }

                finish_transition_state(1.0, true);
                let context = resolve_native_transition_context(handles);
                finalize_open_transition(handles, context, &render_plan, target_height);
            });
        });
    } else {
        let card_count = prepare_close_transition(context, skip_close_card_exit);
        set_transition_cards_state(0.0, false);

        tauri::async_runtime::spawn(async move {
            animate_close_transition(app.clone(), handles, animation_id, start_height, card_count)
                .await;

            let _ = app.run_on_main_thread(move || unsafe {
                if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                    return;
                }

                finish_transition_state(0.0, false);
                let context = resolve_native_transition_context(handles);
                finalize_close_transition(handles, context);
            });
        });
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn begin_native_panel_surface_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    snapshot: RuntimeSnapshot,
) {
    let animation_id = NATIVE_TEST_PANEL_ANIMATION_ID.fetch_add(1, Ordering::SeqCst) + 1;
    apply_snapshot_values_to_panel(handles, &snapshot);
    let _ = take_skip_close_card_exit_and_begin_transition(true);
    set_transition_cards_state(PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS, true);

    let context = resolve_native_transition_context(handles);
    let panel = context.refs.panel;
    let render_plan = resolve_snapshot_render_plan(&snapshot);
    let start_height = panel.frame().size.height;
    let target_height = resolved_expanded_target_height_for_plan(context, &render_plan);

    let card_count = prepare_surface_switch_transition(context, &render_plan);

    tauri::async_runtime::spawn(async move {
        animate_surface_switch_transition(
            app.clone(),
            handles,
            animation_id,
            start_height,
            target_height,
            card_count,
        )
        .await;

        let _ = app.run_on_main_thread(move || unsafe {
            if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                return;
            }

            finish_transition_state(1.0, true);
            let context = resolve_native_transition_context(handles);
            finalize_surface_switch_transition(handles, context, &render_plan, target_height);
        });
    });
}
