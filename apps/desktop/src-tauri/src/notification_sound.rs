#[cfg(target_os = "macos")]
use crate::app_settings::current_app_settings;

#[cfg(target_os = "macos")]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use tracing::warn;

#[cfg(target_os = "macos")]
const CLICK_SOUND_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../web/assets/click.mp3"
));

#[cfg(target_os = "macos")]
const CLICK_SOUND_FILE_NAME: &str = "echoisland-click.mp3";

#[cfg(target_os = "macos")]
const SOUND_PLAY_MIN_INTERVAL_MS: u64 = 120;

#[cfg(target_os = "macos")]
static LAST_SOUND_PLAY_AT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

#[cfg(target_os = "macos")]
static EMBEDDED_SOUND_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

#[cfg(target_os = "macos")]
fn last_sound_play_at() -> &'static Mutex<Option<Instant>> {
    LAST_SOUND_PLAY_AT.get_or_init(|| Mutex::new(None))
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
pub(crate) fn play_message_card_sound() {
    if !current_app_settings().completion_sound_enabled {
        return;
    }

    let Ok(mut last_play_at) = last_sound_play_at().lock() else {
        return;
    };

    let now = Instant::now();
    if last_play_at.is_some_and(|previous| {
        now.saturating_duration_since(previous) < Duration::from_millis(SOUND_PLAY_MIN_INTERVAL_MS)
    }) {
        return;
    }
    *last_play_at = Some(now);
    drop(last_play_at);

    let Some(sound_path) = resolve_click_sound_path() else {
        return;
    };

    if let Err(error) = Command::new("afplay").arg(&sound_path).spawn() {
        warn!(error = %error, path = %sound_path.display(), "failed to play message card sound");
    }
}

#[cfg(not(target_os = "macos"))]
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(crate) fn play_message_card_sound() {}
