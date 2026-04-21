use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use anyhow::{Context, Result};
use echoisland_paths::current_platform_paths;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_completion_sound_enabled")]
    pub completion_sound_enabled: bool,
    #[serde(default = "default_mascot_enabled")]
    pub mascot_enabled: bool,
    #[serde(default)]
    pub preferred_display_index: usize,
    #[serde(default)]
    pub preferred_display_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            completion_sound_enabled: default_completion_sound_enabled(),
            mascot_enabled: default_mascot_enabled(),
            preferred_display_index: 0,
            preferred_display_key: None,
        }
    }
}

static APP_SETTINGS_CACHE: OnceLock<Mutex<AppSettings>> = OnceLock::new();

pub fn app_settings_path() -> PathBuf {
    current_platform_paths()
        .echoisland_app_dir
        .join("app-settings.json")
}

pub fn current_app_settings() -> AppSettings {
    APP_SETTINGS_CACHE
        .get_or_init(|| Mutex::new(load_app_settings_from_disk().unwrap_or_default()))
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

pub fn update_completion_sound_enabled(enabled: bool) -> Result<AppSettings> {
    let cache = APP_SETTINGS_CACHE
        .get_or_init(|| Mutex::new(load_app_settings_from_disk().unwrap_or_default()));
    let mut guard = cache
        .lock()
        .map_err(|_| anyhow::anyhow!("app settings lock poisoned"))?;
    if guard.completion_sound_enabled == enabled {
        return Ok(guard.clone());
    }
    guard.completion_sound_enabled = enabled;
    save_app_settings(&app_settings_path(), &guard)?;
    Ok(guard.clone())
}

pub fn update_mascot_enabled(enabled: bool) -> Result<AppSettings> {
    let cache = APP_SETTINGS_CACHE
        .get_or_init(|| Mutex::new(load_app_settings_from_disk().unwrap_or_default()));
    let mut guard = cache
        .lock()
        .map_err(|_| anyhow::anyhow!("app settings lock poisoned"))?;
    if guard.mascot_enabled == enabled {
        return Ok(guard.clone());
    }
    guard.mascot_enabled = enabled;
    save_app_settings(&app_settings_path(), &guard)?;
    Ok(guard.clone())
}

pub fn update_preferred_display_selection(
    index: usize,
    key: Option<String>,
) -> Result<AppSettings> {
    let cache = APP_SETTINGS_CACHE
        .get_or_init(|| Mutex::new(load_app_settings_from_disk().unwrap_or_default()));
    let mut guard = cache
        .lock()
        .map_err(|_| anyhow::anyhow!("app settings lock poisoned"))?;
    if guard.preferred_display_index == index && guard.preferred_display_key == key {
        return Ok(guard.clone());
    }
    guard.preferred_display_index = index;
    guard.preferred_display_key = key;
    save_app_settings(&app_settings_path(), &guard)?;
    Ok(guard.clone())
}

fn load_app_settings_from_disk() -> Result<AppSettings> {
    load_app_settings(&app_settings_path())
}

fn load_app_settings(path: &Path) -> Result<AppSettings> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let raw = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&raw).context("failed to decode app settings")
}

fn save_app_settings(path: &Path, settings: &AppSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let encoded = serde_json::to_vec_pretty(settings).context("failed to encode app settings")?;
    fs::write(path, encoded).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn default_completion_sound_enabled() -> bool {
    true
}

fn default_mascot_enabled() -> bool {
    true
}
