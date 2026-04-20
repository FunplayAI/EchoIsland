use echoisland_adapters::{
    ClaudeStatus, CodexStatus, OpenClawStatus, claude_default_paths,
    default_paths as codex_default_paths, get_claude_status, get_codex_status, get_openclaw_status,
    openclaw_default_paths,
};
use echoisland_core::ResponseEnvelope;
use echoisland_ipc::DEFAULT_ADDR;
use echoisland_runtime::RuntimeSnapshot;
use tauri::{AppHandle, State};

use crate::{
    app_runtime::{AppRuntime, maybe_refresh_native_ui_for_event},
    command_services::{SampleIngestService, SnapshotCommandService},
    http_receiver::{HttpReceiverStatus, default_http_receiver_status},
    platform::{
        PlatformCapabilities, PlatformPathsPayload, current_platform_capabilities,
        current_platform_paths,
    },
    terminal_focus_service::TerminalFocusService,
    window_surface_service::WindowSurfaceService,
};
const PANEL_STAGE_HEIGHT: f64 = 580.0;

#[tauri::command]
pub async fn get_snapshot(runtime: State<'_, AppRuntime>) -> Result<RuntimeSnapshot, String> {
    SnapshotCommandService::new(runtime.inner())
        .get_snapshot()
        .await
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
    #[cfg(target_os = "macos")]
    if crate::macos_native_test_panel::native_ui_enabled() {
        return crate::macos_native_test_panel::hide_native_island_panel(&app);
    }

    WindowSurfaceService::new(&app).hide_main_window()
}

#[tauri::command]
pub fn open_settings_location() -> Result<(), String> {
    let paths = current_platform_paths();
    let settings_dir = std::path::PathBuf::from(paths.echoisland_app_dir);
    std::fs::create_dir_all(&settings_dir).map_err(|error| error.to_string())?;
    open_path_with_system(&settings_dir)
}

#[tauri::command]
pub fn quit_application(app: AppHandle) {
    app.exit(0);
}

fn open_path_with_system(path: &std::path::Path) -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        std::process::Command::new("explorer")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?
    } else {
        std::process::Command::new("xdg-open")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("failed to open settings location: {status}"))
    }
}

#[tauri::command]
pub fn set_macos_shared_expanded_height(height: f64, app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if crate::macos_native_test_panel::native_ui_enabled() {
        return crate::macos_native_test_panel::set_shared_expanded_body_height(&app, height);
    }

    let _ = (height, app);
    Ok(())
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
