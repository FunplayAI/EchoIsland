use anyhow::Result;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindow},
};

pub(super) fn activate_window(hwnd: HWND) -> Result<bool> {
    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        if SetForegroundWindow(hwnd) == 0 {
            return Err(anyhow::anyhow!("failed to focus terminal window"));
        }
    }
    Ok(true)
}
