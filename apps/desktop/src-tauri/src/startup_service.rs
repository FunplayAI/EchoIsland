use std::path::PathBuf;

use echoisland_adapters::{ClaudeAdapter, CodexAdapter, InstallableAdapter, claude_default_paths};
use echoisland_paths::{bridge_binary_name, bridge_binary_path, codex_dir};
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
            info!("skipping webview window surface initialization in native macOS UI mode");
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
        let Some(source_bridge) = resolve_source_bridge() else {
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

fn resolve_source_bridge() -> Option<PathBuf> {
    let env_bridge = std::env::var_os("CODEISLAND_HOOK_BRIDGE").map(PathBuf::from);
    let current_exe_bridge = std::env::current_exe().ok().and_then(|path| {
        path.parent()
            .map(|parent| parent.join(bridge_binary_name()))
    });
    let build_dir_bridge = source_bridge_candidates_from_repo();
    let installed_bridge = Some(bridge_binary_path());

    [
        env_bridge,
        current_exe_bridge,
        build_dir_bridge,
        installed_bridge,
    ]
    .into_iter()
    .flatten()
    .find(|path| path.is_file())
}

fn source_bridge_candidates_from_repo() -> Option<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let preferred = repo_root
        .join("target")
        .join(profile)
        .join(bridge_binary_name());
    if preferred.is_file() {
        return Some(preferred);
    }

    ["debug", "release"]
        .into_iter()
        .map(|name| {
            repo_root
                .join("target")
                .join(name)
                .join(bridge_binary_name())
        })
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use crate::platform::PlatformBackend;

    use super::StartupPolicy;

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
}
