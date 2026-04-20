#[cfg(target_os = "macos")]
use std::collections::{HashMap, HashSet};

#[cfg(target_os = "macos")]
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[cfg(target_os = "macos")]
use std::time::Instant;

#[cfg(target_os = "macos")]
use chrono::Utc;

#[cfg(target_os = "macos")]
use objc2::{MainThreadMarker, MainThreadOnly};

#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSBackingStoreType, NSClipView, NSColor, NSEvent, NSFont, NSPanel, NSScreen, NSTextAlignment,
    NSTextField, NSView, NSWindowAnimationBehavior, NSWindowCollectionBehavior, NSWindowStyleMask,
};

#[cfg(target_os = "macos")]
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, ns_string};

#[cfg(target_os = "macos")]
use objc2_core_graphics::{
    CGAffineTransformMakeScale, CGAffineTransformTranslate, CGMutablePath, CGPath,
};

#[cfg(target_os = "macos")]
use objc2_quartz_core::{CACornerMask, CALayer, CAShapeLayer, CATransaction};

#[cfg(target_os = "macos")]
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use tokio::time::Duration;

#[cfg(target_os = "macos")]
use tracing::{info, warn};

#[cfg(target_os = "macos")]
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

#[cfg(target_os = "macos")]
use crate::{
    app_runtime::AppRuntime, macos_shared_expanded_window,
    terminal_focus_service::TerminalFocusService,
};

#[cfg(target_os = "macos")]
mod card_animation;
#[cfg(target_os = "macos")]
mod card_metrics;
#[cfg(target_os = "macos")]
mod card_stack;
#[cfg(target_os = "macos")]
mod card_views;
#[cfg(target_os = "macos")]
mod compact_bar;
#[cfg(target_os = "macos")]
mod display_helpers;
#[cfg(target_os = "macos")]
mod mascot;
#[cfg(target_os = "macos")]
mod mascot_render;
#[cfg(target_os = "macos")]
mod panel_assembly;
#[cfg(target_os = "macos")]
mod panel_constants;
#[cfg(target_os = "macos")]
mod panel_entry;
#[cfg(target_os = "macos")]
mod panel_geometry;
#[cfg(target_os = "macos")]
mod panel_handles_init;
#[cfg(target_os = "macos")]
mod panel_helpers;
#[cfg(target_os = "macos")]
mod panel_interaction;
#[cfg(target_os = "macos")]
mod panel_refs;
#[cfg(target_os = "macos")]
mod panel_render;
#[cfg(target_os = "macos")]
mod panel_setup;
#[cfg(target_os = "macos")]
mod panel_shoulder;
#[cfg(target_os = "macos")]
mod panel_snapshot;
#[cfg(target_os = "macos")]
mod panel_state_init;
#[cfg(target_os = "macos")]
mod panel_style;
#[cfg(target_os = "macos")]
mod panel_transition_entry;
#[cfg(target_os = "macos")]
mod panel_types;
#[cfg(target_os = "macos")]
mod panel_views;
#[cfg(target_os = "macos")]
mod panel_window;
#[cfg(target_os = "macos")]
mod queue_logic;
#[cfg(target_os = "macos")]
mod transition_logic;
#[cfg(target_os = "macos")]
mod transition_runner;
#[cfg(target_os = "macos")]
mod transition_ui;

#[cfg(target_os = "macos")]
use card_animation::*;
#[cfg(target_os = "macos")]
use card_metrics::*;
#[cfg(target_os = "macos")]
use card_stack::*;
#[cfg(target_os = "macos")]
use card_views::*;
#[cfg(target_os = "macos")]
use compact_bar::*;
#[cfg(target_os = "macos")]
use display_helpers::*;
#[cfg(target_os = "macos")]
use mascot::*;
#[cfg(target_os = "macos")]
use mascot_render::*;
#[cfg(target_os = "macos")]
use panel_assembly::*;
#[cfg(target_os = "macos")]
use panel_constants::*;
#[cfg(target_os = "macos")]
pub(crate) use panel_entry::{
    create_native_island_panel, hide_main_webview_window, hide_native_island_panel,
    native_ui_enabled, spawn_native_count_marquee_loop, spawn_native_hover_loop,
    spawn_native_snapshot_loop, spawn_native_status_queue_loop,
};
#[cfg(target_os = "macos")]
use panel_geometry::*;
#[cfg(target_os = "macos")]
use panel_handles_init::*;
#[cfg(target_os = "macos")]
use panel_helpers::*;
#[cfg(target_os = "macos")]
use panel_interaction::*;
#[cfg(target_os = "macos")]
use panel_refs::*;
#[cfg(target_os = "macos")]
use panel_render::*;
#[cfg(target_os = "macos")]
use panel_setup::*;
#[cfg(target_os = "macos")]
use panel_snapshot::*;
#[cfg(target_os = "macos")]
pub(crate) use panel_snapshot::{set_shared_expanded_body_height, update_native_island_snapshot};
#[cfg(target_os = "macos")]
use panel_state_init::*;
#[cfg(target_os = "macos")]
use panel_transition_entry::*;
#[cfg(target_os = "macos")]
use panel_types::*;
#[cfg(target_os = "macos")]
use panel_views::*;
#[cfg(target_os = "macos")]
use queue_logic::*;
#[cfg(target_os = "macos")]
use transition_logic::*;
#[cfg(target_os = "macos")]
use transition_runner::*;
#[cfg(target_os = "macos")]
use transition_ui::*;

#[cfg(target_os = "macos")]
static NATIVE_TEST_PANEL_CREATED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
static NATIVE_TEST_PANEL_HANDLES: OnceLock<NativePanelHandles> = OnceLock::new();

#[cfg(target_os = "macos")]
static NATIVE_TEST_PANEL_STATE: OnceLock<Mutex<NativePanelState>> = OnceLock::new();

#[cfg(target_os = "macos")]
static NATIVE_TEST_PANEL_ANIMATION_ID: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "macos")]
static ACTIVE_COUNT_SCROLL_STARTED: OnceLock<Instant> = OnceLock::new();
#[cfg(target_os = "macos")]
static ACTIVE_COUNT_SCROLL_TEXT: OnceLock<Mutex<String>> = OnceLock::new();
#[cfg(target_os = "macos")]
static CARD_ANIMATION_LAYOUTS: OnceLock<Mutex<HashMap<usize, CardAnimationLayout>>> =
    OnceLock::new();

#[cfg(all(test, target_os = "macos"))]
mod tests;

#[cfg(not(target_os = "macos"))]
pub fn native_ui_enabled() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn create_native_island_panel() -> Result<(), String> {
    Ok(())
}
