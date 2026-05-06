#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

use super::{ClaudePaths, ClaudeStatus};
use crate::install_support::{load_json_object, remove_echoisland_entries, write_json_object};
use crate::platform_support::supported_with_note;

const CLAUDE_EVENTS: [(&str, u64); 13] = [
    ("SessionStart", 5),
    ("SessionEnd", 5),
    ("UserPromptSubmit", 5),
    ("PreToolUse", 5),
    ("PostToolUse", 5),
    ("PostToolUseFailure", 5),
    ("Stop", 5),
    ("PermissionRequest", 86_400),
    ("PermissionDenied", 5),
    ("Elicitation", 86_400),
    ("Notification", 86_400),
    ("SubagentStart", 5),
    ("SubagentStop", 5),
];

pub fn install_claude_adapter(paths: &ClaudePaths, source_bridge: &Path) -> Result<ClaudeStatus> {
    fs::create_dir_all(&paths.claude_dir)
        .with_context(|| format!("failed to create {}", paths.claude_dir.display()))?;
    if let Some(parent) = paths.hook_script_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
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
    install_hook_script(paths)?;

    install_settings_json(paths)?;
    get_claude_status(paths)
}

pub fn get_claude_status(paths: &ClaudePaths) -> Result<ClaudeStatus> {
    let bridge_exists = paths.bridge_path.exists();
    let hooks_installed = bridge_exists && hooks_have_echoisland_entries(paths)?;
    let support = supported_with_note(
        "Claude Code hooks are installed through ~/.claude/settings.json. Project-local settings may still override global behavior.",
    );
    Ok(ClaudeStatus {
        claude_dir_exists: paths.claude_dir.exists(),
        bridge_exists,
        hooks_installed,
        live_capture_supported: support.supported,
        live_capture_ready: hooks_installed,
        status_note: support.note,
        claude_dir: paths.claude_dir.display().to_string(),
        settings_path: paths.settings_path.display().to_string(),
        projects_dir: paths.projects_dir.display().to_string(),
        bridge_path: paths.bridge_path.display().to_string(),
    })
}

fn install_settings_json(paths: &ClaudePaths) -> Result<()> {
    let mut root = load_json_object(&paths.settings_path)?;
    let hooks = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks_obj = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("settings.json hooks must be an object"))?;

    let command = hook_script_command(paths);
    remove_echoisland_entries(hooks_obj);

    for (event, timeout) in CLAUDE_EVENTS {
        let entry = json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": command,
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

    write_json_object(&paths.settings_path, &root)
}

fn hook_script_command(paths: &ClaudePaths) -> String {
    let hook_script = bash_path_string(&paths.hook_script_path);
    let home_dir = bash_path_string(&paths.home_dir);
    if hook_script.starts_with(&home_dir) {
        hook_script.replacen(&home_dir, "~", 1)
    } else {
        hook_script
    }
}

fn install_hook_script(paths: &ClaudePaths) -> Result<()> {
    let script = render_hook_script(paths);
    fs::write(&paths.hook_script_path, script.as_bytes())
        .with_context(|| format!("failed to write {}", paths.hook_script_path.display()))?;
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&paths.hook_script_path)
            .with_context(|| format!("failed to read {}", paths.hook_script_path.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&paths.hook_script_path, perms)
            .with_context(|| format!("failed to chmod {}", paths.hook_script_path.display()))?;
    }
    Ok(())
}

fn render_hook_script(paths: &ClaudePaths) -> String {
    format!(
        "#!/bin/bash\nBRIDGE=\"{}\"\nif [ -x \"$BRIDGE\" ]; then\n  exec \"$BRIDGE\" --source claude \"$@\"\nfi\nexit 0\n",
        bash_path_string(&paths.bridge_path)
    )
}

fn bash_path_string(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn hooks_have_echoisland_entries(paths: &ClaudePaths) -> Result<bool> {
    if !paths.settings_path.exists() {
        return Ok(false);
    }
    let root = load_json_object(&paths.settings_path)?;
    let Some(hooks_obj) = root.get("hooks").and_then(Value::as_object) else {
        return Ok(false);
    };

    for (event, _) in CLAUDE_EVENTS {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{bash_path_string, hook_script_command, render_hook_script};
    use crate::claude::ClaudePaths;

    #[test]
    fn claude_hook_command_uses_bash_safe_forward_slashes() {
        let paths = ClaudePaths {
            home_dir: PathBuf::from(r"C:\Users\Adim"),
            claude_dir: PathBuf::from(r"C:\Users\Adim\.claude"),
            settings_path: PathBuf::from(r"C:\Users\Adim\.claude\settings.json"),
            projects_dir: PathBuf::from(r"C:\Users\Adim\.claude\projects"),
            hook_script_path: PathBuf::from(r"C:\Users\Adim\.claude\hooks\echoisland-hook.sh"),
            bridge_install_dir: PathBuf::from(r"C:\Users\Adim\.echoisland\bin"),
            bridge_path: PathBuf::from(r"C:\Users\Adim\.echoisland\bin\echoisland-hook-bridge.exe"),
        };

        assert_eq!(
            hook_script_command(&paths),
            "~/.claude/hooks/echoisland-hook.sh"
        );
        assert!(
            render_hook_script(&paths)
                .contains(r#"BRIDGE="C:/Users/Adim/.echoisland/bin/echoisland-hook-bridge.exe""#)
        );
    }

    #[test]
    fn bash_path_string_preserves_posix_paths() {
        assert_eq!(
            bash_path_string(PathBuf::from("/tmp/.claude/hooks/hook.sh").as_path()),
            "/tmp/.claude/hooks/hook.sh"
        );
    }
}
