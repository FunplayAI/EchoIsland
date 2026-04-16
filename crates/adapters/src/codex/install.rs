use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

use super::{CodexPaths, CodexStatus};
use crate::install_support::{
    load_json_object, platform_bridge_command, remove_echoisland_entries, write_json_object,
};
use crate::platform_support::codex_live_capture_support;

const CODEX_EVENTS: [(&str, u64); 5] = [
    ("SessionStart", 5),
    ("UserPromptSubmit", 5),
    ("PreToolUse", 5),
    ("PostToolUse", 5),
    ("Stop", 5),
];

pub fn install_codex_adapter(paths: &CodexPaths, source_bridge: &Path) -> Result<CodexStatus> {
    if !paths.codex_dir.exists() {
        return Ok(status(paths, false, false, false));
    }

    fs::create_dir_all(&paths.bridge_install_dir)
        .with_context(|| format!("failed to create {}", paths.bridge_install_dir.display()))?;
    if source_bridge != paths.bridge_path {
        fs::copy(source_bridge, &paths.bridge_path).with_context(|| {
            format!(
                "failed to copy bridge {} -> {}",
                source_bridge.display(),
                paths.bridge_path.display()
            )
        })?;
    }

    install_hooks_json(paths)?;
    ensure_codex_hooks_enabled(paths)?;
    get_codex_status(paths)
}

pub fn get_codex_status(paths: &CodexPaths) -> Result<CodexStatus> {
    let bridge_exists = paths.bridge_path.exists();
    let codex_dir_exists = paths.codex_dir.exists();
    let codex_hooks_enabled = is_codex_hooks_enabled(paths)?;
    let hooks_installed = bridge_exists && hooks_have_echoisland_entries(paths)?;
    Ok(status(
        paths,
        codex_dir_exists,
        hooks_installed && bridge_exists,
        codex_hooks_enabled,
    ))
}

fn status(
    paths: &CodexPaths,
    codex_dir_exists: bool,
    hooks_installed: bool,
    codex_hooks_enabled: bool,
) -> CodexStatus {
    let support = codex_live_capture_support();
    CodexStatus {
        codex_dir_exists,
        bridge_exists: paths.bridge_path.exists(),
        hooks_installed,
        codex_hooks_enabled,
        live_capture_supported: support.supported,
        live_capture_ready: hooks_installed && codex_hooks_enabled && support.supported,
        status_note: support.note,
        codex_dir: paths.codex_dir.display().to_string(),
        hooks_path: paths.hooks_path.display().to_string(),
        config_path: paths.config_path.display().to_string(),
        bridge_path: paths.bridge_path.display().to_string(),
    }
}

fn install_hooks_json(paths: &CodexPaths) -> Result<()> {
    let mut root = load_json_object(&paths.hooks_path)?;
    let hooks = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks_obj = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("hooks.json top-level hooks must be an object"))?;

    let command = platform_bridge_command(&paths.bridge_path, "codex");
    remove_echoisland_entries(hooks_obj);

    for (event, timeout) in CODEX_EVENTS {
        let entry = json!({
            "hooks": [{
                "type": "command",
                "command": command.clone(),
                "timeout": timeout
            }]
        });

        let entries = hooks_obj
            .entry(event.to_string())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(|| anyhow::anyhow!("hook entries for {event} must be an array"))?;
        entries.push(entry);
    }

    write_json_object(&paths.hooks_path, &root)
}

fn ensure_codex_hooks_enabled(paths: &CodexPaths) -> Result<()> {
    let mut contents = if paths.config_path.exists() {
        fs::read_to_string(&paths.config_path)
            .with_context(|| format!("failed to read {}", paths.config_path.display()))?
    } else {
        String::new()
    };

    if contents
        .lines()
        .any(|line| line.trim_start().starts_with("codex_hooks = true"))
    {
        return Ok(());
    }

    if contents
        .lines()
        .any(|line| line.trim_start().starts_with("codex_hooks = false"))
    {
        contents = contents.replacen("codex_hooks = false", "codex_hooks = true", 1);
    } else if let Some(features_index) = contents.find("[features]") {
        let insert_at = contents[features_index..]
            .find('\n')
            .map(|offset| features_index + offset + 1)
            .unwrap_or(contents.len());
        contents.insert_str(insert_at, "codex_hooks = true\n");
    } else if contents.trim().is_empty() {
        contents = "[features]\ncodex_hooks = true\n".to_string();
    } else {
        contents.push_str("\n[features]\ncodex_hooks = true\n");
    }

    if let Some(parent) = paths.config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&paths.config_path, contents)
        .with_context(|| format!("failed to write {}", paths.config_path.display()))?;
    Ok(())
}

fn is_codex_hooks_enabled(paths: &CodexPaths) -> Result<bool> {
    if !paths.config_path.exists() {
        return Ok(false);
    }
    let contents = fs::read_to_string(&paths.config_path)
        .with_context(|| format!("failed to read {}", paths.config_path.display()))?;
    Ok(contents
        .lines()
        .any(|line| line.trim_start().starts_with("codex_hooks = true")))
}

fn hooks_have_echoisland_entries(paths: &CodexPaths) -> Result<bool> {
    if !paths.hooks_path.exists() {
        return Ok(false);
    }
    let root = load_json_object(&paths.hooks_path)?;
    let Some(hooks_obj) = root.get("hooks").and_then(Value::as_object) else {
        return Ok(false);
    };

    for (event, _) in CODEX_EVENTS {
        let Some(entries) = hooks_obj.get(event).and_then(Value::as_array) else {
            return Ok(false);
        };
        if !entries.iter().any(entry_contains_echoisland) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn entry_contains_echoisland(entry: &Value) -> bool {
    crate::install_support::entry_contains_echoisland(entry)
}
