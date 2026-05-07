use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};
use tracing::warn;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AppUpdatePhase {
    Idle,
    Checking,
    UpToDate,
    Available,
    Downloading,
    Installing,
    Installed,
    Failed,
    UnsupportedPortable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppUpdateStatus {
    pub(crate) phase: AppUpdatePhase,
    pub(crate) label: String,
    pub(crate) value_text: String,
    pub(crate) version: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) can_check: bool,
    pub(crate) can_install: bool,
    pub(crate) can_open_release_page: bool,
}

impl AppUpdateStatus {
    pub(crate) fn idle() -> Self {
        Self {
            phase: AppUpdatePhase::Idle,
            label: "Update & Upgrade".to_string(),
            value_text: "Check".to_string(),
            version: None,
            notes: None,
            error: None,
            can_check: true,
            can_install: false,
            can_open_release_page: true,
        }
    }

    fn checking() -> Self {
        Self {
            phase: AppUpdatePhase::Checking,
            label: "Checking updates".to_string(),
            value_text: "Checking...".to_string(),
            version: None,
            notes: None,
            error: None,
            can_check: false,
            can_install: false,
            can_open_release_page: false,
        }
    }

    fn up_to_date() -> Self {
        Self {
            phase: AppUpdatePhase::UpToDate,
            label: "Update & Upgrade".to_string(),
            value_text: "Latest".to_string(),
            version: None,
            notes: None,
            error: None,
            can_check: true,
            can_install: false,
            can_open_release_page: true,
        }
    }

    fn available(version: String, notes: Option<String>) -> Self {
        Self {
            phase: AppUpdatePhase::Available,
            label: format!("Version {version} available"),
            value_text: "Install".to_string(),
            version: Some(version),
            notes,
            error: None,
            can_check: true,
            can_install: true,
            can_open_release_page: true,
        }
    }

    fn downloading(version: Option<String>) -> Self {
        Self {
            phase: AppUpdatePhase::Downloading,
            label: "Downloading update".to_string(),
            value_text: "Downloading...".to_string(),
            version,
            notes: None,
            error: None,
            can_check: false,
            can_install: false,
            can_open_release_page: false,
        }
    }

    fn installing(version: Option<String>) -> Self {
        Self {
            phase: AppUpdatePhase::Installing,
            label: "Installing update".to_string(),
            value_text: "Installing...".to_string(),
            version,
            notes: None,
            error: None,
            can_check: false,
            can_install: false,
            can_open_release_page: false,
        }
    }

    fn installed(version: Option<String>) -> Self {
        Self {
            phase: AppUpdatePhase::Installed,
            label: "Update installed".to_string(),
            value_text: "Restarting".to_string(),
            version,
            notes: None,
            error: None,
            can_check: false,
            can_install: false,
            can_open_release_page: false,
        }
    }

    fn failed(error: String) -> Self {
        Self {
            phase: AppUpdatePhase::Failed,
            label: "Update failed".to_string(),
            value_text: "Open Releases".to_string(),
            version: None,
            notes: None,
            error: Some(error),
            can_check: true,
            can_install: false,
            can_open_release_page: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PortableUpdatePolicy {
    Installed,
    Portable,
}

#[derive(Default)]
pub(crate) struct AppUpdateState {
    status: Mutex<AppUpdateStatus>,
    pending_update: Mutex<Option<Update>>,
}

impl Default for AppUpdateStatus {
    fn default() -> Self {
        Self::idle()
    }
}

static FALLBACK_UPDATE_STATUS: OnceLock<Mutex<AppUpdateStatus>> = OnceLock::new();

pub(crate) fn current_update_status() -> AppUpdateStatus {
    fallback_update_status()
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|_| AppUpdateStatus::failed("update status lock poisoned".to_string()))
}

pub(crate) fn update_status_for_portable_policy(policy: PortableUpdatePolicy) -> AppUpdateStatus {
    match policy {
        PortableUpdatePolicy::Installed => AppUpdateStatus::idle(),
        PortableUpdatePolicy::Portable => AppUpdateStatus {
            phase: AppUpdatePhase::UnsupportedPortable,
            label: "Portable version".to_string(),
            value_text: "Open Releases".to_string(),
            version: None,
            notes: None,
            error: Some("Portable builds require manual download.".to_string()),
            can_check: false,
            can_install: false,
            can_open_release_page: true,
        },
    }
}

pub(crate) fn detect_update_policy() -> PortableUpdatePolicy {
    if std::env::var("ECHOISLAND_PORTABLE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
    {
        return PortableUpdatePolicy::Portable;
    }
    if portable_marker_exists_near_current_exe() {
        return PortableUpdatePolicy::Portable;
    }

    if cfg!(debug_assertions) {
        return PortableUpdatePolicy::Installed;
    }

    if tauri::utils::platform::bundle_type().is_none() {
        PortableUpdatePolicy::Portable
    } else {
        PortableUpdatePolicy::Installed
    }
}

pub(crate) fn app_update_status_from_state(state: &AppUpdateState) -> AppUpdateStatus {
    state
        .status
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|_| AppUpdateStatus::failed("update status lock poisoned".to_string()))
}

pub(crate) async fn check_for_update<R: tauri::Runtime>(
    app: &AppHandle<R>,
    state: &AppUpdateState,
) -> AppUpdateStatus {
    if detect_update_policy() == PortableUpdatePolicy::Portable {
        let status = update_status_for_portable_policy(PortableUpdatePolicy::Portable);
        set_update_status(state, status.clone());
        return status;
    }

    set_update_status(state, AppUpdateStatus::checking());
    let update_check = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(error) => Err(error),
    };

    match update_check {
        Ok(Some(update)) => {
            let status = AppUpdateStatus::available(update.version.clone(), update.body.clone());
            if let Ok(mut pending_update) = state.pending_update.lock() {
                pending_update.replace(update);
            }
            set_update_status(state, status.clone());
            status
        }
        Ok(None) => {
            let status = AppUpdateStatus::up_to_date();
            set_update_status(state, status.clone());
            status
        }
        Err(error) => {
            let status = AppUpdateStatus::failed(error.to_string());
            set_update_status(state, status.clone());
            status
        }
    }
}

pub(crate) async fn download_and_install_update<R: tauri::Runtime>(
    app: &AppHandle<R>,
    state: &AppUpdateState,
) -> AppUpdateStatus {
    if detect_update_policy() == PortableUpdatePolicy::Portable {
        let status = update_status_for_portable_policy(PortableUpdatePolicy::Portable);
        set_update_status(state, status.clone());
        return status;
    }

    let update = match take_pending_update(state) {
        Some(update) => update,
        None => match app.updater() {
            Ok(updater) => match updater.check().await {
                Ok(Some(update)) => update,
                Ok(None) => {
                    let status = AppUpdateStatus::up_to_date();
                    set_update_status(state, status.clone());
                    return status;
                }
                Err(error) => {
                    let status = AppUpdateStatus::failed(error.to_string());
                    set_update_status(state, status.clone());
                    return status;
                }
            },
            Err(error) => {
                let status = AppUpdateStatus::failed(error.to_string());
                set_update_status(state, status.clone());
                return status;
            }
        },
    };

    let version = Some(update.version.clone());
    set_update_status(state, AppUpdateStatus::downloading(version.clone()));
    let installing_version = version.clone();
    let result = update
        .download_and_install(
            |_chunk_length, _content_length| {},
            || set_update_status(state, AppUpdateStatus::installing(installing_version)),
        )
        .await;

    match result {
        Ok(()) => {
            let status = AppUpdateStatus::installed(version);
            set_update_status(state, status.clone());
            status
        }
        Err(error) => {
            let status = AppUpdateStatus::failed(error.to_string());
            set_update_status(state, status.clone());
            status
        }
    }
}

pub(crate) fn spawn_native_update_flow<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppUpdateState>();
        let status = app_update_status_from_state(&state);
        if open_release_page_on_portable_or_failed(&status) {
            let _ = crate::commands::open_release_page();
            return;
        }
        let next_status = if status.can_install {
            download_and_install_update(&app, &state).await
        } else {
            check_for_update(&app, &state).await
        };
        if open_release_page_on_portable_or_failed(&next_status) {
            warn!(
                error = ?next_status.error,
                "native update flow falling back to release page"
            );
            let _ = crate::commands::open_release_page();
        }
        refresh_native_panel_after_update_status_change(&app);
    });
}

pub(crate) fn open_release_page_on_portable_or_failed(status: &AppUpdateStatus) -> bool {
    matches!(
        status.phase,
        AppUpdatePhase::UnsupportedPortable | AppUpdatePhase::Failed
    )
}

fn set_update_status(state: &AppUpdateState, status: AppUpdateStatus) {
    if let Ok(mut current) = state.status.lock() {
        *current = status.clone();
    }
    if let Ok(mut fallback) = fallback_update_status().lock() {
        *fallback = status;
    }
}

fn take_pending_update(state: &AppUpdateState) -> Option<Update> {
    state
        .pending_update
        .lock()
        .ok()
        .and_then(|mut pending_update| pending_update.take())
}

fn fallback_update_status() -> &'static Mutex<AppUpdateStatus> {
    FALLBACK_UPDATE_STATUS.get_or_init(|| Mutex::new(AppUpdateStatus::idle()))
}

fn portable_marker_exists_near_current_exe() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join("EchoIsland.portable"))
        })
        .is_some_and(|marker| marker.exists())
}

fn refresh_native_panel_after_update_status_change<R: tauri::Runtime>(app: &AppHandle<R>) {
    use crate::native_panel_renderer::facade::runtime::{
        NativePanelRuntimeBackend, current_native_panel_runtime_backend,
    };

    let backend = current_native_panel_runtime_backend();
    if backend.native_ui_enabled() {
        let _ = backend.refresh_from_last_snapshot(app);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppUpdatePhase, AppUpdateStatus, PortableUpdatePolicy, update_status_for_portable_policy,
    };

    #[test]
    fn portable_policy_reports_manual_download_fallback() {
        let status = update_status_for_portable_policy(PortableUpdatePolicy::Portable);

        assert_eq!(status.phase, AppUpdatePhase::UnsupportedPortable);
        assert_eq!(status.label, "Portable version");
        assert_eq!(status.value_text, "Open Releases");
        assert!(status.can_open_release_page);
        assert!(!status.can_install);
    }

    #[test]
    fn idle_status_is_installable_only_after_update_is_available() {
        let status = AppUpdateStatus::idle();

        assert_eq!(status.phase, AppUpdatePhase::Idle);
        assert_eq!(status.label, "Update & Upgrade");
        assert_eq!(status.value_text, "Check");
        assert!(!status.can_install);
    }
}
