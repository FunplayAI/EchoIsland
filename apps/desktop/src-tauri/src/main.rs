#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use echoisland_runtime::SharedRuntime;
use tauri::WindowEvent;
use tracing_subscriber::{EnvFilter, fmt};

mod app_runtime;
mod app_settings;
mod claude_scan;
mod codex_scan;
mod command_services;
mod commands;
mod constants;
mod display_settings;
mod focus_store;
mod http_receiver;
mod island_window;
#[cfg(target_os = "macos")]
mod macos_native_test_panel;
#[cfg(not(target_os = "macos"))]
#[path = "macos_native_test_panel_stub.rs"]
mod macos_native_test_panel;
mod macos_panel;
mod macos_shared_expanded_window;
mod native_panel_core;
mod native_panel_runtime;
mod native_panel_scene;
mod native_ui_refresh;
mod notification_sound;
mod platform;
mod platform_stub;
mod session_scan_runner;
mod startup_service;
mod terminal_focus;
mod terminal_focus_service;
mod tray;
mod web_panel_scene_service;
mod window_surface_service;

use app_runtime::{AppRuntime, spawn_ipc_server};
use claude_scan::spawn_claude_scan_loop;
use codex_scan::spawn_codex_scan_loop;
use commands::{
    answer_question, approve_permission, bind_session_terminal, build_status_surface_scene,
    claude_status, codex_status, deny_permission, focus_session_terminal, get_app_settings,
    get_available_displays, get_snapshot, get_snapshot_status_surface_bundle, hide_main_window,
    http_receiver_status, ingest_sample, ipc_addr, open_release_page, open_settings_location,
    openclaw_status, platform_capabilities, platform_paths, quit_application,
    set_completion_sound_enabled, set_island_bar_stage, set_island_bar_stage_passive,
    set_island_expanded, set_island_expanded_passive, set_island_panel_stage,
    set_island_panel_stage_passive, set_macos_shared_expanded_height, set_mascot_enabled,
    set_preferred_display_index, show_main_window_interactive, skip_question,
};
use http_receiver::spawn_http_receiver;
use startup_service::AppStartupService;
use web_panel_scene_service::WebPanelSceneState;

fn main() {
    setup_tracing();

    let runtime = Arc::new(SharedRuntime::new());
    let app_runtime = AppRuntime::new(runtime.clone());

    let builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .manage(app_runtime.clone())
        .manage(WebPanelSceneState::default())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let app_handle = app.handle().clone();
            #[cfg(target_os = "macos")]
            if macos_native_test_panel::native_ui_enabled() {
                macos_native_test_panel::create_native_island_panel()
                    .map_err(std::io::Error::other)?;
                if macos_shared_expanded_window::shared_expanded_enabled() {
                    macos_shared_expanded_window::create_shared_expanded_window(&app_handle)
                        .map_err(std::io::Error::other)?;
                }
                macos_native_test_panel::hide_main_webview_window(&app_handle)
                    .map_err(std::io::Error::other)?;
                native_panel_runtime::spawn_native_snapshot_loop(
                    app_handle.clone(),
                    app_runtime.clone(),
                );
                macos_native_test_panel::spawn_native_hover_loop(app_handle.clone());
                macos_native_test_panel::spawn_native_count_marquee_loop(app_handle.clone());
                macos_native_test_panel::spawn_native_status_queue_loop(app_handle.clone());
            } else {
                macos_panel::create_main_panel(&app_handle).map_err(std::io::Error::other)?;
            }
            #[cfg(not(target_os = "macos"))]
            macos_panel::create_main_panel(&app_handle).map_err(std::io::Error::other)?;
            AppStartupService::new(app)
                .initialize()
                .map_err(std::io::Error::other)?;
            spawn_codex_scan_loop(runtime.clone());
            spawn_claude_scan_loop(runtime.clone());
            spawn_ipc_server(app_handle, app_runtime.clone());
            spawn_http_receiver(app.handle().clone(), runtime.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_snapshot_status_surface_bundle,
            build_status_surface_scene,
            get_app_settings,
            get_available_displays,
            ingest_sample,
            ipc_addr,
            codex_status,
            claude_status,
            openclaw_status,
            http_receiver_status,
            platform_capabilities,
            platform_paths,
            approve_permission,
            deny_permission,
            answer_question,
            skip_question,
            set_island_bar_stage,
            set_island_bar_stage_passive,
            set_island_panel_stage,
            set_island_panel_stage_passive,
            set_island_expanded,
            set_island_expanded_passive,
            show_main_window_interactive,
            set_macos_shared_expanded_height,
            hide_main_window,
            open_settings_location,
            open_release_page,
            set_completion_sound_enabled,
            set_mascot_enabled,
            set_preferred_display_index,
            quit_application,
            focus_session_terminal,
            bind_session_terminal
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}

fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}
