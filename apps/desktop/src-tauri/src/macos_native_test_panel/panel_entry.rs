use std::sync::atomic::Ordering;

use objc2::MainThreadMarker;

use super::compact_bar::{CompactBarViews, create_compact_bar_views};
use super::mascot_views::{MascotViews, create_mascot_views};
use super::panel_assembly::{NativePanelAssemblyViews, assemble_native_panel_views};
use super::panel_globals::NATIVE_TEST_PANEL_CREATED;
use super::panel_handles_init::{NativePanelHandleViews, initialize_native_panel_handles};
use super::panel_host_descriptor::native_panel_host_window_descriptor;
use super::panel_screen_geometry::screen_has_camera_housing;
use super::panel_setup::{
    NativePanelColors, NativePanelSetup, native_panel_colors, resolve_native_panel_setup,
};
use super::panel_state_init::{initialize_active_count_scroll_text, initialize_native_panel_state};
use super::panel_views::{PanelBaseViews, create_panel_base_views};
use super::panel_window;
use crate::native_panel_renderer::facade::env::native_panel_enabled_from_webview_env_value;

use tracing::info;

pub(crate) fn native_ui_enabled() -> bool {
    native_panel_enabled_from_webview_env_value(std::env::var("ECHOISLAND_USE_WEBVIEW").ok())
}

pub(crate) fn create_native_island_panel() -> Result<(), String> {
    if NATIVE_TEST_PANEL_CREATED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let Some(mtm) = MainThreadMarker::new() else {
        return Err("native test panel must be created on the main thread".to_string());
    };

    let setup = resolve_native_panel_setup(mtm)?;
    let colors = native_panel_colors();
    let NativePanelSetup {
        screen,
        compact_height,
        compact_width,
        expanded_width,
        size,
        pill_size,
        screen_frame,
        frame,
        pill_frame,
    } = setup;
    let panel = panel_window::create_native_panel_window(mtm, frame);
    let NativePanelColors {
        pill_background,
        pill_border,
        pill_highlight,
        mascot_shell_border,
        mascot_body_fill,
        mascot_stroke,
        mascot_face,
        expanded_background,
        expanded_border,
        text_primary,
        accent_active,
        separator_color,
    } = colors;

    let PanelBaseViews {
        content_view,
        left_shoulder,
        right_shoulder,
        pill_view,
        expanded_container,
        cards_container,
        completion_glow,
        top_highlight,
        body_separator,
        settings_button,
        settings_button_label,
        quit_button,
        quit_button_label,
    } = create_panel_base_views(
        mtm,
        size,
        pill_frame,
        pill_size,
        compact_width,
        expanded_width,
        compact_height,
        &pill_background,
        &pill_border,
        &pill_highlight,
        &expanded_background,
        &expanded_border,
        &separator_color,
    );
    let MascotViews {
        shell: mascot_shell,
        body: mascot_body,
        left_eye: mascot_left_eye,
        right_eye: mascot_right_eye,
        mouth: mascot_mouth,
        bubble: mascot_bubble,
        sleep_label: mascot_sleep_label,
        completion_badge: mascot_completion_badge,
        completion_badge_label: mascot_completion_badge_label,
    } = create_mascot_views(
        mtm,
        &mascot_shell_border,
        &mascot_body_fill,
        &mascot_stroke,
        &mascot_face,
    );

    let CompactBarViews {
        headline,
        active_count_clip,
        active_count,
        active_count_next,
        slash,
        total_count,
    } = create_compact_bar_views(
        mtm,
        pill_size,
        screen_has_camera_housing(&screen),
        &text_primary,
        &accent_active,
    );

    assemble_native_panel_views(NativePanelAssemblyViews {
        content_view: &content_view,
        left_shoulder: &left_shoulder,
        right_shoulder: &right_shoulder,
        pill_view: &pill_view,
        expanded_container: &expanded_container,
        completion_glow: &completion_glow,
        top_highlight: &top_highlight,
        body_separator: &body_separator,
        settings_button: &settings_button,
        quit_button: &quit_button,
        mascot_shell: &mascot_shell,
        headline: &headline,
        active_count_clip: &active_count_clip,
        slash: &slash,
        total_count: &total_count,
    });

    unsafe {
        panel_window::configure_native_panel_window(&panel, &content_view, frame);
    }

    initialize_native_panel_handles(NativePanelHandleViews {
        panel: &panel,
        content_view: &content_view,
        left_shoulder: &left_shoulder,
        right_shoulder: &right_shoulder,
        pill_view: &pill_view,
        expanded_container: &expanded_container,
        cards_container: &cards_container,
        completion_glow: &completion_glow,
        top_highlight: &top_highlight,
        body_separator: &body_separator,
        settings_button: &settings_button,
        settings_button_label: &settings_button_label,
        quit_button: &quit_button,
        quit_button_label: &quit_button_label,
        mascot_shell: &mascot_shell,
        mascot_body: &mascot_body,
        mascot_left_eye: &mascot_left_eye,
        mascot_right_eye: &mascot_right_eye,
        mascot_mouth: &mascot_mouth,
        mascot_bubble: &mascot_bubble,
        mascot_sleep_label: &mascot_sleep_label,
        mascot_completion_badge: &mascot_completion_badge,
        mascot_completion_badge_label: &mascot_completion_badge_label,
        headline: &headline,
        active_count_clip: &active_count_clip,
        active_count: &active_count,
        active_count_next: &active_count_next,
        slash: &slash,
        total_count: &total_count,
    });
    initialize_native_panel_state(native_panel_host_window_descriptor(
        true,
        crate::app_settings::current_app_settings().preferred_display_index,
        Some(screen_frame),
        None,
        None,
    ));
    initialize_active_count_scroll_text();

    info!(
        panel_x = frame.origin.x,
        panel_y = frame.origin.y,
        panel_width = frame.size.width,
        panel_height = frame.size.height,
        screen_height = screen_frame.size.height,
        "created native macOS island panel"
    );

    let _: &'static mut _ = Box::leak(Box::new(panel));

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn native_ui_enabled() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn create_native_island_panel() -> Result<(), String> {
    Ok(())
}
