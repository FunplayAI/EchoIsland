#[cfg(target_os = "macos")]
mod card_animation;
#[cfg(target_os = "macos")]
mod card_metrics;
#[cfg(target_os = "macos")]
mod card_stack;
#[cfg(target_os = "macos")]
mod card_views;
#[cfg(target_os = "macos")]
mod compact_bar;
#[cfg(target_os = "macos")]
mod compact_bar_layout;
#[cfg(target_os = "macos")]
mod completion_glow_view;
#[cfg(target_os = "macos")]
mod display_helpers;
#[cfg(target_os = "macos")]
mod mascot;
#[cfg(target_os = "macos")]
mod mascot_motion;
#[cfg(target_os = "macos")]
mod mascot_render;
#[cfg(target_os = "macos")]
mod mascot_scene;
#[cfg(target_os = "macos")]
mod mascot_views;
#[cfg(target_os = "macos")]
mod panel_action_buttons;
#[cfg(target_os = "macos")]
mod panel_assembly;
#[cfg(target_os = "macos")]
mod panel_base_container_views;
#[cfg(target_os = "macos")]
mod panel_constants;
#[cfg(target_os = "macos")]
mod panel_entry;
#[cfg(target_os = "macos")]
mod panel_geometry;
#[cfg(target_os = "macos")]
mod panel_globals;
#[cfg(target_os = "macos")]
mod panel_handles_init;
#[cfg(target_os = "macos")]
mod panel_helpers;
#[cfg(target_os = "macos")]
mod panel_hit_testing;
#[cfg(target_os = "macos")]
mod panel_interaction;
#[cfg(target_os = "macos")]
mod panel_interaction_effects;
#[cfg(target_os = "macos")]
mod panel_loops;
#[cfg(target_os = "macos")]
mod panel_refs;
#[cfg(target_os = "macos")]
mod panel_render;
#[cfg(target_os = "macos")]
mod panel_scene_adapter;
#[cfg(target_os = "macos")]
mod panel_screen_geometry;
#[cfg(target_os = "macos")]
mod panel_setup;
#[cfg(target_os = "macos")]
mod panel_shoulder;
#[cfg(target_os = "macos")]
mod panel_snapshot;
#[cfg(target_os = "macos")]
mod panel_state_init;
#[cfg(target_os = "macos")]
mod panel_style;
#[cfg(target_os = "macos")]
mod panel_transition_entry;
#[cfg(target_os = "macos")]
mod panel_types;
#[cfg(target_os = "macos")]
mod panel_view_updates;
#[cfg(target_os = "macos")]
mod panel_views;
#[cfg(target_os = "macos")]
mod panel_window;
#[cfg(target_os = "macos")]
mod panel_window_control;
#[cfg(all(test, target_os = "macos"))]
mod queue_logic;
#[cfg(target_os = "macos")]
mod transition_logic;
#[cfg(target_os = "macos")]
mod transition_runner;
#[cfg(target_os = "macos")]
mod transition_ui;

#[cfg(target_os = "macos")]
pub(crate) use panel_entry::{create_native_island_panel, native_ui_enabled};
#[cfg(target_os = "macos")]
pub(crate) use panel_loops::{
    spawn_native_count_marquee_loop, spawn_native_hover_loop, spawn_native_status_queue_loop,
};
#[cfg(target_os = "macos")]
pub(crate) use panel_snapshot::{set_shared_expanded_body_height, update_native_island_snapshot};
#[cfg(target_os = "macos")]
pub(crate) use panel_window_control::{
    hide_main_webview_window, hide_native_island_panel, refresh_native_panel_from_last_snapshot,
    reposition_native_panel_to_selected_display,
};

#[cfg(all(test, target_os = "macos"))]
mod tests;

#[cfg(not(target_os = "macos"))]
pub fn native_ui_enabled() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn create_native_island_panel() -> Result<(), String> {
    Ok(())
}
