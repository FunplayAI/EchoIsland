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
use tokio::time::{Duration, MissedTickBehavior};

#[cfg(target_os = "macos")]
use tracing::{info, warn};

#[cfg(target_os = "macos")]
use echoisland_runtime::{
    PendingPermissionView, PendingQuestionView, RuntimeSnapshot, SessionSnapshotView,
};

#[cfg(target_os = "macos")]
use crate::{
    app_runtime::AppRuntime, constants::MAIN_WINDOW_LABEL, macos_shared_expanded_window,
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
mod panel_geometry;
#[cfg(target_os = "macos")]
mod panel_handles_init;
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
use panel_geometry::*;
#[cfg(target_os = "macos")]
use panel_handles_init::*;
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

#[cfg(target_os = "macos")]
const DEFAULT_PANEL_CANVAS_WIDTH: f64 = 420.0;
#[cfg(target_os = "macos")]
const COLLAPSED_PANEL_HEIGHT: f64 = 80.0;
#[cfg(target_os = "macos")]
const DEFAULT_COMPACT_PILL_WIDTH: f64 = 253.0;
#[cfg(target_os = "macos")]
const DEFAULT_COMPACT_PILL_HEIGHT: f64 = 37.0;
#[cfg(target_os = "macos")]
const COMPACT_SHOULDER_SIZE: f64 = 6.0;
#[cfg(target_os = "macos")]
const COMPACT_PILL_RADIUS: f64 = 12.5;
#[cfg(target_os = "macos")]
const COMPACT_HEADLINE_LABEL_HEIGHT: f64 = 24.0;
#[cfg(target_os = "macos")]
const COMPACT_HEADLINE_VERTICAL_NUDGE_Y: f64 = -1.5;
#[cfg(target_os = "macos")]
const EXPANDED_PANEL_RADIUS: f64 = 12.0;
#[cfg(target_os = "macos")]
const CARD_RADIUS: f64 = 9.0;
#[cfg(target_os = "macos")]
const SHOULDER_CURVE_FACTOR: f64 = 0.62;
#[cfg(target_os = "macos")]
const EXPANDED_MAX_BODY_HEIGHT: f64 = 420.0;
#[cfg(target_os = "macos")]
const EXPANDED_CARD_GAP: f64 = 12.0;
#[cfg(target_os = "macos")]
const EXPANDED_CONTENT_TOP_GAP: f64 = 9.0;
#[cfg(target_os = "macos")]
const EXPANDED_CONTENT_BOTTOM_INSET: f64 = 10.0;
#[cfg(target_os = "macos")]
const EXPANDED_CARDS_SIDE_INSET: f64 = 10.0;
#[cfg(target_os = "macos")]
const EXPANDED_CARD_OVERHANG: f64 = 0.0;
#[cfg(target_os = "macos")]
const CARD_INSET_X: f64 = 10.0;
#[cfg(target_os = "macos")]
const CARD_HEADER_HEIGHT: f64 = 52.0;
#[cfg(target_os = "macos")]
const CARD_CHAT_PREFIX_WIDTH: f64 = 15.0;
#[cfg(target_os = "macos")]
const CARD_CHAT_LINE_HEIGHT: f64 = 14.0;
#[cfg(target_os = "macos")]
const CARD_CONTENT_BOTTOM_INSET: f64 = 6.0;
#[cfg(target_os = "macos")]
const CARD_CHAT_GAP: f64 = 4.0;
#[cfg(target_os = "macos")]
const CARD_TOOL_GAP: f64 = 7.0;
#[cfg(target_os = "macos")]
const CARD_PENDING_ACTION_Y: f64 = 9.0;
#[cfg(target_os = "macos")]
const CARD_PENDING_ACTION_HEIGHT: f64 = 18.0;
#[cfg(target_os = "macos")]
const CARD_PENDING_ACTION_GAP: f64 = 6.0;
#[cfg(target_os = "macos")]
const PENDING_QUESTION_CARD_MIN_HEIGHT: f64 = 108.0;
#[cfg(target_os = "macos")]
const PENDING_QUESTION_CARD_MAX_HEIGHT: f64 = 132.0;
#[cfg(target_os = "macos")]
const MAX_VISIBLE_SESSIONS: usize = 5;
#[cfg(target_os = "macos")]
const PROMPT_ASSIST_RUNNING_SECONDS: i64 = 12;
#[cfg(target_os = "macos")]
const PROMPT_ASSIST_PROCESSING_SECONDS: i64 = 18;
#[cfg(target_os = "macos")]
const PROMPT_ASSIST_RECENT_SECONDS: i64 = 20 * 60;
#[cfg(target_os = "macos")]
const STATUS_COMPLETION_VISIBLE_SECONDS: u64 = 10;
#[cfg(target_os = "macos")]
const STATUS_APPROVAL_VISIBLE_SECONDS: u64 = 30;
#[cfg(target_os = "macos")]
const STATUS_QUEUE_EXIT_EXTRA_MS: u64 = 80;
#[cfg(target_os = "macos")]
const STATUS_QUEUE_REFRESH_MS: u64 = 33;
#[cfg(target_os = "macos")]
const HOVER_POLL_MS: u64 = 120;
#[cfg(target_os = "macos")]
const HOVER_DELAY_MS: u64 = 500;
#[cfg(target_os = "macos")]
const PENDING_CARD_MIN_VISIBLE_MS: u64 = 2200;
#[cfg(target_os = "macos")]
const PENDING_CARD_RELEASE_GRACE_MS: u64 = 1600;
#[cfg(target_os = "macos")]
const PANEL_OPEN_TOTAL_MS: u64 = 660;
#[cfg(target_os = "macos")]
const PANEL_CLOSE_TOTAL_MS: u64 = 660;
#[cfg(target_os = "macos")]
const PANEL_MORPH_DELAY_MS: u64 = 120;
#[cfg(target_os = "macos")]
const PANEL_MORPH_MS: u64 = 270;
#[cfg(target_os = "macos")]
const PANEL_SHOULDER_HIDE_MS: u64 = 120;
#[cfg(target_os = "macos")]
const PANEL_HEIGHT_MS: u64 = 270;
#[cfg(target_os = "macos")]
const PANEL_CLOSE_MORPH_DELAY_MS: u64 = 270;
#[cfg(target_os = "macos")]
const PANEL_CLOSE_SHOULDER_DELAY_MS: u64 = 540;
#[cfg(target_os = "macos")]
const PANEL_CLOSE_SHOULDER_MS: u64 = 120;
#[cfg(target_os = "macos")]
const PANEL_DROP_DISTANCE: f64 = 4.5;
#[cfg(target_os = "macos")]
const PANEL_MORPH_PILL_RADIUS: f64 = 14.0;
#[cfg(target_os = "macos")]
const PANEL_ANIMATION_FRAME_MS: u64 = 16;
#[cfg(target_os = "macos")]
const PANEL_CARD_REVEAL_MS: u64 = 320;
#[cfg(target_os = "macos")]
const PANEL_CARD_REVEAL_STAGGER_MS: u64 = 70;
#[cfg(target_os = "macos")]
const PANEL_CARD_EXIT_MS: u64 = 220;
#[cfg(target_os = "macos")]
const PANEL_CARD_EXIT_STAGGER_MS: u64 = 38;
#[cfg(target_os = "macos")]
const PANEL_CARD_EXIT_SETTLE_MS: u64 = 40;
#[cfg(target_os = "macos")]
const PANEL_SURFACE_SWITCH_HEIGHT_MS: u64 = 220;
#[cfg(target_os = "macos")]
const PANEL_SURFACE_SWITCH_CARD_REVEAL_MS: u64 = 220;
#[cfg(target_os = "macos")]
const PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS: u64 = 42;
#[cfg(target_os = "macos")]
const PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS: f64 = 0.24;
#[cfg(target_os = "macos")]
const PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS: f64 = 0.18;
#[cfg(target_os = "macos")]
const PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS: f64 = 0.30;
#[cfg(target_os = "macos")]
const PANEL_CARD_REVEAL_Y: f64 = -8.0;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SLOT_WIDTH: f64 = 11.0;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SLOT_NUDGE_X: f64 = 2.0;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SCROLL_TRAVEL: f64 = 10.0;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_TEXT_WIDTH: f64 = 22.0;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_TEXT_OFFSET_X: f64 = ACTIVE_COUNT_SLOT_WIDTH - ACTIVE_COUNT_TEXT_WIDTH;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_LABEL_HEIGHT: f64 = 24.0;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SCROLL_STEP_MS: u128 = 2300;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SCROLL_HOLD_MS: u128 = 1000;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SCROLL_MOVE_MS: u128 = 300;
#[cfg(target_os = "macos")]
const ACTIVE_COUNT_SCROLL_REFRESH_MS: u64 = 33;
#[cfg(target_os = "macos")]
const MASCOT_ANIMATION_REFRESH_MS: u64 = 33;
#[cfg(target_os = "macos")]
const MASCOT_STATE_TRANSITION_SECONDS: f64 = 0.24;
#[cfg(target_os = "macos")]
const MASCOT_IDLE_LONG_SECONDS: u64 = 120;
#[cfg(target_os = "macos")]
const MASCOT_WAKE_ANGRY_SECONDS: f64 = 0.82;
#[cfg(target_os = "macos")]
const MASCOT_VERTICAL_NUDGE_Y: f64 = -2.0;
#[cfg(target_os = "macos")]
const SHARED_CONTENT_REVEAL_PROGRESS: f64 = 0.94;
#[cfg(target_os = "macos")]
const SHARED_CONTENT_INTERACTIVE_PROGRESS: f64 = 0.985;

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativePanelHandles {
    panel: usize,
    content_view: usize,
    left_shoulder: usize,
    right_shoulder: usize,
    pill_view: usize,
    expanded_container: usize,
    cards_container: usize,
    top_highlight: usize,
    body_separator: usize,
    mascot_shell: usize,
    mascot_body: usize,
    mascot_left_eye: usize,
    mascot_right_eye: usize,
    mascot_mouth: usize,
    mascot_bubble: usize,
    mascot_sleep_label: usize,
    headline: usize,
    active_count_clip: usize,
    active_count: usize,
    active_count_next: usize,
    slash: usize,
    total_count: usize,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct CardAnimationLayout {
    frame: NSRect,
    collapsed_height: f64,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativePanelTransitionFrame {
    canvas_height: f64,
    visible_height: f64,
    bar_progress: f64,
    height_progress: f64,
    shoulder_progress: f64,
    drop_progress: f64,
    cards_progress: f64,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativePanelGeometryMetrics {
    compact_height: f64,
    compact_width: f64,
    expanded_width: f64,
    panel_width: f64,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativePanelLayout {
    panel_frame: NSRect,
    content_frame: NSRect,
    pill_frame: NSRect,
    left_shoulder_frame: NSRect,
    right_shoulder_frame: NSRect,
    expanded_frame: NSRect,
    cards_frame: NSRect,
    separator_frame: NSRect,
    shared_content_frame: NSRect,
    shell_visible: bool,
    separator_visibility: f64,
}

#[cfg(target_os = "macos")]
impl NativePanelTransitionFrame {
    fn expanded(height: f64) -> Self {
        Self {
            canvas_height: height,
            visible_height: height,
            bar_progress: 1.0,
            height_progress: 1.0,
            shoulder_progress: 1.0,
            drop_progress: 1.0,
            cards_progress: 1.0,
        }
    }

    fn collapsed(height: f64) -> Self {
        Self {
            canvas_height: height,
            visible_height: height,
            bar_progress: 0.0,
            height_progress: 0.0,
            shoulder_progress: 0.0,
            drop_progress: 0.0,
            cards_progress: 0.0,
        }
    }
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
enum NativeStatusQueuePayload {
    Approval(PendingPermissionView),
    Completion(SessionSnapshotView),
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
struct NativeStatusQueueItem {
    key: String,
    session_id: String,
    sort_time: chrono::DateTime<Utc>,
    expires_at: Instant,
    is_live: bool,
    is_removing: bool,
    remove_after: Option<Instant>,
    payload: NativeStatusQueuePayload,
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
struct NativePendingPermissionCard {
    request_id: String,
    payload: PendingPermissionView,
    started_at: Instant,
    last_seen_at: Instant,
    visible_until: Instant,
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
struct NativePendingQuestionCard {
    request_id: String,
    payload: PendingQuestionView,
    started_at: Instant,
    last_seen_at: Instant,
    visible_until: Instant,
}

#[cfg(target_os = "macos")]
#[derive(Clone)]
struct NativeCardHitTarget {
    session_id: String,
    frame: NSRect,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeExpandedSurface {
    Default,
    Status,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeHoverTransition {
    Expand,
    Collapse,
}

#[cfg(target_os = "macos")]
struct NativePanelState {
    expanded: bool,
    transitioning: bool,
    transition_cards_progress: f64,
    transition_cards_entering: bool,
    skip_next_close_card_exit: bool,
    last_raw_snapshot: Option<RuntimeSnapshot>,
    last_snapshot: Option<RuntimeSnapshot>,
    status_queue: Vec<NativeStatusQueueItem>,
    pending_permission_card: Option<NativePendingPermissionCard>,
    pending_question_card: Option<NativePendingQuestionCard>,
    status_auto_expanded: bool,
    surface_mode: NativeExpandedSurface,
    shared_body_height: Option<f64>,
    pointer_inside_since: Option<Instant>,
    pointer_outside_since: Option<Instant>,
    primary_mouse_down: bool,
    last_focus_click: Option<(String, Instant)>,
    card_hit_targets: Vec<NativeCardHitTarget>,
    mascot_runtime: NativeMascotRuntime,
}

const CARD_FOCUS_CLICK_DEBOUNCE_MS: u128 = 600;

#[cfg(target_os = "macos")]
pub fn native_ui_enabled() -> bool {
    !matches!(
        std::env::var("CODEISLAND_USE_WEBVIEW").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

#[cfg(target_os = "macos")]
pub fn create_native_island_panel() -> Result<(), String> {
    if NATIVE_TEST_PANEL_CREATED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let Some(mtm) = MainThreadMarker::new() else {
        return Err("native test panel must be created on the main thread".to_string());
    };

    let setup = resolve_native_panel_setup(mtm)?;
    let colors = native_panel_colors();
    let NativePanelSetup {
        screen,
        compact_height,
        compact_width,
        expanded_width,
        size,
        pill_size,
        screen_frame,
        frame,
        pill_frame,
    } = setup;
    let panel = panel_window::create_native_panel_window(mtm, frame);
    let NativePanelColors {
        pill_background,
        pill_border,
        pill_highlight,
        mascot_shell_border,
        mascot_body_fill,
        mascot_stroke,
        mascot_face,
        expanded_background,
        expanded_border,
        text_primary,
        accent_active,
        separator_color,
    } = colors;

    let PanelBaseViews {
        content_view,
        left_shoulder,
        right_shoulder,
        pill_view,
        expanded_container,
        cards_container,
        top_highlight,
        body_separator,
    } = create_panel_base_views(
        mtm,
        size,
        pill_frame,
        pill_size,
        compact_width,
        expanded_width,
        compact_height,
        &pill_background,
        &pill_border,
        &pill_highlight,
        &expanded_background,
        &expanded_border,
        &separator_color,
    );
    let MascotViews {
        shell: mascot_shell,
        body: mascot_body,
        left_eye: mascot_left_eye,
        right_eye: mascot_right_eye,
        mouth: mascot_mouth,
        bubble: mascot_bubble,
        sleep_label: mascot_sleep_label,
    } = create_mascot_views(
        mtm,
        &mascot_shell_border,
        &mascot_body_fill,
        &mascot_stroke,
        &mascot_face,
    );

    let CompactBarViews {
        headline,
        active_count_clip,
        active_count,
        active_count_next,
        slash,
        total_count,
    } = create_compact_bar_views(
        mtm,
        pill_size,
        screen_has_camera_housing(&screen),
        &text_primary,
        &accent_active,
    );

    assemble_native_panel_views(NativePanelAssemblyViews {
        content_view: &content_view,
        left_shoulder: &left_shoulder,
        right_shoulder: &right_shoulder,
        pill_view: &pill_view,
        expanded_container: &expanded_container,
        top_highlight: &top_highlight,
        body_separator: &body_separator,
        mascot_shell: &mascot_shell,
        headline: &headline,
        active_count_clip: &active_count_clip,
        slash: &slash,
        total_count: &total_count,
    });

    unsafe {
        panel_window::configure_native_panel_window(&panel, &content_view, frame);
    }

    initialize_native_panel_handles(NativePanelHandleViews {
        panel: &panel,
        content_view: &content_view,
        left_shoulder: &left_shoulder,
        right_shoulder: &right_shoulder,
        pill_view: &pill_view,
        expanded_container: &expanded_container,
        cards_container: &cards_container,
        top_highlight: &top_highlight,
        body_separator: &body_separator,
        mascot_shell: &mascot_shell,
        mascot_body: &mascot_body,
        mascot_left_eye: &mascot_left_eye,
        mascot_right_eye: &mascot_right_eye,
        mascot_mouth: &mascot_mouth,
        mascot_bubble: &mascot_bubble,
        mascot_sleep_label: &mascot_sleep_label,
        headline: &headline,
        active_count_clip: &active_count_clip,
        active_count: &active_count,
        active_count_next: &active_count_next,
        slash: &slash,
        total_count: &total_count,
    });
    initialize_native_panel_state();
    initialize_active_count_scroll_text();

    info!(
        panel_x = frame.origin.x,
        panel_y = frame.origin.y,
        panel_width = frame.size.width,
        panel_height = frame.size.height,
        screen_height = screen_frame.size.height,
        "created native macOS island panel"
    );

    let _: &'static mut _ = Box::leak(Box::new(panel));

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn native_ui_enabled() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn create_native_island_panel() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn hide_native_island_panel<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let _ = macos_shared_expanded_window::hide_shared_expanded_window(app);

    app.run_on_main_thread(move || {
        if let Some(handles) = native_panel_handles() {
            unsafe {
                panel_from_ptr(handles.panel).orderOut(None);
            }
        }
    })
    .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
pub fn spawn_native_snapshot_loop<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    runtime: AppRuntime,
) {
    tauri::async_runtime::spawn(async move {
        sync_native_snapshot_once(&app, &runtime).await;

        let mut interval = tokio::time::interval(Duration::from_millis(1500));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            sync_native_snapshot_once(&app, &runtime).await;
        }
    });
}

#[cfg(target_os = "macos")]
pub fn spawn_native_hover_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(HOVER_POLL_MS));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if !native_ui_enabled() {
                continue;
            }

            let app_for_hover = app.clone();
            let _ = app.run_on_main_thread(move || unsafe {
                sync_hover_state_on_main_thread(app_for_hover);
            });
        }
    });
}

#[cfg(target_os = "macos")]
pub fn spawn_native_count_marquee_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(
            ACTIVE_COUNT_SCROLL_REFRESH_MS.min(MASCOT_ANIMATION_REFRESH_MS),
        ));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if !native_ui_enabled() {
                continue;
            }

            let _ = app.run_on_main_thread(move || unsafe {
                let Some(handles) = native_panel_handles() else {
                    return;
                };
                let refs = resolve_native_panel_refs(handles);
                sync_active_count_marquee(&refs);
                sync_native_mascot(handles);
                panel_from_ptr(handles.panel).displayIfNeeded();
            });
        }
    });
}

#[cfg(target_os = "macos")]
pub fn spawn_native_status_queue_loop<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(STATUS_QUEUE_REFRESH_MS));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            if !native_ui_enabled() {
                continue;
            }

            let snapshot = native_panel_state().and_then(|state| {
                state.lock().ok().and_then(|guard| {
                    if guard.status_queue.is_empty()
                        && guard.pending_permission_card.is_none()
                        && guard.pending_question_card.is_none()
                    {
                        None
                    } else {
                        guard.last_raw_snapshot.clone()
                    }
                })
            });
            let Some(snapshot) = snapshot else {
                continue;
            };

            if let Err(error) = update_native_island_snapshot(&app, &snapshot) {
                warn!(error = %error, "failed to refresh native macOS status queue animation");
            }
        }
    });
}

#[cfg(target_os = "macos")]
pub fn hide_main_webview_window<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.hide().map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn native_panel_content_visibility() -> f64 {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                if guard.transitioning {
                    card_content_visibility_phase(
                        guard.transition_cards_progress,
                        guard.transition_cards_entering,
                    )
                } else if guard.expanded {
                    1.0
                } else {
                    0.0
                }
            })
        })
        .unwrap_or(0.0)
}

#[cfg(target_os = "macos")]
fn card_chat_body_width(card_width: f64) -> f64 {
    (card_width - (CARD_INSET_X * 2.0) - CARD_CHAT_PREFIX_WIDTH).max(1.0)
}

#[cfg(target_os = "macos")]
fn estimated_chat_body_height(body: &str, width: f64, max_lines: isize) -> f64 {
    estimated_chat_line_count(body, width, max_lines) as f64 * CARD_CHAT_LINE_HEIGHT
}

#[cfg(target_os = "macos")]
fn estimated_chat_line_count(body: &str, width: f64, max_lines: isize) -> isize {
    let max_lines = max_lines.max(1);
    let line_count = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                1
            } else {
                (estimated_text_width(trimmed, 10.0) / width.max(1.0)).ceil() as isize
            }
        })
        .sum::<isize>()
        .max(1);
    line_count.min(max_lines)
}

#[cfg(target_os = "macos")]
fn estimated_text_width(text: &str, font_size: f64) -> f64 {
    text.chars()
        .map(|ch| {
            let factor = if ch.is_ascii_whitespace() {
                0.34
            } else if ch.is_ascii_uppercase() {
                0.66
            } else if ch.is_ascii_punctuation() {
                0.42
            } else if ch.is_ascii() {
                0.60
            } else {
                1.0
            };
            factor * font_size
        })
        .sum::<f64>()
        .max(font_size)
}

#[cfg(target_os = "macos")]
fn estimated_default_chat_body_width() -> f64 {
    card_chat_body_width(expanded_cards_width(DEFAULT_PANEL_CANVAS_WIDTH))
}

#[cfg(target_os = "macos")]
fn summarize_headline(snapshot: &RuntimeSnapshot) -> String {
    let status_queue = native_status_queue_surface_items();
    if !status_queue.is_empty() {
        let approval_count = status_queue
            .iter()
            .filter(|item| matches!(&item.payload, NativeStatusQueuePayload::Approval(_)))
            .count();
        if approval_count > 0 {
            return if approval_count > 1 {
                "Approvals waiting".to_string()
            } else {
                "Approval waiting".to_string()
            };
        }

        let completion_count = status_queue
            .iter()
            .filter(|item| matches!(&item.payload, NativeStatusQueuePayload::Completion(_)))
            .count();
        if completion_count > 1 {
            return format!("{completion_count} tasks complete");
        }
        if let Some(NativeStatusQueueItem {
            payload: NativeStatusQueuePayload::Completion(session),
            ..
        }) = status_queue.first()
        {
            return display_snippet(
                session
                    .last_assistant_message
                    .as_deref()
                    .or(session.tool_description.as_deref()),
                30,
            )
            .unwrap_or_else(|| "Task complete".to_string());
        }
    }

    let active_count = compact_active_count_value(snapshot);
    if active_count > 0 {
        format!(
            "{} active task{}",
            active_count,
            if active_count > 1 { "s" } else { "" }
        )
    } else {
        "No active tasks".to_string()
    }
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + ((end - start) * progress.clamp(0.0, 1.0))
}

#[cfg(target_os = "macos")]
fn animation_phase(elapsed_ms: u64, delay_ms: u64, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        return 1.0;
    }

    elapsed_ms.saturating_sub(delay_ms) as f64 / duration_ms as f64
}

#[cfg(target_os = "macos")]
fn ease_in_cubic(progress: f64) -> f64 {
    progress.clamp(0.0, 1.0).powi(3)
}

#[cfg(target_os = "macos")]
fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

#[cfg(target_os = "macos")]
fn status_queue_exit_duration() -> Duration {
    Duration::from_millis(PANEL_CARD_EXIT_MS.max(220) + STATUS_QUEUE_EXIT_EXTRA_MS)
}

#[cfg(target_os = "macos")]
fn status_pill_colors(status: &str, emphasize: bool) -> ([f64; 4], [f64; 4]) {
    if emphasize {
        return ([0.40, 0.87, 0.57, 0.14], [0.40, 0.87, 0.57, 1.0]);
    }

    match status {
        "running" => ([0.30, 0.87, 0.46, 0.14], [0.49, 0.95, 0.64, 1.0]),
        "processing" => ([1.0, 0.74, 0.41, 0.14], [1.0, 0.81, 0.46, 1.0]),
        "waitingapproval" | "waitingquestion" => ([1.0, 0.61, 0.26, 0.16], [1.0, 0.68, 0.40, 1.0]),
        _ => ([1.0, 1.0, 1.0, 0.08], [0.96, 0.97, 0.99, 1.0]),
    }
}

#[cfg(target_os = "macos")]
fn ns_color(rgba: [f64; 4]) -> objc2::rc::Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(rgba[0], rgba[1], rgba[2], rgba[3])
}

#[cfg(target_os = "macos")]
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use std::time::{Duration, Instant};

    use chrono::Utc;
    use echoisland_runtime::{PendingPermissionView, RuntimeSnapshot, SessionSnapshotView};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::{
        HOVER_DELAY_MS, NativeExpandedSurface, NativeHoverTransition, NativeMascotRuntime,
        NativePanelGeometryMetrics, NativePanelState, NativePanelTransitionFrame,
        NativeStatusQueuePayload, PANEL_CLOSE_MORPH_DELAY_MS, PANEL_CLOSE_SHOULDER_DELAY_MS,
        PANEL_MORPH_DELAY_MS, PANEL_SHOULDER_HIDE_MS, PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        card_content_visibility_phase, centered_top_frame, compact_active_count_text,
        panel_transition_canvas_height, resolve_close_transition_frame,
        resolve_native_panel_layout, resolve_open_transition_frame,
        resolve_surface_switch_transition_frame, shared_expanded_content_state, summarize_headline,
        surface_switch_card_progress, sync_native_hover_expansion_state,
        sync_native_pending_card_visibility, sync_native_status_queue,
    };

    fn snapshot(active: usize, total: usize) -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: "Idle".to_string(),
            primary_source: "claude".to_string(),
            active_session_count: active,
            total_session_count: total,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: Vec::new(),
            pending_questions: Vec::new(),
            sessions: Vec::new(),
        }
    }

    fn session(status: &str) -> SessionSnapshotView {
        SessionSnapshotView {
            session_id: "session-1".to_string(),
            source: "claude".to_string(),
            project_name: None,
            cwd: None,
            model: None,
            terminal_app: None,
            terminal_bundle: None,
            host_app: None,
            window_title: None,
            tty: None,
            terminal_pid: None,
            cli_pid: None,
            iterm_session_id: None,
            kitty_window_id: None,
            tmux_env: None,
            tmux_pane: None,
            tmux_client_tty: None,
            status: status.to_string(),
            current_tool: None,
            tool_description: None,
            last_user_prompt: None,
            last_assistant_message: None,
            tool_history_count: 0,
            tool_history: Vec::new(),
            last_activity: Utc::now(),
        }
    }

    fn pending_permission(request_id: &str, session_id: &str) -> PendingPermissionView {
        PendingPermissionView {
            request_id: request_id.to_string(),
            session_id: session_id.to_string(),
            source: "claude".to_string(),
            tool_name: Some("Bash".to_string()),
            tool_description: Some("Run command".to_string()),
            requested_at: Utc::now(),
        }
    }

    fn snapshot_with_permission(request_id: &str, session_id: &str) -> RuntimeSnapshot {
        let mut snapshot = snapshot(1, 1);
        let pending = pending_permission(request_id, session_id);
        snapshot.pending_permission_count = 1;
        snapshot.pending_permission = Some(pending.clone());
        snapshot.pending_permissions = vec![pending];
        snapshot.sessions = vec![session("WaitingApproval")];
        snapshot
    }

    fn panel_state() -> NativePanelState {
        NativePanelState {
            expanded: false,
            transitioning: false,
            transition_cards_progress: 0.0,
            transition_cards_entering: false,
            skip_next_close_card_exit: false,
            last_raw_snapshot: None,
            last_snapshot: None,
            status_queue: Vec::new(),
            pending_permission_card: None,
            pending_question_card: None,
            status_auto_expanded: false,
            surface_mode: NativeExpandedSurface::Default,
            shared_body_height: None,
            pointer_inside_since: None,
            pointer_outside_since: None,
            primary_mouse_down: false,
            last_focus_click: None,
            card_hit_targets: Vec::new(),
            mascot_runtime: NativeMascotRuntime::new(Instant::now()),
        }
    }

    #[test]
    fn summarizes_empty_snapshot() {
        assert_eq!(summarize_headline(&snapshot(0, 0)), "No active tasks");
    }

    #[test]
    fn summarizes_active_snapshot() {
        let mut snapshot = snapshot(2, 5);
        snapshot.sessions = vec![session("Running"), session("Processing")];
        assert_eq!(summarize_headline(&snapshot), "2 active tasks");
    }

    #[test]
    fn formats_empty_active_count_as_single_zero() {
        let mut snapshot = snapshot(0, 1);
        snapshot.sessions = vec![session("Idle")];
        assert_eq!(compact_active_count_text(&snapshot), "0");
    }

    #[test]
    fn resolved_approval_enters_status_queue_exit_instead_of_waiting_for_expiry() {
        let mut state = panel_state();
        let live_snapshot = snapshot_with_permission("request-1", "session-1");

        assert!(sync_native_status_queue(&mut state, &live_snapshot));
        assert_eq!(state.status_queue.len(), 1);
        state.last_raw_snapshot = Some(live_snapshot);

        let empty_snapshot = snapshot(0, 1);
        assert!(!sync_native_status_queue(&mut state, &empty_snapshot));

        assert_eq!(state.status_queue.len(), 1);
        assert!(state.status_queue[0].is_removing);
        assert!(state.status_queue[0].remove_after.is_some());
        assert!(matches!(
            state.status_queue[0].payload,
            NativeStatusQueuePayload::Approval(_)
        ));
    }

    #[test]
    fn pending_card_grace_snapshot_does_not_readd_status_approval() {
        let mut state = panel_state();
        let live_snapshot = snapshot_with_permission("request-1", "session-1");
        let held_snapshot = sync_native_pending_card_visibility(&mut state, &live_snapshot);

        assert_eq!(held_snapshot.pending_permission_count, 1);
        state.last_raw_snapshot = Some(live_snapshot);

        let empty_snapshot = snapshot(0, 1);
        let held_after_resolve = sync_native_pending_card_visibility(&mut state, &empty_snapshot);

        assert_eq!(held_after_resolve.pending_permission_count, 1);
        assert!(!sync_native_status_queue(&mut state, &empty_snapshot));
        assert!(state.status_queue.is_empty());
    }

    #[test]
    fn surface_switch_card_progress_starts_above_zero_for_continuity() {
        assert_eq!(
            surface_switch_card_progress(0, 220),
            PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS
        );
    }

    #[test]
    fn surface_switch_card_progress_reaches_full_visibility() {
        assert_eq!(surface_switch_card_progress(220, 220), 1.0);
        assert_eq!(surface_switch_card_progress(999, 220), 1.0);
        assert_eq!(surface_switch_card_progress(0, 0), 1.0);
    }

    #[test]
    fn entering_content_visibility_waits_for_reveal_delay() {
        assert_eq!(card_content_visibility_phase(0.10, true), 0.0);
        assert_eq!(card_content_visibility_phase(0.18, true), 0.0);
        assert!(card_content_visibility_phase(0.24, true) > 0.0);
    }

    #[test]
    fn exiting_content_visibility_fades_to_zero() {
        assert_eq!(card_content_visibility_phase(0.0, false), 1.0);
        assert!(card_content_visibility_phase(0.30, false) < 1.0);
        assert_eq!(card_content_visibility_phase(1.0, false), 0.0);
    }

    #[test]
    fn shared_content_waits_for_card_content_reveal() {
        let (visible, interactive) =
            shared_expanded_content_state(true, true, 1.0, 1.0, 120.0, false, 0.80);

        assert!(!visible);
        assert!(!interactive);
    }

    #[test]
    fn shared_content_becomes_visible_and_interactive_after_reveal() {
        let (visible, interactive) =
            shared_expanded_content_state(true, true, 1.0, 1.0, 120.0, false, 1.0);

        assert!(visible);
        assert!(interactive);
    }

    #[test]
    fn shared_content_stays_hidden_for_status_surface() {
        let (visible, interactive) =
            shared_expanded_content_state(true, true, 1.0, 1.0, 120.0, true, 1.0);

        assert!(!visible);
        assert!(!interactive);
    }

    #[test]
    fn centered_top_frame_snaps_panel_geometry_to_whole_points() {
        let frame = centered_top_frame(
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1512.0, 982.0)),
            NSSize::new(419.6, 152.4),
        );

        assert_eq!(frame.origin.x.fract(), 0.0);
        assert_eq!(frame.origin.y.fract(), 0.0);
        assert_eq!(frame.size.width, 420.0);
        assert_eq!(frame.size.height, 152.0);
        assert_eq!(frame.origin.y + frame.size.height, 982.0);
    }

    #[test]
    fn transition_canvas_height_uses_max_height_during_animation() {
        assert_eq!(panel_transition_canvas_height(80.0, 164.0), 164.0);
        assert_eq!(panel_transition_canvas_height(196.0, 80.0), 196.0);
        assert_eq!(panel_transition_canvas_height(148.0, 168.0), 168.0);
    }

    #[test]
    fn transition_frame_uses_named_fields_for_progress() {
        let frame = NativePanelTransitionFrame {
            canvas_height: 196.0,
            visible_height: 148.0,
            bar_progress: 0.4,
            height_progress: 0.6,
            shoulder_progress: 0.8,
            drop_progress: 0.3,
            cards_progress: 0.7,
        };

        assert_eq!(frame.canvas_height, 196.0);
        assert_eq!(frame.visible_height, 148.0);
        assert_eq!(frame.bar_progress, 0.4);
        assert_eq!(frame.height_progress, 0.6);
        assert_eq!(frame.shoulder_progress, 0.8);
        assert_eq!(frame.drop_progress, 0.3);
        assert_eq!(frame.cards_progress, 0.7);
    }

    #[test]
    fn static_transition_frames_match_expected_end_states() {
        let expanded = NativePanelTransitionFrame::expanded(164.0);
        let collapsed = NativePanelTransitionFrame::collapsed(80.0);

        assert_eq!(expanded.canvas_height, 164.0);
        assert_eq!(expanded.visible_height, 164.0);
        assert_eq!(expanded.bar_progress, 1.0);
        assert_eq!(expanded.cards_progress, 1.0);
        assert_eq!(collapsed.canvas_height, 80.0);
        assert_eq!(collapsed.visible_height, 80.0);
        assert_eq!(collapsed.bar_progress, 0.0);
        assert_eq!(collapsed.cards_progress, 0.0);
    }

    #[test]
    fn native_panel_layout_clamps_visible_height_to_canvas() {
        let layout = resolve_native_panel_layout(
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1512.0, 982.0)),
            NativePanelGeometryMetrics {
                compact_height: 38.0,
                compact_width: 126.0,
                expanded_width: 356.0,
                panel_width: 356.0,
            },
            120.0,
            220.0,
            1.0,
            1.0,
            0.0,
            1.0,
        );

        assert_eq!(layout.content_frame.size.height, 120.0);
        assert_eq!(layout.expanded_frame.size.height, 120.0);
        assert!(layout.shell_visible);
        assert_eq!(layout.separator_visibility, 0.88);
    }

    #[test]
    fn open_transition_sampler_starts_collapsed_and_reveals_cards_later() {
        let frame = resolve_open_transition_frame(0, 164.0, 164.0, 220);

        assert_eq!(frame.canvas_height, 164.0);
        assert_eq!(frame.visible_height, 80.0);
        assert_eq!(frame.bar_progress, 0.0);
        assert_eq!(frame.height_progress, 0.0);
        assert_eq!(frame.shoulder_progress, 0.0);
        assert_eq!(frame.cards_progress, 0.0);
    }

    #[test]
    fn open_transition_contracts_shoulders_before_rounding_top_corners() {
        let shoulder_frame =
            resolve_open_transition_frame(PANEL_SHOULDER_HIDE_MS / 2, 164.0, 164.0, 220);
        assert_eq!(shoulder_frame.bar_progress, 0.0);
        assert!(shoulder_frame.shoulder_progress > 0.0);
        assert!(shoulder_frame.shoulder_progress < 1.0);

        let morph_start = resolve_open_transition_frame(PANEL_MORPH_DELAY_MS, 164.0, 164.0, 220);
        assert_eq!(morph_start.bar_progress, 0.0);
        assert_eq!(morph_start.shoulder_progress, 1.0);
    }

    #[test]
    fn surface_switch_sampler_keeps_shell_fully_open() {
        let frame = resolve_surface_switch_transition_frame(0, 164.0, 120.0, 164.0, 220);

        assert_eq!(frame.bar_progress, 1.0);
        assert_eq!(frame.height_progress, 1.0);
        assert_eq!(frame.shoulder_progress, 1.0);
        assert_eq!(frame.drop_progress, 1.0);
        assert_eq!(
            frame.cards_progress,
            PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS
        );
    }

    #[test]
    fn close_transition_sampler_reports_completed_exit_after_delays() {
        let frame = resolve_close_transition_frame(999, 164.0, 164.0, 220, 220);

        assert_eq!(frame.canvas_height, 164.0);
        assert_eq!(frame.visible_height, 80.0);
        assert_eq!(frame.bar_progress, 0.0);
        assert_eq!(frame.height_progress, 0.0);
        assert_eq!(frame.shoulder_progress, 0.0);
        assert_eq!(frame.drop_progress, 0.0);
        assert!(frame.cards_progress >= 1.0);
    }

    #[test]
    fn close_transition_squares_top_corners_before_expanding_shoulders() {
        let close_delay_ms = 220;
        let contracting = resolve_close_transition_frame(
            close_delay_ms + PANEL_CLOSE_MORPH_DELAY_MS + 135,
            164.0,
            164.0,
            close_delay_ms,
            220,
        );
        assert!(contracting.bar_progress > 0.0);
        assert!(contracting.bar_progress < 1.0);
        assert_eq!(contracting.shoulder_progress, 1.0);

        let shoulder_frame = resolve_close_transition_frame(
            close_delay_ms + PANEL_CLOSE_SHOULDER_DELAY_MS + 60,
            164.0,
            164.0,
            close_delay_ms,
            220,
        );
        assert_eq!(shoulder_frame.bar_progress, 0.0);
        assert!(shoulder_frame.shoulder_progress > 0.0);
        assert!(shoulder_frame.shoulder_progress < 1.0);
    }

    #[test]
    fn hover_does_not_collapse_during_transition_even_after_delay() {
        let now = Instant::now();
        let mut state = panel_state();
        state.expanded = true;
        state.transitioning = true;
        state.pointer_outside_since =
            Some(now - Duration::from_millis(HOVER_DELAY_MS.saturating_add(100)));

        let transition = sync_native_hover_expansion_state(&mut state, false, now);

        assert_eq!(transition, None);
        assert!(state.expanded);
    }

    #[test]
    fn hover_collapse_reuses_existing_timer_after_transition_finishes() {
        let now = Instant::now();
        let mut state = panel_state();
        state.expanded = true;
        state.transitioning = false;
        state.pointer_outside_since =
            Some(now - Duration::from_millis(HOVER_DELAY_MS.saturating_add(100)));

        let transition = sync_native_hover_expansion_state(&mut state, false, now);

        assert_eq!(transition, Some(NativeHoverTransition::Collapse));
        assert!(!state.expanded);
    }

    #[test]
    fn status_auto_hover_keeps_live_status_surface_open_outside() {
        let now = Instant::now();
        let mut state = panel_state();
        let live_snapshot = snapshot_with_permission("request-1", "session-1");
        assert!(sync_native_status_queue(&mut state, &live_snapshot));
        state.expanded = true;
        state.status_auto_expanded = true;
        state.surface_mode = NativeExpandedSurface::Status;
        state.pointer_outside_since =
            Some(now - Duration::from_millis(HOVER_DELAY_MS.saturating_add(100)));

        let transition = sync_native_hover_expansion_state(&mut state, false, now);

        assert_eq!(transition, None);
        assert!(state.expanded);
        assert_eq!(state.surface_mode, NativeExpandedSurface::Status);
    }
}
