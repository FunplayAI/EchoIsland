#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use echoisland_runtime::SharedRuntime;
use tauri::{RunEvent, WindowEvent};
use tracing_subscriber::{EnvFilter, fmt};

mod app_runtime;
mod app_settings;
mod claude_scan;
mod codex_scan;
mod command_services;
mod commands;
mod constants;
mod diagnostics;
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
mod native_panel_renderer;
mod native_panel_runtime;
mod native_panel_scene;
mod native_panel_scene_input;
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
mod windows_native_panel;

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
use native_panel_renderer::facade::runtime::{
    NativePanelRuntimeBackend, current_native_panel_runtime_backend,
};
use startup_service::AppStartupService;
use web_panel_scene_service::WebPanelSceneState;

fn main() {
    setup_tracing();
    diagnostics::log_diagnostic_event(
        "app_start",
        &[
            diagnostics::current_process_fields(),
            vec![(
                "diagnostic_log",
                diagnostics::diagnostic_log_path().display().to_string(),
            )],
        ]
        .concat(),
    );
    if app_settings::current_app_settings().debug_mode_enabled {
        diagnostics::log_debug_mode_snapshot();
    }

    let runtime = Arc::new(SharedRuntime::new());
    let app_runtime = AppRuntime::new(runtime.clone());

    let builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .manage(app_runtime.clone())
        .manage(WebPanelSceneState::default())
        .on_window_event(|window, event| {
            diagnostics::log_diagnostic_event(
                "tauri_window_event",
                &[
                    ("label", window.label().to_string()),
                    ("event", window_event_name(event).to_string()),
                ],
            );
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            diagnostics::log_diagnostic_event("tauri_setup_begin", &[]);
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let app_handle = app.handle().clone();
            let native_panel_backend = current_native_panel_runtime_backend();
            if native_panel_backend.native_ui_enabled() {
                native_panel_backend
                    .create_panel()
                    .map_err(std::io::Error::other)?;
                if macos_shared_expanded_window::shared_expanded_enabled() {
                    macos_shared_expanded_window::create_shared_expanded_window(&app_handle)
                        .map_err(std::io::Error::other)?;
                }
                native_panel_backend
                    .hide_main_webview_window(&app_handle)
                    .map_err(std::io::Error::other)?;
                native_panel_runtime::spawn_native_snapshot_loop(
                    app_handle.clone(),
                    app_runtime.clone(),
                );
                native_panel_backend.spawn_platform_loops(app_handle.clone());
            } else {
                macos_panel::create_main_panel(&app_handle).map_err(std::io::Error::other)?;
            }
            AppStartupService::new(app)
                .initialize()
                .map_err(std::io::Error::other)?;
            spawn_codex_scan_loop(runtime.clone());
            spawn_claude_scan_loop(runtime.clone());
            spawn_ipc_server(app_handle, app_runtime.clone());
            spawn_http_receiver(app.handle().clone(), runtime.clone());
            diagnostics::log_diagnostic_event("tauri_setup_complete", &[]);
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
        .build(tauri::generate_context!())
        .expect("failed to build tauri app")
        .run(|_app_handle, event| {
            log_tauri_run_event(&event);
        });
}

fn window_event_name(event: &WindowEvent) -> &'static str {
    match event {
        WindowEvent::Resized(_) => "resized",
        WindowEvent::Moved(_) => "moved",
        WindowEvent::CloseRequested { .. } => "close_requested",
        WindowEvent::Destroyed => "destroyed",
        WindowEvent::Focused(true) => "focused_true",
        WindowEvent::Focused(false) => "focused_false",
        WindowEvent::ScaleFactorChanged { .. } => "scale_factor_changed",
        WindowEvent::DragDrop(_) => "drag_drop",
        WindowEvent::ThemeChanged(_) => "theme_changed",
        _ => "unknown",
    }
}

fn log_tauri_run_event(event: &RunEvent) {
    let Some((event_name, mut fields)) = run_event_diagnostic_fields(event) else {
        return;
    };
    fields.extend(diagnostics::current_process_fields());
    diagnostics::log_diagnostic_event(event_name, &fields);
}

fn run_event_diagnostic_fields(
    event: &RunEvent,
) -> Option<(&'static str, Vec<(&'static str, String)>)> {
    match event {
        RunEvent::Ready => Some(("tauri_run_event", vec![("event", "ready".to_string())])),
        RunEvent::Resumed => Some(("tauri_run_event", vec![("event", "resumed".to_string())])),
        RunEvent::Exit => Some(("tauri_run_event", vec![("event", "exit".to_string())])),
        RunEvent::ExitRequested { code, .. } => Some((
            "tauri_run_event",
            vec![
                ("event", "exit_requested".to_string()),
                (
                    "code",
                    code.map(|value| value.to_string()).unwrap_or_default(),
                ),
            ],
        )),
        #[cfg(target_os = "macos")]
        RunEvent::Opened { urls } => Some((
            "tauri_run_event",
            vec![
                ("event", "opened".to_string()),
                ("url_count", urls.len().to_string()),
                (
                    "urls",
                    urls.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            ],
        )),
        #[cfg(target_os = "macos")]
        RunEvent::Reopen {
            has_visible_windows,
            ..
        } => Some((
            "tauri_run_event",
            vec![
                ("event", "reopen".to_string()),
                ("has_visible_windows", has_visible_windows.to_string()),
            ],
        )),
        _ => None,
    }
}

fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}
