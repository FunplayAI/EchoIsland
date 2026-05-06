#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::app_settings::current_app_settings;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use tracing::warn;

#[cfg(any(target_os = "macos", target_os = "windows"))]
const CLICK_SOUND_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../web/assets/click.mp3"
));

#[cfg(any(target_os = "macos", target_os = "windows"))]
const CLICK_SOUND_FILE_NAME: &str = "echoisland-click.mp3";

#[cfg(any(target_os = "macos", target_os = "windows"))]
const SOUND_PLAY_MIN_INTERVAL_MS: u64 = 120;

#[cfg(any(target_os = "macos", target_os = "windows"))]
static LAST_SOUND_PLAY_AT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

#[cfg(any(target_os = "macos", target_os = "windows"))]
static EMBEDDED_SOUND_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn last_sound_play_at() -> &'static Mutex<Option<Instant>> {
    LAST_SOUND_PLAY_AT.get_or_init(|| Mutex::new(None))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn ensure_embedded_click_sound_file() -> Option<&'static Path> {
    EMBEDDED_SOUND_PATH
        .get_or_init(|| {
            let path = std::env::temp_dir().join(CLICK_SOUND_FILE_NAME);
            match fs::write(&path, CLICK_SOUND_BYTES) {
                Ok(()) => Some(path),
                Err(error) => {
                    warn!(error = %error, "failed to write embedded click sound");
                    None
                }
            }
        })
        .as_deref()
}

#[cfg(target_os = "macos")]
fn resolve_click_sound_path() -> Option<PathBuf> {
    let custom_path = PathBuf::from("/Users/wenuts/Downloads/click.mp3");
    if custom_path.is_file() {
        return Some(custom_path);
    }

    ensure_embedded_click_sound_file().map(Path::to_path_buf)
}

#[cfg(target_os = "windows")]
fn resolve_click_sound_path() -> Option<PathBuf> {
    ensure_embedded_click_sound_file().map(Path::to_path_buf)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn should_throttle_sound_play() -> bool {
    let Ok(mut last_play_at) = last_sound_play_at().lock() else {
        return true;
    };

    let now = Instant::now();
    if last_play_at.is_some_and(|previous| {
        now.saturating_duration_since(previous) < Duration::from_millis(SOUND_PLAY_MIN_INTERVAL_MS)
    }) {
        return true;
    }
    *last_play_at = Some(now);
    false
}

#[cfg(target_os = "macos")]
pub(crate) fn play_message_card_sound() {
    if !current_app_settings().completion_sound_enabled {
        return;
    }

    if should_throttle_sound_play() {
        return;
    }

    let Some(sound_path) = resolve_click_sound_path() else {
        return;
    };

    if let Err(error) = Command::new("afplay").arg(&sound_path).spawn() {
        warn!(error = %error, path = %sound_path.display(), "failed to play message card sound");
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn play_message_card_sound() {
    if !current_app_settings().completion_sound_enabled {
        return;
    }

    if should_throttle_sound_play() {
        return;
    }

    let Some(sound_path) = resolve_click_sound_path() else {
        return;
    };

    if let Err(error) = windows_play_mp3_via_mci(&sound_path) {
        warn!(error = %error, path = %sound_path.display(), "failed to play message card sound");
    }
}

/// Play the MP3 asynchronously via the Windows MCI (Media Control Interface)
/// API. We close any previous instance under the same alias first so back-to-
/// back plays work, then re-open and start the new playback. The throttle in
/// `should_throttle_sound_play` guarantees calls are at least 120ms apart.
#[cfg(target_os = "windows")]
fn windows_play_mp3_via_mci(sound_path: &Path) -> std::io::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Media::Multimedia::mciSendStringW;

    const ALIAS: &str = "echoisland_click";

    fn to_wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }

    fn send(command: &str) -> u32 {
        let wide = to_wide(command);
        unsafe { mciSendStringW(wide.as_ptr(), null_mut(), 0, null_mut()) }
    }

    // Closing a non-existent alias is harmless; ignore its result.
    let _ = send(&format!("close {ALIAS}"));

    let path_str = sound_path
        .to_str()
        .ok_or_else(|| std::io::Error::other("sound path is not valid UTF-8"))?;
    let open_status = send(&format!("open \"{path_str}\" type mpegvideo alias {ALIAS}"));
    if open_status != 0 {
        return Err(std::io::Error::other(format!(
            "MCI open returned {open_status}"
        )));
    }

    let play_status = send(&format!("play {ALIAS}"));
    if play_status != 0 {
        let _ = send(&format!("close {ALIAS}"));
        return Err(std::io::Error::other(format!(
            "MCI play returned {play_status}"
        )));
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn play_message_card_sound() {}
