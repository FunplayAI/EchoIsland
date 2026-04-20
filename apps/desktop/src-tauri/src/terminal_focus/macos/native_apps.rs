use super::{
    SessionFocusTarget,
    osascript::{escape_applescript, run_osascript},
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
    let opened = run_command_success("/usr/bin/open", &["-b", bundle_id], None);
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
    run_osascript(&script, display_name) || opened
}

pub(super) fn activate_app_name(display_name: &str) -> bool {
    let opened = run_command_success("/usr/bin/open", &["-a", display_name], None);
    let script = format!(
        r#"tell application "{display_name}" to activate
"#,
        display_name = escape_applescript(display_name)
    );
    run_osascript(&script, display_name) || opened
}
