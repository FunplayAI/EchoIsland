use std::path::Path;

use echoisland_paths::user_home_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveCaptureSupport {
    pub supported: bool,
    pub note: Option<String>,
}

pub fn codex_live_capture_support() -> LiveCaptureSupport {
    LiveCaptureSupport {
        supported: true,
        note: None,
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
        windows_codex_process_count()
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn windows_codex_process_count() -> Option<usize> {
    use std::mem::size_of;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

        let mut count = 0;
        if Process32FirstW(snapshot, &mut entry) == 0 {
            CloseHandle(snapshot);
            return None;
        }

        loop {
            if process_entry_name(&entry).eq_ignore_ascii_case("codex.exe")
                || process_entry_name(&entry).eq_ignore_ascii_case("codex")
            {
                count += 1;
            }

            if Process32NextW(snapshot, &mut entry) == 0 {
                break;
            }
        }

        CloseHandle(snapshot);
        Some(count)
    }
}

#[cfg(target_os = "windows")]
fn process_entry_name(
    entry: &windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W,
) -> String {
    let len = entry
        .szExeFile
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(entry.szExeFile.len());
    String::from_utf16_lossy(&entry.szExeFile[..len])
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

        assert!(support.supported);
        assert!(support.note.is_none());
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
