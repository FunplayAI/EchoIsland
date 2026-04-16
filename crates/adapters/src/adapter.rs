use std::path::Path;

use anyhow::Result;
use echoisland_core::SessionRecord;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AdapterPath {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AdapterStatus {
    pub adapter: String,
    pub config_dir_exists: bool,
    pub bridge_exists: bool,
    pub hooks_installed: bool,
    pub hooks_enabled: bool,
    pub live_capture_supported: bool,
    pub live_capture_ready: bool,
    pub status_note: Option<String>,
    pub paths: Vec<AdapterPath>,
}

pub trait InstallableAdapter {
    fn adapter_id(&self) -> &'static str;
    fn status(&self) -> Result<AdapterStatus>;
    fn install(&self, source_bridge: &Path) -> Result<AdapterStatus>;
}

pub trait SessionScanningAdapter {
    fn adapter_id(&self) -> &'static str;
    fn scan_sessions(&self) -> Result<Vec<SessionRecord>>;
}
