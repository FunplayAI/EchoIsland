use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Context;
use chrono::Utc;
use echoisland_core::{PROTOCOL_VERSION, ResponseEnvelope};
use echoisland_ipc::{DEFAULT_ADDR, send_raw};
use echoisland_paths::bridge_log_path as default_bridge_log_path;
use serde_json::{Map, Value, json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut source = "codex".to_string();
    while let Some(arg) = args.next() {
        if arg == "--source" {
            source = args.next().unwrap_or_else(|| "codex".to_string());
        }
    }

    use tokio::io::{self, AsyncReadExt};
    let mut stdin = io::stdin();
    let mut raw = Vec::new();
    stdin.read_to_end(&mut raw).await?;
    if raw.is_empty() {
        append_bridge_log("empty stdin; exiting");
        return Ok(());
    }

    let value = match serde_json::from_slice::<Value>(&raw) {
        Ok(value) => value,
        Err(error) => {
            append_bridge_log(&format!("failed to parse stdin json: {error}"));
            return Ok(());
        }
    };
    let mut obj = match value {
        Value::Object(obj) => obj,
        _ => {
            append_bridge_log("stdin json is not an object");
            return Ok(());
        }
    };

    let raw_event_name = obj
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    enrich_event(&mut obj, &source);

    if obj
        .get("session_id")
        .and_then(Value::as_str)
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        append_bridge_log("missing session_id after enrichment");
        return Ok(());
    }

    let event_name = obj
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let session_id = obj
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    append_bridge_log(&format!(
        "invoked source={source} event={event_name} session_id={session_id}"
    ));
    if let Some(metadata) = obj.get("metadata").and_then(Value::as_object) {
        append_bridge_log(&format!(
            "metadata source={source} event={event_name} session_id={session_id} terminal_app={:?} terminal_bundle={:?} tty={:?} pid={:?} cli_pid={:?}",
            metadata.get("terminal_app").and_then(Value::as_str),
            metadata.get("terminal_bundle").and_then(Value::as_str),
            metadata.get("tty").and_then(Value::as_str),
            metadata.get("pid").and_then(Value::as_u64),
            metadata.get("cli_pid").and_then(Value::as_u64),
        ));
    }

    let payload =
        serde_json::to_vec(&Value::Object(obj.clone())).context("failed to encode payload")?;
    let response = match send_raw(DEFAULT_ADDR, &payload).await {
        Ok(response) => {
            append_bridge_log(&format!(
                "sent to ipc source={source} event={event_name} session_id={session_id} ok={}",
                response.ok
            ));
            response
        }
        Err(error) => {
            append_bridge_log(&format!(
                "ipc send failed source={source} event={event_name} session_id={session_id}: {error}"
            ));
            fallback_response(&payload)
        }
    };

    let output = format_output(&source, &raw_event_name, &obj, &response);
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn enrich_event(obj: &mut Map<String, Value>, source: &str) {
    if source.eq_ignore_ascii_case("claude") {
        enrich_claude_event(obj);
    } else if source.eq_ignore_ascii_case("codex") {
        enrich_codex_event(obj);
    }
    obj.entry("protocol_version".to_string())
        .or_insert_with(|| Value::String(PROTOCOL_VERSION.to_string()));
    obj.insert("source".to_string(), Value::String(source.to_string()));
    obj.entry("timestamp".to_string())
        .or_insert_with(|| Value::String(Utc::now().to_rfc3339()));
    if obj.get("cwd").is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            obj.insert("cwd".to_string(), Value::String(cwd.display().to_string()));
        }
    }
    let metadata = obj
        .entry("metadata".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if let Some(metadata_obj) = metadata.as_object_mut() {
        metadata_obj
            .entry("pid".to_string())
            .or_insert_with(|| json!(std::process::id()));
        metadata_obj
            .entry("host_app".to_string())
            .or_insert_with(|| Value::String(source.to_string()));
        enrich_terminal_metadata(metadata_obj);
    }
}

fn enrich_terminal_metadata(metadata_obj: &mut Map<String, Value>) {
    for (key, value) in collect_terminal_metadata() {
        metadata_obj.entry(key).or_insert(value);
    }
}

fn collect_terminal_metadata() -> Map<String, Value> {
    collect_terminal_metadata_with(
        &|name| env::var(name).ok().filter(|value| !value.trim().is_empty()),
        detect_tty_path(),
        current_parent_pid(),
    )
}

fn collect_terminal_metadata_with(
    env_get: &dyn Fn(&str) -> Option<String>,
    tty: Option<String>,
    parent_pid: Option<u32>,
) -> Map<String, Value> {
    let mut metadata = Map::new();

    if let Some(terminal_app) = env_get("TERM_PROGRAM") {
        metadata.insert("terminal_app".to_string(), Value::String(terminal_app));
    }
    if let Some(terminal_bundle) = env_get("__CFBundleIdentifier") {
        metadata.insert(
            "terminal_bundle".to_string(),
            Value::String(terminal_bundle),
        );
    }
    if let Some(iterm_session_id) = env_get("ITERM_SESSION_ID")
        .map(|value| parse_iterm_session_id(&value))
        .filter(|value| !value.is_empty())
    {
        metadata.insert(
            "iterm_session_id".to_string(),
            Value::String(iterm_session_id),
        );
    }
    if let Some(kitty_window_id) = env_get("KITTY_WINDOW_ID") {
        metadata.insert(
            "kitty_window_id".to_string(),
            Value::String(kitty_window_id),
        );
    }
    if let Some(tmux_env) = env_get("TMUX") {
        metadata.insert("tmux_env".to_string(), Value::String(tmux_env.clone()));
        if let Some(tmux_pane) = env_get("TMUX_PANE") {
            metadata.insert("tmux_pane".to_string(), Value::String(tmux_pane.clone()));
            if let Some(tmux_client_tty) = tmux_client_tty(&tmux_pane, Some(&tmux_env)) {
                metadata.insert(
                    "tmux_client_tty".to_string(),
                    Value::String(tmux_client_tty),
                );
            }
        }
    }
    let tty = tty
        .filter(|value| is_precise_tty_path(value))
        .or_else(|| parent_pid.and_then(detect_tty_path_from_pid));
    if let Some(tty) = tty.filter(|value| !value.is_empty()) {
        metadata.insert("tty".to_string(), Value::String(tty));
    }
    if let Some(parent_pid) = parent_pid {
        metadata.insert("cli_pid".to_string(), json!(parent_pid));
    }

    metadata
}

fn parse_iterm_session_id(raw: &str) -> String {
    raw.split_once(':')
        .map(|(_, value)| value)
        .unwrap_or(raw)
        .trim()
        .to_string()
}

fn find_binary(name: &str) -> Option<String> {
    [
        format!("/opt/homebrew/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
    ]
    .into_iter()
    .find(|path| std::fs::metadata(path).is_ok())
}

fn run_command(path: &str, args: &[&str], extra_env: Option<&[(&str, &str)]>) -> Option<String> {
    let mut command = Command::new(path);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(extra_env) = extra_env {
        command.envs(extra_env.iter().copied());
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!stdout.is_empty()).then_some(stdout)
}

fn tmux_client_tty(tmux_pane: &str, tmux_env: Option<&str>) -> Option<String> {
    let tmux_bin = find_binary("tmux")?;
    let mut env_pairs = Vec::new();
    if let Some(tmux_env) = tmux_env.filter(|value| !value.trim().is_empty()) {
        env_pairs.push(("TMUX", tmux_env));
    }
    run_command(
        &tmux_bin,
        &[
            "display-message",
            "-p",
            "-t",
            tmux_pane,
            "-F",
            "#{client_tty}",
        ],
        (!env_pairs.is_empty()).then_some(env_pairs.as_slice()),
    )
}

#[cfg(unix)]
fn detect_tty_path() -> Option<String> {
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        let tty_name = unsafe { libc::ttyname(fd) };
        if tty_name.is_null() {
            continue;
        }
        let value = unsafe { std::ffi::CStr::from_ptr(tty_name) }
            .to_string_lossy()
            .trim()
            .to_string();
        if is_precise_tty_path(&value) {
            return Some(value);
        }
    }

    let path = b"/dev/tty\0";
    let fd = unsafe { libc::open(path.as_ptr().cast(), libc::O_RDONLY | libc::O_NOCTTY) };
    if fd < 0 {
        return None;
    }
    let tty_name = unsafe { libc::ttyname(fd) };
    let _ = unsafe { libc::close(fd) };
    if tty_name.is_null() {
        return None;
    }
    let value = unsafe { std::ffi::CStr::from_ptr(tty_name) }
        .to_string_lossy()
        .trim()
        .to_string();
    is_precise_tty_path(&value).then_some(value)
}

#[cfg(not(unix))]
fn detect_tty_path() -> Option<String> {
    None
}

#[cfg(unix)]
fn current_parent_pid() -> Option<u32> {
    let pid = unsafe { libc::getppid() };
    (pid > 0).then_some(pid as u32)
}

#[cfg(not(unix))]
fn current_parent_pid() -> Option<u32> {
    None
}

fn detect_tty_path_from_pid(pid: u32) -> Option<String> {
    let pid_text = pid.to_string();
    let tty = run_command("/bin/ps", &["-p", &pid_text, "-o", "tty="], None)?
        .trim()
        .to_string();
    if tty.is_empty() || tty == "??" {
        return None;
    }
    let tty_path = if tty.starts_with("/dev/") {
        tty
    } else {
        format!("/dev/{tty}")
    };
    is_precise_tty_path(&tty_path).then_some(tty_path)
}

fn is_precise_tty_path(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value != "/dev/tty" && value != "/dev/console"
}

fn enrich_codex_event(obj: &mut Map<String, Value>) {
    if obj.get("message").is_none() {
        if let Some(message) = obj
            .get("prompt")
            .and_then(Value::as_str)
            .or_else(|| obj.get("last_assistant_message").and_then(Value::as_str))
            .or_else(|| {
                obj.get("tool_input")
                    .and_then(|tool| tool.get("command"))
                    .and_then(Value::as_str)
            })
        {
            obj.insert("message".to_string(), Value::String(message.to_string()));
        }
    }
}

fn enrich_claude_event(obj: &mut Map<String, Value>) {
    if let Some(event_name) = obj.get("hook_event_name").and_then(Value::as_str) {
        let normalized = match event_name {
            "Elicitation" => "AskUserQuestion",
            "TaskCompleted" => "Notification",
            "StopFailure" => "Notification",
            other => other,
        };
        obj.insert(
            "hook_event_name".to_string(),
            Value::String(normalized.to_string()),
        );
    }

    if obj.get("message").is_none() {
        if let Some(message) = obj
            .get("last_assistant_message")
            .and_then(Value::as_str)
            .or_else(|| obj.get("task_subject").and_then(Value::as_str))
            .or_else(|| obj.get("prompt").and_then(Value::as_str))
            .or_else(|| obj.get("question").and_then(Value::as_str))
            .or_else(|| {
                obj.get("tool_input")
                    .and_then(|tool| tool.get("description"))
                    .and_then(Value::as_str)
            })
        {
            obj.insert("message".to_string(), Value::String(message.to_string()));
        }
    }

    if obj.get("question").is_none()
        && obj
            .get("hook_event_name")
            .and_then(Value::as_str)
            .map(|value| value == "AskUserQuestion")
            .unwrap_or(false)
    {
        if let Some(question) = build_claude_question_payload(obj) {
            obj.insert("question".to_string(), question);
        }
    }
}

fn build_claude_question_payload(obj: &Map<String, Value>) -> Option<Value> {
    let requested_schema = obj.get("requested_schema").and_then(Value::as_object);
    let field_specs = requested_schema
        .and_then(parse_elicitation_fields)
        .unwrap_or_default();

    let header = obj
        .get("mcp_server_name")
        .and_then(Value::as_str)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null);

    let mode = obj.get("mode").and_then(Value::as_str).unwrap_or("form");
    let base_message = obj
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| obj.get("question").and_then(Value::as_str))
        .unwrap_or("Question");

    let text = match mode {
        "url" => {
            let url = obj.get("url").and_then(Value::as_str).unwrap_or("");
            if url.is_empty() {
                base_message.to_string()
            } else {
                format!("{base_message}\nOpen URL: {url}")
            }
        }
        _ => {
            let guidance = elicitation_guidance(&field_specs);
            if guidance.is_empty() {
                base_message.to_string()
            } else {
                format!("{base_message}\n{guidance}")
            }
        }
    };

    let options = build_elicitation_options(&field_specs);

    Some(json!({
        "header": header,
        "text": text,
        "options": options,
    }))
}

fn format_output(
    source: &str,
    raw_event_name: &str,
    request: &Map<String, Value>,
    response: &ResponseEnvelope,
) -> Value {
    if source.eq_ignore_ascii_case("claude") {
        return format_claude_output(raw_event_name, request, response);
    }
    if source.eq_ignore_ascii_case("codex") {
        return format_codex_output(raw_event_name, response);
    }

    serde_json::to_value(response)
        .unwrap_or_else(|_| json!({ "ok": false, "error": "encode_failed" }))
}

fn format_codex_output(raw_event_name: &str, _response: &ResponseEnvelope) -> Value {
    match raw_event_name {
        "Stop" | "SessionStart" | "UserPromptSubmit" | "PreToolUse" | "PostToolUse" => {
            Value::Object(Map::new())
        }
        _ => Value::Object(Map::new()),
    }
}

fn format_claude_output(
    raw_event_name: &str,
    request: &Map<String, Value>,
    response: &ResponseEnvelope,
) -> Value {
    match raw_event_name {
        "PermissionRequest" => {
            let behavior = response
                .decision
                .as_ref()
                .map(|decision| decision.behavior.clone())
                .unwrap_or_else(|| "deny".to_string());
            let denied = behavior == "deny";

            let mut decision = Map::new();
            decision.insert("behavior".to_string(), Value::String(behavior));
            if denied {
                decision.insert(
                    "message".to_string(),
                    Value::String("Denied by EchoIsland approval workflow".to_string()),
                );
                decision.insert("interrupt".to_string(), Value::Bool(false));
            }
            if let Some(updated_permissions) = response
                .decision
                .as_ref()
                .and_then(|decision| decision.updated_permissions.clone())
            {
                decision.insert("updatedPermissions".to_string(), updated_permissions);
            }

            json!({
                "hookSpecificOutput": {
                    "hookEventName": "PermissionRequest",
                    "decision": Value::Object(decision),
                }
            })
        }
        "Elicitation" => {
            if response
                .answer
                .as_ref()
                .map(|answer| answer.skipped)
                .unwrap_or(false)
            {
                return json!({
                    "hookSpecificOutput": {
                        "hookEventName": "Elicitation",
                        "action": "cancel"
                    }
                });
            }

            let answer = response
                .answer
                .as_ref()
                .and_then(|answer| answer.value.clone());
            if let Some(value) = answer {
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "Elicitation",
                        "action": "accept",
                        "content": build_elicitation_content(request, &value),
                    }
                })
            } else {
                json!({
                    "hookSpecificOutput": {
                        "hookEventName": "Elicitation",
                        "action": "cancel"
                    }
                })
            }
        }
        _ => json!({}),
    }
}

fn build_elicitation_content(request: &Map<String, Value>, answer: &str) -> Value {
    let fields = request
        .get("requested_schema")
        .and_then(Value::as_object)
        .and_then(parse_elicitation_fields)
        .unwrap_or_default();

    if let Ok(value) = serde_json::from_str::<Value>(answer) {
        if value.is_object() {
            return value;
        }
    }

    if fields.len() == 1 {
        return json!({
            fields[0].name.clone(): answer,
        });
    }

    json!({
        "value": answer,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ElicitationField {
    name: String,
    label: String,
    description: Option<String>,
    enum_values: Vec<String>,
}

fn parse_elicitation_fields(schema: &Map<String, Value>) -> Option<Vec<ElicitationField>> {
    let properties = schema.get("properties")?.as_object()?;
    let mut fields = Vec::new();

    for (name, value) in properties {
        let field = value.as_object()?;
        let label = field
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(name)
            .to_string();
        let description = field
            .get("description")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let enum_values = field
            .get("enum")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        fields.push(ElicitationField {
            name: name.clone(),
            label,
            description,
            enum_values,
        });
    }

    fields.sort_by(|left, right| left.name.cmp(&right.name));

    Some(fields)
}

fn elicitation_guidance(fields: &[ElicitationField]) -> String {
    if fields.is_empty() {
        return String::new();
    }

    if fields.len() == 1 {
        let field = &fields[0];
        if !field.enum_values.is_empty() {
            return format!(
                "Choose one value for {}: {}",
                field.label,
                field.enum_values.join(", ")
            );
        }
        return format!("Provide a value for {}.", field.label);
    }

    let names = fields
        .iter()
        .map(|field| format!("\"{}\"", field.name))
        .collect::<Vec<_>>()
        .join(", ");
    format!("Reply with JSON containing fields: {names}")
}

fn build_elicitation_options(fields: &[ElicitationField]) -> Vec<Value> {
    if fields.len() != 1 || fields[0].enum_values.is_empty() {
        return Vec::new();
    }

    let field = &fields[0];
    field
        .enum_values
        .iter()
        .map(|value| {
            json!({
                "label": value,
                "description": field.description.clone(),
            })
        })
        .collect()
}

fn fallback_response(payload: &[u8]) -> ResponseEnvelope {
    let event_name = serde_json::from_slice::<Value>(payload)
        .ok()
        .and_then(|value| {
            value
                .get("hook_event_name")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();

    match event_name.as_str() {
        "PermissionRequest" => ResponseEnvelope::deny(),
        "AskUserQuestion" => ResponseEnvelope::skipped(),
        _ => ResponseEnvelope::ok(),
    }
}

fn append_bridge_log(message: &str) {
    let Some(path) = bridge_log_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "[{}] {}", Utc::now().to_rfc3339(), message);
    }
}

fn bridge_log_path() -> Option<PathBuf> {
    Some(default_bridge_log_path())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::PathBuf};

    use echoisland_core::{EventEnvelope, ResponseEnvelope, protocol::DecisionResponse};
    use serde_json::{Map, Value, json};

    use super::{
        build_claude_question_payload, collect_terminal_metadata_with, enrich_event, format_output,
    };

    #[test]
    fn codex_prompt_is_copied_to_message() {
        let mut event = Map::new();
        event.insert(
            "hook_event_name".to_string(),
            Value::String("UserPromptSubmit".to_string()),
        );
        event.insert(
            "prompt".to_string(),
            Value::String("Refactor the session scanner".to_string()),
        );

        enrich_event(&mut event, "codex");

        assert_eq!(
            event.get("message").and_then(Value::as_str),
            Some("Refactor the session scanner")
        );
        assert_eq!(event.get("source").and_then(Value::as_str), Some("codex"));
    }

    #[test]
    fn codex_stop_uses_last_assistant_message_as_message() {
        let mut event = Map::new();
        event.insert(
            "hook_event_name".to_string(),
            Value::String("Stop".to_string()),
        );
        event.insert(
            "last_assistant_message".to_string(),
            Value::String("Done, the queue logic is stable.".to_string()),
        );

        enrich_event(&mut event, "codex");

        assert_eq!(
            event.get("message").and_then(Value::as_str),
            Some("Done, the queue logic is stable.")
        );
    }

    #[test]
    fn codex_session_start_source_is_overridden_to_adapter_source() {
        let mut event = Map::new();
        event.insert(
            "hook_event_name".to_string(),
            Value::String("SessionStart".to_string()),
        );
        event.insert("source".to_string(), Value::String("startup".to_string()));

        enrich_event(&mut event, "codex");

        assert_eq!(event.get("source").and_then(Value::as_str), Some("codex"));
    }

    #[test]
    fn terminal_metadata_is_collected_from_environment() {
        let env = HashMap::from([
            ("TERM_PROGRAM", "Warp".to_string()),
            ("__CFBundleIdentifier", "dev.warp.Warp-Stable".to_string()),
            ("ITERM_SESSION_ID", "w0t0p0:ABC-123".to_string()),
            ("KITTY_WINDOW_ID", "42".to_string()),
            ("TMUX", "/tmp/tmux-501/default,123,0".to_string()),
            ("TMUX_PANE", "%7".to_string()),
        ]);
        let metadata = collect_terminal_metadata_with(
            &|name| env.get(name).cloned(),
            Some("/dev/ttys001".to_string()),
            Some(777),
        );

        assert_eq!(
            metadata.get("terminal_app").and_then(Value::as_str),
            Some("Warp")
        );
        assert_eq!(
            metadata.get("terminal_bundle").and_then(Value::as_str),
            Some("dev.warp.Warp-Stable")
        );
        assert_eq!(
            metadata.get("iterm_session_id").and_then(Value::as_str),
            Some("ABC-123")
        );
        assert_eq!(
            metadata.get("kitty_window_id").and_then(Value::as_str),
            Some("42")
        );
        assert_eq!(
            metadata.get("tty").and_then(Value::as_str),
            Some("/dev/ttys001")
        );
        assert_eq!(metadata.get("cli_pid").and_then(Value::as_u64), Some(777));
    }

    #[test]
    fn generic_dev_tty_is_not_used_as_precise_terminal_tty() {
        let env = HashMap::from([("TERM_PROGRAM", "Apple_Terminal".to_string())]);
        let metadata = collect_terminal_metadata_with(
            &|name| env.get(name).cloned(),
            Some("/dev/tty".to_string()),
            None,
        );

        assert_eq!(
            metadata.get("terminal_app").and_then(Value::as_str),
            Some("Apple_Terminal")
        );
        assert!(metadata.get("tty").is_none());
    }

    #[test]
    fn codex_output_uses_empty_json_object_shape() {
        let output = format_output("codex", "Stop", &Map::new(), &ResponseEnvelope::ok());
        assert_eq!(output, Value::Object(Map::new()));
    }

    fn sample_hooks_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("samples")
            .join("hooks")
    }

    fn load_hook_sample(name: &str) -> Map<String, Value> {
        let path = sample_hooks_dir().join(name);
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        serde_json::from_str::<Value>(&raw)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
            .as_object()
            .cloned()
            .unwrap_or_else(|| panic!("sample {} is not a json object", path.display()))
    }

    #[test]
    fn claude_elicitation_is_normalized_to_question() {
        let mut event = Map::new();
        event.insert(
            "hook_event_name".to_string(),
            Value::String("Elicitation".to_string()),
        );
        event.insert(
            "message".to_string(),
            Value::String("Choose deployment target".to_string()),
        );
        event.insert(
            "requested_schema".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "environment": { "type": "string", "title": "Environment" }
                }
            }),
        );

        enrich_event(&mut event, "claude");

        assert_eq!(
            event.get("hook_event_name").and_then(Value::as_str),
            Some("AskUserQuestion")
        );
        assert!(event.get("question").is_some());
    }

    #[test]
    fn claude_permission_output_uses_hook_specific_shape() {
        let response = ResponseEnvelope {
            ok: true,
            error: None,
            decision: Some(DecisionResponse {
                behavior: "allow".to_string(),
                updated_permissions: None,
            }),
            answer: None,
        };

        let output = format_output("claude", "PermissionRequest", &Map::new(), &response);
        assert_eq!(
            output
                .get("hookSpecificOutput")
                .and_then(|value| value.get("decision"))
                .and_then(|value| value.get("behavior"))
                .and_then(Value::as_str),
            Some("allow")
        );
    }

    #[test]
    fn claude_elicitation_output_wraps_single_answer() {
        let mut request = Map::new();
        request.insert(
            "requested_schema".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "username": { "type": "string" }
                }
            }),
        );

        let output = format_output(
            "claude",
            "Elicitation",
            &request,
            &ResponseEnvelope::answer("alice"),
        );

        assert_eq!(
            output
                .get("hookSpecificOutput")
                .and_then(|value| value.get("content"))
                .and_then(|value| value.get("username"))
                .and_then(Value::as_str),
            Some("alice")
        );
    }

    #[test]
    fn claude_question_payload_uses_requested_schema_titles() {
        let event = json!({
            "message": "Provide credentials",
            "requested_schema": {
                "type": "object",
                "properties": {
                    "username": { "title": "Username" },
                    "password": { "description": "Password" }
                }
            }
        });
        let payload = build_claude_question_payload(event.as_object().unwrap()).unwrap();
        assert_eq!(
            payload
                .get("options")
                .and_then(Value::as_array)
                .map(|values| values.len()),
            Some(0)
        );
        let text = payload
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(text.starts_with("Provide credentials\nReply with JSON containing fields:"));
        assert!(text.contains("\"username\""));
        assert!(text.contains("\"password\""));
    }

    #[test]
    fn claude_elicitation_payload_uses_enum_as_choices() {
        let event = json!({
            "message": "Choose environment",
            "requested_schema": {
                "type": "object",
                "properties": {
                    "environment": {
                        "title": "Environment",
                        "enum": ["staging", "production"]
                    }
                }
            }
        });
        let payload = build_claude_question_payload(event.as_object().unwrap()).unwrap();
        assert_eq!(
            payload
                .get("options")
                .and_then(Value::as_array)
                .map(|values| values.len()),
            Some(2)
        );
    }

    #[test]
    fn claude_stop_uses_last_assistant_message_as_message() {
        let mut event = Map::new();
        event.insert(
            "hook_event_name".to_string(),
            Value::String("Stop".to_string()),
        );
        event.insert(
            "last_assistant_message".to_string(),
            Value::String("Done, tests are green.".to_string()),
        );

        enrich_event(&mut event, "claude");

        assert_eq!(
            event.get("message").and_then(Value::as_str),
            Some("Done, tests are green.")
        );
    }

    #[test]
    fn claude_task_completed_becomes_notification() {
        let mut event = Map::new();
        event.insert(
            "hook_event_name".to_string(),
            Value::String("TaskCompleted".to_string()),
        );
        event.insert(
            "task_subject".to_string(),
            Value::String("Implement login".to_string()),
        );

        enrich_event(&mut event, "claude");

        assert_eq!(
            event.get("hook_event_name").and_then(Value::as_str),
            Some("Notification")
        );
        assert_eq!(
            event.get("message").and_then(Value::as_str),
            Some("Implement login")
        );
    }

    #[test]
    fn claude_elicitation_multi_field_accepts_json_answer() {
        let mut request = Map::new();
        request.insert(
            "requested_schema".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "username": { "type": "string" },
                    "password": { "type": "string" }
                }
            }),
        );

        let output = format_output(
            "claude",
            "Elicitation",
            &request,
            &ResponseEnvelope::answer("{\"username\":\"alice\",\"password\":\"secret\"}"),
        );

        assert_eq!(
            output
                .get("hookSpecificOutput")
                .and_then(|value| value.get("content"))
                .and_then(|value| value.get("password"))
                .and_then(Value::as_str),
            Some("secret")
        );
    }

    #[test]
    fn raw_claude_hook_samples_enrich_to_valid_events() {
        for name in [
            "claude_permission_request_hook.json",
            "claude_elicitation_enum_hook.json",
            "claude_elicitation_form_hook.json",
            "claude_task_completed_hook.json",
            "claude_stop_hook.json",
        ] {
            let mut event = load_hook_sample(name);
            enrich_event(&mut event, "claude");

            let envelope: EventEnvelope = serde_json::from_value(Value::Object(event))
                .unwrap_or_else(|error| panic!("failed to decode enriched sample {name}: {error}"));
            envelope
                .validate()
                .unwrap_or_else(|error| panic!("invalid enriched sample {name}: {error}"));
        }
    }

    #[test]
    fn raw_claude_form_sample_can_round_trip_json_answer() {
        let request = load_hook_sample("claude_elicitation_form_hook.json");
        let output = format_output(
            "claude",
            "Elicitation",
            &request,
            &ResponseEnvelope::answer("{\"username\":\"alice\",\"password\":\"secret\"}"),
        );

        assert_eq!(
            output
                .get("hookSpecificOutput")
                .and_then(|value| value.get("content"))
                .and_then(|value| value.get("username"))
                .and_then(Value::as_str),
            Some("alice")
        );
    }
}
