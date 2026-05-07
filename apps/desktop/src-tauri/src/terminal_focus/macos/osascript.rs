use std::{
    io::Write,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};

use tracing::{debug, warn};

static DID_WARN_ACCESSIBILITY_PERMISSION: AtomicBool = AtomicBool::new(false);
static DID_WARN_AUTOMATION_PERMISSION: AtomicBool = AtomicBool::new(false);

pub(super) fn run_osascript(source: &str, label: &str) -> bool {
    let Some(output) = run_osascript_raw(source, label) else {
        return false;
    };
    debug!(label, output = %output, "osascript command succeeded");
    true
}

pub(super) fn run_osascript_raw(source: &str, label: &str) -> Option<String> {
    let mut begin_fields = osascript_diagnostic_fields(label);
    begin_fields.push(("source_len", source.len().to_string()));
    crate::diagnostics::log_diagnostic_event("macos_osascript_begin", &begin_fields);
    let mut child = match Command::new("/usr/bin/osascript")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let mut fields = osascript_diagnostic_fields(label);
            fields.push(("error", error.to_string()));
            crate::diagnostics::log_diagnostic_event("macos_osascript_spawn_error", &fields);
            warn!(label, error = %error, "failed to spawn osascript");
            return None;
        }
    };

    if let Some(stdin) = child.stdin.as_mut()
        && let Err(error) = stdin.write_all(source.as_bytes())
    {
        let mut fields = osascript_diagnostic_fields(label);
        fields.push(("error", error.to_string()));
        crate::diagnostics::log_diagnostic_event("macos_osascript_write_error", &fields);
        warn!(label, error = %error, "failed to write osascript source");
        let _ = child.kill();
        return None;
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mut fields = osascript_diagnostic_fields(label);
            fields.extend([
                ("status", output.status.to_string()),
                ("stdout", stdout.clone()),
            ]);
            crate::diagnostics::log_diagnostic_event("macos_osascript_complete", &fields);
            Some(stdout)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            warn_macos_permission_hint(&stderr);
            let mut fields = osascript_diagnostic_fields(label);
            fields.extend([
                ("status", output.status.to_string()),
                ("stderr", stderr.clone()),
            ]);
            crate::diagnostics::log_diagnostic_event("macos_osascript_failed", &fields);
            warn!(label, stderr, "osascript command failed");
            None
        }
        Err(error) => {
            let mut fields = osascript_diagnostic_fields(label);
            fields.push(("error", error.to_string()));
            crate::diagnostics::log_diagnostic_event("macos_osascript_wait_error", &fields);
            warn!(label, error = %error, "failed waiting for osascript");
            None
        }
    }
}

fn osascript_diagnostic_fields(label: &str) -> Vec<(&'static str, String)> {
    let mut fields = vec![("label", label.to_string())];
    fields.extend(crate::diagnostics::current_context_fields());
    fields
}

fn warn_macos_permission_hint(stderr: &str) {
    if stderr.is_empty() {
        return;
    }

    if is_accessibility_permission_error(stderr) {
        if !DID_WARN_ACCESSIBILITY_PERMISSION.swap(true, Ordering::Relaxed) {
            warn!(
                "macOS terminal focus requires Accessibility permission. Enable the terminal app that launched EchoIsland in System Settings > Privacy & Security > Accessibility."
            );
        }
        return;
    }

    if is_automation_permission_error(stderr)
        && !DID_WARN_AUTOMATION_PERMISSION.swap(true, Ordering::Relaxed)
    {
        warn!(
            "macOS terminal focus requires Automation permission. Allow the launcher app to control System Events and your terminal app in System Settings > Privacy & Security > Automation."
        );
    }
}

fn is_accessibility_permission_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("(-25211)") || lower.contains("assistive access") || lower.contains("辅助访问")
}

fn is_automation_permission_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("(-1743)")
        || lower.contains("not authorized to send apple events")
        || lower.contains("apple events")
        || lower.contains("不允许发送 apple event")
        || lower.contains("不允许控制")
}

pub(super) fn escape_applescript(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}
