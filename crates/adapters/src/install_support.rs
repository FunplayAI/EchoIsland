use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Map, Value};

pub fn direct_bridge_command(bridge_path: &Path, source: &str) -> String {
    format!("\"{}\" --source {}", bridge_path.display(), source)
}

pub fn platform_bridge_command(bridge_path: &Path, source: &str) -> String {
    direct_bridge_command(bridge_path, source)
}

pub fn load_json_object(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let value = serde_json::from_str::<Value>(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(value.as_object().cloned().unwrap_or_default())
}

pub fn write_json_object(path: &Path, root: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let encoded =
        serde_json::to_vec_pretty(&Value::Object(root.clone())).context("failed to encode json")?;
    fs::write(path, encoded).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn entry_contains_echoisland(entry: &Value) -> bool {
    command_values(entry).into_iter().any(is_echoisland_command)
}

fn command_values(entry: &Value) -> Vec<&str> {
    let mut commands = Vec::new();
    if let Some(command) = entry.get("command").and_then(Value::as_str) {
        commands.push(command);
    }
    if let Some(command) = entry.get("bash").and_then(Value::as_str) {
        commands.push(command);
    }
    if let Some(hooks) = entry.get("hooks").and_then(Value::as_array) {
        for hook in hooks {
            if let Some(command) = hook.get("command").and_then(Value::as_str) {
                commands.push(command);
            }
            if let Some(command) = hook.get("bash").and_then(Value::as_str) {
                commands.push(command);
            }
        }
    }
    commands
}

fn is_echoisland_command(command: &str) -> bool {
    command.contains("codeisland-hook-bridge")
        || command.contains("echoisland-hook-bridge")
        || command.contains("codeisland-bridge")
        || command.contains("codeisland-hook.sh")
        || command.contains("vibenotch")
}

pub fn remove_echoisland_entries(hooks_obj: &mut Map<String, Value>) {
    for key in hooks_obj.clone().keys().cloned().collect::<Vec<_>>() {
        if let Some(entries) = hooks_obj.get_mut(&key).and_then(Value::as_array_mut) {
            entries.retain(|entry| !entry_contains_echoisland(entry));
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use echoisland_paths::bridge_binary_name;

    use super::{
        direct_bridge_command, entry_contains_echoisland, platform_bridge_command,
        remove_echoisland_entries,
    };

    #[test]
    fn builds_bridge_commands() {
        let path_string = format!("C:/CodeIsland/{}", bridge_binary_name());
        let path = std::path::Path::new(&path_string);
        let direct = direct_bridge_command(path, "codex");
        let platform = platform_bridge_command(path, "claude");

        assert!(direct.contains("--source codex"));
        assert!(platform.contains("--source claude"));
        assert!(!platform.contains("powershell.exe"));
    }

    #[test]
    fn removes_echoisland_entries_from_hook_map() {
        let bridge_command = format!("\"C:/CodeIsland/{}\" --source codex", bridge_binary_name());
        let mut hooks = serde_json::Map::from_iter([(
            "SessionStart".to_string(),
            json!([
              { "hooks": [{ "command": bridge_command }] },
              { "hooks": [{ "command": "echo hello" }] }
            ]),
        )]);

        remove_echoisland_entries(&mut hooks);
        let entries = hooks
            .get("SessionStart")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!entry_contains_echoisland(&entries[0]));
    }
}
