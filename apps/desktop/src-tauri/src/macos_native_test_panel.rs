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
mod panel_geometry;
#[cfg(target_os = "macos")]
mod panel_refs;
#[cfg(target_os = "macos")]
mod panel_render;
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
use panel_geometry::*;
#[cfg(target_os = "macos")]
use panel_refs::*;
#[cfg(target_os = "macos")]
use panel_render::*;
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

    let screen = NSScreen::mainScreen(mtm)
        .or_else(|| {
            let screens = NSScreen::screens(mtm);
            if screens.is_empty() {
                None
            } else {
                Some(screens.objectAtIndex(0))
            }
        })
        .ok_or_else(|| "failed to resolve a macOS screen".to_string())?;

    let compact_height = compact_pill_height_for_screen(&screen);
    let compact_width = compact_pill_width_for_screen(&screen, compact_height);
    let expanded_width = expanded_panel_width_for_screen(&screen);
    let panel_width = panel_canvas_width_for_screen(&screen, compact_height);
    let size = NSSize::new(panel_width, COLLAPSED_PANEL_HEIGHT);
    let pill_size = NSSize::new(compact_width, compact_height);
    let screen_frame = screen.frame();
    let frame = centered_top_frame(screen_frame, size);

    let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
        NSPanel::alloc(mtm),
        frame,
        style,
        NSBackingStoreType::Buffered,
        false,
    );

    let content_view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), size),
    );
    content_view.setWantsLayer(true);
    let content_layer = CALayer::layer();
    content_layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    content_layer.setMasksToBounds(false);
    content_view.setLayer(Some(&content_layer));
    let pill_frame = island_bar_frame(
        size,
        0.0,
        compact_width,
        expanded_width,
        compact_height,
        0.0,
    );
    let left_shoulder = NSView::initWithFrame(NSView::alloc(mtm), left_shoulder_frame(pill_frame));
    let right_shoulder =
        NSView::initWithFrame(NSView::alloc(mtm), right_shoulder_frame(pill_frame));
    let pill_view = NSView::initWithFrame(NSView::alloc(mtm), pill_frame);
    let expanded_container = NSView::initWithFrame(
        NSView::alloc(mtm),
        expanded_background_frame(
            size,
            COLLAPSED_PANEL_HEIGHT,
            0.0,
            0.0,
            compact_width,
            expanded_width,
            compact_height,
            0.0,
        ),
    );
    expanded_container.setHidden(true);
    let cards_container = NSView::initWithFrame(
        NSView::alloc(mtm),
        expanded_cards_frame(expanded_container.frame(), pill_size.height),
    );
    expanded_container.addSubview(&cards_container);

    let panel_background = NSColor::clearColor();
    let pill_background = NSColor::colorWithSRGBRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0);
    let pill_border = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.055);
    let pill_highlight = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.16);
    let mascot_shell_border = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.08);
    let expanded_background = NSColor::colorWithSRGBRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0);
    let expanded_border = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.0);
    let text_primary = NSColor::colorWithSRGBRed_green_blue_alpha(0.96, 0.97, 0.99, 1.0);
    let accent_active = NSColor::colorWithSRGBRed_green_blue_alpha(0.40, 0.87, 0.57, 1.0);
    let separator_color = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.11);

    apply_shoulder_layer(&left_shoulder, &pill_background, true);
    apply_shoulder_layer(&right_shoulder, &pill_background, false);

    expanded_container.setWantsLayer(true);
    let expanded_layer = CALayer::layer();
    expanded_layer.setCornerRadius(EXPANDED_PANEL_RADIUS);
    expanded_layer.setMasksToBounds(true);
    expanded_layer.setBackgroundColor(Some(&expanded_background.CGColor()));
    expanded_layer.setBorderWidth(0.0);
    expanded_layer.setBorderColor(Some(&expanded_border.CGColor()));
    expanded_container.setLayer(Some(&expanded_layer));

    let top_highlight = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(12.0, pill_size.height - 1.0),
            NSSize::new(pill_size.width - 24.0, 1.0),
        ),
    );
    top_highlight.setWantsLayer(true);
    let top_highlight_layer = CALayer::layer();
    top_highlight_layer.setCornerRadius(0.5);
    top_highlight_layer.setBackgroundColor(Some(&pill_highlight.CGColor()));
    top_highlight_layer.setOpacity(0.0);
    top_highlight.setLayer(Some(&top_highlight_layer));
    top_highlight.setHidden(true);

    let body_separator = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(14.0, 0.0),
            NSSize::new(expanded_width - 28.0, 1.0),
        ),
    );
    body_separator.setWantsLayer(true);
    let body_separator_layer = CALayer::layer();
    body_separator_layer.setCornerRadius(0.5);
    body_separator_layer.setBackgroundColor(Some(&separator_color.CGColor()));
    body_separator_layer.setOpacity(0.0);
    body_separator.setLayer(Some(&body_separator_layer));
    let metrics_trailing = 2.0;
    let metrics_gap = 0.0;
    let active_width = ACTIVE_COUNT_SLOT_WIDTH;
    let slash_width = 10.0;
    let total_width = 24.0;
    let total_x = pill_size.width - metrics_trailing - total_width;
    let slash_x = total_x - metrics_gap - slash_width;
    let active_x = slash_x - metrics_gap - active_width + ACTIVE_COUNT_SLOT_NUDGE_X;

    let mascot_shell = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(10.0, 6.0 + MASCOT_VERTICAL_NUDGE_Y),
            NSSize::new(28.0, 28.0),
        ),
    );
    mascot_shell.setWantsLayer(true);
    let mascot_shell_layer = CALayer::layer();
    mascot_shell_layer.setCornerRadius(7.0);
    mascot_shell_layer.setMasksToBounds(false);
    mascot_shell_layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    mascot_shell_layer.setBorderWidth(0.0);
    mascot_shell_layer.setBorderColor(Some(&mascot_shell_border.CGColor()));
    mascot_shell.setLayer(Some(&mascot_shell_layer));

    let mascot_body_fill = NSColor::colorWithSRGBRed_green_blue_alpha(0.02, 0.02, 0.02, 1.0);
    let mascot_stroke = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.48, 0.14, 1.0);
    let mascot_face = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);

    let mascot_body = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(2.0, 4.0 + MASCOT_VERTICAL_NUDGE_Y),
            NSSize::new(24.0, 20.0),
        ),
    );
    mascot_body.setWantsLayer(true);
    let mascot_body_layer = CALayer::layer();
    mascot_body_layer.setCornerRadius(6.0);
    mascot_body_layer.setMasksToBounds(false);
    mascot_body_layer.setBackgroundColor(Some(&mascot_body_fill.CGColor()));
    mascot_body_layer.setBorderWidth(2.2);
    mascot_body_layer.setBorderColor(Some(&mascot_stroke.CGColor()));
    mascot_body_layer.setShadowColor(Some(&mascot_stroke.CGColor()));
    mascot_body_layer.setShadowOpacity(0.18);
    mascot_body_layer.setShadowRadius(4.0);
    mascot_body.setLayer(Some(&mascot_body_layer));
    mascot_shell.addSubview(&mascot_body);

    let mascot_left_eye = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(7.0, 14.1), NSSize::new(5.7, 4.8)),
    );
    mascot_left_eye.setWantsLayer(true);
    let mascot_left_eye_layer = CALayer::layer();
    mascot_left_eye_layer.setCornerRadius(2.4);
    mascot_left_eye_layer.setMasksToBounds(true);
    mascot_left_eye_layer.setBackgroundColor(Some(&mascot_face.CGColor()));
    mascot_left_eye_layer.setShadowColor(Some(&mascot_face.CGColor()));
    mascot_left_eye_layer.setShadowOpacity(0.22);
    mascot_left_eye_layer.setShadowRadius(6.0);
    mascot_left_eye.setLayer(Some(&mascot_left_eye_layer));
    mascot_shell.addSubview(&mascot_left_eye);

    let mascot_right_eye = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(15.3, 14.1), NSSize::new(5.7, 4.8)),
    );
    mascot_right_eye.setWantsLayer(true);
    let mascot_right_eye_layer = CALayer::layer();
    mascot_right_eye_layer.setCornerRadius(2.4);
    mascot_right_eye_layer.setMasksToBounds(true);
    mascot_right_eye_layer.setBackgroundColor(Some(&mascot_face.CGColor()));
    mascot_right_eye_layer.setShadowColor(Some(&mascot_face.CGColor()));
    mascot_right_eye_layer.setShadowOpacity(0.22);
    mascot_right_eye_layer.setShadowRadius(6.0);
    mascot_right_eye.setLayer(Some(&mascot_right_eye_layer));
    mascot_shell.addSubview(&mascot_right_eye);

    let mascot_mouth = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(10.5, 9.0), NSSize::new(7.0, 2.2)),
    );
    mascot_mouth.setWantsLayer(true);
    let mascot_mouth_layer = CALayer::layer();
    mascot_mouth_layer.setCornerRadius(1.1);
    mascot_mouth_layer.setMasksToBounds(true);
    mascot_mouth_layer.setBackgroundColor(Some(&mascot_face.CGColor()));
    mascot_mouth_layer.setShadowColor(Some(&mascot_face.CGColor()));
    mascot_mouth_layer.setShadowOpacity(0.20);
    mascot_mouth_layer.setShadowRadius(6.0);
    mascot_mouth.setLayer(Some(&mascot_mouth_layer));
    mascot_shell.addSubview(&mascot_mouth);

    let mascot_bubble = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(18.0, 19.5), NSSize::new(14.0, 7.5)),
    );
    mascot_bubble.setWantsLayer(true);
    let mascot_bubble_layer = CALayer::layer();
    mascot_bubble_layer.setCornerRadius(3.7);
    mascot_bubble_layer.setMasksToBounds(true);
    mascot_bubble_layer.setBackgroundColor(Some(&mascot_face.CGColor()));
    mascot_bubble_layer.setShadowColor(Some(&mascot_face.CGColor()));
    mascot_bubble_layer.setShadowOpacity(0.24);
    mascot_bubble_layer.setShadowRadius(7.0);
    mascot_bubble.setLayer(Some(&mascot_bubble_layer));
    mascot_bubble.setHidden(true);
    for x in [3.6, 6.8, 10.0] {
        let dot = NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(NSPoint::new(x, 3.0), NSSize::new(1.6, 1.6)),
        );
        dot.setWantsLayer(true);
        let dot_layer = CALayer::layer();
        dot_layer.setCornerRadius(0.8);
        dot_layer.setMasksToBounds(true);
        dot_layer.setBackgroundColor(Some(
            &NSColor::colorWithSRGBRed_green_blue_alpha(0.02, 0.02, 0.02, 0.72).CGColor(),
        ));
        dot.setLayer(Some(&dot_layer));
        mascot_bubble.addSubview(&dot);
    }
    mascot_shell.addSubview(&mascot_bubble);

    let mascot_sleep_label = NSTextField::labelWithString(ns_string!("Z"), mtm);
    mascot_sleep_label.setFrame(NSRect::new(
        NSPoint::new(20.0, 18.0),
        NSSize::new(10.0, 10.0),
    ));
    mascot_sleep_label.setAlignment(NSTextAlignment::Center);
    mascot_sleep_label.setTextColor(Some(&mascot_face));
    mascot_sleep_label.setFont(Some(&NSFont::boldSystemFontOfSize(8.0)));
    mascot_sleep_label.setDrawsBackground(false);
    mascot_sleep_label.setBezeled(false);
    mascot_sleep_label.setBordered(false);
    mascot_sleep_label.setEditable(false);
    mascot_sleep_label.setSelectable(false);
    mascot_sleep_label.setHidden(true);
    mascot_shell.addSubview(&mascot_sleep_label);

    let headline = NSTextField::labelWithString(ns_string!("No active tasks"), mtm);
    headline.setFrame(NSRect::new(
        NSPoint::new(44.0, compact_headline_y(pill_size.height)),
        NSSize::new(136.0, COMPACT_HEADLINE_LABEL_HEIGHT),
    ));
    headline.setAlignment(NSTextAlignment::Center);
    headline.setTextColor(Some(&text_primary));
    headline.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    headline.setDrawsBackground(false);
    headline.setBezeled(false);
    headline.setBordered(false);
    headline.setEditable(false);
    headline.setSelectable(false);
    headline.setHidden(screen_has_camera_housing(&screen));

    let active_count_clip = NSClipView::initWithFrame(
        NSClipView::alloc(mtm),
        NSRect::new(
            NSPoint::new(
                active_x,
                ((pill_size.height - ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 0.5,
            ),
            NSSize::new(active_width, ACTIVE_COUNT_LABEL_HEIGHT),
        ),
    );
    active_count_clip.setDrawsBackground(false);
    active_count_clip.setBackgroundColor(&NSColor::clearColor());

    let active_count_doc = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(
                ACTIVE_COUNT_SCROLL_TRAVEL + ACTIVE_COUNT_TEXT_WIDTH,
                ACTIVE_COUNT_LABEL_HEIGHT,
            ),
        ),
    );

    let active_count = NSTextField::labelWithString(ns_string!("1"), mtm);
    active_count.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    active_count.setAlignment(NSTextAlignment::Right);
    active_count.setTextColor(Some(&accent_active));
    active_count.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        15.0,
        unsafe { objc2_app_kit::NSFontWeightSemibold },
    )));
    active_count.setDrawsBackground(false);
    active_count.setBezeled(false);
    active_count.setBordered(false);
    active_count.setEditable(false);
    active_count.setSelectable(false);
    active_count.setWantsLayer(true);
    active_count_doc.addSubview(&active_count);

    let active_count_next = NSTextField::labelWithString(ns_string!("2"), mtm);
    active_count_next.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_SCROLL_TRAVEL + ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    active_count_next.setAlignment(NSTextAlignment::Right);
    active_count_next.setTextColor(Some(&accent_active));
    active_count_next.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        15.0,
        unsafe { objc2_app_kit::NSFontWeightSemibold },
    )));
    active_count_next.setDrawsBackground(false);
    active_count_next.setBezeled(false);
    active_count_next.setBordered(false);
    active_count_next.setEditable(false);
    active_count_next.setSelectable(false);
    active_count_next.setWantsLayer(true);
    active_count_doc.addSubview(&active_count_next);
    active_count_clip.setDocumentView(Some(&active_count_doc));

    let slash = NSTextField::labelWithString(ns_string!("/"), mtm);
    slash.setFrame(NSRect::new(
        NSPoint::new(
            slash_x,
            ((pill_size.height - ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 0.5,
        ),
        NSSize::new(slash_width, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    slash.setAlignment(NSTextAlignment::Center);
    slash.setTextColor(Some(&text_primary));
    slash.setFont(Some(&NSFont::systemFontOfSize_weight(15.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    slash.setDrawsBackground(false);
    slash.setBezeled(false);
    slash.setBordered(false);
    slash.setEditable(false);
    slash.setSelectable(false);

    let total_count = NSTextField::labelWithString(ns_string!("99"), mtm);
    total_count.setFrame(NSRect::new(
        NSPoint::new(total_x, ((pill_size.height - 24.0) / 2.0).round() - 0.5),
        NSSize::new(total_width, 24.0),
    ));
    total_count.setAlignment(NSTextAlignment::Left);
    total_count.setTextColor(Some(&text_primary));
    total_count.setFont(Some(&NSFont::systemFontOfSize_weight(15.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    total_count.setDrawsBackground(false);
    total_count.setBezeled(false);
    total_count.setBordered(false);
    total_count.setEditable(false);
    total_count.setSelectable(false);

    pill_view.setWantsLayer(true);
    let pill_layer = CALayer::layer();
    pill_layer.setCornerRadius(COMPACT_PILL_RADIUS);
    pill_layer.setMasksToBounds(true);
    pill_layer.setMaskedCorners(compact_pill_corner_mask());
    pill_layer.setBackgroundColor(Some(&pill_background.CGColor()));
    pill_layer.setBorderWidth(1.0);
    pill_layer.setBorderColor(Some(&pill_border.CGColor()));
    pill_view.setLayer(Some(&pill_layer));
    pill_view.addSubview(&top_highlight);
    pill_view.addSubview(&mascot_shell);
    pill_view.addSubview(&headline);
    pill_view.addSubview(&active_count_clip);
    pill_view.addSubview(&slash);
    pill_view.addSubview(&total_count);
    content_view.addSubview(&expanded_container);
    expanded_container.addSubview(&body_separator);
    content_view.addSubview(&left_shoulder);
    content_view.addSubview(&right_shoulder);
    content_view.addSubview(&pill_view);

    unsafe {
        panel.setReleasedWhenClosed(false);
    }
    panel.setFloatingPanel(true);
    panel.setBecomesKeyOnlyIfNeeded(false);
    panel.setWorksWhenModal(true);
    panel.setLevel(26);
    panel.setBackgroundColor(Some(&panel_background));
    panel.setOpaque(false);
    panel.setHasShadow(false);
    panel.setAnimationBehavior(NSWindowAnimationBehavior::None);
    panel.setMovableByWindowBackground(false);
    panel.setHidesOnDeactivate(false);
    panel.setAcceptsMouseMovedEvents(true);
    panel.setIgnoresMouseEvents(true);
    panel.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    panel.setContentView(Some(&content_view));
    panel.setFrame_display(frame, true);
    panel.orderFrontRegardless();
    panel.displayIfNeeded();

    let _ = NATIVE_TEST_PANEL_HANDLES.set(NativePanelHandles {
        panel: (&*panel as *const NSPanel) as usize,
        content_view: (&*content_view as *const NSView) as usize,
        left_shoulder: (&*left_shoulder as *const NSView) as usize,
        right_shoulder: (&*right_shoulder as *const NSView) as usize,
        pill_view: (&*pill_view as *const NSView) as usize,
        expanded_container: (&*expanded_container as *const NSView) as usize,
        cards_container: (&*cards_container as *const NSView) as usize,
        top_highlight: (&*top_highlight as *const NSView) as usize,
        body_separator: (&*body_separator as *const NSView) as usize,
        mascot_shell: (&*mascot_shell as *const NSView) as usize,
        mascot_body: (&*mascot_body as *const NSView) as usize,
        mascot_left_eye: (&*mascot_left_eye as *const NSView) as usize,
        mascot_right_eye: (&*mascot_right_eye as *const NSView) as usize,
        mascot_mouth: (&*mascot_mouth as *const NSView) as usize,
        mascot_bubble: (&*mascot_bubble as *const NSView) as usize,
        mascot_sleep_label: (&*mascot_sleep_label as *const NSTextField) as usize,
        headline: (&*headline as *const NSTextField) as usize,
        active_count_clip: (&*active_count_clip as *const NSClipView) as usize,
        active_count: (&*active_count as *const NSTextField) as usize,
        active_count_next: (&*active_count_next as *const NSTextField) as usize,
        slash: (&*slash as *const NSTextField) as usize,
        total_count: (&*total_count as *const NSTextField) as usize,
    });
    let _ = NATIVE_TEST_PANEL_STATE.set(Mutex::new(NativePanelState {
        expanded: false,
        transitioning: false,
        transition_cards_progress: 1.0,
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
    }));
    let _ = ACTIVE_COUNT_SCROLL_TEXT.set(Mutex::new("0".to_string()));

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
pub fn update_native_island_snapshot<R: tauri::Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let Some(handles) = native_panel_handles() else {
        return Ok(());
    };

    let raw_snapshot = snapshot.clone();
    let snapshot = {
        let Some(state) = native_panel_state() else {
            return Ok(());
        };
        let mut state = state
            .lock()
            .map_err(|_| "native panel state poisoned".to_string())?;
        sync_native_pending_card_visibility(&mut state, &raw_snapshot)
    };
    if macos_shared_expanded_window::shared_expanded_enabled() {
        if let Err(error) =
            macos_shared_expanded_window::sync_shared_expanded_snapshot(app, &snapshot)
        {
            warn!(error = %error, "failed to sync shared expanded snapshot");
        }
    }
    let mut transition_snapshot: Option<(RuntimeSnapshot, bool)> = None;
    let mut surface_transition_snapshot: Option<RuntimeSnapshot> = None;
    let (
        expanded,
        shared_body_height,
        transitioning,
        transition_cards_progress,
        transition_cards_entering,
    ) = {
        let Some(state) = native_panel_state() else {
            return Ok(());
        };
        let mut state = state
            .lock()
            .map_err(|_| "native panel state poisoned".to_string())?;
        let was_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        let status_queue_added = sync_native_status_queue(&mut state, &raw_snapshot);
        if status_queue_added && !state.expanded && !state.transitioning {
            state.expanded = true;
            state.status_auto_expanded = true;
            state.surface_mode = NativeExpandedSurface::Status;
            transition_snapshot = Some((snapshot.clone(), true));
        } else if status_queue_added
            && state.expanded
            && !state.transitioning
            && state.pointer_inside_since.is_none()
        {
            state.status_auto_expanded = true;
            state.surface_mode = NativeExpandedSurface::Status;
        } else if state.status_auto_expanded
            && state.status_queue.is_empty()
            && state.expanded
            && !state.transitioning
            && state.pointer_inside_since.is_none()
        {
            state.expanded = false;
            state.status_auto_expanded = false;
            state.surface_mode = NativeExpandedSurface::Default;
            state.skip_next_close_card_exit = true;
            transition_snapshot = Some((snapshot.clone(), false));
        } else if state.status_queue.is_empty()
            && state.surface_mode == NativeExpandedSurface::Status
        {
            state.surface_mode = NativeExpandedSurface::Default;
            state.status_auto_expanded = false;
        }
        let is_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        if was_status_surface != is_status_surface && state.expanded && !state.transitioning {
            surface_transition_snapshot = Some(snapshot.clone());
        }
        state.last_raw_snapshot = Some(raw_snapshot.clone());
        state.last_snapshot = Some(snapshot.clone());
        (
            state.expanded,
            state.shared_body_height,
            state.transitioning,
            state.transition_cards_progress,
            state.transition_cards_entering,
        )
    };

    if let Some((transition_snapshot, expanded)) = transition_snapshot {
        let app_for_transition = app.clone();
        return app
            .run_on_main_thread(move || unsafe {
                begin_native_panel_transition(
                    app_for_transition,
                    handles,
                    transition_snapshot,
                    expanded,
                );
            })
            .map_err(|error| error.to_string());
    }

    if let Some(snapshot) = surface_transition_snapshot {
        let app_for_transition = app.clone();
        return app
            .run_on_main_thread(move || unsafe {
                begin_native_panel_surface_transition(app_for_transition, handles, snapshot);
            })
            .map_err(|error| error.to_string());
    }

    app.run_on_main_thread(move || unsafe {
        apply_snapshot_to_panel(
            handles,
            &snapshot,
            expanded,
            shared_body_height,
            transitioning,
            transition_cards_progress,
            transition_cards_entering,
        );
    })
    .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
pub fn set_shared_expanded_body_height<R: tauri::Runtime>(
    app: &AppHandle<R>,
    body_height: f64,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    let Some(handles) = native_panel_handles() else {
        return Ok(());
    };
    let Some(state_mutex) = native_panel_state() else {
        return Ok(());
    };

    let rerender_payload = {
        let mut state = state_mutex
            .lock()
            .map_err(|_| "native panel state poisoned".to_string())?;
        let next_height = body_height.max(0.0);
        if state
            .shared_body_height
            .is_some_and(|current| (current - next_height).abs() < 1.0)
        {
            return Ok(());
        }
        state.shared_body_height = Some(next_height);
        state.last_snapshot.clone().map(|snapshot| {
            (
                snapshot,
                state.expanded,
                state.shared_body_height,
                state.transitioning,
                state.transition_cards_progress,
                state.transition_cards_entering,
            )
        })
    };

    if let Some((
        snapshot,
        expanded,
        shared_body_height,
        transitioning,
        transition_cards_progress,
        transition_cards_entering,
    )) = rerender_payload
    {
        app.run_on_main_thread(move || unsafe {
            apply_snapshot_to_panel(
                handles,
                &snapshot,
                expanded,
                shared_body_height,
                transitioning,
                transition_cards_progress,
                transition_cards_entering,
            );
        })
        .map_err(|error| error.to_string())?;
    }

    Ok(())
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
async fn sync_native_snapshot_once<R: tauri::Runtime>(app: &AppHandle<R>, runtime: &AppRuntime) {
    let raw_snapshot = runtime.runtime.snapshot().await;
    if raw_snapshot.pending_permission_count > 0 || raw_snapshot.pending_question_count > 0 {
        warn!(
            active_session_count = raw_snapshot.active_session_count,
            pending_permission_count = raw_snapshot.pending_permission_count,
            pending_question_count = raw_snapshot.pending_question_count,
            "native snapshot loop observed pending items"
        );
    }
    if raw_snapshot.active_session_count > 0 {
        if let Err(error) = TerminalFocusService::new(runtime)
            .sync_snapshot_focus_bindings(&raw_snapshot)
            .await
        {
            warn!(error = %error, "failed to sync focus bindings during native snapshot refresh");
        }
    }

    if let Err(error) = update_native_island_snapshot(app, &raw_snapshot) {
        warn!(error = %error, "failed to update native macOS island panel");
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sync_hover_state_on_main_thread<R: tauri::Runtime + 'static>(app: AppHandle<R>) {
    let Some(handles) = native_panel_handles() else {
        return;
    };
    let Some(state_mutex) = native_panel_state() else {
        return;
    };

    let refs = resolve_native_panel_refs(handles);
    sync_active_count_marquee(&refs);

    let panel = panel_from_ptr(handles.panel);
    let mouse = NSEvent::mouseLocation();
    let primary_mouse_down = (NSEvent::pressedMouseButtons() & 1) != 0;
    let panel_frame = panel.frame();
    let pill_frame = absolute_rect(panel_frame, compact_pill_frame(panel, panel_frame.size));
    let expanded_container = view_from_ptr(handles.expanded_container);
    let cards_container = view_from_ptr(handles.cards_container);
    let inside = if panel_frame.size.height > COLLAPSED_PANEL_HEIGHT + 0.5 {
        point_in_rect(
            mouse,
            absolute_rect(panel_frame, expanded_container.frame()),
        )
    } else {
        point_in_rect(mouse, pill_frame)
    };
    panel.setIgnoresMouseEvents(!inside);

    let now = Instant::now();
    let mut transition_snapshot: Option<(RuntimeSnapshot, bool)> = None;
    let mut surface_transition_snapshot: Option<RuntimeSnapshot> = None;
    let mut clicked_session_id: Option<String> = None;

    {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if primary_mouse_down
            && !state.primary_mouse_down
            && inside
            && state.expanded
            && !state.transitioning
            && !cards_container.isHidden()
        {
            clicked_session_id = find_clicked_card_session_id(
                &state.card_hit_targets,
                panel_frame,
                expanded_container.frame(),
                cards_container.frame(),
                mouse,
            );
            if let Some(session_id) = clicked_session_id.as_ref() {
                let suppressed =
                    state
                        .last_focus_click
                        .as_ref()
                        .is_some_and(|(last_session_id, last_at)| {
                            last_session_id == session_id
                                && now.duration_since(*last_at).as_millis()
                                    < CARD_FOCUS_CLICK_DEBOUNCE_MS
                        });
                if suppressed {
                    clicked_session_id = None;
                } else {
                    state.last_focus_click = Some((session_id.clone(), now));
                }
            }
        }
        state.primary_mouse_down = primary_mouse_down;
        let was_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();

        if let Some(hover_transition) = sync_native_hover_expansion_state(&mut state, inside, now) {
            if let Some(snapshot) = state.last_snapshot.clone() {
                transition_snapshot =
                    Some((snapshot, hover_transition == NativeHoverTransition::Expand));
            }
        }

        let is_status_surface =
            state.surface_mode == NativeExpandedSurface::Status && !state.status_queue.is_empty();
        if was_status_surface != is_status_surface && state.expanded && !state.transitioning {
            if let Some(snapshot) = state.last_snapshot.clone() {
                surface_transition_snapshot = Some(snapshot);
            }
        }
    }

    if let Some((snapshot, expanded)) = transition_snapshot {
        begin_native_panel_transition(app.clone(), handles, snapshot, expanded);
    } else if let Some(snapshot) = surface_transition_snapshot {
        begin_native_panel_surface_transition(app.clone(), handles, snapshot);
    }

    if let Some(session_id) = clicked_session_id {
        spawn_native_focus_session(app, session_id);
    }
}

#[cfg(target_os = "macos")]
fn sync_native_hover_expansion_state(
    state: &mut NativePanelState,
    inside: bool,
    now: Instant,
) -> Option<NativeHoverTransition> {
    if inside {
        state.pointer_outside_since = None;
        state.pointer_inside_since.get_or_insert(now);
        if !state.expanded
            && !state.transitioning
            && state.pointer_inside_since.is_some_and(|entered_at| {
                now.duration_since(entered_at).as_millis() >= HOVER_DELAY_MS as u128
            })
        {
            state.expanded = true;
            state.status_auto_expanded = false;
            state.surface_mode = NativeExpandedSurface::Default;
            return Some(NativeHoverTransition::Expand);
        }
    } else {
        state.pointer_inside_since = None;
        state.pointer_outside_since.get_or_insert(now);
        let keep_open_for_status = state.status_auto_expanded
            && state.surface_mode == NativeExpandedSurface::Status
            && !state.status_queue.is_empty();
        if state.expanded
            && !state.transitioning
            && !keep_open_for_status
            && state.pointer_outside_since.is_some_and(|left_at| {
                now.duration_since(left_at).as_millis() >= HOVER_DELAY_MS as u128
            })
        {
            state.expanded = false;
            state.status_auto_expanded = false;
            state.surface_mode = NativeExpandedSurface::Default;
            return Some(NativeHoverTransition::Collapse);
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn native_status_surface_active() -> bool {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                guard.surface_mode == NativeExpandedSurface::Status
                    && !guard.status_queue.is_empty()
            })
        })
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn replace_native_card_hit_targets(targets: Vec<NativeCardHitTarget>) {
    if let Some(state) = native_panel_state() {
        if let Ok(mut guard) = state.lock() {
            guard.card_hit_targets = targets;
        }
    }
}

#[cfg(target_os = "macos")]
fn find_clicked_card_session_id(
    targets: &[NativeCardHitTarget],
    panel_frame: NSRect,
    expanded_frame: NSRect,
    cards_frame: NSRect,
    mouse: NSPoint,
) -> Option<String> {
    targets
        .iter()
        .find(|target| {
            point_in_rect(
                mouse,
                absolute_rect(
                    panel_frame,
                    compose_local_rect(
                        expanded_frame,
                        compose_local_rect(cards_frame, target.frame),
                    ),
                ),
            )
        })
        .map(|target| target.session_id.clone())
}

#[cfg(target_os = "macos")]
fn spawn_native_focus_session<R: tauri::Runtime + 'static>(app: AppHandle<R>, session_id: String) {
    let runtime = app.state::<AppRuntime>().inner().clone();
    tauri::async_runtime::spawn(async move {
        match TerminalFocusService::new(&runtime)
            .focus_session(&session_id)
            .await
        {
            Ok(true) => {
                info!(session_id = %session_id, "native card click focused terminal session");
            }
            Ok(false) => {
                warn!(
                    session_id = %session_id,
                    "native card click did not find a focusable terminal target"
                );
            }
            Err(error) => {
                warn!(
                    session_id = %session_id,
                    error = %error,
                    "native card click failed to focus terminal session"
                );
            }
        }
    });
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_snapshot_to_panel(
    handles: NativePanelHandles,
    snapshot: &RuntimeSnapshot,
    expanded: bool,
    shared_body_height: Option<f64>,
    transitioning: bool,
    transition_cards_progress: f64,
    transition_cards_entering: bool,
) {
    apply_snapshot_values_to_panel(handles, snapshot);
    let context = resolve_native_transition_context(handles);
    let panel = context.refs.panel;

    if transitioning {
        if expanded {
            if context.refs.cards_container.subviews().is_empty() {
                render_transition_cards(context, snapshot);
            }
            apply_card_stack_transition(
                context.refs.cards_container,
                transition_cards_progress,
                transition_cards_entering,
            );
            context.refs.panel.displayIfNeeded();
        }
        return;
    }

    let total_height = if expanded {
        let shared_body_height = if macos_shared_expanded_window::shared_expanded_enabled()
            && !native_status_surface_active()
        {
            shared_body_height
        } else {
            None
        };
        expanded_total_height(
            snapshot,
            compact_pill_height_for_screen_rect(panel.screen().as_deref(), context.screen_frame),
            shared_body_height,
        )
    } else {
        COLLAPSED_PANEL_HEIGHT
    };
    if expanded {
        apply_panel_geometry(handles, NativePanelTransitionFrame::expanded(total_height));
    } else {
        apply_panel_geometry(handles, NativePanelTransitionFrame::collapsed(total_height));
    }

    if expanded {
        render_transition_cards(context, snapshot);
    } else {
        reset_collapsed_cards(context);
    }

    context.refs.panel.displayIfNeeded();
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn begin_native_panel_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    snapshot: RuntimeSnapshot,
    expanded: bool,
) {
    let animation_id = NATIVE_TEST_PANEL_ANIMATION_ID.fetch_add(1, Ordering::SeqCst) + 1;
    apply_snapshot_values_to_panel(handles, &snapshot);
    let skip_close_card_exit = take_skip_close_card_exit_and_begin_transition(expanded);

    let context = resolve_native_transition_context(handles);
    let panel = context.refs.panel;

    let start_height = panel.frame().size.height;
    let target_height = if expanded {
        resolved_expanded_target_height(context, &snapshot)
    } else {
        COLLAPSED_PANEL_HEIGHT
    };

    if expanded {
        let card_count = prepare_open_transition(context, &snapshot);
        set_transition_cards_state(0.0, true);
        tauri::async_runtime::spawn(async move {
            animate_open_transition(
                app.clone(),
                handles,
                animation_id,
                start_height,
                target_height,
                card_count,
            )
            .await;

            let _ = app.run_on_main_thread(move || unsafe {
                if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                    return;
                }

                finish_transition_state(1.0, true);
                let context = resolve_native_transition_context(handles);
                finalize_open_transition(handles, context, &snapshot, target_height);
            });
        });
    } else {
        let card_count = prepare_close_transition(context, skip_close_card_exit);
        set_transition_cards_state(0.0, false);

        tauri::async_runtime::spawn(async move {
            animate_close_transition(app.clone(), handles, animation_id, start_height, card_count)
                .await;

            let _ = app.run_on_main_thread(move || unsafe {
                if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                    return;
                }

                finish_transition_state(0.0, false);
                let context = resolve_native_transition_context(handles);
                finalize_close_transition(handles, context);
            });
        });
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn begin_native_panel_surface_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    snapshot: RuntimeSnapshot,
) {
    let animation_id = NATIVE_TEST_PANEL_ANIMATION_ID.fetch_add(1, Ordering::SeqCst) + 1;
    apply_snapshot_values_to_panel(handles, &snapshot);
    let _ = take_skip_close_card_exit_and_begin_transition(true);
    set_transition_cards_state(PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS, true);

    let context = resolve_native_transition_context(handles);
    let panel = context.refs.panel;
    let start_height = panel.frame().size.height;
    let target_height = resolved_expanded_target_height(context, &snapshot);

    let card_count = prepare_surface_switch_transition(context, &snapshot);

    tauri::async_runtime::spawn(async move {
        animate_surface_switch_transition(
            app.clone(),
            handles,
            animation_id,
            start_height,
            target_height,
            card_count,
        )
        .await;

        let _ = app.run_on_main_thread(move || unsafe {
            if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                return;
            }

            finish_transition_state(1.0, true);
            let context = resolve_native_transition_context(handles);
            finalize_surface_switch_transition(handles, context, &snapshot, target_height);
        });
    });
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_snapshot_values_to_panel(handles: NativePanelHandles, snapshot: &RuntimeSnapshot) {
    let refs = resolve_native_panel_refs(handles);
    let headline = refs.headline;
    let active_count = refs.active_count;
    let active_count_next = refs.active_count_next;
    let total_count = refs.total_count;

    let headline_value = NSString::from_str(&summarize_headline(snapshot));
    let active_count_text = compact_active_count_text(snapshot);
    let total_count_text = snapshot.total_session_count.to_string();
    let active_count_value = NSString::from_str(&active_count_text);
    let total_count_value = NSString::from_str(&total_count_text);
    let style = compact_style(snapshot);
    let headline_color = ns_color(style.headline_color);
    let active_count_color = ns_color(style.active_count_color);
    let total_count_color = ns_color(style.total_count_color);

    headline.setStringValue(&headline_value);
    headline.setTextColor(Some(&headline_color));
    headline.setHidden(compact_headline_should_hide(&refs));
    active_count.setTextColor(Some(&active_count_color));
    active_count_next.setTextColor(Some(&active_count_color));
    total_count.setStringValue(&total_count_value);
    total_count.setTextColor(Some(&total_count_color));
    if let Some(source) = ACTIVE_COUNT_SCROLL_TEXT.get() {
        if let Ok(mut value) = source.lock() {
            *value = active_count_text;
        }
    }
    active_count.setStringValue(&active_count_value);
    sync_active_count_marquee(&refs);

    headline.displayIfNeeded();
    refs.active_count_clip.displayIfNeeded();
    active_count.displayIfNeeded();
    active_count_next.displayIfNeeded();
    total_count.displayIfNeeded();
}

#[cfg(target_os = "macos")]
fn with_disabled_layer_actions<T>(f: impl FnOnce() -> T) -> T {
    CATransaction::begin();
    CATransaction::setDisableActions(true);
    let result = f();
    CATransaction::commit();
    result
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
fn compact_pill_corner_mask() -> CACornerMask {
    CACornerMask::LayerMinXMinYCorner | CACornerMask::LayerMaxXMinYCorner
}

#[cfg(target_os = "macos")]
fn all_corner_mask() -> CACornerMask {
    CACornerMask::LayerMinXMinYCorner
        | CACornerMask::LayerMaxXMinYCorner
        | CACornerMask::LayerMinXMaxYCorner
        | CACornerMask::LayerMaxXMaxYCorner
}

fn apply_shoulder_layer(view: &NSView, background: &NSColor, align_right: bool) {
    let size = COMPACT_SHOULDER_SIZE;
    let control = size * SHOULDER_CURVE_FACTOR;
    let path = CGMutablePath::new();

    unsafe {
        if align_right {
            CGMutablePath::move_to_point(Some(path.as_ref()), std::ptr::null(), 0.0, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), size, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), size, 0.0);
            CGMutablePath::add_curve_to_point(
                Some(path.as_ref()),
                std::ptr::null(),
                size,
                control,
                control,
                size,
                0.0,
                size,
            );
            CGMutablePath::close_subpath(Some(path.as_ref()));
        } else {
            CGMutablePath::move_to_point(Some(path.as_ref()), std::ptr::null(), size, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), 0.0, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), 0.0, 0.0);
            CGMutablePath::add_curve_to_point(
                Some(path.as_ref()),
                std::ptr::null(),
                0.0,
                control,
                control,
                size,
                size,
                size,
            );
            CGMutablePath::close_subpath(Some(path.as_ref()));
        }
    }

    let immutable_path = CGPath::new_copy(Some(path.as_ref())).expect("shoulder path copy");
    let shape_layer = CAShapeLayer::layer();
    shape_layer.setMasksToBounds(false);
    shape_layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    shape_layer.setFillColor(Some(&background.CGColor()));
    shape_layer.setPath(Some(immutable_path.as_ref()));
    view.setWantsLayer(true);
    view.setLayer(Some(shape_layer.as_ref()));
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
        NativeStatusQueuePayload, PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
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
        assert_eq!(frame.cards_progress, 0.0);
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
