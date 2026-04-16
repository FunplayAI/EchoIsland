use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use echoisland_paths::current_platform_paths;
use serde::{Deserialize, Serialize};

use crate::terminal_focus::SessionTabCache;

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedFocusBindings {
    session_tabs: HashMap<String, SessionTabCache>,
}

pub fn default_focus_bindings_path() -> PathBuf {
    current_platform_paths().focus_bindings_path
}

pub fn load_focus_bindings(path: &Path) -> Result<HashMap<String, SessionTabCache>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let raw = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let decoded: PersistedFocusBindings =
        serde_json::from_slice(&raw).context("failed to decode persisted focus bindings")?;
    Ok(decoded.session_tabs)
}

pub fn save_focus_bindings(path: &Path, bindings: &HashMap<String, SessionTabCache>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let encoded = serde_json::to_vec_pretty(&PersistedFocusBindings {
        session_tabs: bindings.clone(),
    })
    .context("failed to encode focus bindings")?;
    fs::write(path, encoded).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{load_focus_bindings, save_focus_bindings};
    use crate::terminal_focus::SessionTabCache;

    fn temp_file() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("echoisland-focus-bindings-{suffix}.json"))
    }

    #[test]
    fn saves_and_loads_focus_bindings() {
        let path = temp_file();
        let mut bindings = std::collections::HashMap::new();
        bindings.insert(
            "session-1".to_string(),
            SessionTabCache {
                terminal_pid: 1234,
                window_hwnd: 5678,
                runtime_id: "1,2,3".to_string(),
                title: "Claude".to_string(),
            },
        );

        save_focus_bindings(&path, &bindings).unwrap();
        let loaded = load_focus_bindings(&path).unwrap();
        let cached = loaded.get("session-1").unwrap();
        assert_eq!(cached.terminal_pid, 1234);
        assert_eq!(cached.window_hwnd, 5678);
        assert_eq!(cached.runtime_id, "1,2,3");
        assert_eq!(cached.title, "Claude");

        let _ = std::fs::remove_file(path);
    }
}
