use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM},
    System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    },
};

#[derive(Clone, Debug)]
pub(super) struct WindowCandidate {
    pub hwnd: HWND,
    pub pid: u32,
    pub title: String,
    pub process_name: String,
    pub is_terminal_like: bool,
}

pub(super) fn collect_windows() -> Vec<WindowCandidate> {
    let mut windows = Vec::new();
    unsafe {
        EnumWindows(Some(enum_windows), &mut windows as *mut _ as LPARAM);
    }
    windows
}

unsafe extern "system" fn enum_windows(hwnd: HWND, lparam: LPARAM) -> i32 {
    if unsafe { IsWindowVisible(hwnd) } == 0 {
        return 1;
    }

    let text_len = unsafe { GetWindowTextLengthW(hwnd) };
    if text_len <= 0 {
        return 1;
    }

    let mut text = vec![0u16; text_len as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32) };
    if copied <= 0 {
        return 1;
    }

    let title = String::from_utf16_lossy(&text[..copied as usize])
        .trim()
        .to_string();
    if title.is_empty() {
        return 1;
    }

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == 0 {
        return 1;
    }

    let process_name = process_name_from_pid(pid).unwrap_or_default();
    let windows = unsafe { &mut *(lparam as *mut Vec<WindowCandidate>) };
    windows.push(WindowCandidate {
        hwnd,
        pid,
        is_terminal_like: terminal_like_process(&process_name),
        process_name,
        title,
    });
    1
}

fn terminal_like_process(process_name: &str) -> bool {
    matches!(
        process_name,
        "windowsterminal"
            | "wezterm-gui"
            | "alacritty"
            | "mintty"
            | "conemu64"
            | "conemu"
            | "hyper"
            | "tabby"
            | "rio"
            | "code"
            | "cursor"
            | "windsurf"
            | "powershell"
            | "pwsh"
            | "cmd"
    )
}

fn process_name_from_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut buffer = vec![0u16; 1024];
        let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) != 0;
        let _ = CloseHandle(handle);
        if !ok || length == 0 {
            return None;
        }

        let path = String::from_utf16_lossy(&buffer[..length as usize]);
        path.rsplit(['\\', '/'])
            .next()
            .map(|value| value.trim_end_matches(".exe").to_ascii_lowercase())
    }
}
