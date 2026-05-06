use super::{
    SessionFocusTarget,
    osascript::{escape_applescript, run_osascript, run_osascript_raw},
    util::run_command_success,
};

pub(super) fn native_app_bundle_for_terminal(
    target: &SessionFocusTarget,
) -> Option<(&'static str, &'static str)> {
    target
        .terminal_bundle
        .as_deref()
        .and_then(native_app_bundle_display_name)
}

fn native_app_bundle_display_name(bundle_id: &str) -> Option<(&'static str, &'static str)> {
    match bundle_id {
        "com.openai.codex" => Some(("com.openai.codex", "Codex")),
        "com.trae.app" => Some(("com.trae.app", "Trae")),
        "com.qoder.ide" => Some(("com.qoder.ide", "Qoder")),
        "com.factory.app" => Some(("com.factory.app", "Factory")),
        "com.tencent.codebuddy" => Some(("com.tencent.codebuddy", "CodeBuddy")),
        "com.tencent.codebuddy.cn" => Some(("com.tencent.codebuddy.cn", "CodyBuddyCN")),
        "com.stepfun.app" => Some(("com.stepfun.app", "StepFun")),
        "ai.opencode.desktop" => Some(("ai.opencode.desktop", "OpenCode")),
        _ => None,
    }
}

pub(super) fn source_native_app_bundle(source: &str) -> Option<(&'static str, &'static str)> {
    match source {
        "codex" => Some(("com.openai.codex", "Codex")),
        "trae" | "traecn" => Some(("com.trae.app", "Trae")),
        "qoder" => Some(("com.qoder.ide", "Qoder")),
        "droid" => Some(("com.factory.app", "Factory")),
        "codebuddy" => Some(("com.tencent.codebuddy", "CodeBuddy")),
        "codybuddycn" => Some(("com.tencent.codebuddy.cn", "CodyBuddyCN")),
        "stepfun" => Some(("com.stepfun.app", "StepFun")),
        "opencode" => Some(("ai.opencode.desktop", "OpenCode")),
        _ => None,
    }
}

pub(super) fn activate_app_bundle(bundle_id: &str, display_name: &str) -> bool {
    crate::diagnostics::log_diagnostic_event(
        "macos_activate_app_bundle_begin",
        &[
            ("bundle_id", bundle_id.to_string()),
            ("display_name", display_name.to_string()),
        ],
    );
    if activate_running_app_bundle(bundle_id, display_name) {
        crate::diagnostics::log_diagnostic_event(
            "macos_activate_app_bundle_running_process_hit",
            &[
                ("bundle_id", bundle_id.to_string()),
                ("display_name", display_name.to_string()),
            ],
        );
        return true;
    }
    let script = format!(
        r#"try
tell application id "{bundle_id}" to activate
on error
    tell application "{display_name}" to activate
end try
"#,
        bundle_id = escape_applescript(bundle_id),
        display_name = escape_applescript(display_name)
    );
    crate::diagnostics::log_diagnostic_event(
        "macos_activate_app_bundle_osascript_begin",
        &[
            ("bundle_id", bundle_id.to_string()),
            ("display_name", display_name.to_string()),
        ],
    );
    let osascript_activated = run_osascript(&script, display_name);
    crate::diagnostics::log_diagnostic_event(
        "macos_activate_app_bundle_osascript_complete",
        &[
            ("bundle_id", bundle_id.to_string()),
            ("display_name", display_name.to_string()),
            ("activated", osascript_activated.to_string()),
        ],
    );
    let fallback_activated = if osascript_activated {
        false
    } else {
        crate::diagnostics::log_diagnostic_event(
            "macos_activate_app_bundle_open_fallback_begin",
            &[
                ("bundle_id", bundle_id.to_string()),
                ("display_name", display_name.to_string()),
                ("path", "/usr/bin/open".to_string()),
                ("args", format!("-b {bundle_id}")),
            ],
        );
        let activated = run_command_success("/usr/bin/open", &["-b", bundle_id], None);
        crate::diagnostics::log_diagnostic_event(
            "macos_activate_app_bundle_open_fallback_complete",
            &[
                ("bundle_id", bundle_id.to_string()),
                ("display_name", display_name.to_string()),
                ("activated", activated.to_string()),
            ],
        );
        activated
    };
    let activated = osascript_activated || fallback_activated;
    crate::diagnostics::log_diagnostic_event(
        "macos_activate_app_bundle_complete",
        &[
            ("bundle_id", bundle_id.to_string()),
            ("display_name", display_name.to_string()),
            ("activated", activated.to_string()),
            (
                "activation_stage",
                if osascript_activated {
                    "osascript"
                } else if fallback_activated {
                    "open_fallback"
                } else {
                    "failed"
                }
                .to_string(),
            ),
        ],
    );
    activated
}

pub(super) fn activate_running_app_bundle(bundle_id: &str, display_name: &str) -> bool {
    let script = format!(
        r#"
tell application "System Events"
    repeat with appProcess in application processes
        try
            if bundle identifier of appProcess is "{bundle_id}" then
                set frontmost of appProcess to true
                return true
            end if
        end try
    end repeat
end tell
return false
"#,
        bundle_id = escape_applescript(bundle_id)
    );
    let activated = run_osascript_raw(&script, display_name)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"));
    crate::diagnostics::log_diagnostic_event(
        "macos_activate_running_app_bundle_complete",
        &[
            ("bundle_id", bundle_id.to_string()),
            ("display_name", display_name.to_string()),
            ("activated", activated.to_string()),
        ],
    );
    activated
}

pub(super) fn activate_running_app_process(display_name: &str) -> bool {
    let script = format!(
        r#"
tell application "System Events"
    if exists process "{display_name}" then
        tell process "{display_name}" to set frontmost to true
        return true
    end if
end tell
return false
"#,
        display_name = escape_applescript(display_name)
    );
    let activated = run_osascript_raw(&script, display_name)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("true"));
    crate::diagnostics::log_diagnostic_event(
        "macos_activate_running_app_process_complete",
        &[
            ("display_name", display_name.to_string()),
            ("activated", activated.to_string()),
        ],
    );
    activated
}
