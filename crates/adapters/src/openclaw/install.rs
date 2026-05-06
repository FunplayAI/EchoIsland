use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

use super::{OpenClawPaths, OpenClawStatus};
use crate::install_support::{load_json_object, write_json_object};
use crate::platform_support::supported_with_note;

pub const DEFAULT_OPENCLAW_RECEIVER_URL: &str = "http://127.0.0.1:37892/event";
const OPENCLAW_HOOK_ID: &str = "echoisland";

pub fn install_openclaw_adapter(paths: &OpenClawPaths) -> Result<OpenClawStatus> {
    fs::create_dir_all(&paths.openclaw_dir)
        .with_context(|| format!("failed to create {}", paths.openclaw_dir.display()))?;
    fs::create_dir_all(&paths.hook_dir)
        .with_context(|| format!("failed to create {}", paths.hook_dir.display()))?;

    fs::write(&paths.hook_manifest_path, render_hook_manifest())
        .with_context(|| format!("failed to write {}", paths.hook_manifest_path.display()))?;
    fs::write(&paths.hook_handler_path, render_hook_handler(paths)?)
        .with_context(|| format!("failed to write {}", paths.hook_handler_path.display()))?;

    ensure_hook_enabled(paths)?;
    get_openclaw_status(paths)
}

pub fn get_openclaw_status(paths: &OpenClawPaths) -> Result<OpenClawStatus> {
    let hook_installed =
        paths.hook_manifest_path.exists() && hook_has_echoisland_marker(&paths.hook_handler_path)?;
    let hook_enabled = hook_enabled(paths).unwrap_or(false);
    let token_exists = paths.token_path.exists();
    let support = supported_with_note(
        "OpenClaw support currently uses a managed internal hook pack for session/message live capture. Tool approval hooks can be added later through the richer plugin SDK.",
    );

    Ok(OpenClawStatus {
        openclaw_dir_exists: paths.openclaw_dir.exists(),
        config_path_exists: paths.config_path.exists(),
        hooks_dir_exists: paths.hooks_dir.exists(),
        hook_installed,
        hook_enabled,
        token_exists,
        live_capture_supported: support.supported,
        live_capture_ready: hook_installed && hook_enabled && token_exists,
        status_note: support.note,
        openclaw_dir: paths.openclaw_dir.display().to_string(),
        config_path: paths.config_path.display().to_string(),
        hooks_dir: paths.hooks_dir.display().to_string(),
        hook_dir: paths.hook_dir.display().to_string(),
        hook_manifest_path: paths.hook_manifest_path.display().to_string(),
        hook_handler_path: paths.hook_handler_path.display().to_string(),
        token_path: paths.token_path.display().to_string(),
        receiver_url: paths.receiver_url.clone(),
    })
}

fn hook_has_echoisland_marker(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(raw.contains("echoisland-openclaw-hook"))
}

fn hook_enabled(paths: &OpenClawPaths) -> Result<bool> {
    if !paths.config_path.exists() {
        return Ok(false);
    }
    let root = load_json_object(&paths.config_path)?;
    Ok(root
        .get("hooks")
        .and_then(Value::as_object)
        .and_then(|hooks| hooks.get("internal"))
        .and_then(Value::as_object)
        .and_then(|internal| internal.get("entries"))
        .and_then(Value::as_object)
        .and_then(|entries| entries.get(OPENCLAW_HOOK_ID))
        .and_then(Value::as_object)
        .and_then(|entry| entry.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

fn ensure_hook_enabled(paths: &OpenClawPaths) -> Result<()> {
    let mut root = if paths.config_path.exists() {
        load_json_object(&paths.config_path)?
    } else {
        Map::new()
    };
    let hooks = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks_obj = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("openclaw hooks config must be an object"))?;
    let internal = hooks_obj
        .entry("internal".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let internal_obj = internal
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("openclaw hooks.internal config must be an object"))?;
    let entries = internal_obj
        .entry("entries".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let entries_obj = entries.as_object_mut().ok_or_else(|| {
        anyhow::anyhow!("openclaw hooks.internal.entries config must be an object")
    })?;

    entries_obj.insert(OPENCLAW_HOOK_ID.to_string(), json!({ "enabled": true }));

    write_json_object(&paths.config_path, &root)
}

fn render_hook_manifest() -> &'static str {
    r#"---
metadata:
  openclaw:
    name: echoisland
    description: Forward OpenClaw session activity to EchoIsland.
    events:
      - command:new
      - command:reset
      - command:stop
      - message:received
      - message:sent
      - session:patch
---

# EchoIsland OpenClaw Hook

Managed by EchoIsland. Forwards OpenClaw session/message events to the local EchoIsland receiver.
"#
}

fn render_hook_handler(paths: &OpenClawPaths) -> Result<String> {
    let receiver_url = serde_json::to_string(&paths.receiver_url)?;
    let token_path = serde_json::to_string(&paths.token_path.display().to_string())?;

    Ok(format!(
        r#"// echoisland-openclaw-hook
import {{ readFile }} from "node:fs/promises";

const RECEIVER_URL = {receiver_url};
const TOKEN_PATH = {token_path};

let cachedToken = null;

function getPath(root, path) {{
  let current = root;
  for (const key of path) {{
    if (current == null) return undefined;
    current = current[key];
  }}
  return current;
}}

function firstDefined(values) {{
  for (const value of values) {{
    if (value == null) continue;
    if (typeof value === "string" && value.trim() === "") continue;
    return value;
  }}
  return undefined;
}}

function textValue(value) {{
  if (value == null) return undefined;
  if (typeof value === "string") {{
    const trimmed = value.trim();
    return trimmed || undefined;
  }}
  if (Array.isArray(value)) {{
    return firstDefined(value.map(textValue));
  }}
  if (typeof value === "object") {{
    return firstDefined([
      textValue(value.content),
      textValue(value.text),
      textValue(value.message),
      textValue(value.title),
    ]);
  }}
  return undefined;
}}

function eventNameFor(action) {{
  switch (String(action ?? "").toLowerCase()) {{
    case "command:new":
    case "command:reset":
    case "session:patch":
      return "SessionStart";
    case "message:received":
      return "UserPromptSubmit";
    case "message:sent":
      return "AfterAgentResponse";
    case "command:stop":
      return "Stop";
    default:
      return null;
  }}
}}

function buildEnvelope(input) {{
  const action = String(input?.action ?? input?.type ?? "").toLowerCase();
  const hookEventName = eventNameFor(action);
  if (!hookEventName) return null;

  const sessionId = textValue(
    firstDefined([
      input?.sessionKey,
      getPath(input, ["context", "sessionKey"]),
      getPath(input, ["context", "sessionEntry", "sessionKey"]),
      getPath(input, ["context", "sessionEntry", "id"]),
    ]),
  );
  if (!sessionId) return null;

  const cwd = textValue(
    firstDefined([
      getPath(input, ["context", "sessionEntry", "workspaceDir"]),
      getPath(input, ["context", "workspaceDir"]),
      getPath(input, ["context", "patch", "workspaceDir"]),
    ]),
  );

  return {{
    protocol_version: "1",
    hook_event_name: hookEventName,
    source: "openclaw",
    session_id: sessionId,
    timestamp: new Date().toISOString(),
    cwd,
    message: textValue(
      firstDefined([
        getPath(input, ["context", "content"]),
        getPath(input, ["context", "message"]),
        getPath(input, ["context", "patch", "title"]),
      ]),
    ),
    metadata: {{
      terminal_app: "openclaw",
      host_app: "cli",
      window_title: "OpenClaw",
    }},
  }};
}}

async function token() {{
  if (cachedToken) return cachedToken;
  cachedToken = (await readFile(TOKEN_PATH, "utf8")).trim();
  return cachedToken;
}}

export default async function handler(input) {{
  const envelope = buildEnvelope(input);
  if (!envelope) return;

  try {{
    const authToken = await token();
    if (!authToken) return;

    await fetch(RECEIVER_URL, {{
      method: "POST",
      headers: {{
        "content-type": "application/json",
        "x-echoisland-token": authToken,
      }},
      body: JSON.stringify({{ event: envelope }}),
    }});
  }} catch (_error) {{
  }}
}}
"#
    ))
}
