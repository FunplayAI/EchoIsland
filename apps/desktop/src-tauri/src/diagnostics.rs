use std::{
    fs::{OpenOptions, create_dir_all},
    io::Write,
    path::PathBuf,
};

use chrono::Utc;
use tracing::info;

const DIAGNOSTIC_LOG_FILE_NAME: &str = "diagnostics.log";

pub(crate) fn diagnostic_log_path() -> PathBuf {
    echoisland_paths::echoisland_app_dir().join(DIAGNOSTIC_LOG_FILE_NAME)
}

pub(crate) fn log_diagnostic_event(event: &str, fields: &[(&str, String)]) {
    if !diagnostic_logging_enabled() {
        return;
    }

    info!(event, fields = %format_fields(fields), "diagnostic event");

    let path = diagnostic_log_path();
    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let _ = writeln!(
        file,
        "{} event={} {}",
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        event,
        format_fields(fields)
    );
}

fn diagnostic_logging_enabled() -> bool {
    crate::app_settings::current_app_settings().debug_mode_enabled
}

pub(crate) fn log_debug_mode_snapshot() {
    let paths = echoisland_paths::current_platform_paths();
    log_diagnostic_event(
        "debug_mode_snapshot",
        &[
            ("pid", std::process::id().to_string()),
            (
                "current_exe",
                std::env::current_exe()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|error| format!("error:{error}")),
            ),
            (
                "current_dir",
                std::env::current_dir()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|error| format!("error:{error}")),
            ),
            ("app_dir", paths.echoisland_app_dir.display().to_string()),
            ("log_path", diagnostic_log_path().display().to_string()),
            ("version", env!("CARGO_PKG_VERSION").to_string()),
        ],
    );
}

pub(crate) fn current_process_fields() -> Vec<(&'static str, String)> {
    vec![
        ("pid", std::process::id().to_string()),
        (
            "current_exe",
            std::env::current_exe()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|error| format!("error:{error}")),
        ),
        (
            "cwd",
            std::env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|error| format!("error:{error}")),
        ),
    ]
}

pub(crate) fn current_context_fields() -> Vec<(&'static str, String)> {
    let mut fields = current_process_fields();
    fields.extend(current_foreground_app_fields());
    fields
}

#[cfg(target_os = "macos")]
pub(crate) fn current_foreground_app_fields() -> Vec<(&'static str, String)> {
    use objc2_app_kit::NSWorkspace;

    if !diagnostic_logging_enabled() {
        return Vec::new();
    }

    let workspace = NSWorkspace::sharedWorkspace();
    let Some(app) = workspace.frontmostApplication() else {
        return vec![("frontmost_app_available", "false".to_string())];
    };

    vec![
        ("frontmost_app_available", "true".to_string()),
        (
            "frontmost_app_name",
            app.localizedName()
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ),
        (
            "frontmost_bundle_id",
            app.bundleIdentifier()
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ),
        ("frontmost_pid", app.processIdentifier().to_string()),
    ]
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn current_foreground_app_fields() -> Vec<(&'static str, String)> {
    Vec::new()
}

pub(crate) fn command_output_preview(bytes: &[u8]) -> String {
    const MAX_OUTPUT_CHARS: usize = 600;
    let output = String::from_utf8_lossy(bytes).trim().to_string();
    if output.chars().count() <= MAX_OUTPUT_CHARS {
        return output;
    }

    output.chars().take(MAX_OUTPUT_CHARS).collect::<String>() + "...<truncated>"
}

fn format_fields(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={}", quote_value(value)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_value(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
