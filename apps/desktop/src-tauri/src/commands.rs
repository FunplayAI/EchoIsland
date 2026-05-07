use echoisland_adapters::{
    ClaudeStatus, CodexStatus, OpenClawStatus, claude_default_paths,
    default_paths as codex_default_paths, get_claude_status, get_codex_status, get_openclaw_status,
    openclaw_default_paths,
};
use echoisland_core::ResponseEnvelope;
use echoisland_ipc::DEFAULT_ADDR;
use echoisland_runtime::RuntimeSnapshot;
use serde::Serialize;
use std::{path::Path, process::Command};
use tauri::{AppHandle, State};

use crate::{
    app_runtime::AppRuntime,
    app_settings::{
        AppSettings, current_app_settings, update_completion_sound_enabled, update_mascot_enabled,
        update_preferred_display_selection,
    },
    command_services::{SampleIngestService, SnapshotCommandService},
    display_settings::{DisplayOption, list_available_displays},
    http_receiver::{HttpReceiverStatus, default_http_receiver_status},
    native_panel_renderer::facade::{
        command::execute_native_panel_cycle_display_command,
        runtime::{NativePanelRuntimeBackend, current_native_panel_runtime_backend},
    },
    native_panel_scene::{
        PanelSceneBuildInput, SessionSurfaceScene, SettingsSurfaceScene, StatusSurfaceScene,
        SurfaceScene,
    },
    native_panel_scene_input::{
        panel_scene_build_input_from_display_options,
        resolve_selected_display_index_from_display_options,
    },
    native_ui_refresh::maybe_refresh_native_ui_for_event,
    panel_scene_service::PanelSceneState,
    platform::{
        PlatformCapabilities, PlatformPathsPayload, current_platform_capabilities,
        current_platform_paths,
    },
    terminal_focus_service::TerminalFocusService,
    updater_service::{self, AppUpdateState, AppUpdateStatus},
    window_surface_service::WindowSurfaceService,
};
const PANEL_STAGE_HEIGHT: f64 = 580.0;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotStatusSurfaceBundle {
    pub snapshot: RuntimeSnapshot,
    pub surface_scene: SurfaceScene,
    pub status_surface_scene: StatusSurfaceScene,
    pub session_surface_scene: SessionSurfaceScene,
    pub settings_surface_scene: SettingsSurfaceScene,
}

#[tauri::command]
pub async fn get_snapshot(runtime: State<'_, AppRuntime>) -> Result<RuntimeSnapshot, String> {
    SnapshotCommandService::new(runtime.inner())
        .get_snapshot()
        .await
}

#[tauri::command]
pub async fn get_snapshot_status_surface_bundle(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    scene_state: State<'_, PanelSceneState>,
) -> Result<SnapshotStatusSurfaceBundle, String> {
    let snapshot = SnapshotCommandService::new(runtime.inner())
        .get_snapshot()
        .await?;
    let input = panel_scene_build_input(&app);
    let (surface_scene, status_surface_scene, session_surface_scene, settings_surface_scene) =
        scene_state.build_surface_scenes(&snapshot, &input)?;
    Ok(SnapshotStatusSurfaceBundle {
        snapshot,
        surface_scene,
        status_surface_scene,
        session_surface_scene,
        settings_surface_scene,
    })
}

fn panel_scene_build_input(app: &AppHandle) -> PanelSceneBuildInput {
    let settings = current_app_settings();
    let displays = list_available_displays(app).unwrap_or_default();
    panel_scene_build_input_from_display_options(
        &displays,
        &settings,
        Some(settings.preferred_display_index),
    )
}

#[tauri::command]
pub async fn build_status_surface_scene(
    runtime: State<'_, AppRuntime>,
    scene_state: State<'_, PanelSceneState>,
) -> Result<StatusSurfaceScene, String> {
    let snapshot = SnapshotCommandService::new(runtime.inner())
        .get_snapshot()
        .await?;
    scene_state.build_status_surface_scene(&snapshot)
}

#[tauri::command]
pub fn get_app_settings(app: AppHandle) -> AppSettings {
    let mut settings = current_app_settings();
    if let Ok(displays) = list_available_displays(&app) {
        settings.preferred_display_index = resolve_selected_display_index_from_display_options(
            &displays,
            &settings,
            Some(settings.preferred_display_index),
        );
    }
    settings
}

#[tauri::command]
pub fn get_available_displays(app: AppHandle) -> Result<Vec<DisplayOption>, String> {
    list_available_displays(&app)
}

#[tauri::command]
pub async fn ingest_sample(
    file_name: String,
    runtime: State<'_, AppRuntime>,
) -> Result<ResponseEnvelope, String> {
    SampleIngestService::new(runtime.inner())
        .ingest_sample(file_name)
        .await
}

#[tauri::command]
pub fn ipc_addr() -> String {
    DEFAULT_ADDR.to_string()
}

#[tauri::command]
pub fn codex_status() -> Result<CodexStatus, String> {
    get_codex_status(&codex_default_paths()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn claude_status() -> Result<ClaudeStatus, String> {
    get_claude_status(&claude_default_paths()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn openclaw_status() -> Result<OpenClawStatus, String> {
    get_openclaw_status(&openclaw_default_paths()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn http_receiver_status() -> HttpReceiverStatus {
    default_http_receiver_status()
}

#[tauri::command]
pub fn platform_capabilities() -> PlatformCapabilities {
    current_platform_capabilities()
}

#[tauri::command]
pub fn platform_paths() -> PlatformPathsPayload {
    current_platform_paths()
}

#[tauri::command]
pub async fn approve_permission(
    request_id: String,
    runtime: State<'_, AppRuntime>,
    app: AppHandle,
) -> Result<(), String> {
    runtime.runtime.approve_permission(&request_id).await?;
    maybe_refresh_native_ui_for_event(app, runtime.runtime.clone(), "PermissionResponse");
    Ok(())
}

#[tauri::command]
pub async fn deny_permission(
    request_id: String,
    runtime: State<'_, AppRuntime>,
    app: AppHandle,
) -> Result<(), String> {
    runtime.runtime.deny_permission(&request_id).await?;
    maybe_refresh_native_ui_for_event(app, runtime.runtime.clone(), "PermissionResponse");
    Ok(())
}

#[tauri::command]
pub async fn answer_question(
    request_id: String,
    answer: String,
    runtime: State<'_, AppRuntime>,
    app: AppHandle,
) -> Result<(), String> {
    runtime
        .runtime
        .answer_question(&request_id, &answer)
        .await?;
    maybe_refresh_native_ui_for_event(app, runtime.runtime.clone(), "QuestionResponse");
    Ok(())
}

#[tauri::command]
pub async fn skip_question(
    request_id: String,
    runtime: State<'_, AppRuntime>,
    app: AppHandle,
) -> Result<(), String> {
    runtime.runtime.skip_question(&request_id).await?;
    maybe_refresh_native_ui_for_event(app, runtime.runtime.clone(), "QuestionResponse");
    Ok(())
}

#[tauri::command]
pub fn set_island_expanded(expanded: bool, app: AppHandle) -> Result<(), String> {
    WindowSurfaceService::new(&app).set_expanded_passive(expanded)
}

#[tauri::command]
pub fn set_island_expanded_passive(expanded: bool, app: AppHandle) -> Result<(), String> {
    WindowSurfaceService::new(&app).set_expanded_passive(expanded)
}

#[tauri::command]
pub fn set_island_bar_stage(app: AppHandle) -> Result<(), String> {
    WindowSurfaceService::new(&app).set_bar_stage_passive()
}

#[tauri::command]
pub fn set_island_bar_stage_passive(app: AppHandle) -> Result<(), String> {
    WindowSurfaceService::new(&app).set_bar_stage_passive()
}

#[tauri::command]
pub fn set_island_panel_stage(app: AppHandle, height: Option<f64>) -> Result<(), String> {
    WindowSurfaceService::new(&app).set_panel_stage_passive(height.unwrap_or(PANEL_STAGE_HEIGHT))
}

#[tauri::command]
pub fn set_island_panel_stage_passive(app: AppHandle, height: Option<f64>) -> Result<(), String> {
    WindowSurfaceService::new(&app).set_panel_stage_passive(height.unwrap_or(PANEL_STAGE_HEIGHT))
}

#[tauri::command]
pub fn show_main_window_interactive(app: AppHandle) -> Result<(), String> {
    WindowSurfaceService::new(&app).show_main_window_interactive()
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    let native_panel_backend = current_native_panel_runtime_backend();
    if native_panel_backend.native_ui_enabled() {
        return native_panel_backend.hide_panel(&app);
    }

    WindowSurfaceService::new(&app).hide_main_window()
}

#[tauri::command]
pub fn open_settings_location() -> Result<(), String> {
    let paths = current_platform_paths();
    let settings_dir = std::path::PathBuf::from(paths.echoisland_app_dir);
    std::fs::create_dir_all(&settings_dir).map_err(|error| error.to_string())?;
    open_path_with_system(&settings_dir, "open_settings_location")
}

#[tauri::command]
pub fn set_completion_sound_enabled(enabled: bool) -> Result<AppSettings, String> {
    update_completion_sound_enabled(enabled).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_mascot_enabled(enabled: bool, app: AppHandle) -> Result<AppSettings, String> {
    let settings = update_mascot_enabled(enabled).map_err(|error| error.to_string())?;
    refresh_desktop_after_settings_change(&app);
    Ok(settings)
}

#[tauri::command]
pub fn set_preferred_display_index(index: usize, app: AppHandle) -> Result<AppSettings, String> {
    let displays = list_available_displays(&app)?;
    if index >= displays.len() {
        return Err(format!("display index out of range: {index}"));
    }
    let selected = &displays[index];
    let settings = update_preferred_display_selection(index, Some(selected.key.clone()))
        .map_err(|error| error.to_string())?;
    refresh_desktop_after_settings_change(&app);
    reposition_desktop_to_selected_display(&app)?;
    Ok(settings)
}

#[tauri::command]
pub fn cycle_display(app: AppHandle) -> Result<AppSettings, String> {
    let settings = execute_native_panel_cycle_display_command(&app, |app| {
        reposition_desktop_to_selected_display(app)
    })?;
    refresh_desktop_after_settings_change(&app);
    Ok(settings)
}

#[tauri::command]
pub fn quit_application(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn open_release_page() -> Result<(), String> {
    open_url_with_system(
        "https://github.com/FunplayAI/EchoIsland/releases/latest",
        "open_release_page",
    )
}

#[tauri::command]
pub fn get_update_status(state: State<'_, AppUpdateState>) -> AppUpdateStatus {
    updater_service::app_update_status_from_state(state.inner())
}

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: State<'_, AppUpdateState>,
) -> Result<AppUpdateStatus, String> {
    Ok(updater_service::check_for_update(&app, state.inner()).await)
}

#[tauri::command]
pub async fn download_and_install_update(
    app: AppHandle,
    state: State<'_, AppUpdateState>,
) -> Result<AppUpdateStatus, String> {
    Ok(updater_service::download_and_install_update(&app, state.inner()).await)
}

fn open_path_with_system(path: &Path, caller: &str) -> Result<(), String> {
    let program = system_open_program();
    let mut fields = path_open_diagnostic_fields(path, program);
    fields.push(("caller", caller.to_string()));
    crate::diagnostics::log_diagnostic_event("system_open_path_begin", &fields);

    let output = match Command::new(program).arg(path).output() {
        Ok(output) => output,
        Err(error) => {
            fields.push(("error", error.to_string()));
            crate::diagnostics::log_diagnostic_event("system_open_path_spawn_error", &fields);
            return Err(error.to_string());
        }
    };

    fields.extend([
        ("status", output.status.to_string()),
        ("success", output.status.success().to_string()),
        (
            "stdout",
            crate::diagnostics::command_output_preview(&output.stdout),
        ),
        (
            "stderr",
            crate::diagnostics::command_output_preview(&output.stderr),
        ),
    ]);
    crate::diagnostics::log_diagnostic_event("system_open_path_complete", &fields);
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to open settings location: {}",
            output.status
        ))
    }
}

fn open_url_with_system(url: &str, caller: &str) -> Result<(), String> {
    let program = system_open_program();
    let mut fields = url_open_diagnostic_fields(url, program);
    fields.push(("caller", caller.to_string()));
    crate::diagnostics::log_diagnostic_event("system_open_url_begin", &fields);

    let output = match Command::new(program).arg(url).output() {
        Ok(output) => output,
        Err(error) => {
            fields.push(("error", error.to_string()));
            crate::diagnostics::log_diagnostic_event("system_open_url_spawn_error", &fields);
            return Err(error.to_string());
        }
    };

    fields.extend([
        ("status", output.status.to_string()),
        ("success", output.status.success().to_string()),
        (
            "stdout",
            crate::diagnostics::command_output_preview(&output.stdout),
        ),
        (
            "stderr",
            crate::diagnostics::command_output_preview(&output.stderr),
        ),
    ]);
    crate::diagnostics::log_diagnostic_event("system_open_url_complete", &fields);
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("failed to open url: {}", output.status))
    }
}

fn system_open_program() -> &'static str {
    if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    }
}

fn path_open_diagnostic_fields(path: &Path, program: &'static str) -> Vec<(&'static str, String)> {
    let mut fields = vec![
        ("program", program.to_string()),
        ("target_type", "path".to_string()),
        ("path", path.display().to_string()),
        ("path_exists", path.exists().to_string()),
        ("path_is_file", path.is_file().to_string()),
        ("path_is_dir", path.is_dir().to_string()),
        (
            "canonical_path",
            path.canonicalize()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|error| format!("error:{error}")),
        ),
    ];
    fields.extend(crate::diagnostics::current_context_fields());
    fields
}

fn url_open_diagnostic_fields(url: &str, program: &'static str) -> Vec<(&'static str, String)> {
    let mut fields = vec![
        ("program", program.to_string()),
        ("target_type", "url".to_string()),
        ("url", url.to_string()),
    ];
    fields.extend(crate::diagnostics::current_context_fields());
    fields
}

fn refresh_desktop_after_settings_change<R: tauri::Runtime>(app: &AppHandle<R>) {
    let native_panel_backend = current_native_panel_runtime_backend();
    if native_panel_backend.native_ui_enabled() {
        let _ = native_panel_backend.refresh_from_last_snapshot(app);
    }
}

fn reposition_desktop_to_selected_display<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    let native_panel_backend = current_native_panel_runtime_backend();
    if native_panel_backend.native_ui_enabled() {
        return native_panel_backend.reposition_to_selected_display(app);
    }

    crate::window_surface_service::WindowSurfaceService::new(app).reposition_to_selected_display()
}

#[tauri::command]
pub async fn focus_session_terminal(
    session_id: String,
    runtime: State<'_, AppRuntime>,
) -> Result<bool, String> {
    TerminalFocusService::new(runtime.inner())
        .focus_session(&session_id)
        .await
}

#[tauri::command]
pub async fn bind_session_terminal(
    session_id: String,
    runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    TerminalFocusService::new(runtime.inner())
        .bind_session(&session_id)
        .await
}
