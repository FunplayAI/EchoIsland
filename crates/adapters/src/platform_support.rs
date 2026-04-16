use std::path::Path;

#[cfg(target_os = "windows")]
use std::process::Command;

use echoisland_paths::user_home_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveCaptureSupport {
    pub supported: bool,
    pub note: Option<String>,
}

pub fn codex_live_capture_support() -> LiveCaptureSupport {
    if cfg!(target_os = "windows") {
        LiveCaptureSupport {
            supported: false,
            note: Some(
                "Current Codex CLI Windows builds disable hooks.json lifecycle hooks at runtime, so EchoIsland cannot receive live Codex hook events yet.".to_string(),
            ),
        }
    } else {
        LiveCaptureSupport {
            supported: true,
            note: None,
        }
    }
}

pub fn supported_with_note(note: impl Into<String>) -> LiveCaptureSupport {
    LiveCaptureSupport {
        supported: true,
        note: Some(note.into()),
    }
}

pub fn codex_running_process_limit(home_dir: &Path) -> Option<usize> {
    if !should_limit_to_current_user_home(home_dir) {
        return None;
    }

    active_codex_process_count()
}

fn should_limit_to_current_user_home(home_dir: &Path) -> bool {
    user_home_dir() == home_dir
}

fn active_codex_process_count() -> Option<usize> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-Process codex -ErrorAction SilentlyContinue | Measure-Object).Count",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        raw.trim().parse::<usize>().ok()
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use echoisland_paths::user_home_dir;

    use super::{
        codex_live_capture_support, codex_running_process_limit, should_limit_to_current_user_home,
        supported_with_note,
    };

    #[test]
    fn supported_with_note_marks_live_capture_supported() {
        let support = supported_with_note("ok");
        assert!(support.supported);
        assert_eq!(support.note.as_deref(), Some("ok"));
    }

    #[test]
    fn codex_support_matches_current_platform() {
        let support = codex_live_capture_support();

        #[cfg(target_os = "windows")]
        {
            assert!(!support.supported);
            assert!(support.note.is_some());
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(support.supported);
            assert!(support.note.is_none());
        }
    }

    #[test]
    fn only_limits_processes_for_current_home() {
        let current_home = user_home_dir();
        let other_home = PathBuf::from("__echoisland_test_other_home__");

        assert!(should_limit_to_current_user_home(&current_home));
        assert!(!should_limit_to_current_user_home(&other_home));
        assert_eq!(codex_running_process_limit(&other_home), None);
    }
}
