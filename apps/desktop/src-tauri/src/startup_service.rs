use std::path::{Path, PathBuf};

use echoisland_adapters::{ClaudeAdapter, CodexAdapter, InstallableAdapter, claude_default_paths};
use echoisland_paths::{bridge_binary_name, bridge_binary_path, codex_dir};
use tauri::Manager;
use tracing::{info, warn};

use crate::{
    native_panel_renderer::facade::runtime::{
        NativePanelRuntimeBackend, current_native_panel_runtime_backend,
    },
    platform::{PlatformBackend, current_platform_capabilities},
    tray::build_tray,
    window_surface_service::WindowSurfaceService,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StartupPolicy {
    platform_backend: PlatformBackend,
    supports_tray: bool,
    can_shape_window_region: bool,
    fail_fast: bool,
}

impl StartupPolicy {
    fn current() -> Self {
        let capabilities = current_platform_capabilities();
        let fail_fast = matches!(capabilities.platform_backend, PlatformBackend::Windows);

        Self {
            platform_backend: capabilities.platform_backend,
            supports_tray: capabilities.supports_tray,
            can_shape_window_region: capabilities.can_shape_window_region,
            fail_fast,
        }
    }
}

pub struct AppStartupService<'a, R: tauri::Runtime> {
    app: &'a mut tauri::App<R>,
    policy: StartupPolicy,
}

impl<'a, R: tauri::Runtime> AppStartupService<'a, R> {
    pub fn new(app: &'a mut tauri::App<R>) -> Self {
        Self {
            app,
            policy: StartupPolicy::current(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        self.initialize_window_surface()?;
        self.initialize_tray()?;
        self.initialize_hooks()?;
        Ok(())
    }

    fn initialize_window_surface(&self) -> Result<(), String> {
        let native_panel_backend = current_native_panel_runtime_backend();
        if native_panel_backend.native_ui_enabled() {
            info!(
                backend = ?self.policy.platform_backend,
                "skipping webview window surface initialization because native panel backend is enabled"
            );
            return Ok(());
        }

        if !self.policy.can_shape_window_region {
            info!(
                backend = ?self.policy.platform_backend,
                "initializing compact window surface without platform region shaping"
            );
        }

        let app_handle = self.app.handle().clone();

        match WindowSurfaceService::new(&app_handle).initialize_compact() {
            Ok(()) => {
                info!(backend = ?self.policy.platform_backend, "initialized compact window surface");
                Ok(())
            }
            Err(error) if self.policy.fail_fast => Err(error),
            Err(error) => {
                warn!(
                    backend = ?self.policy.platform_backend,
                    error = %error,
                    "window surface initialization failed; continuing with degraded startup"
                );
                Ok(())
            }
        }
    }

    fn initialize_tray(&mut self) -> Result<(), String> {
        if !self.policy.supports_tray {
            info!(
                backend = ?self.policy.platform_backend,
                "skipping tray initialization because platform reports no tray support"
            );
            return Ok(());
        }

        match build_tray(self.app) {
            Ok(()) => {
                info!(backend = ?self.policy.platform_backend, "initialized app tray");
                Ok(())
            }
            Err(error) if self.policy.fail_fast => Err(error.to_string()),
            Err(error) => {
                warn!(
                    backend = ?self.policy.platform_backend,
                    error = %error,
                    "tray initialization failed; continuing with degraded startup"
                );
                Ok(())
            }
        }
    }

    fn initialize_hooks(&self) -> Result<(), String> {
        let resource_dir = self.app.path().resource_dir().ok();
        let Some(source_bridge) = resolve_source_bridge(resource_dir.as_deref()) else {
            warn!("hook bridge binary not found; skipping Claude/Codex hook installation");
            return Ok(());
        };

        let claude_paths = claude_default_paths();
        if claude_paths.claude_dir.exists() || claude_paths.settings_path.exists() {
            match ClaudeAdapter::with_default_paths().install(&source_bridge) {
                Ok(status) => {
                    info!(
                        bridge_exists = status.bridge_exists,
                        hooks_installed = status.hooks_installed,
                        "installed or repaired Claude hooks"
                    );
                }
                Err(error) => {
                    warn!(error = %error, "failed to install or repair Claude hooks");
                }
            }
        } else {
            info!("skipping Claude hook installation because ~/.claude is absent");
        }

        if codex_dir().exists() {
            match CodexAdapter::with_default_paths().install(&source_bridge) {
                Ok(status) => {
                    info!(
                        bridge_exists = status.bridge_exists,
                        hooks_installed = status.hooks_installed,
                        hooks_enabled = status.hooks_enabled,
                        "installed or repaired Codex hooks"
                    );
                }
                Err(error) => {
                    warn!(error = %error, "failed to install or repair Codex hooks");
                }
            }
        } else {
            info!("skipping Codex hook installation because ~/.codex is absent");
        }

        Ok(())
    }
}

fn resolve_source_bridge(resource_dir: Option<&Path>) -> Option<PathBuf> {
    source_bridge_candidates(resource_dir)
        .into_iter()
        .find(|path| path.is_file())
}

fn source_bridge_candidates(resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(env_bridge) = std::env::var_os("ECHOISLAND_HOOK_BRIDGE") {
        candidates.push(PathBuf::from(env_bridge));
    }
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        candidates.push(parent.join(bridge_binary_name()));
    }
    if let Some(resource_dir) = resource_dir {
        candidates.push(resource_dir.join(bridge_binary_name()));
    }
    candidates.extend(source_bridge_candidates_from_repo());

    candidates
}

fn source_bridge_candidates_from_repo() -> Vec<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let current_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let mut candidates = Vec::new();
    for profile in [current_profile, "debug", "release"] {
        candidates.push(
            repo_root
                .join("target")
                .join(profile)
                .join(bridge_binary_name()),
        );
    }
    candidates.push(bridge_binary_path());
    candidates
}

#[cfg(test)]
mod tests {
    use crate::platform::PlatformBackend;

    use std::path::PathBuf;

    use super::{StartupPolicy, source_bridge_candidates_from_repo};

    #[test]
    fn startup_policy_matches_current_backend() {
        let policy = StartupPolicy::current();

        #[cfg(target_os = "windows")]
        {
            assert_eq!(policy.platform_backend, PlatformBackend::Windows);
            assert!(policy.fail_fast);
            assert!(policy.supports_tray);
            assert!(policy.can_shape_window_region);
        }

        #[cfg(not(target_os = "windows"))]
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(policy.platform_backend, PlatformBackend::Stub);
            assert!(!policy.fail_fast);
            assert!(!policy.supports_tray);
            assert!(!policy.can_shape_window_region);
        }

        #[cfg(target_os = "macos")]
        {
            assert_eq!(policy.platform_backend, PlatformBackend::Macos);
            assert!(!policy.fail_fast);
            assert!(!policy.supports_tray);
            assert!(!policy.can_shape_window_region);
        }
    }

    #[test]
    fn source_bridge_candidates_use_echoisland_bridge() {
        let candidates = source_bridge_candidates_from_repo();

        assert!(!candidates.is_empty());
        assert!(candidates.contains(&echoisland_paths::bridge_binary_path()));
        assert!(candidates.iter().all(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with("echoisland-hook-bridge"))
        }));
    }

    #[test]
    fn source_bridge_candidates_include_resource_dir_bridge() {
        let resource_dir = PathBuf::from("C:/Program Files/EchoIsland");
        let candidates = super::source_bridge_candidates(Some(&resource_dir));

        assert!(candidates.contains(&resource_dir.join(echoisland_paths::bridge_binary_name())));
    }
}
