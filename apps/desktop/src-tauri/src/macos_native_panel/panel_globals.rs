use std::collections::HashMap;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU64},
};
use std::time::Instant;

use super::panel_types::{CardAnimationLayout, NativePanelHandles, NativePanelState};

pub(super) static NATIVE_TEST_PANEL_CREATED: AtomicBool = AtomicBool::new(false);
pub(super) static NATIVE_TEST_PANEL_HANDLES: OnceLock<NativePanelHandles> = OnceLock::new();
pub(super) static NATIVE_TEST_PANEL_STATE: OnceLock<Mutex<NativePanelState>> = OnceLock::new();
pub(super) static NATIVE_TEST_PANEL_ANIMATION_ID: AtomicU64 = AtomicU64::new(0);
pub(super) static ACTIVE_COUNT_SCROLL_STARTED: OnceLock<Instant> = OnceLock::new();
pub(super) static ACTIVE_COUNT_SCROLL_TEXT: OnceLock<Mutex<String>> = OnceLock::new();
pub(super) static CARD_ANIMATION_LAYOUTS: OnceLock<Mutex<HashMap<usize, CardAnimationLayout>>> =
    OnceLock::new();
