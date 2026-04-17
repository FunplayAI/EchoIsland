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
#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeMascotState {
    Idle,
    Bouncing,
    Approval,
    Question,
    MessageBubble,
    Sleepy,
    WakeAngry,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativeMascotMotion {
    offset_x: f64,
    offset_y: f64,
    scale_x: f64,
    scale_y: f64,
    shell_alpha: f64,
    shadow_opacity: f32,
    shadow_radius: f64,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativeMascotFrame {
    state: NativeMascotState,
    t: f64,
    motion: NativeMascotMotion,
    color: [f64; 4],
}

#[cfg(target_os = "macos")]
struct NativeMascotRuntime {
    animation_started_at: Instant,
    last_non_idle_at: Instant,
    last_resolved_state: NativeMascotState,
    wake_started_at: Option<Instant>,
    wake_next_state: NativeMascotState,
    transition_target: NativeMascotState,
    transition_started_at: Instant,
    transition_start_motion: NativeMascotMotion,
    last_motion: NativeMascotMotion,
}

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
                sync_active_count_marquee(handles);
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

    sync_active_count_marquee(handles);

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
fn sync_native_pending_card_visibility(
    state: &mut NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> RuntimeSnapshot {
    let now = Instant::now();
    let next_permission = resolve_native_pending_permission_card(
        displayed_pending_permissions(snapshot).into_iter().next(),
        state.pending_permission_card.as_ref(),
        now,
    );
    let next_question = resolve_native_pending_question_card(
        displayed_pending_questions(snapshot).into_iter().next(),
        state.pending_question_card.as_ref(),
        now,
    );

    state.pending_permission_card = next_permission.clone();
    state.pending_question_card = next_question.clone();

    apply_native_pending_cards_to_snapshot(snapshot, next_permission, next_question)
}

#[cfg(target_os = "macos")]
fn resolve_native_pending_permission_card(
    current_payload: Option<PendingPermissionView>,
    previous: Option<&NativePendingPermissionCard>,
    now: Instant,
) -> Option<NativePendingPermissionCard> {
    if let Some(payload) = current_payload {
        let started_at = previous
            .filter(|card| card.request_id == payload.request_id)
            .map(|card| card.started_at)
            .unwrap_or(now);
        return Some(NativePendingPermissionCard {
            request_id: payload.request_id.clone(),
            payload,
            started_at,
            last_seen_at: now,
            visible_until: previous
                .map(|card| card.visible_until)
                .unwrap_or(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS))
                .max(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS)),
        });
    }

    let previous = previous?;
    let keep_visible_until = previous
        .visible_until
        .max(previous.last_seen_at + Duration::from_millis(PENDING_CARD_RELEASE_GRACE_MS));
    if now > keep_visible_until {
        return None;
    }

    Some(NativePendingPermissionCard {
        request_id: previous.request_id.clone(),
        payload: previous.payload.clone(),
        started_at: previous.started_at,
        last_seen_at: previous.last_seen_at,
        visible_until: keep_visible_until,
    })
}

#[cfg(target_os = "macos")]
fn resolve_native_pending_question_card(
    current_payload: Option<PendingQuestionView>,
    previous: Option<&NativePendingQuestionCard>,
    now: Instant,
) -> Option<NativePendingQuestionCard> {
    if let Some(payload) = current_payload {
        let started_at = previous
            .filter(|card| card.request_id == payload.request_id)
            .map(|card| card.started_at)
            .unwrap_or(now);
        return Some(NativePendingQuestionCard {
            request_id: payload.request_id.clone(),
            payload,
            started_at,
            last_seen_at: now,
            visible_until: previous
                .map(|card| card.visible_until)
                .unwrap_or(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS))
                .max(started_at + Duration::from_millis(PENDING_CARD_MIN_VISIBLE_MS)),
        });
    }

    let previous = previous?;
    let keep_visible_until = previous
        .visible_until
        .max(previous.last_seen_at + Duration::from_millis(PENDING_CARD_RELEASE_GRACE_MS));
    if now > keep_visible_until {
        return None;
    }

    Some(NativePendingQuestionCard {
        request_id: previous.request_id.clone(),
        payload: previous.payload.clone(),
        started_at: previous.started_at,
        last_seen_at: previous.last_seen_at,
        visible_until: keep_visible_until,
    })
}

#[cfg(target_os = "macos")]
fn apply_native_pending_cards_to_snapshot(
    snapshot: &RuntimeSnapshot,
    pending_permission_card: Option<NativePendingPermissionCard>,
    pending_question_card: Option<NativePendingQuestionCard>,
) -> RuntimeSnapshot {
    let mut next_snapshot = snapshot.clone();

    if let Some(card) = pending_permission_card {
        let mut permissions = vec![card.payload];
        let held_request_id = permissions[0].request_id.clone();
        permissions.extend(
            displayed_pending_permissions(snapshot)
                .into_iter()
                .filter(|item| item.request_id != held_request_id),
        );
        next_snapshot.pending_permission_count = permissions.len();
        next_snapshot.pending_permission = permissions.first().cloned();
        next_snapshot.pending_permissions = permissions;
    }

    if let Some(card) = pending_question_card {
        let mut questions = vec![card.payload];
        let held_request_id = questions[0].request_id.clone();
        questions.extend(
            displayed_pending_questions(snapshot)
                .into_iter()
                .filter(|item| item.request_id != held_request_id),
        );
        next_snapshot.pending_question_count = questions.len();
        next_snapshot.pending_question = questions.first().cloned();
        next_snapshot.pending_questions = questions;
    }

    next_snapshot
}

#[cfg(target_os = "macos")]
fn sync_native_status_queue(state: &mut NativePanelState, snapshot: &RuntimeSnapshot) -> bool {
    let now = Instant::now();
    let utc_now = Utc::now();
    let previous_snapshot = state.last_raw_snapshot.as_ref();
    let completed_session_ids = previous_snapshot.map_or_else(Vec::new, |previous| {
        detect_completed_sessions(previous, snapshot, utc_now)
    });
    let previous_live_permission_ids = previous_snapshot
        .map(displayed_pending_permissions)
        .unwrap_or_default()
        .into_iter()
        .map(|pending| pending.request_id)
        .collect::<HashSet<_>>();
    let previous_queue_keys = state
        .status_queue
        .iter()
        .map(|item| item.key.clone())
        .collect::<HashSet<_>>();
    let previous_approval_count = state
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Approval(_)))
        .count();
    let previous_completion_count = state
        .status_queue
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)))
        .count();

    let mut existing_items = state
        .status_queue
        .drain(..)
        .filter(|item| {
            if item.is_removing {
                item.remove_after
                    .is_some_and(|remove_after| remove_after > now)
            } else {
                true
            }
        })
        .map(|item| (item.key.clone(), item))
        .collect::<HashMap<_, _>>();
    let mut next_items = Vec::new();
    let mut added = false;

    for pending in displayed_pending_permissions(snapshot) {
        let key = format!("approval:{}", pending.request_id);
        let existing = existing_items.remove(&key);
        let is_new_live_permission = !previous_live_permission_ids.contains(&pending.request_id);
        if let Some(existing_item) = existing.as_ref() {
            if existing_item.is_removing
                && existing_item
                    .remove_after
                    .is_some_and(|remove_after| remove_after > now)
            {
                next_items.push(NativeStatusQueueItem {
                    key,
                    session_id: pending.session_id.clone(),
                    sort_time: pending.requested_at,
                    expires_at: existing_item.expires_at,
                    is_live: false,
                    is_removing: true,
                    remove_after: existing_item.remove_after,
                    payload: NativeStatusQueuePayload::Approval(pending),
                });
                continue;
            }
        }
        if existing.is_none() && !is_new_live_permission {
            continue;
        }
        if existing.is_none() && is_new_live_permission {
            added = true;
        }
        next_items.push(NativeStatusQueueItem {
            key,
            session_id: pending.session_id.clone(),
            sort_time: pending.requested_at,
            expires_at: existing
                .as_ref()
                .map(|item| item.expires_at)
                .unwrap_or_else(|| now + Duration::from_secs(STATUS_APPROVAL_VISIBLE_SECONDS)),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: NativeStatusQueuePayload::Approval(pending),
        });
    }

    for session_id in completed_session_ids {
        let Some(session) = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == session_id)
            .cloned()
        else {
            continue;
        };
        let key = format!("completion:{}", session.session_id);
        let existing = existing_items.remove(&key);
        if existing.is_none() {
            added = true;
        }
        next_items.push(NativeStatusQueueItem {
            key,
            session_id: session.session_id.clone(),
            sort_time: session.last_activity,
            expires_at: existing
                .as_ref()
                .map(|item| item.expires_at)
                .unwrap_or_else(|| now + Duration::from_secs(STATUS_COMPLETION_VISIBLE_SECONDS)),
            is_live: true,
            is_removing: false,
            remove_after: None,
            payload: NativeStatusQueuePayload::Completion(session),
        });
    }

    for mut item in existing_items.into_values() {
        if item.is_removing {
            if item
                .remove_after
                .is_some_and(|remove_after| remove_after > now)
            {
                next_items.push(item);
            }
            continue;
        }

        if now >= item.expires_at {
            item.is_live = false;
            item.is_removing = true;
            item.remove_after = Some(now + status_queue_exit_duration());
            next_items.push(item);
            continue;
        }

        match &mut item.payload {
            NativeStatusQueuePayload::Approval(_) => {
                item.is_live = false;
                item.is_removing = true;
                item.remove_after = Some(now + status_queue_exit_duration());
                next_items.push(item);
            }
            NativeStatusQueuePayload::Completion(session) => {
                if let Some(latest) = snapshot
                    .sessions
                    .iter()
                    .find(|candidate| candidate.session_id == item.session_id)
                {
                    *session = latest.clone();
                    item.sort_time = latest.last_activity;
                }
                item.is_live = false;
                item.is_removing = false;
                item.remove_after = None;
                next_items.push(item);
            }
        }
    }

    next_items.sort_by(compare_native_status_queue_items);
    next_items.retain(|item| {
        if item.is_removing {
            return item
                .remove_after
                .is_some_and(|remove_after| remove_after > now);
        }
        item.expires_at > now
    });
    let next_queue_keys = next_items
        .iter()
        .map(|item| item.key.clone())
        .collect::<HashSet<_>>();
    let next_approval_count = next_items
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Approval(_)))
        .count();
    let next_completion_count = next_items
        .iter()
        .filter(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)))
        .count();
    let displayed_permission_count = displayed_pending_permissions(snapshot).len();
    let displayed_question_count = displayed_pending_questions(snapshot).len();
    if added
        || previous_queue_keys != next_queue_keys
        || previous_approval_count != next_approval_count
        || previous_completion_count != next_completion_count
        || (snapshot.pending_permission_count > 0 && next_approval_count == 0)
    {
        tracing::debug!(
            snapshot_pending_permission_count = snapshot.pending_permission_count,
            snapshot_pending_question_count = snapshot.pending_question_count,
            displayed_permission_count,
            displayed_question_count,
            previous_approval_count,
            previous_completion_count,
            next_approval_count,
            next_completion_count,
            queue_len = next_items.len(),
            expanded = state.expanded,
            status_auto_expanded = state.status_auto_expanded,
            status_surface_active = state.surface_mode == NativeExpandedSurface::Status,
            added,
            "native status queue sync"
        );
    }
    state.status_queue = next_items;
    added
}

#[cfg(target_os = "macos")]
fn detect_completed_sessions(
    previous: &RuntimeSnapshot,
    snapshot: &RuntimeSnapshot,
    now: chrono::DateTime<Utc>,
) -> Vec<String> {
    let previous_by_id = previous
        .sessions
        .iter()
        .map(|session| (&session.session_id, session))
        .collect::<HashMap<_, _>>();
    snapshot
        .sessions
        .iter()
        .filter_map(|session| {
            let previous = previous_by_id.get(&session.session_id)?;
            let previous_status = normalize_status(&previous.status);
            let current_status = normalize_status(&session.status);
            let became_idle_from_active = current_status == "idle"
                && (previous_status == "processing" || previous_status == "running");
            let idle_message_updated = current_status == "idle"
                && previous_status == "idle"
                && (now - session.last_activity).num_seconds() <= 20
                && session
                    .last_assistant_message
                    .as_deref()
                    .is_some_and(|message| !message.trim().is_empty())
                && session.last_assistant_message != previous.last_assistant_message;

            if became_idle_from_active || idle_message_updated {
                Some(session.session_id.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn compare_native_status_queue_items(
    left: &NativeStatusQueueItem,
    right: &NativeStatusQueueItem,
) -> std::cmp::Ordering {
    let left_priority = native_status_queue_priority(left);
    let right_priority = native_status_queue_priority(right);
    right_priority
        .cmp(&left_priority)
        .then_with(|| match (&left.payload, &right.payload) {
            (NativeStatusQueuePayload::Approval(_), NativeStatusQueuePayload::Approval(_)) => {
                left.sort_time.cmp(&right.sort_time)
            }
            _ => right.sort_time.cmp(&left.sort_time),
        })
        .then_with(|| left.session_id.cmp(&right.session_id))
}

#[cfg(target_os = "macos")]
fn native_status_queue_priority(item: &NativeStatusQueueItem) -> u8 {
    match &item.payload {
        NativeStatusQueuePayload::Approval(_) => 2,
        NativeStatusQueuePayload::Completion(_) => 1,
    }
}

#[cfg(target_os = "macos")]
fn native_status_queue_surface_items() -> Vec<NativeStatusQueueItem> {
    native_panel_state()
        .and_then(|state| {
            state.lock().ok().map(|guard| {
                if guard.surface_mode == NativeExpandedSurface::Status
                    && !guard.status_queue.is_empty()
                {
                    guard.status_queue.clone()
                } else {
                    Vec::new()
                }
            })
        })
        .unwrap_or_default()
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
    let panel = panel_from_ptr(handles.panel);
    let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
    let expanded_width =
        expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);

    if transitioning {
        if expanded {
            let cards_container = view_from_ptr(handles.cards_container);
            if cards_container.subviews().is_empty() {
                render_expanded_cards(cards_container, snapshot, expanded_width);
            }
            apply_card_stack_transition(
                cards_container,
                transition_cards_progress,
                transition_cards_entering,
            );
            panel_from_ptr(handles.panel).displayIfNeeded();
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
            compact_pill_height_for_screen_rect(panel.screen().as_deref(), screen_frame),
            shared_body_height,
        )
    } else {
        COLLAPSED_PANEL_HEIGHT
    };
    if expanded {
        apply_panel_geometry(handles, total_height, total_height, 1.0, 1.0, 1.0, 1.0);
    } else {
        apply_panel_geometry(handles, total_height, total_height, 0.0, 0.0, 0.0, 0.0);
    }

    let cards_container = view_from_ptr(handles.cards_container);
    if expanded {
        render_expanded_cards(cards_container, snapshot, expanded_width);
    } else {
        clear_subviews(cards_container);
        cards_container.setFrame(NSRect::new(
            NSPoint::new(EXPANDED_CARDS_SIDE_INSET, 0.0),
            NSSize::new(expanded_cards_width(expanded_width), 0.0),
        ));
    }

    panel_from_ptr(handles.panel).displayIfNeeded();
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
    let mut skip_close_card_exit = false;
    if let Some(state_mutex) = native_panel_state() {
        if let Ok(mut state) = state_mutex.lock() {
            state.transitioning = true;
            if !expanded {
                skip_close_card_exit = state.skip_next_close_card_exit;
                state.skip_next_close_card_exit = false;
            }
        }
    }

    let panel = panel_from_ptr(handles.panel);
    let cards_container = view_from_ptr(handles.cards_container);
    let expanded_container = view_from_ptr(handles.expanded_container);
    let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
    let expanded_width =
        expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);

    let start_height = panel.frame().size.height;
    let target_height = if expanded {
        let shared_body_height = if macos_shared_expanded_window::shared_expanded_enabled()
            && !native_status_surface_active()
        {
            native_panel_state()
                .and_then(|state| state.lock().ok().and_then(|guard| guard.shared_body_height))
        } else {
            None
        };
        expanded_total_height(
            &snapshot,
            compact_pill_height_for_screen_rect(panel.screen().as_deref(), screen_frame),
            shared_body_height,
        )
    } else {
        COLLAPSED_PANEL_HEIGHT
    };

    if expanded {
        render_expanded_cards(cards_container, &snapshot, expanded_width);
        let card_count = cards_container.subviews().len();
        apply_card_stack_transition(cards_container, 0.0, true);
        cards_container.setHidden(false);
        cards_container.setAlphaValue(1.0);
        expanded_container.setHidden(true);
        expanded_container.setAlphaValue(0.0);
        if let Some(state_mutex) = native_panel_state() {
            if let Ok(mut state) = state_mutex.lock() {
                state.transition_cards_progress = 0.0;
                state.transition_cards_entering = true;
            }
        }
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

                if let Some(state_mutex) = native_panel_state() {
                    if let Ok(mut state) = state_mutex.lock() {
                        state.transitioning = false;
                        state.transition_cards_progress = 1.0;
                        state.transition_cards_entering = true;
                    }
                }

                let cards_container = view_from_ptr(handles.cards_container);
                let panel = panel_from_ptr(handles.panel);
                let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
                let expanded_width =
                    expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);
                render_expanded_cards(cards_container, &snapshot, expanded_width);
                cards_container.setHidden(false);
                cards_container.setAlphaValue(1.0);
                apply_card_stack_transition(cards_container, 1.0, true);
                apply_panel_geometry(handles, target_height, target_height, 1.0, 1.0, 1.0, 1.0);
                panel_from_ptr(handles.panel).displayIfNeeded();
            });
        });
    } else {
        if skip_close_card_exit {
            clear_subviews(cards_container);
            cards_container.setHidden(true);
            cards_container.setAlphaValue(0.0);
        }
        let card_count = if skip_close_card_exit {
            0
        } else {
            cards_container.subviews().len()
        };
        expanded_container.setHidden(false);
        if !skip_close_card_exit {
            cards_container.setHidden(false);
            cards_container.setAlphaValue(1.0);
            apply_card_stack_transition(cards_container, 0.0, false);
        }
        if let Some(state_mutex) = native_panel_state() {
            if let Ok(mut state) = state_mutex.lock() {
                state.transition_cards_progress = 0.0;
                state.transition_cards_entering = false;
            }
        }

        tauri::async_runtime::spawn(async move {
            animate_close_transition(app.clone(), handles, animation_id, start_height, card_count)
                .await;

            let _ = app.run_on_main_thread(move || unsafe {
                if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                    return;
                }

                if let Some(state_mutex) = native_panel_state() {
                    if let Ok(mut state) = state_mutex.lock() {
                        state.transitioning = false;
                        state.transition_cards_progress = 0.0;
                        state.transition_cards_entering = false;
                    }
                }

                let cards_container = view_from_ptr(handles.cards_container);
                let expanded_container = view_from_ptr(handles.expanded_container);
                let panel = panel_from_ptr(handles.panel);
                let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
                let expanded_width =
                    expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);
                clear_subviews(cards_container);
                cards_container.setFrame(NSRect::new(
                    NSPoint::new(EXPANDED_CARDS_SIDE_INSET, 0.0),
                    NSSize::new(expanded_cards_width(expanded_width), 0.0),
                ));
                expanded_container.setHidden(true);
                expanded_container.setAlphaValue(1.0);
                apply_panel_geometry(
                    handles,
                    COLLAPSED_PANEL_HEIGHT,
                    COLLAPSED_PANEL_HEIGHT,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                );
                panel_from_ptr(handles.panel).displayIfNeeded();
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

    if let Some(state_mutex) = native_panel_state() {
        if let Ok(mut state) = state_mutex.lock() {
            state.transitioning = true;
            state.transition_cards_progress = PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS;
            state.transition_cards_entering = true;
        }
    }

    let panel = panel_from_ptr(handles.panel);
    let cards_container = view_from_ptr(handles.cards_container);
    let expanded_container = view_from_ptr(handles.expanded_container);
    let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
    let expanded_width =
        expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);
    let start_height = panel.frame().size.height;
    let shared_body_height = if macos_shared_expanded_window::shared_expanded_enabled()
        && !native_status_surface_active()
    {
        native_panel_state()
            .and_then(|state| state.lock().ok().and_then(|guard| guard.shared_body_height))
    } else {
        None
    };
    let target_height = expanded_total_height(
        &snapshot,
        compact_pill_height_for_screen_rect(panel.screen().as_deref(), screen_frame),
        shared_body_height,
    );

    render_expanded_cards(cards_container, &snapshot, expanded_width);
    let card_count = cards_container.subviews().len();
    apply_card_stack_transition(
        cards_container,
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        true,
    );
    cards_container.setHidden(false);
    cards_container.setAlphaValue(1.0);
    expanded_container.setHidden(false);
    expanded_container.setAlphaValue(1.0);

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

            if let Some(state_mutex) = native_panel_state() {
                if let Ok(mut state) = state_mutex.lock() {
                    state.transitioning = false;
                    state.transition_cards_progress = 1.0;
                    state.transition_cards_entering = true;
                }
            }

            let cards_container = view_from_ptr(handles.cards_container);
            let panel = panel_from_ptr(handles.panel);
            let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
            let expanded_width =
                expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);
            render_expanded_cards(cards_container, &snapshot, expanded_width);
            cards_container.setHidden(false);
            cards_container.setAlphaValue(1.0);
            apply_card_stack_transition(cards_container, 1.0, true);
            apply_panel_geometry(handles, target_height, target_height, 1.0, 1.0, 1.0, 1.0);
            panel_from_ptr(handles.panel).displayIfNeeded();
        });
    });
}

#[cfg(target_os = "macos")]
async fn animate_open_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) {
    let card_total_ms = card_transition_total_ms(
        card_count,
        PANEL_CARD_REVEAL_MS,
        PANEL_CARD_REVEAL_STAGGER_MS,
    );
    let total_ms = PANEL_OPEN_TOTAL_MS + card_total_ms;
    let canvas_height = panel_transition_canvas_height(start_height, target_height);
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        total_ms,
        move |elapsed_ms| {
            let morph_phase = animation_phase(elapsed_ms, PANEL_MORPH_DELAY_MS, PANEL_MORPH_MS);
            let height_phase = animation_phase(
                elapsed_ms,
                PANEL_MORPH_DELAY_MS + PANEL_MORPH_MS,
                PANEL_HEIGHT_MS,
            );
            let morph_progress = morph_phase.clamp(0.0, 1.0);
            let height_progress = height_phase.clamp(0.0, 1.0);
            let shoulder_progress =
                ease_in_cubic(animation_phase(elapsed_ms, 0, PANEL_SHOULDER_HIDE_MS));
            let drop_progress = ease_out_cubic(morph_phase);
            let cards_progress = animation_phase(elapsed_ms, PANEL_OPEN_TOTAL_MS, card_total_ms);
            (
                canvas_height,
                lerp(COLLAPSED_PANEL_HEIGHT, target_height, height_progress),
                morph_progress,
                height_progress,
                shoulder_progress,
                drop_progress,
                cards_progress,
            )
        },
        true,
    )
    .await;
}

#[cfg(target_os = "macos")]
async fn animate_surface_switch_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    start_height: f64,
    target_height: f64,
    card_count: usize,
) {
    let card_total_ms = card_transition_total_ms(
        card_count,
        PANEL_SURFACE_SWITCH_CARD_REVEAL_MS,
        PANEL_SURFACE_SWITCH_CARD_REVEAL_STAGGER_MS,
    );
    let total_ms = PANEL_SURFACE_SWITCH_HEIGHT_MS.max(card_total_ms);
    let canvas_height = panel_transition_canvas_height(start_height, target_height);
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        total_ms,
        move |elapsed_ms| {
            let height_progress = ease_out_cubic(animation_phase(
                elapsed_ms,
                0,
                PANEL_SURFACE_SWITCH_HEIGHT_MS,
            ));
            let cards_progress = surface_switch_card_progress(elapsed_ms, card_total_ms);
            (
                canvas_height,
                lerp(start_height, target_height, height_progress),
                1.0,
                1.0,
                1.0,
                1.0,
                cards_progress,
            )
        },
        true,
    )
    .await;
}

#[cfg(target_os = "macos")]
fn surface_switch_card_progress(elapsed_ms: u64, card_total_ms: u64) -> f64 {
    if card_total_ms == 0 {
        return 1.0;
    }
    lerp(
        PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        1.0,
        animation_phase(elapsed_ms, 0, card_total_ms),
    )
}

#[cfg(target_os = "macos")]
async fn animate_close_transition<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    start_height: f64,
    card_count: usize,
) {
    let card_total_ms =
        card_transition_total_ms(card_count, PANEL_CARD_EXIT_MS, PANEL_CARD_EXIT_STAGGER_MS);
    let card_exit_settle_ms = if card_count > 0 {
        PANEL_CARD_EXIT_SETTLE_MS
    } else {
        0
    };
    let close_delay_ms = card_total_ms + card_exit_settle_ms;
    let total_ms = close_delay_ms + PANEL_CLOSE_TOTAL_MS;
    let canvas_height = panel_transition_canvas_height(start_height, COLLAPSED_PANEL_HEIGHT);
    animate_panel_timeline(
        app,
        handles,
        animation_id,
        total_ms,
        move |elapsed_ms| {
            let height_phase = animation_phase(elapsed_ms, close_delay_ms, PANEL_HEIGHT_MS);
            let morph_phase = animation_phase(
                elapsed_ms,
                close_delay_ms + PANEL_CLOSE_MORPH_DELAY_MS,
                PANEL_MORPH_MS,
            );
            let shoulder_phase = animation_phase(
                elapsed_ms,
                close_delay_ms + PANEL_CLOSE_SHOULDER_DELAY_MS,
                PANEL_CLOSE_SHOULDER_MS,
            );
            let height_progress = 1.0 - height_phase.clamp(0.0, 1.0);
            let morph_progress = 1.0 - ease_in_cubic(morph_phase);
            let shoulder_progress = 1.0 - ease_out_cubic(shoulder_phase);
            let drop_progress = 1.0 - ease_out_cubic(morph_phase);
            let cards_progress = animation_phase(elapsed_ms, 0, card_total_ms);
            (
                canvas_height,
                lerp(COLLAPSED_PANEL_HEIGHT, start_height, height_progress),
                morph_progress,
                height_progress,
                shoulder_progress,
                drop_progress,
                cards_progress,
            )
        },
        false,
    )
    .await;
}

#[cfg(target_os = "macos")]
async fn animate_panel_timeline<R, F>(
    app: AppHandle<R>,
    handles: NativePanelHandles,
    animation_id: u64,
    total_ms: u64,
    mut sample: F,
    cards_entering: bool,
) where
    R: tauri::Runtime + 'static,
    F: FnMut(u64) -> (f64, f64, f64, f64, f64, f64, f64) + Send + 'static,
{
    let started = Instant::now();
    loop {
        let elapsed_ms = started.elapsed().as_millis().min(total_ms as u128) as u64;
        let (
            canvas_height,
            visible_height,
            bar_progress,
            height_progress,
            shoulder_progress,
            drop_progress,
            cards_progress,
        ) = sample(elapsed_ms);
        let _ = app.run_on_main_thread(move || unsafe {
            if NATIVE_TEST_PANEL_ANIMATION_ID.load(Ordering::SeqCst) != animation_id {
                return;
            }
            with_disabled_layer_actions(|| {
                if let Some(state_mutex) = native_panel_state() {
                    if let Ok(mut state) = state_mutex.lock() {
                        state.transition_cards_progress = cards_progress.clamp(0.0, 1.0);
                        state.transition_cards_entering = cards_entering;
                    }
                }
                apply_panel_geometry(
                    handles,
                    canvas_height,
                    visible_height,
                    bar_progress,
                    height_progress,
                    shoulder_progress,
                    drop_progress,
                );
                let cards_container = view_from_ptr(handles.cards_container);
                apply_card_stack_transition(cards_container, cards_progress, cards_entering);
                panel_from_ptr(handles.panel).displayIfNeeded();
            });
        });

        if elapsed_ms >= total_ms {
            break;
        }

        tokio::time::sleep(Duration::from_millis(PANEL_ANIMATION_FRAME_MS)).await;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_snapshot_values_to_panel(handles: NativePanelHandles, snapshot: &RuntimeSnapshot) {
    let headline = text_field_from_ptr(handles.headline);
    let active_count = text_field_from_ptr(handles.active_count);
    let active_count_next = text_field_from_ptr(handles.active_count_next);
    let total_count = text_field_from_ptr(handles.total_count);

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
    headline.setHidden(compact_headline_should_hide(handles));
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
    sync_active_count_marquee(handles);

    headline.displayIfNeeded();
    clip_view_from_ptr(handles.active_count_clip).displayIfNeeded();
    active_count.displayIfNeeded();
    active_count_next.displayIfNeeded();
    total_count.displayIfNeeded();
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_panel_geometry(
    handles: NativePanelHandles,
    canvas_height: f64,
    visible_height: f64,
    bar_progress: f64,
    height_progress: f64,
    shoulder_progress: f64,
    drop_progress: f64,
) {
    let panel = panel_from_ptr(handles.panel);
    let content_view = view_from_ptr(handles.content_view);
    let left_shoulder = view_from_ptr(handles.left_shoulder);
    let right_shoulder = view_from_ptr(handles.right_shoulder);
    let pill_view = view_from_ptr(handles.pill_view);
    let expanded_container = view_from_ptr(handles.expanded_container);
    let cards_container = view_from_ptr(handles.cards_container);
    let body_separator = view_from_ptr(handles.body_separator);

    let bar_progress = bar_progress.clamp(0.0, 1.0);
    let height_progress = height_progress.clamp(0.0, 1.0);
    let shoulder_progress = shoulder_progress.clamp(0.0, 1.0);
    let drop_progress = drop_progress.clamp(0.0, 1.0);
    let screen_frame = resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame());
    let compact_height =
        compact_pill_height_for_screen_rect(panel.screen().as_deref(), screen_frame);
    let compact_width =
        compact_pill_width_for_screen_rect(panel.screen().as_deref(), compact_height);
    let expanded_width =
        expanded_panel_width_for_screen_rect(panel.screen().as_deref(), screen_frame);
    let panel_width =
        panel_canvas_width_for_screen_rect(panel.screen().as_deref(), compact_height, screen_frame);
    let canvas_height = canvas_height.max(COLLAPSED_PANEL_HEIGHT);
    let visible_height = visible_height.clamp(COLLAPSED_PANEL_HEIGHT, canvas_height);
    let size = NSSize::new(panel_width, canvas_height);
    let drop_offset = PANEL_DROP_DISTANCE * drop_progress;
    let panel_frame = centered_top_frame(screen_frame, size);
    apply_panel_frame(panel, panel_frame);
    content_view.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), size));
    let pill_frame = island_bar_frame(
        size,
        bar_progress,
        compact_width,
        expanded_width,
        compact_height,
        drop_offset,
    );
    let expanded_frame = expanded_background_frame(
        size,
        visible_height,
        bar_progress,
        height_progress,
        compact_width,
        expanded_width,
        compact_height,
        drop_offset,
    );
    pill_view.setFrame(pill_frame);
    left_shoulder.setFrame(left_shoulder_frame(pill_frame));
    right_shoulder.setFrame(right_shoulder_frame(pill_frame));
    left_shoulder.setAlphaValue(1.0 - shoulder_progress);
    right_shoulder.setAlphaValue(1.0 - shoulder_progress);
    left_shoulder.setHidden(shoulder_progress >= 0.98);
    right_shoulder.setHidden(shoulder_progress >= 0.98);
    relayout_compact_content(handles, pill_frame.size);
    expanded_container.setFrame(expanded_frame);
    let cards_frame = expanded_cards_frame(expanded_frame, compact_height);
    cards_container.setFrame(cards_frame);
    body_separator.setFrame(expanded_separator_frame(expanded_frame, compact_height));
    let shell_visible = bar_progress > 0.01 || height_progress > 0.01;
    let content_visibility = native_panel_content_visibility();
    let transitioning = native_panel_state()
        .and_then(|state| state.lock().ok().map(|guard| guard.transitioning))
        .unwrap_or(false);
    let shared_expanded_enabled = macos_shared_expanded_window::shared_expanded_enabled();
    let status_surface_active = native_status_surface_active();
    let (shared_content_visible, shared_content_interactive) = shared_expanded_content_state(
        shared_expanded_enabled,
        shell_visible,
        height_progress,
        bar_progress,
        cards_frame.size.height,
        status_surface_active,
        content_visibility,
    );
    let shared_content_visible = shared_content_visible && !transitioning;
    let shared_content_interactive = shared_content_interactive && !transitioning;
    expanded_container.setHidden(!shell_visible);
    expanded_container.setAlphaValue(if shell_visible { 1.0 } else { 0.0 });
    let separator_visibility = (height_progress.min(content_visibility) * 0.88).clamp(0.0, 0.88);
    body_separator.setHidden(separator_visibility <= 0.02);
    body_separator.setAlphaValue(separator_visibility);
    cards_container.setHidden(shared_content_visible);

    if let Some(layer) = pill_view.layer() {
        layer.setCornerRadius(lerp(
            COMPACT_PILL_RADIUS,
            PANEL_MORPH_PILL_RADIUS,
            bar_progress,
        ));
        if bar_progress <= 0.01 {
            layer.setMaskedCorners(compact_pill_corner_mask());
        } else {
            layer.setMaskedCorners(all_corner_mask());
        }
        layer.setBorderWidth(lerp(1.0, 0.0, bar_progress));
    }
    if let Some(layer) = expanded_container.layer() {
        layer.setCornerRadius(lerp(
            COMPACT_PILL_RADIUS,
            EXPANDED_PANEL_RADIUS,
            bar_progress.max(height_progress),
        ));
        layer.setBorderWidth(0.0);
        layer.setOpacity(if shell_visible { 1.0 } else { 0.0 });
    }

    if shared_expanded_enabled {
        let shared_content_frame =
            absolute_rect(panel_frame, compose_local_rect(expanded_frame, cards_frame));
        let _ = macos_shared_expanded_window::sync_shared_expanded_frame(
            shared_content_frame,
            shared_content_visible,
            shared_content_interactive,
        );
    }

    pill_view.displayIfNeeded();
    expanded_container.displayIfNeeded();
    left_shoulder.setNeedsDisplay(true);
    right_shoulder.setNeedsDisplay(true);
    pill_view.setNeedsDisplay(true);
    expanded_container.setNeedsDisplay(true);
    content_view.setNeedsDisplay(true);
    content_view.layoutSubtreeIfNeeded();
    content_view.displayIfNeededIgnoringOpacity();
    panel.displayIfNeeded();
}

#[cfg(target_os = "macos")]
fn shared_expanded_content_state(
    shared_expanded_enabled: bool,
    shell_visible: bool,
    height_progress: f64,
    bar_progress: f64,
    cards_height: f64,
    status_surface_active: bool,
    content_visibility: f64,
) -> (bool, bool) {
    let visible = shared_expanded_enabled
        && shell_visible
        && height_progress > SHARED_CONTENT_REVEAL_PROGRESS
        && content_visibility > SHARED_CONTENT_REVEAL_PROGRESS
        && cards_height > 4.0
        && !status_surface_active;
    let interactive = visible
        && height_progress > SHARED_CONTENT_INTERACTIVE_PROGRESS
        && bar_progress > SHARED_CONTENT_INTERACTIVE_PROGRESS
        && content_visibility > SHARED_CONTENT_INTERACTIVE_PROGRESS;
    (visible, interactive)
}

#[cfg(target_os = "macos")]
fn rects_nearly_equal(a: NSRect, b: NSRect) -> bool {
    (a.origin.x - b.origin.x).abs() < 0.5
        && (a.origin.y - b.origin.y).abs() < 0.5
        && (a.size.width - b.size.width).abs() < 0.5
        && (a.size.height - b.size.height).abs() < 0.5
}

#[cfg(target_os = "macos")]
fn apply_panel_frame(panel: &NSPanel, frame: NSRect) {
    let current = panel.frame();
    if rects_nearly_equal(current, frame) {
        return;
    }
    let top_left = NSPoint::new(frame.origin.x, frame.origin.y + frame.size.height);
    panel.setContentSize(frame.size);
    panel.setFrameTopLeftPoint(top_left);
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
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_expanded_cards(
    cards_container: &NSView,
    snapshot: &RuntimeSnapshot,
    expanded_width: f64,
) {
    clear_card_animation_layouts();
    clear_subviews(cards_container);
    let mut card_hit_targets = Vec::new();
    let cards_width = expanded_cards_width(expanded_width);

    let status_queue = native_status_queue_surface_items();
    if !status_queue.is_empty() {
        let body_height = estimated_expanded_body_height(snapshot);
        let current_frame = cards_container.frame();
        cards_container.setFrame(NSRect::new(
            current_frame.origin,
            NSSize::new(cards_width, body_height),
        ));

        let mut cursor_y = body_height;
        let mut rendered_count = 0usize;
        for item in status_queue.iter() {
            let card_height = native_status_queue_card_height(item);
            let Some(frame) = next_expanded_card_frame(
                &mut cursor_y,
                rendered_count > 0,
                card_height,
                cards_width,
            ) else {
                break;
            };
            let card = create_status_queue_card(frame, item);
            apply_status_queue_item_visual_state(&card, item);
            cards_container.addSubview(&card);
            card_hit_targets.push(NativeCardHitTarget {
                session_id: item.session_id.clone(),
                frame,
            });
            rendered_count += 1;
        }
        replace_native_card_hit_targets(card_hit_targets);
        return;
    }

    let pending_permissions = displayed_default_pending_permissions(snapshot);
    let pending_questions = displayed_default_pending_questions(snapshot);
    let prompt_assist_sessions = displayed_prompt_assist_sessions(snapshot);
    let sessions = displayed_sessions(snapshot, &prompt_assist_sessions);
    let body_height = estimated_expanded_body_height(snapshot);
    let current_frame = cards_container.frame();
    cards_container.setFrame(NSRect::new(
        current_frame.origin,
        NSSize::new(cards_width, body_height),
    ));

    let mut cursor_y = body_height;
    let mut rendered_count = 0usize;

    for pending in pending_permissions.iter() {
        if let Some(frame) = next_expanded_card_frame(
            &mut cursor_y,
            rendered_count > 0,
            pending_permission_card_height(pending),
            cards_width,
        ) {
            let card =
                create_pending_permission_card(frame, pending, snapshot.pending_permission_count);
            cards_container.addSubview(&card);
            card_hit_targets.push(NativeCardHitTarget {
                session_id: pending.session_id.clone(),
                frame,
            });
            rendered_count += 1;
        }
    }

    for pending in pending_questions.iter() {
        if let Some(frame) = next_expanded_card_frame(
            &mut cursor_y,
            rendered_count > 0,
            pending_question_card_height(pending),
            cards_width,
        ) {
            let card =
                create_pending_question_card(frame, pending, snapshot.pending_question_count);
            cards_container.addSubview(&card);
            card_hit_targets.push(NativeCardHitTarget {
                session_id: pending.session_id.clone(),
                frame,
            });
            rendered_count += 1;
        }
    }

    for session in prompt_assist_sessions.iter() {
        let Some(frame) = next_expanded_card_frame(
            &mut cursor_y,
            rendered_count > 0,
            prompt_assist_card_height(session),
            cards_width,
        ) else {
            break;
        };
        let card = create_prompt_assist_card(frame, session);
        cards_container.addSubview(&card);
        card_hit_targets.push(NativeCardHitTarget {
            session_id: session.session_id.clone(),
            frame,
        });
        rendered_count += 1;
    }

    if sessions.is_empty() && rendered_count == 0 {
        if let Some(frame) = next_expanded_card_frame(&mut cursor_y, false, 84.0, cards_width) {
            let empty = create_empty_card(frame);
            cards_container.addSubview(&empty);
        }
        replace_native_card_hit_targets(card_hit_targets);
        return;
    }

    for session in sessions.iter() {
        let card_height = estimated_card_height(session);
        let Some(frame) =
            next_expanded_card_frame(&mut cursor_y, rendered_count > 0, card_height, cards_width)
        else {
            break;
        };
        let card = create_session_card(frame, session, false);
        cards_container.addSubview(&card);
        card_hit_targets.push(NativeCardHitTarget {
            session_id: session.session_id.clone(),
            frame,
        });
        rendered_count += 1;
    }

    replace_native_card_hit_targets(card_hit_targets);
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_card_stack_transition(
    cards_container: &NSView,
    cards_progress: f64,
    entering: bool,
) {
    let subviews = cards_container.subviews();
    for index in 0..subviews.len() {
        let card = subviews.objectAtIndex(index);
        let phase = staggered_card_phase(cards_progress, index, subviews.len(), entering);
        let base_layout = card_animation_layout(&card).unwrap_or(CardAnimationLayout {
            frame: card.frame(),
            collapsed_height: (card.frame().size.height * 0.58).round().max(34.0),
        });
        let (shell_opacity, current_height, scale_x, scale_y, translate_y, content_progress) =
            if entering {
                let shell_progress = ease_out_cubic(phase);
                (
                    shell_progress,
                    lerp(
                        base_layout.collapsed_height,
                        base_layout.frame.size.height,
                        shell_progress,
                    ),
                    lerp(0.96, 1.0, shell_progress),
                    lerp(0.82, 1.0, shell_progress),
                    lerp(PANEL_CARD_REVEAL_Y, 0.0, shell_progress),
                    card_content_visibility_phase(phase, true),
                )
            } else {
                let squeeze_phase = (phase / 0.28).clamp(0.0, 1.0);
                let exit_phase = ((phase - 0.28) / 0.72).clamp(0.0, 1.0);
                let visible_ratio = if phase <= 0.28 { 1.0 } else { 1.0 - exit_phase };
                (
                    if phase <= 0.28 {
                        1.0
                    } else {
                        1.0 - ease_in_cubic(exit_phase)
                    },
                    (base_layout.frame.size.height * visible_ratio).max(1.0),
                    if phase <= 0.28 {
                        lerp(1.0, 1.003, squeeze_phase)
                    } else {
                        lerp(1.003, 0.985, exit_phase)
                    },
                    if phase <= 0.28 {
                        lerp(1.0, 0.94, squeeze_phase)
                    } else {
                        lerp(0.94, 0.76, exit_phase)
                    },
                    0.0,
                    card_content_visibility_phase(phase, false),
                )
            };

        let frame = NSRect::new(
            NSPoint::new(
                base_layout.frame.origin.x,
                base_layout.frame.origin.y + (base_layout.frame.size.height - current_height),
            ),
            NSSize::new(base_layout.frame.size.width, current_height),
        );
        card.setFrame(frame);
        card.setHidden(shell_opacity <= 0.001);
        card.setAlphaValue(shell_opacity);
        if let Some(layer) = card.layer() {
            let transform = CGAffineTransformTranslate(
                CGAffineTransformMakeScale(scale_x, scale_y),
                0.0,
                translate_y,
            );
            layer.setAffineTransform(transform);
            layer.setShadowOpacity((shell_opacity * 0.08).clamp(0.0, 0.08) as f32);
            layer.setShadowRadius(lerp(0.0, 8.0, shell_opacity));
            layer.setShadowOffset(NSSize::new(0.0, lerp(0.0, -2.0, shell_opacity)));
        }

        apply_card_content_phase(&card, phase, entering, content_progress);
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_card_exit_phase(card: &NSView, phase: f64) {
    let phase = phase.clamp(0.0, 1.0);
    let base_layout = card_animation_layout(card).unwrap_or(CardAnimationLayout {
        frame: card.frame(),
        collapsed_height: (card.frame().size.height * 0.58).round().max(34.0),
    });

    let squeeze_phase = (phase / 0.28).clamp(0.0, 1.0);
    let exit_phase = ((phase - 0.28) / 0.72).clamp(0.0, 1.0);
    let visible_ratio = if phase <= 0.28 { 1.0 } else { 1.0 - exit_phase };
    let shell_opacity = if phase <= 0.28 {
        1.0
    } else {
        1.0 - ease_in_cubic(exit_phase)
    };
    let content_progress = card_content_visibility_phase(phase, false);
    let current_height = (base_layout.frame.size.height * visible_ratio).max(1.0);
    let scale_x = if phase <= 0.28 {
        lerp(1.0, 1.003, squeeze_phase)
    } else {
        lerp(1.003, 0.985, exit_phase)
    };
    let scale_y = if phase <= 0.28 {
        lerp(1.0, 0.94, squeeze_phase)
    } else {
        lerp(0.94, 0.76, exit_phase)
    };

    let frame = NSRect::new(
        NSPoint::new(
            base_layout.frame.origin.x,
            base_layout.frame.origin.y + (base_layout.frame.size.height - current_height),
        ),
        NSSize::new(base_layout.frame.size.width, current_height),
    );
    card.setFrame(frame);
    card.setHidden(shell_opacity <= 0.001);
    card.setAlphaValue(shell_opacity);

    if let Some(layer) = card.layer() {
        let transform =
            CGAffineTransformTranslate(CGAffineTransformMakeScale(scale_x, scale_y), 0.0, 0.0);
        layer.setAffineTransform(transform);
        layer.setShadowOpacity((shell_opacity * 0.08).clamp(0.0, 0.08) as f32);
        layer.setShadowRadius(lerp(0.0, 8.0, shell_opacity));
        layer.setShadowOffset(NSSize::new(0.0, lerp(0.0, -2.0, shell_opacity)));
    }

    apply_card_content_phase(card, phase, false, content_progress);
}

#[cfg(target_os = "macos")]
fn card_content_visibility_phase(phase: f64, entering: bool) -> f64 {
    let phase = phase.clamp(0.0, 1.0);
    if entering {
        ease_out_cubic(
            ((phase - PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS)
                / (1.0 - PANEL_CARD_CONTENT_REVEAL_DELAY_PROGRESS))
                .clamp(0.0, 1.0),
        )
    } else if phase <= PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS {
        1.0 - (0.06 * (phase / PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS).clamp(0.0, 1.0))
    } else {
        0.94 * (1.0
            - ease_in_cubic(
                ((phase - PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS)
                    / (1.0 - PANEL_CARD_CONTENT_EARLY_EXIT_PROGRESS))
                    .clamp(0.0, 1.0),
            ))
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_card_content_phase(
    card: &NSView,
    phase: f64,
    entering: bool,
    content_progress: f64,
) {
    let children = card.subviews();
    for child_index in 0..children.len() {
        let child = children.objectAtIndex(child_index);
        child.setHidden(content_progress <= 0.001);
        child.setAlphaValue(content_progress);
        child.setWantsLayer(true);

        if let Some(layer) = child.layer() {
            let transform = if entering {
                let reveal_progress = phase.clamp(0.0, 1.0);
                CGAffineTransformTranslate(
                    CGAffineTransformMakeScale(1.0, lerp(0.92, 1.0, reveal_progress)),
                    0.0,
                    lerp(-5.0, 0.0, reveal_progress),
                )
            } else if phase <= 0.30 {
                let early_phase = (phase / 0.30).clamp(0.0, 1.0);
                CGAffineTransformTranslate(
                    CGAffineTransformMakeScale(1.0, lerp(1.0, 0.92, early_phase)),
                    0.0,
                    0.0,
                )
            } else {
                let late_phase = ((phase - 0.30) / 0.70).clamp(0.0, 1.0);
                CGAffineTransformTranslate(
                    CGAffineTransformMakeScale(1.0, lerp(0.92, 0.82, late_phase)),
                    0.0,
                    0.0,
                )
            };
            layer.setAffineTransform(transform);
        }
    }
}

#[cfg(target_os = "macos")]
fn staggered_card_phase(progress: f64, index: usize, total: usize, entering: bool) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    let duration_ms = if entering {
        PANEL_CARD_REVEAL_MS
    } else {
        PANEL_CARD_EXIT_MS
    };
    let stagger_ms = if entering {
        PANEL_CARD_REVEAL_STAGGER_MS
    } else {
        PANEL_CARD_EXIT_STAGGER_MS
    };
    let total_ms = card_transition_total_ms(total, duration_ms, stagger_ms) as f64;
    let order_index = if entering {
        index
    } else {
        total.saturating_sub(index + 1)
    };
    let elapsed_ms = progress * total_ms;
    let delay_ms = order_index as f64 * stagger_ms as f64;

    ((elapsed_ms - delay_ms) / duration_ms as f64).clamp(0.0, 1.0)
}

#[cfg(target_os = "macos")]
fn card_transition_total_ms(card_count: usize, duration_ms: u64, stagger_ms: u64) -> u64 {
    if card_count == 0 {
        return 0;
    }
    duration_ms + card_count.saturating_sub(1) as u64 * stagger_ms
}

#[cfg(target_os = "macos")]
fn next_expanded_card_frame(
    cursor_y: &mut f64,
    needs_gap: bool,
    height: f64,
    expanded_width: f64,
) -> Option<NSRect> {
    if needs_gap {
        *cursor_y -= EXPANDED_CARD_GAP;
    }
    if *cursor_y < height {
        return None;
    }

    *cursor_y -= height;
    Some(NSRect::new(
        NSPoint::new(-EXPANDED_CARD_OVERHANG, *cursor_y),
        NSSize::new(expanded_width + (EXPANDED_CARD_OVERHANG * 2.0), height),
    ))
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_empty_card(frame: NSRect) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 1.0, 1.0, 0.055], [1.0, 1.0, 1.0, 0.08]);
    register_card_animation_layout(&view, frame, 34.0);

    let label = make_label(
        mtm,
        "No sessions yet.",
        NSRect::new(NSPoint::new(0.0, 31.0), NSSize::new(frame.size.width, 20.0)),
        12.0,
        [0.67, 0.70, 0.76, 1.0],
        true,
        false,
    );
    view.addSubview(&label);
    view
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_pending_permission_card(
    frame: NSRect,
    pending: &PendingPermissionView,
    _waiting_count: usize,
) -> objc2::rc::Retained<NSView> {
    let title = pending
        .tool_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Tool permission");
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());
    create_pending_card(
        frame,
        "Approval",
        &compact_title(title, 34),
        &body,
        "Allow / Deny in terminal",
        &pending.source,
        &pending.session_id,
        [1.0, 0.61, 0.26, 0.13],
        [1.0, 0.61, 0.26, 0.24],
        [1.0, 0.68, 0.40, 1.0],
    )
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_pending_question_card(
    frame: NSRect,
    pending: &PendingQuestionView,
    _waiting_count: usize,
) -> objc2::rc::Retained<NSView> {
    let title = pending
        .header
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Need your input");
    let body = display_snippet(Some(&pending.text), 82)
        .unwrap_or_else(|| "Waiting for your answer".to_string());
    let action_hint = pending_question_action_hint(pending);
    create_pending_card(
        frame,
        "Question",
        &compact_title(title, 34),
        &body,
        &action_hint,
        &pending.source,
        &pending.session_id,
        [0.69, 0.55, 1.0, 0.13],
        [0.69, 0.55, 1.0, 0.24],
        [0.79, 0.69, 1.0, 1.0],
    )
}

#[cfg(target_os = "macos")]
fn pending_question_action_hint(pending: &PendingQuestionView) -> String {
    if pending.options.is_empty() {
        return "Answer in terminal".to_string();
    }

    let options = pending
        .options
        .iter()
        .take(3)
        .map(|option| compact_title(option, 12))
        .collect::<Vec<_>>()
        .join(" / ");
    if pending.options.len() > 3 {
        format!("{options} / …")
    } else {
        options
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::too_many_arguments)]
unsafe fn create_pending_card(
    frame: NSRect,
    label: &str,
    title: &str,
    body: &str,
    action_hint: &str,
    source: &str,
    session_id: &str,
    background: [f64; 4],
    border: [f64; 4],
    accent: [f64; 4],
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, background, border);
    register_card_animation_layout(&view, frame, 46.0);

    let status_badge = make_badge_view(
        mtm,
        label,
        badge_width(label, 10.0, 16.0),
        [1.0, 1.0, 1.0, 0.08],
        accent,
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let formatted_source = format_source(source);
    let source_badge = make_badge_view(
        mtm,
        &formatted_source,
        badge_width(&formatted_source, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let title_label = make_label(
        mtm,
        title,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let meta_label = make_label(
        mtm,
        &format!("#{} · {}", short_session_id(session_id), label),
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    let prefix = if label.eq_ignore_ascii_case("Approval") {
        "!"
    } else {
        "?"
    };
    let prefix_color = if label.eq_ignore_ascii_case("Approval") {
        [1.0, 0.68, 0.40, 1.0]
    } else {
        [0.79, 0.69, 1.0, 1.0]
    };
    let body_width = card_chat_body_width(frame.size.width);
    let body_height = estimated_chat_body_height(body, body_width, 2);
    let header_bottom = frame.size.height - 40.0;
    let body_gap_from_header = 6.0;
    let body_origin_y = (header_bottom - body_gap_from_header - body_height)
        .max(CARD_PENDING_ACTION_Y + CARD_PENDING_ACTION_HEIGHT + CARD_PENDING_ACTION_GAP);
    let prefix_y = body_origin_y + body_height - 12.0;
    let prefix_label = make_label(
        mtm,
        prefix,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, prefix_y),
            NSSize::new(10.0, 12.0),
        ),
        10.0,
        prefix_color,
        true,
        true,
    );
    prefix_label.setFont(Some(&NSFont::boldSystemFontOfSize(10.0)));
    view.addSubview(&prefix_label);

    let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(body), mtm);
    body_label.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X + CARD_CHAT_PREFIX_WIDTH, body_origin_y),
        NSSize::new(body_width, body_height),
    ));
    body_label.setTextColor(Some(&ns_color([0.86, 0.88, 0.92, 0.78])));
    body_label.setFont(Some(&NSFont::systemFontOfSize(10.0)));
    body_label.setDrawsBackground(false);
    body_label.setBezeled(false);
    body_label.setBordered(false);
    body_label.setEditable(false);
    body_label.setSelectable(false);
    body_label.setMaximumNumberOfLines(2);
    view.addSubview(&body_label);

    let action_badge = make_badge_view(
        mtm,
        action_hint,
        badge_width(action_hint, 10.0, 18.0).min(frame.size.width - (CARD_INSET_X * 2.0)),
        [1.0, 1.0, 1.0, 0.07],
        [0.90, 0.92, 0.96, 0.86],
    );
    action_badge.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X, CARD_PENDING_ACTION_Y),
        action_badge.frame().size,
    ));
    view.addSubview(&action_badge);

    view
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_status_queue_card(
    frame: NSRect,
    item: &NativeStatusQueueItem,
) -> objc2::rc::Retained<NSView> {
    match &item.payload {
        NativeStatusQueuePayload::Approval(pending) => {
            create_pending_permission_card(frame, pending, 1)
        }
        NativeStatusQueuePayload::Completion(session) => create_completion_card(frame, session),
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_status_queue_item_visual_state(card: &NSView, item: &NativeStatusQueueItem) {
    if !item.is_removing {
        return;
    }

    let Some(remove_after) = item.remove_after else {
        return;
    };
    let exit_duration = status_queue_exit_duration();
    let elapsed =
        exit_duration.saturating_sub(remove_after.saturating_duration_since(Instant::now()));
    let progress = (elapsed.as_secs_f64() / exit_duration.as_secs_f64()).clamp(0.0, 1.0);
    apply_card_exit_phase(card, progress);
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_completion_card(
    frame: NSRect,
    session: &SessionSnapshotView,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [0.40, 0.87, 0.57, 0.08], [0.40, 0.87, 0.57, 0.28]);
    register_card_animation_layout(&view, frame, 52.0);

    let status_text = "Complete";
    let status_badge = make_badge_view(
        mtm,
        status_text,
        badge_width(status_text, 10.0, 16.0),
        [0.40, 0.87, 0.57, 0.14],
        [0.40, 0.87, 0.57, 1.0],
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let source_text = format_source(&session.source);
    let source_badge = make_badge_view(
        mtm,
        &source_text,
        badge_width(&source_text, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let project_name = session_title(session);
    let title = compact_title(&project_name, 30);
    let title_label = make_label(
        mtm,
        &title,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let meta_label = make_label(
        mtm,
        &format!(
            "#{} · {}",
            short_session_id(&session.session_id),
            time_ago(session.last_activity)
        ),
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    let preview = completion_preview_text(session);
    add_chat_line_with_max_lines_from_bottom(
        &view,
        mtm,
        CARD_CONTENT_BOTTOM_INSET,
        "$",
        &preview,
        [0.40, 0.87, 0.57, 0.96],
        [0.86, 0.88, 0.92, 0.78],
        frame.size.width,
        2,
        0.0,
    );

    view
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_prompt_assist_card(
    frame: NSRect,
    session: &SessionSnapshotView,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    apply_card_layer(&view, [1.0, 0.61, 0.26, 0.08], [1.0, 0.61, 0.26, 0.32]);
    register_card_animation_layout(&view, frame, 52.0);

    let status_text = "Check";
    let status_badge = make_badge_view(
        mtm,
        status_text,
        badge_width(status_text, 10.0, 16.0),
        [1.0, 0.61, 0.26, 0.16],
        [1.0, 0.70, 0.40, 1.0],
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let source_badge = make_badge_view(
        mtm,
        "Codex",
        badge_width("Codex", 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let title_label = make_label(
        mtm,
        "Codex needs attention",
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let project_name = compact_title(&session_title(session), 24);
    let meta_label = make_label(
        mtm,
        &format!(
            "#{} · {} · {}",
            short_session_id(&session.session_id),
            project_name,
            time_ago(session.last_activity)
        ),
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    add_chat_line_with_max_lines_from_bottom(
        &view,
        mtm,
        CARD_PENDING_ACTION_Y + CARD_PENDING_ACTION_HEIGHT + CARD_PENDING_ACTION_GAP,
        "!",
        "Approval may be required in the Codex terminal.",
        [1.0, 0.68, 0.40, 1.0],
        [0.86, 0.88, 0.92, 0.78],
        frame.size.width,
        2,
        0.0,
    );

    let action_badge = make_badge_view(
        mtm,
        "Open terminal to check",
        badge_width("Open terminal to check", 10.0, 18.0)
            .min(frame.size.width - (CARD_INSET_X * 2.0)),
        [1.0, 1.0, 1.0, 0.07],
        [0.90, 0.92, 0.96, 0.86],
    );
    action_badge.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X, CARD_PENDING_ACTION_Y),
        action_badge.frame().size,
    ));
    view.addSubview(&action_badge);

    view
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn create_session_card(
    frame: NSRect,
    session: &SessionSnapshotView,
    emphasize: bool,
) -> objc2::rc::Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main thread required");
    let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
    let status = normalize_status(&session.status);
    let prompt = session_prompt_preview(session);
    let reply = session_reply_preview(session);
    let tool_preview = session_tool_preview(session);
    let has_body_content = prompt.is_some()
        || reply.is_some()
        || tool_preview
            .as_ref()
            .map(|(name, _)| !name.is_empty())
            .unwrap_or(false);
    let is_compact = is_long_idle_session(session) || !has_body_content;
    let background = if emphasize {
        [0.40, 0.87, 0.57, 0.08]
    } else {
        [1.0, 1.0, 1.0, 0.055]
    };
    let border = if emphasize {
        [0.40, 0.87, 0.57, 0.20]
    } else {
        [1.0, 1.0, 1.0, 0.08]
    };
    apply_card_layer(&view, background, border);
    register_card_animation_layout(
        &view,
        frame,
        session_card_collapsed_height(frame.size.height, is_compact),
    );

    let (status_bg, status_fg) = status_pill_colors(&status, emphasize);
    let status_text = format_status(&session.status);
    let status_badge = make_badge_view(
        mtm,
        &status_text,
        badge_width(&status_text, 10.0, 16.0),
        status_bg,
        status_fg,
    );
    let status_width = status_badge.frame().size.width;
    let status_x = frame.size.width - status_width - CARD_INSET_X;
    status_badge.setFrame(NSRect::new(
        NSPoint::new(status_x, frame.size.height - 27.0),
        status_badge.frame().size,
    ));
    view.addSubview(&status_badge);

    let source_text = format_source(&session.source);
    let source_badge = make_badge_view(
        mtm,
        &source_text,
        badge_width(&source_text, 10.0, 16.0),
        [0.47, 0.65, 1.0, 0.12],
        [0.47, 0.65, 1.0, 1.0],
    );
    let source_width = source_badge.frame().size.width;
    let source_x = (status_x - source_width - 6.0).max(CARD_INSET_X + 76.0);
    source_badge.setFrame(NSRect::new(
        NSPoint::new(source_x, frame.size.height - 27.0),
        source_badge.frame().size,
    ));
    view.addSubview(&source_badge);

    let title = compact_title(&session_title(session), 30);
    let title_label = make_label(
        mtm,
        &title,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 24.0),
            NSSize::new((source_x - CARD_INSET_X - 8.0).max(92.0), 16.0),
        ),
        12.0,
        [0.96, 0.97, 0.99, 1.0],
        false,
        true,
    );
    title_label.setFont(Some(&NSFont::boldSystemFontOfSize(12.0)));
    view.addSubview(&title_label);

    let meta_text = session_meta_line(session);
    let meta_label = make_label(
        mtm,
        &meta_text,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, frame.size.height - 40.0),
            NSSize::new(frame.size.width - (CARD_INSET_X * 2.0), 12.0),
        ),
        9.0,
        [0.67, 0.70, 0.76, 1.0],
        false,
        true,
    );
    meta_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightRegular },
    )));
    view.addSubview(&meta_label);

    if !is_compact {
        let mut content_top = CARD_CONTENT_BOTTOM_INSET;
        let has_prompt = prompt.is_some();
        let has_reply = reply.is_some();

        if let Some((tool_name, tool_description)) = tool_preview.as_ref() {
            let tool_text = tool_description
                .as_ref()
                .map(|description| format!("{tool_name} · {description}"))
                .unwrap_or_else(|| tool_name.to_string());
            let tool_view = make_live_tool_view(
                mtm,
                tool_name,
                tool_description.as_deref(),
                (frame.size.width - (CARD_INSET_X * 2.0)).min(badge_width(&tool_text, 9.0, 20.0)),
            );
            let tool_size = tool_view.frame().size;
            tool_view.setFrame(NSRect::new(
                NSPoint::new(CARD_INSET_X, content_top),
                NSSize::new(
                    (frame.size.width - (CARD_INSET_X * 2.0)).min(tool_size.width),
                    tool_size.height,
                ),
            ));
            view.addSubview(&tool_view);
            content_top += tool_size.height;
            if has_reply || has_prompt {
                content_top += CARD_TOOL_GAP;
            }
        }

        if let Some(reply) = reply.as_deref() {
            content_top = add_chat_line_with_max_lines_from_bottom(
                &view,
                mtm,
                content_top,
                "$",
                &reply,
                [0.85, 0.47, 0.34, 0.96],
                [0.86, 0.88, 0.92, 0.74],
                frame.size.width,
                2,
                if has_prompt { CARD_CHAT_GAP } else { 0.0 },
            );
        }

        if let Some(prompt) = prompt.as_deref() {
            add_chat_line_with_max_lines_from_bottom(
                &view,
                mtm,
                content_top,
                ">",
                &prompt,
                [0.40, 0.87, 0.57, 0.96],
                [0.96, 0.97, 0.99, 0.86],
                frame.size.width,
                1,
                0.0,
            );
        }
    }

    view
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
#[allow(clippy::too_many_arguments)]
unsafe fn add_chat_line_with_max_lines_from_bottom(
    parent: &NSView,
    mtm: MainThreadMarker,
    bottom_y: f64,
    prefix: &str,
    body: &str,
    prefix_color: [f64; 4],
    body_color: [f64; 4],
    width: f64,
    max_lines: isize,
    gap_after: f64,
) -> f64 {
    let body_width = card_chat_body_width(width);
    let body_height = estimated_chat_body_height(body, body_width, max_lines);
    let prefix_y = bottom_y + body_height - 12.0;
    let prefix_label = make_label(
        mtm,
        prefix,
        NSRect::new(
            NSPoint::new(CARD_INSET_X, prefix_y),
            NSSize::new(10.0, 12.0),
        ),
        10.0,
        prefix_color,
        true,
        true,
    );
    prefix_label.setFont(Some(&NSFont::boldSystemFontOfSize(10.0)));
    parent.addSubview(&prefix_label);

    let body_label = NSTextField::wrappingLabelWithString(&NSString::from_str(body), mtm);
    body_label.setFrame(NSRect::new(
        NSPoint::new(CARD_INSET_X + CARD_CHAT_PREFIX_WIDTH, bottom_y),
        NSSize::new(body_width, body_height),
    ));
    body_label.setTextColor(Some(&ns_color(body_color)));
    body_label.setFont(Some(&NSFont::systemFontOfSize(10.0)));
    body_label.setDrawsBackground(false);
    body_label.setBezeled(false);
    body_label.setBordered(false);
    body_label.setEditable(false);
    body_label.setSelectable(false);
    body_label.setMaximumNumberOfLines(max_lines);
    parent.addSubview(&body_label);

    bottom_y + body_height + gap_after.max(0.0)
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

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct NativeCompactStyle {
    headline_color: [f64; 4],
    active_count_color: [f64; 4],
    total_count_color: [f64; 4],
}

#[cfg(target_os = "macos")]
fn compact_active_count_value(snapshot: &RuntimeSnapshot) -> usize {
    snapshot
        .sessions
        .iter()
        .filter(|session| !should_hide_legacy_opencode_session(session))
        .filter(|session| normalize_status(&session.status) != "idle")
        .count()
}

#[cfg(target_os = "macos")]
fn compact_active_count_text(snapshot: &RuntimeSnapshot) -> String {
    compact_active_count_value(snapshot).to_string()
}

#[cfg(target_os = "macos")]
fn compact_style(snapshot: &RuntimeSnapshot) -> NativeCompactStyle {
    let active_count = compact_active_count_value(snapshot);

    NativeCompactStyle {
        headline_color: if active_count > 0 {
            [0.96, 0.97, 0.99, 1.0]
        } else {
            [0.90, 0.92, 0.96, 0.92]
        },
        active_count_color: if active_count > 0 {
            [0.40, 0.87, 0.57, 1.0]
        } else {
            [0.61, 0.65, 0.72, 1.0]
        },
        total_count_color: [0.96, 0.97, 0.99, 1.0],
    }
}

#[cfg(target_os = "macos")]
impl NativeMascotMotion {
    fn idle() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shell_alpha: 1.0,
            shadow_opacity: 0.34,
            shadow_radius: 4.0,
        }
    }
}

#[cfg(target_os = "macos")]
impl NativeMascotRuntime {
    fn new(now: Instant) -> Self {
        let idle_motion = NativeMascotMotion::idle();
        Self {
            animation_started_at: now,
            last_non_idle_at: now,
            last_resolved_state: NativeMascotState::Idle,
            wake_started_at: None,
            wake_next_state: NativeMascotState::Idle,
            transition_target: NativeMascotState::Idle,
            transition_started_at: now,
            transition_start_motion: idle_motion,
            last_motion: idle_motion,
        }
    }

    fn next_frame(
        &mut self,
        base_state: NativeMascotState,
        expanded: bool,
        now: Instant,
    ) -> NativeMascotFrame {
        let t = now
            .saturating_duration_since(self.animation_started_at)
            .as_secs_f64();
        let visual_state = self.resolve_visual_state(base_state, expanded, now);
        let target_motion = native_mascot_target_motion(visual_state, t, self.wake_started_at, now);

        if self.transition_target != visual_state {
            self.transition_target = visual_state;
            self.transition_started_at = now;
            self.transition_start_motion = self.last_motion;
        }

        let transition_progress = now
            .saturating_duration_since(self.transition_started_at)
            .as_secs_f64()
            / MASCOT_STATE_TRANSITION_SECONDS;
        let motion = native_mascot_lerp_motion(
            self.transition_start_motion,
            target_motion,
            smoothstep_unit(transition_progress),
        );
        self.last_motion = motion;

        NativeMascotFrame {
            state: visual_state,
            t,
            motion,
            color: native_mascot_color(visual_state, t, self.wake_started_at, now),
        }
    }

    fn resolve_visual_state(
        &mut self,
        base_state: NativeMascotState,
        expanded: bool,
        now: Instant,
    ) -> NativeMascotState {
        let mut next_state = if base_state != NativeMascotState::Idle {
            self.last_non_idle_at = now;
            base_state
        } else if expanded {
            self.last_non_idle_at = now;
            NativeMascotState::Idle
        } else if now
            .saturating_duration_since(self.last_non_idle_at)
            .as_secs()
            >= MASCOT_IDLE_LONG_SECONDS
        {
            NativeMascotState::Sleepy
        } else {
            NativeMascotState::Idle
        };

        let waking_from_sleep = next_state != NativeMascotState::Sleepy
            && self.wake_started_at.is_none()
            && self.last_resolved_state == NativeMascotState::Sleepy;
        if waking_from_sleep {
            self.wake_started_at = Some(now);
            self.wake_next_state = next_state;
            self.last_resolved_state = NativeMascotState::WakeAngry;
            return NativeMascotState::WakeAngry;
        }

        if let Some(started_at) = self.wake_started_at {
            self.wake_next_state = if next_state == NativeMascotState::Sleepy {
                NativeMascotState::Idle
            } else {
                next_state
            };

            if now.saturating_duration_since(started_at).as_secs_f64() < MASCOT_WAKE_ANGRY_SECONDS {
                self.last_resolved_state = NativeMascotState::WakeAngry;
                return NativeMascotState::WakeAngry;
            }

            self.wake_started_at = None;
            next_state = self.wake_next_state;
        }

        self.last_resolved_state = next_state;
        next_state
    }
}

#[cfg(target_os = "macos")]
fn infer_native_mascot_base_state(
    snapshot: Option<&RuntimeSnapshot>,
    has_status_completion: bool,
) -> NativeMascotState {
    let Some(snapshot) = snapshot else {
        return NativeMascotState::Idle;
    };

    if snapshot.pending_permission_count > 0 {
        return NativeMascotState::Approval;
    }
    if snapshot.pending_question_count > 0 {
        return NativeMascotState::Question;
    }
    if has_status_completion {
        return NativeMascotState::MessageBubble;
    }
    if compact_active_count_value(snapshot) > 0 || snapshot.active_session_count > 0 {
        return NativeMascotState::Bouncing;
    }

    NativeMascotState::Idle
}

#[cfg(target_os = "macos")]
fn native_mascot_target_motion(
    state: NativeMascotState,
    t: f64,
    wake_started_at: Option<Instant>,
    now: Instant,
) -> NativeMascotMotion {
    match state {
        NativeMascotState::Bouncing => {
            let bounce = (t * 5.8).sin().abs();
            let hang = bounce.powf(0.72);
            let landing = (1.0 - bounce).powf(3.2);
            NativeMascotMotion {
                offset_x: (t * 3.1).sin() * 0.28,
                offset_y: hang * 5.6,
                scale_x: 1.0 + landing * 0.18 + hang * 0.018,
                scale_y: 1.0 - landing * 0.16 + hang * 0.018,
                shell_alpha: 1.0,
                shadow_opacity: 0.46,
                shadow_radius: 5.4,
            }
        }
        NativeMascotState::Approval => {
            let pulse = ((t * 7.2).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: (t * 9.0).sin() * 0.34,
                offset_y: 0.0,
                scale_x: 1.0 + pulse * 0.025,
                scale_y: 1.0 - pulse * 0.018,
                shell_alpha: 1.0,
                shadow_opacity: 0.52,
                shadow_radius: 6.0,
            }
        }
        NativeMascotState::Question => {
            let tilt = (t * 4.4).sin();
            NativeMascotMotion {
                offset_x: tilt * 0.28,
                offset_y: (t * 5.1).sin() * 0.55,
                scale_x: 1.0 + tilt.abs() * 0.012,
                scale_y: 1.0,
                shell_alpha: 1.0,
                shadow_opacity: 0.50,
                shadow_radius: 5.8,
            }
        }
        NativeMascotState::MessageBubble => {
            let bob = ((t * 3.2).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: bob * 1.6,
                scale_x: 1.0 + bob * 0.012,
                scale_y: 1.0 - bob * 0.008,
                shell_alpha: 1.0,
                shadow_opacity: 0.46,
                shadow_radius: 5.2,
            }
        }
        NativeMascotState::Sleepy => {
            let breath = ((t * 0.9).sin() + 1.0) * 0.5;
            let sleepy_phase = (t + 0.9).rem_euclid(7.6);
            let nod = if sleepy_phase > 5.1 && sleepy_phase < 5.95 {
                (((sleepy_phase - 5.1) / 0.85) * std::f64::consts::PI).sin()
            } else {
                0.0
            };
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: nod * -0.7,
                scale_x: 1.0 + breath * 0.012,
                scale_y: 0.96 - breath * 0.012 + nod * 0.01,
                shell_alpha: 0.70,
                shadow_opacity: 0.18,
                shadow_radius: 3.0,
            }
        }
        NativeMascotState::WakeAngry => {
            let elapsed = wake_started_at
                .map(|started_at| now.saturating_duration_since(started_at).as_secs_f64())
                .unwrap_or(0.0);
            let fade = 1.0 - smoothstep_range(0.52, MASCOT_WAKE_ANGRY_SECONDS, elapsed);
            NativeMascotMotion {
                offset_x: (elapsed * 30.0).sin() * 1.85 * fade,
                offset_y: 0.0,
                scale_x: 1.0 + 0.045 * fade,
                scale_y: 1.0 - 0.04 * fade,
                shell_alpha: 1.0,
                shadow_opacity: 0.56,
                shadow_radius: 6.4,
            }
        }
        NativeMascotState::Idle => {
            let breath = ((t * 1.1).sin() + 1.0) * 0.5;
            NativeMascotMotion {
                offset_x: 0.0,
                offset_y: 0.0,
                scale_x: 1.0 + breath * 0.006,
                scale_y: 1.0 - breath * 0.004,
                shell_alpha: 1.0,
                shadow_opacity: 0.34,
                shadow_radius: 4.0,
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn native_mascot_color(
    state: NativeMascotState,
    _t: f64,
    wake_started_at: Option<Instant>,
    now: Instant,
) -> [f64; 4] {
    match state {
        NativeMascotState::Approval | NativeMascotState::Question => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::MessageBubble => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::Bouncing => [1.0, 0.48, 0.14, 1.0],
        NativeMascotState::Sleepy => [0.72, 0.30, 0.13, 1.0],
        NativeMascotState::WakeAngry => {
            let elapsed = wake_started_at
                .map(|started_at| now.saturating_duration_since(started_at).as_secs_f64())
                .unwrap_or(0.0);
            let blink = if (elapsed * 12.0).sin() >= 0.0 {
                1.0
            } else {
                0.0
            };
            [
                lerp(1.0, 1.0, blink),
                lerp(0.38, 0.48, blink),
                lerp(0.24, 0.14, blink),
                1.0,
            ]
        }
        NativeMascotState::Idle => [1.0, 0.48, 0.14, 1.0],
    }
}

#[cfg(target_os = "macos")]
fn native_mascot_lerp_motion(
    start: NativeMascotMotion,
    end: NativeMascotMotion,
    progress: f64,
) -> NativeMascotMotion {
    NativeMascotMotion {
        offset_x: lerp(start.offset_x, end.offset_x, progress),
        offset_y: lerp(start.offset_y, end.offset_y, progress),
        scale_x: lerp(start.scale_x, end.scale_x, progress),
        scale_y: lerp(start.scale_y, end.scale_y, progress),
        shell_alpha: lerp(start.shell_alpha, end.shell_alpha, progress),
        shadow_opacity: lerp(
            start.shadow_opacity as f64,
            end.shadow_opacity as f64,
            progress,
        ) as f32,
        shadow_radius: lerp(start.shadow_radius, end.shadow_radius, progress),
    }
}

#[cfg(target_os = "macos")]
fn smoothstep_unit(progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - (2.0 * progress))
}

#[cfg(target_os = "macos")]
fn smoothstep_range(edge0: f64, edge1: f64, value: f64) -> f64 {
    if (edge1 - edge0).abs() <= f64::EPSILON {
        return if value >= edge1 { 1.0 } else { 0.0 };
    }
    smoothstep_unit((value - edge0) / (edge1 - edge0))
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sync_native_mascot(handles: NativePanelHandles) {
    let now = Instant::now();
    let Some(state_mutex) = native_panel_state() else {
        return;
    };

    let frame = {
        let mut state = match state_mutex.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let has_status_completion = state
            .status_queue
            .iter()
            .any(|item| matches!(item.payload, NativeStatusQueuePayload::Completion(_)));
        let base_state =
            infer_native_mascot_base_state(state.last_snapshot.as_ref(), has_status_completion);
        let expanded = state.expanded;
        state.mascot_runtime.next_frame(base_state, expanded, now)
    };

    apply_native_mascot_frame(handles, frame);
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_native_mascot_frame(handles: NativePanelHandles, frame: NativeMascotFrame) {
    let mascot_shell = view_from_ptr(handles.mascot_shell);
    let mascot_body = view_from_ptr(handles.mascot_body);
    let mascot_left_eye = view_from_ptr(handles.mascot_left_eye);
    let mascot_right_eye = view_from_ptr(handles.mascot_right_eye);
    let mascot_mouth = view_from_ptr(handles.mascot_mouth);
    let mascot_bubble = view_from_ptr(handles.mascot_bubble);
    let mascot_sleep_label = text_field_from_ptr(handles.mascot_sleep_label);
    let motion = frame.motion;

    mascot_shell.setAlphaValue(motion.shell_alpha.clamp(0.0, 1.0));
    let body_width = 24.0 * motion.scale_x;
    let body_height = 20.0 * motion.scale_y;
    let body_x = 14.0 - (body_width / 2.0) + motion.offset_x;
    let body_y = 4.0 + MASCOT_VERTICAL_NUDGE_Y + motion.offset_y;
    mascot_body.setFrame(NSRect::new(
        NSPoint::new(body_x, body_y),
        NSSize::new(body_width, body_height),
    ));
    let stroke_color = ns_color(frame.color);
    let body_fill = if frame.state == NativeMascotState::Sleepy {
        ns_color([0.012, 0.012, 0.012, 1.0])
    } else {
        ns_color([0.02, 0.02, 0.02, 1.0])
    };
    if let Some(layer) = mascot_body.layer() {
        layer.setCornerRadius((body_width.min(body_height) * 0.28).max(4.0));
        layer.setBackgroundColor(Some(&body_fill.CGColor()));
        layer.setBorderColor(Some(&stroke_color.CGColor()));
        layer.setShadowColor(Some(&stroke_color.CGColor()));
        layer.setShadowOpacity(motion.shadow_opacity.clamp(0.0, 1.0));
        layer.setShadowRadius(motion.shadow_radius);
    }

    let blink_scale = native_mascot_blink_scale(frame.t, frame.state);
    let open_pct = if frame.state == NativeMascotState::Bouncing {
        ((motion.offset_y - 0.4) / 5.2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (eye_width_factor, eye_height_factor, eye_offset_factor) =
        native_mascot_eye_metrics(frame.state, open_pct);
    let eye_width = (body_width * eye_width_factor).max(2.4);
    let eye_height = (body_height * eye_height_factor * blink_scale).max(
        if matches!(
            frame.state,
            NativeMascotState::Question | NativeMascotState::Sleepy
        ) {
            1.6
        } else {
            2.4
        },
    );
    let eye_center_y = body_y + body_height * 0.58;
    let eye_offset_x = body_width * eye_offset_factor;
    set_mascot_face_part_frame(
        mascot_left_eye,
        body_x + (body_width / 2.0) - eye_offset_x,
        eye_center_y,
        eye_width,
        eye_height,
    );
    set_mascot_face_part_frame(
        mascot_right_eye,
        body_x + (body_width / 2.0) + eye_offset_x,
        eye_center_y,
        eye_width,
        eye_height,
    );

    let mouth_center_y = body_y + body_height * 0.32;
    let (mouth_width, mouth_height, mouth_alpha) =
        native_mascot_mouth_metrics(frame.state, body_width, body_height, open_pct);
    set_mascot_face_part_frame(
        mascot_mouth,
        body_x + (body_width / 2.0),
        mouth_center_y,
        mouth_width,
        mouth_height,
    );
    mascot_mouth.setAlphaValue(mouth_alpha);
    if let Some(layer) = mascot_mouth.layer() {
        layer.setCornerRadius((mouth_height / 2.0).max(0.8));
    }

    let bubble_visible = frame.state == NativeMascotState::MessageBubble;
    let bubble_phase = (frame.t % 1.8) / 1.8;
    let bubble_pop = smoothstep_range(0.0, 0.28, bubble_phase)
        * (1.0 - smoothstep_range(0.78, 1.0, bubble_phase));
    mascot_bubble.setHidden(!bubble_visible || bubble_pop <= 0.06);
    mascot_bubble.setAlphaValue(if bubble_visible { bubble_pop } else { 0.0 });
    mascot_bubble.setFrame(NSRect::new(
        NSPoint::new(
            body_x + body_width * 0.58,
            body_y + body_height * 0.86 + (bubble_pop * 1.4),
        ),
        NSSize::new(body_width * 0.54, body_height * 0.30),
    ));

    let sleep_visible = frame.state == NativeMascotState::Sleepy;
    let sleep_phase = (frame.t % 2.5) / 2.5;
    let sleep_rise = smoothstep_range(0.0, 0.66, sleep_phase);
    let sleep_fade = 1.0 - smoothstep_range(0.58, 1.0, sleep_phase);
    let sleep_alpha = if sleep_visible {
        sleep_rise * sleep_fade
    } else {
        0.0
    };
    mascot_sleep_label.setHidden(sleep_alpha <= 0.03);
    mascot_sleep_label.setAlphaValue(sleep_alpha);
    mascot_sleep_label.setFrame(NSRect::new(
        NSPoint::new(
            body_x + body_width * 0.66 + sleep_rise * body_width * 0.16,
            body_y + body_height * 0.78 + sleep_rise * body_width * 0.16,
        ),
        NSSize::new(10.0, 10.0),
    ));

    for face_part in [mascot_left_eye, mascot_right_eye, mascot_mouth] {
        face_part.setHidden(false);
        if let Some(layer) = face_part.layer() {
            layer.setShadowOpacity(0.22);
            layer.setShadowRadius(6.0);
        }
    }

    mascot_shell.displayIfNeeded();
    mascot_body.displayIfNeeded();
    mascot_left_eye.displayIfNeeded();
    mascot_right_eye.displayIfNeeded();
    mascot_mouth.displayIfNeeded();
    mascot_bubble.displayIfNeeded();
    mascot_sleep_label.displayIfNeeded();
}

#[cfg(target_os = "macos")]
fn native_mascot_blink_scale(t: f64, state: NativeMascotState) -> f64 {
    if state == NativeMascotState::WakeAngry {
        return 1.0;
    }

    let phase = (t + 0.35).rem_euclid(4.8);
    let mut scale = if phase < 0.09 {
        1.0 - (phase / 0.09) * 0.9
    } else if phase < 0.18 {
        0.1 + ((phase - 0.09) / 0.09) * 0.9
    } else {
        1.0
    };

    if state == NativeMascotState::Sleepy {
        let sleepy_phase = (t + 1.1).rem_euclid(7.4);
        scale *= 0.72;
        if sleepy_phase > 4.7 && sleepy_phase < 5.45 {
            let pct = (sleepy_phase - 4.7) / 0.75;
            scale = if pct < 0.5 {
                0.18
            } else {
                0.18 + (pct - 0.5) * 0.36
            };
        }
        return scale.max(0.16);
    }

    match state {
        NativeMascotState::Approval => (scale * 0.92).max(0.34),
        NativeMascotState::Question => scale.max(0.48),
        NativeMascotState::Bouncing => scale.max(0.72),
        _ => scale,
    }
}

#[cfg(target_os = "macos")]
fn native_mascot_eye_metrics(state: NativeMascotState, open_pct: f64) -> (f64, f64, f64) {
    match state {
        NativeMascotState::Bouncing => {
            (lerp(0.24, 0.20, open_pct), lerp(0.24, 0.20, open_pct), 0.18)
        }
        NativeMascotState::Approval => (0.22, 0.22, 0.18),
        NativeMascotState::Question => (0.26, 0.055, 0.20),
        NativeMascotState::Sleepy => (0.22, 0.085, 0.20),
        NativeMascotState::WakeAngry => (0.20, 0.12, 0.18),
        NativeMascotState::MessageBubble => (0.14, 0.16, 0.20),
        NativeMascotState::Idle => (0.24, 0.24, 0.21),
    }
}

#[cfg(target_os = "macos")]
fn native_mascot_mouth_metrics(
    state: NativeMascotState,
    body_width: f64,
    body_height: f64,
    open_pct: f64,
) -> (f64, f64, f64) {
    match state {
        NativeMascotState::Approval => (body_width * 0.34, body_height * 0.11, 1.0),
        NativeMascotState::Question => (body_width * 0.18, body_height * 0.10, 1.0),
        NativeMascotState::Sleepy => (body_width * 0.16, body_height * 0.095, 0.92),
        NativeMascotState::WakeAngry => (body_width * 0.34, body_height * 0.105, 1.0),
        NativeMascotState::MessageBubble => (body_width * 0.16, body_height * 0.085, 1.0),
        NativeMascotState::Bouncing => (
            lerp(body_width * 0.21, body_width * 0.28, open_pct),
            lerp(body_height * 0.08, body_height * 0.30, open_pct),
            1.0,
        ),
        NativeMascotState::Idle => (
            lerp(body_width * 0.20, body_width * 0.32, open_pct),
            lerp(body_height * 0.09, body_height * 0.13, open_pct),
            1.0,
        ),
    }
}

#[cfg(target_os = "macos")]
fn set_mascot_face_part_frame(
    view: &NSView,
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
) {
    view.setFrame(NSRect::new(
        NSPoint::new(center_x - (width / 2.0), center_y - (height / 2.0)),
        NSSize::new(width.max(1.0), height.max(1.0)),
    ));
}

#[cfg(target_os = "macos")]
fn centered_top_frame(screen_frame: NSRect, size: NSSize) -> NSRect {
    let snapped_width = size.width.max(1.0).round();
    let snapped_height = size.height.max(1.0).round();
    let top_edge = screen_frame.origin.y + screen_frame.size.height;
    NSRect::new(
        NSPoint::new(
            (screen_frame.origin.x + ((screen_frame.size.width - snapped_width) / 2.0).max(0.0))
                .round(),
            (top_edge - snapped_height).round(),
        ),
        NSSize::new(snapped_width, snapped_height),
    )
}

#[cfg(target_os = "macos")]
fn compact_pill_height_for_screen(screen: &NSScreen) -> f64 {
    let safe_top = screen.safeAreaInsets().top;
    if safe_top > 0.0 {
        return safe_top;
    }

    let frame = screen.frame();
    let visible = screen.visibleFrame();
    let menu_bar_height =
        (frame.origin.y + frame.size.height) - (visible.origin.y + visible.size.height);
    if menu_bar_height > 5.0 {
        return menu_bar_height;
    }

    if let Some(mtm) = MainThreadMarker::new() {
        if let Some(main_screen) = NSScreen::mainScreen(mtm) {
            let main_frame = main_screen.frame();
            let main_visible = main_screen.visibleFrame();
            let main_menu = (main_frame.origin.y + main_frame.size.height)
                - (main_visible.origin.y + main_visible.size.height);
            if main_menu > 5.0 {
                return main_menu;
            }
        }
    }

    DEFAULT_COMPACT_PILL_HEIGHT
}

#[cfg(target_os = "macos")]
fn notch_width_for_screen(screen: &NSScreen) -> f64 {
    let left_width = screen.auxiliaryTopLeftArea().size.width;
    let right_width = screen.auxiliaryTopRightArea().size.width;
    if left_width > 0.0 || right_width > 0.0 {
        return (screen.frame().size.width - left_width - right_width).max(0.0);
    }

    (screen.frame().size.width * 0.18).clamp(160.0, 240.0)
}

#[cfg(target_os = "macos")]
fn screen_has_camera_housing(screen: &NSScreen) -> bool {
    let left_width = screen.auxiliaryTopLeftArea().size.width;
    let right_width = screen.auxiliaryTopRightArea().size.width;
    let center_gap = (screen.frame().size.width - left_width - right_width).max(0.0);
    (left_width > 0.0 || right_width > 0.0) && center_gap > 40.0
}

#[cfg(target_os = "macos")]
fn shell_width_for_screen(screen: &NSScreen, compact_height: f64) -> f64 {
    let mascot_size = (compact_height - 6.0).min(27.0).max(20.0);
    let compact_wing = mascot_size + 14.0;
    let notch_width = notch_width_for_screen(screen);
    let screen_extra = (screen.frame().size.width * 0.012).clamp(10.0, 22.0);
    let max_width = (screen.frame().size.width - 24.0)
        .min(DEFAULT_PANEL_CANVAS_WIDTH)
        .max(DEFAULT_COMPACT_PILL_WIDTH);
    (notch_width + compact_wing * 2.0 + 10.0 + screen_extra)
        .clamp(DEFAULT_COMPACT_PILL_WIDTH, max_width)
}

#[cfg(target_os = "macos")]
fn compact_pill_width_for_screen(screen: &NSScreen, compact_height: f64) -> f64 {
    shell_width_for_screen(screen, compact_height)
}

#[cfg(target_os = "macos")]
fn compact_pill_height_for_screen_rect(screen: Option<&NSScreen>, fallback_rect: NSRect) -> f64 {
    screen
        .map(compact_pill_height_for_screen)
        .unwrap_or_else(|| {
            if fallback_rect.size.height > 0.0 {
                DEFAULT_COMPACT_PILL_HEIGHT
            } else {
                25.0
            }
        })
}

#[cfg(target_os = "macos")]
fn compact_pill_width_for_screen_rect(screen: Option<&NSScreen>, compact_height: f64) -> f64 {
    screen
        .map(|screen| compact_pill_width_for_screen(screen, compact_height))
        .unwrap_or(DEFAULT_COMPACT_PILL_WIDTH)
}

#[cfg(target_os = "macos")]
fn expanded_panel_width_for_screen(screen: &NSScreen) -> f64 {
    let compact_height = compact_pill_height_for_screen(screen);
    shell_width_for_screen(screen, compact_height)
}

#[cfg(target_os = "macos")]
fn expanded_panel_width_for_screen_rect(screen: Option<&NSScreen>, fallback_rect: NSRect) -> f64 {
    screen
        .map(expanded_panel_width_for_screen)
        .unwrap_or(DEFAULT_COMPACT_PILL_WIDTH.min(fallback_rect.size.width.max(1.0)))
}

#[cfg(target_os = "macos")]
fn panel_canvas_width_for_screen(screen: &NSScreen, compact_height: f64) -> f64 {
    let compact_width = compact_pill_width_for_screen(screen, compact_height);
    expanded_panel_width_for_screen(screen)
        .max(compact_width + 24.0)
        .max(DEFAULT_PANEL_CANVAS_WIDTH)
}

#[cfg(target_os = "macos")]
fn panel_canvas_width_for_screen_rect(
    screen: Option<&NSScreen>,
    compact_height: f64,
    fallback_rect: NSRect,
) -> f64 {
    screen
        .map(|screen| panel_canvas_width_for_screen(screen, compact_height))
        .unwrap_or_else(|| fallback_rect.size.width.max(DEFAULT_PANEL_CANVAS_WIDTH))
}

#[cfg(target_os = "macos")]
fn compact_pill_frame(panel: &NSPanel, content_size: NSSize) -> NSRect {
    let compact_height = compact_pill_height_for_screen_rect(
        panel.screen().as_deref(),
        resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame()),
    );
    let compact_width =
        compact_pill_width_for_screen_rect(panel.screen().as_deref(), compact_height);
    let expanded_width = expanded_panel_width_for_screen_rect(
        panel.screen().as_deref(),
        resolve_screen_frame_for_panel(panel).unwrap_or(panel.frame()),
    );
    island_bar_frame(
        content_size,
        0.0,
        compact_width,
        expanded_width,
        compact_height,
        0.0,
    )
}

#[cfg(target_os = "macos")]
fn island_bar_frame(
    content_size: NSSize,
    _progress: f64,
    compact_width: f64,
    _expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> NSRect {
    NSRect::new(
        NSPoint::new(
            (content_size.width - compact_width) / 2.0,
            content_size.height - compact_height - drop_offset,
        ),
        NSSize::new(compact_width, compact_height),
    )
}

#[cfg(target_os = "macos")]
fn left_shoulder_frame(pill_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            pill_frame.origin.x - COMPACT_SHOULDER_SIZE,
            pill_frame.origin.y + pill_frame.size.height - COMPACT_SHOULDER_SIZE,
        ),
        NSSize::new(COMPACT_SHOULDER_SIZE, COMPACT_SHOULDER_SIZE),
    )
}

#[cfg(target_os = "macos")]
fn right_shoulder_frame(pill_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            pill_frame.origin.x + pill_frame.size.width,
            pill_frame.origin.y + pill_frame.size.height - COMPACT_SHOULDER_SIZE,
        ),
        NSSize::new(COMPACT_SHOULDER_SIZE, COMPACT_SHOULDER_SIZE),
    )
}

fn expanded_background_frame(
    content_size: NSSize,
    visible_height: f64,
    _bar_progress: f64,
    height_progress: f64,
    _compact_width: f64,
    expanded_width: f64,
    compact_height: f64,
    drop_offset: f64,
) -> NSRect {
    let height_progress = height_progress.clamp(0.0, 1.0);
    let visible_height = visible_height
        .max(COLLAPSED_PANEL_HEIGHT)
        .min(content_size.height.max(COLLAPSED_PANEL_HEIGHT));
    let height = lerp(
        compact_height,
        (visible_height - drop_offset).max(compact_height),
        height_progress,
    );
    NSRect::new(
        NSPoint::new(
            (content_size.width - expanded_width) / 2.0,
            content_size.height - drop_offset - height,
        ),
        NSSize::new(expanded_width, height),
    )
}

#[cfg(target_os = "macos")]
fn expanded_cards_frame(container_frame: NSRect, compact_height: f64) -> NSRect {
    let body_height = (container_frame.size.height
        - compact_height
        - EXPANDED_CONTENT_TOP_GAP
        - EXPANDED_CONTENT_BOTTOM_INSET)
        .max(0.0);
    NSRect::new(
        NSPoint::new(EXPANDED_CARDS_SIDE_INSET, EXPANDED_CONTENT_BOTTOM_INSET),
        NSSize::new(
            expanded_cards_width(container_frame.size.width),
            body_height,
        ),
    )
}

#[cfg(target_os = "macos")]
fn expanded_separator_frame(container_frame: NSRect, compact_height: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(
            14.0,
            (container_frame.size.height - compact_height - 0.5).max(0.0),
        ),
        NSSize::new((container_frame.size.width - 28.0).max(0.0), 1.0),
    )
}

#[cfg(target_os = "macos")]
fn expanded_total_height(
    snapshot: &RuntimeSnapshot,
    compact_height: f64,
    shared_body_height: Option<f64>,
) -> f64 {
    let estimated_height = estimated_expanded_body_height(snapshot);
    let body_height = shared_body_height
        .map(|shared_height| shared_height.max(estimated_height))
        .unwrap_or(estimated_height)
        .min(EXPANDED_MAX_BODY_HEIGHT);
    compact_height + EXPANDED_CONTENT_TOP_GAP + EXPANDED_CONTENT_BOTTOM_INSET + body_height
}

#[cfg(target_os = "macos")]
fn panel_transition_canvas_height(start_height: f64, target_height: f64) -> f64 {
    start_height.max(target_height).max(COLLAPSED_PANEL_HEIGHT)
}

#[cfg(target_os = "macos")]
fn expanded_cards_width(container_width: f64) -> f64 {
    (container_width - (EXPANDED_CARDS_SIDE_INSET * 2.0)).max(0.0)
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
fn estimated_expanded_body_height(snapshot: &RuntimeSnapshot) -> f64 {
    estimated_expanded_content_height(snapshot).min(EXPANDED_MAX_BODY_HEIGHT)
}

#[cfg(target_os = "macos")]
fn estimated_expanded_content_height(snapshot: &RuntimeSnapshot) -> f64 {
    let status_queue = native_status_queue_surface_items();
    if !status_queue.is_empty() {
        let cards = status_queue
            .iter()
            .map(native_status_queue_card_height)
            .sum::<f64>();
        let gaps = EXPANDED_CARD_GAP * (status_queue.len().saturating_sub(1) as f64);
        return cards + gaps;
    }

    let pending_permissions = displayed_default_pending_permissions(snapshot);
    let pending_questions = displayed_default_pending_questions(snapshot);
    let prompt_assist_sessions = displayed_prompt_assist_sessions(snapshot);
    let sessions = displayed_sessions(snapshot, &prompt_assist_sessions);
    let mut heights = Vec::new();

    for pending in pending_permissions.iter() {
        heights.push(pending_permission_card_height(pending));
    }
    for pending in pending_questions.iter() {
        heights.push(pending_question_card_height(pending));
    }
    heights.extend(prompt_assist_sessions.iter().map(prompt_assist_card_height));
    heights.extend(sessions.iter().map(estimated_card_height));

    if heights.is_empty() {
        return 84.0;
    }

    let cards = heights.iter().sum::<f64>();
    let gaps = EXPANDED_CARD_GAP * (heights.len().saturating_sub(1) as f64);
    cards + gaps
}

#[cfg(target_os = "macos")]
fn native_status_queue_card_height(item: &NativeStatusQueueItem) -> f64 {
    match &item.payload {
        NativeStatusQueuePayload::Approval(pending) => pending_permission_card_height(pending),
        NativeStatusQueuePayload::Completion(session) => completion_card_height(session),
    }
}

#[cfg(target_os = "macos")]
fn status_queue_exit_duration() -> Duration {
    Duration::from_millis(PANEL_CARD_EXIT_MS.max(220) + STATUS_QUEUE_EXIT_EXTRA_MS)
}

#[cfg(target_os = "macos")]
fn pending_permission_card_height(pending: &PendingPermissionView) -> f64 {
    let body = display_snippet(pending.tool_description.as_deref(), 78)
        .unwrap_or_else(|| "Waiting for your approval".to_string());
    pending_like_card_height(&body, 92.0, 120.0)
}

#[cfg(target_os = "macos")]
fn pending_question_card_height(pending: &PendingQuestionView) -> f64 {
    let body = display_snippet(Some(&pending.text), 82)
        .unwrap_or_else(|| "Waiting for your answer".to_string());
    let min_height = if pending.options.is_empty() {
        PENDING_QUESTION_CARD_MIN_HEIGHT
    } else {
        PENDING_QUESTION_CARD_MIN_HEIGHT + 6.0
    };
    pending_like_card_height(
        &body,
        min_height,
        144.0_f64.max(PENDING_QUESTION_CARD_MAX_HEIGHT),
    )
}

#[cfg(target_os = "macos")]
fn prompt_assist_card_height(_session: &SessionSnapshotView) -> f64 {
    pending_like_card_height(
        "A command may be waiting for approval in the Codex terminal. Allow or deny it there.",
        92.0,
        108.0,
    )
}

#[cfg(target_os = "macos")]
fn completion_card_height(session: &SessionSnapshotView) -> f64 {
    let preview = completion_preview_text(session);
    let body_height = estimated_chat_body_height(&preview, estimated_default_chat_body_width(), 2);
    (CARD_HEADER_HEIGHT + CARD_CONTENT_BOTTOM_INSET + body_height).max(92.0)
}

#[cfg(target_os = "macos")]
fn pending_like_card_height(body: &str, min_height: f64, max_height: f64) -> f64 {
    let body_height = estimated_chat_body_height(body, estimated_default_chat_body_width(), 2);
    (58.0
        + CARD_PENDING_ACTION_Y
        + CARD_PENDING_ACTION_HEIGHT
        + CARD_PENDING_ACTION_GAP
        + body_height)
        .clamp(min_height, max_height)
}

#[cfg(target_os = "macos")]
fn session_card_collapsed_height(target_height: f64, is_compact: bool) -> f64 {
    let limit = if is_compact { 52.0 } else { 64.0 };
    let factor = if is_compact { 0.76 } else { 0.58 };
    target_height
        .mul_add(factor, 0.0)
        .round()
        .clamp(34.0, limit)
}

#[cfg(target_os = "macos")]
fn clear_card_animation_layouts() {
    if let Some(layouts) = CARD_ANIMATION_LAYOUTS.get() {
        if let Ok(mut layouts) = layouts.lock() {
            layouts.clear();
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn register_card_animation_layout(view: &NSView, frame: NSRect, collapsed_height: f64) {
    let layouts = CARD_ANIMATION_LAYOUTS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut layouts) = layouts.lock() {
        layouts.insert(
            (view as *const NSView) as usize,
            CardAnimationLayout {
                frame,
                collapsed_height: collapsed_height.clamp(1.0, frame.size.height.max(1.0)),
            },
        );
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn card_animation_layout(view: &NSView) -> Option<CardAnimationLayout> {
    CARD_ANIMATION_LAYOUTS
        .get()
        .and_then(|layouts| layouts.lock().ok())
        .and_then(|layouts| layouts.get(&((view as *const NSView) as usize)).copied())
}

#[cfg(target_os = "macos")]
fn displayed_sessions(
    snapshot: &RuntimeSnapshot,
    prompt_assist_sessions: &[SessionSnapshotView],
) -> Vec<SessionSnapshotView> {
    let blocked_session_ids = blocked_session_ids(snapshot, prompt_assist_sessions);
    let mut sessions = snapshot
        .sessions
        .iter()
        .filter(|session| !should_hide_legacy_opencode_session(session))
        .filter(|session| !blocked_session_ids.contains(&session.session_id))
        .cloned()
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| {
        let priority_diff = status_priority(&left.status).cmp(&status_priority(&right.status));
        if priority_diff == std::cmp::Ordering::Equal {
            right.last_activity.cmp(&left.last_activity)
        } else {
            priority_diff
        }
    });
    sessions.truncate(MAX_VISIBLE_SESSIONS);
    sessions
}

#[cfg(target_os = "macos")]
fn displayed_pending_permissions(snapshot: &RuntimeSnapshot) -> Vec<PendingPermissionView> {
    let mut permissions = if snapshot.pending_permissions.is_empty() {
        snapshot.pending_permission.clone().into_iter().collect()
    } else {
        snapshot.pending_permissions.clone()
    };
    permissions.sort_by(|left, right| left.requested_at.cmp(&right.requested_at));
    permissions
}

#[cfg(target_os = "macos")]
fn displayed_default_pending_permissions(snapshot: &RuntimeSnapshot) -> Vec<PendingPermissionView> {
    let permission = snapshot
        .pending_permission
        .clone()
        .or_else(|| displayed_pending_permissions(snapshot).into_iter().next());
    permission.into_iter().collect()
}

#[cfg(target_os = "macos")]
fn displayed_pending_questions(snapshot: &RuntimeSnapshot) -> Vec<PendingQuestionView> {
    let mut questions = if snapshot.pending_questions.is_empty() {
        snapshot.pending_question.clone().into_iter().collect()
    } else {
        snapshot.pending_questions.clone()
    };
    questions.sort_by(|left, right| left.requested_at.cmp(&right.requested_at));
    questions
}

#[cfg(target_os = "macos")]
fn displayed_default_pending_questions(snapshot: &RuntimeSnapshot) -> Vec<PendingQuestionView> {
    let question = snapshot
        .pending_question
        .clone()
        .or_else(|| displayed_pending_questions(snapshot).into_iter().next());
    question.into_iter().collect()
}

#[cfg(target_os = "macos")]
fn blocked_session_ids(
    snapshot: &RuntimeSnapshot,
    prompt_assist_sessions: &[SessionSnapshotView],
) -> HashSet<String> {
    displayed_pending_permissions(snapshot)
        .into_iter()
        .map(|pending| pending.session_id)
        .chain(
            displayed_pending_questions(snapshot)
                .into_iter()
                .map(|pending| pending.session_id),
        )
        .chain(
            prompt_assist_sessions
                .iter()
                .map(|session| session.session_id.clone()),
        )
        .filter(|session_id| !session_id.trim().is_empty())
        .collect()
}

#[cfg(target_os = "macos")]
fn displayed_prompt_assist_sessions(snapshot: &RuntimeSnapshot) -> Vec<SessionSnapshotView> {
    let live_pending_session_ids = live_pending_session_ids(snapshot);
    let now = Utc::now();
    let mut sessions = snapshot
        .sessions
        .iter()
        .filter(|session| !live_pending_session_ids.contains(&session.session_id))
        .filter(|session| is_prompt_assist_session(session, now))
        .cloned()
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| right.last_activity.cmp(&left.last_activity));
    sessions.truncate(1);
    sessions
}

#[cfg(target_os = "macos")]
fn live_pending_session_ids(snapshot: &RuntimeSnapshot) -> HashSet<String> {
    displayed_pending_permissions(snapshot)
        .into_iter()
        .map(|pending| pending.session_id)
        .chain(
            displayed_pending_questions(snapshot)
                .into_iter()
                .map(|pending| pending.session_id),
        )
        .filter(|session_id| !session_id.trim().is_empty())
        .collect()
}

#[cfg(target_os = "macos")]
fn is_prompt_assist_session(session: &SessionSnapshotView, now: chrono::DateTime<Utc>) -> bool {
    if session.source.to_ascii_lowercase() != "codex" {
        return false;
    }

    let status = normalize_status(&session.status);
    if status != "processing" && status != "running" {
        return false;
    }

    let age_seconds = (now - session.last_activity).num_seconds();
    let stale_seconds = if status == "running" {
        PROMPT_ASSIST_RUNNING_SECONDS
    } else {
        PROMPT_ASSIST_PROCESSING_SECONDS
    };
    age_seconds >= stale_seconds && age_seconds <= PROMPT_ASSIST_RECENT_SECONDS
}

#[cfg(target_os = "macos")]
fn estimated_card_height(session: &SessionSnapshotView) -> f64 {
    if is_long_idle_session(session) || !session_has_visible_card_body(session) {
        return 58.0;
    }

    let content_height = estimated_session_card_content_height(
        session_prompt_preview(session).as_deref(),
        session_reply_preview(session).as_deref(),
        session_tool_preview(session).is_some(),
    );
    (CARD_HEADER_HEIGHT + content_height).max(58.0)
}

#[cfg(target_os = "macos")]
fn estimated_session_card_content_height(
    prompt: Option<&str>,
    reply: Option<&str>,
    has_tool: bool,
) -> f64 {
    let mut content_height = CARD_CONTENT_BOTTOM_INSET;
    let has_prompt = prompt.is_some_and(|value| !value.is_empty());
    let has_reply = reply.is_some_and(|value| !value.is_empty());

    if has_tool {
        content_height += 22.0;
        if has_reply || has_prompt {
            content_height += CARD_TOOL_GAP;
        }
    }
    if let Some(body) = reply.filter(|value| !value.is_empty()) {
        content_height += estimated_chat_body_height(body, estimated_default_chat_body_width(), 2);
        if has_prompt {
            content_height += CARD_CHAT_GAP;
        }
    }
    if let Some(body) = prompt.filter(|value| !value.is_empty()) {
        content_height += estimated_chat_body_height(body, estimated_default_chat_body_width(), 1);
    }

    content_height
}

#[cfg(target_os = "macos")]
fn session_prompt_preview(session: &SessionSnapshotView) -> Option<String> {
    display_snippet(session.last_user_prompt.as_deref(), 68)
}

#[cfg(target_os = "macos")]
fn session_reply_preview(session: &SessionSnapshotView) -> Option<String> {
    display_snippet(
        session
            .last_assistant_message
            .as_deref()
            .or(session.tool_description.as_deref()),
        92,
    )
}

#[cfg(target_os = "macos")]
fn session_tool_preview(session: &SessionSnapshotView) -> Option<(String, Option<String>)> {
    let tool_name = session.current_tool.as_deref()?.trim();
    if tool_name.is_empty() {
        return None;
    }

    Some((
        tool_name.to_string(),
        display_snippet(session.tool_description.as_deref(), 48),
    ))
}

#[cfg(target_os = "macos")]
fn session_has_visible_card_body(session: &SessionSnapshotView) -> bool {
    session_prompt_preview(session).is_some()
        || session_reply_preview(session).is_some()
        || session_tool_preview(session).is_some()
}

#[cfg(target_os = "macos")]
fn completion_preview_text(session: &SessionSnapshotView) -> String {
    session_reply_preview(session).unwrap_or_else(|| "Task complete".to_string())
}

#[cfg(target_os = "macos")]
fn is_long_idle_session(session: &SessionSnapshotView) -> bool {
    normalize_status(&session.status) == "idle"
        && (Utc::now() - session.last_activity).num_minutes() > 15
}

#[cfg(target_os = "macos")]
fn should_hide_legacy_opencode_session(session: &SessionSnapshotView) -> bool {
    let source = session.source.to_ascii_lowercase();
    source == "opencode"
        && session.session_id.starts_with("open-")
        && session.cwd.is_none()
        && session.project_name.is_none()
        && session.model.is_none()
        && session.current_tool.is_none()
        && session.tool_description.is_none()
        && session.last_user_prompt.is_none()
        && session.last_assistant_message.is_none()
}

#[cfg(target_os = "macos")]
fn status_priority(status: &str) -> u8 {
    match normalize_status(status).as_str() {
        "waitingapproval" | "waitingquestion" => 0,
        "running" => 1,
        "processing" => 2,
        _ => 3,
    }
}

#[cfg(target_os = "macos")]
fn normalize_status(status: &str) -> String {
    status.to_ascii_lowercase()
}

#[cfg(target_os = "macos")]
fn format_source(source: &str) -> String {
    match source.to_ascii_lowercase().as_str() {
        "claude" => "Claude".to_string(),
        "codex" => "Codex".to_string(),
        "cursor" => "Cursor".to_string(),
        "gemini" => "Gemini".to_string(),
        "copilot" => "Copilot".to_string(),
        "qoder" => "Qoder".to_string(),
        "codebuddy" => "CodeBuddy".to_string(),
        "opencode" => "OpenCode".to_string(),
        "openclaw" => "OpenClaw".to_string(),
        other => {
            let mut chars = other.chars();
            if let Some(first) = chars.next() {
                format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.collect::<String>()
                )
            } else {
                "Unknown".to_string()
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn format_status(status: &str) -> String {
    match normalize_status(status).as_str() {
        "running" => "Running".to_string(),
        "processing" => "Thinking".to_string(),
        "waitingapproval" => "Approval".to_string(),
        "waitingquestion" => "Question".to_string(),
        "idle" => "Idle".to_string(),
        other => other.to_string(),
    }
}

#[cfg(target_os = "macos")]
fn session_title(session: &SessionSnapshotView) -> String {
    let project_name = display_project_name(session);
    if project_name != "Session" {
        return project_name;
    }
    format!(
        "{} {}",
        format_source(&session.source),
        short_session_id(&session.session_id)
    )
}

#[cfg(target_os = "macos")]
fn display_project_name(session: &SessionSnapshotView) -> String {
    let raw = session
        .project_name
        .as_deref()
        .or(session.cwd.as_deref())
        .unwrap_or("Session");
    raw.split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
        .next_back()
        .map(|segment| segment.replace(':', ""))
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "Session".to_string())
}

#[cfg(target_os = "macos")]
fn compact_title(value: &str, max_length: usize) -> String {
    let text = value.trim();
    if text.chars().count() <= max_length {
        return text.to_string();
    }
    let head_length = (((max_length - 1) as f64) * 0.58).ceil() as usize;
    let tail_length = max_length.saturating_sub(1 + head_length);
    let head = text.chars().take(head_length).collect::<String>();
    let tail = text
        .chars()
        .rev()
        .take(tail_length)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{head}…{tail}")
}

#[cfg(target_os = "macos")]
fn short_session_id(session_id: &str) -> String {
    session_id
        .split_once('-')
        .map(|(_, tail)| tail.chars().take(6).collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "------".to_string())
}

#[cfg(target_os = "macos")]
fn time_ago(last_activity: chrono::DateTime<chrono::Utc>) -> String {
    let diff = Utc::now() - last_activity;
    if diff.num_minutes() < 1 {
        "now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h", diff.num_hours())
    } else {
        format!("{}d", diff.num_days())
    }
}

#[cfg(target_os = "macos")]
fn session_meta_line(session: &SessionSnapshotView) -> String {
    let mut parts = vec![format!("#{}", short_session_id(&session.session_id))];
    if let Some(model) = session
        .model
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        parts.push(model.to_string());
    }
    parts.push(time_ago(session.last_activity));
    parts.join(" · ")
}

#[cfg(target_os = "macos")]
fn display_snippet(value: Option<&str>, max_chars: usize) -> Option<String> {
    let value = value?.replace(['\r', '\n'], " ");
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return None;
    }
    let text = compact.replace(['`', '*', '_', '~', '|'], "");
    if text.chars().count() <= max_chars {
        Some(text)
    } else {
        Some(format!(
            "{}…",
            text.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        ))
    }
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

#[cfg(target_os = "macos")]
fn compact_headline_y(bar_height: f64) -> f64 {
    ((bar_height - COMPACT_HEADLINE_LABEL_HEIGHT) / 2.0).round() + COMPACT_HEADLINE_VERTICAL_NUDGE_Y
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn compact_headline_should_hide(handles: NativePanelHandles) -> bool {
    let panel = panel_from_ptr(handles.panel);
    panel
        .screen()
        .as_deref()
        .is_some_and(screen_has_camera_housing)
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn relayout_compact_content(handles: NativePanelHandles, bar_size: NSSize) {
    let top_highlight = view_from_ptr(handles.top_highlight);
    let mascot_shell = view_from_ptr(handles.mascot_shell);
    let headline = text_field_from_ptr(handles.headline);
    let active_count_clip = clip_view_from_ptr(handles.active_count_clip);
    let slash = text_field_from_ptr(handles.slash);
    let total_count = text_field_from_ptr(handles.total_count);

    let mascot_size = (bar_size.height - 9.0).clamp(24.0, 28.0);
    let left_inset = ((bar_size.height - mascot_size) / 2.0).clamp(8.0, 12.0);
    let title_x = left_inset + mascot_size + 8.0;
    let right_padding = 4.0;
    let active_width = ACTIVE_COUNT_SLOT_WIDTH;
    let slash_width = 10.0;
    let total_width = 24.0;
    let metrics_gap = 0.0;
    let group_right = (bar_size.width - right_padding).max(208.0);
    let total_x = group_right - total_width;
    let slash_x = total_x - metrics_gap - slash_width;
    let right_start = (slash_x - metrics_gap - active_width + ACTIVE_COUNT_SLOT_NUDGE_X).max(168.0);
    let headline_width = (right_start - title_x - 8.0).max(96.0);
    let digit_y = ((bar_size.height - ACTIVE_COUNT_LABEL_HEIGHT) / 2.0).round() - 0.5;
    let slash_y = digit_y;

    top_highlight.setFrame(NSRect::new(
        NSPoint::new(12.0, bar_size.height - 1.0),
        NSSize::new((bar_size.width - 24.0).max(0.0), 1.0),
    ));
    mascot_shell.setFrame(NSRect::new(
        NSPoint::new(
            left_inset,
            ((bar_size.height - mascot_size) / 2.0).round() + MASCOT_VERTICAL_NUDGE_Y,
        ),
        NSSize::new(mascot_size, mascot_size),
    ));
    headline.setFrame(NSRect::new(
        NSPoint::new(title_x, compact_headline_y(bar_size.height)),
        NSSize::new(headline_width, COMPACT_HEADLINE_LABEL_HEIGHT),
    ));
    headline.setHidden(compact_headline_should_hide(handles));
    active_count_clip.setFrame(NSRect::new(
        NSPoint::new(right_start, digit_y),
        NSSize::new(active_width, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    slash.setFrame(NSRect::new(
        NSPoint::new(slash_x, slash_y),
        NSSize::new(slash_width, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    total_count.setFrame(NSRect::new(
        NSPoint::new(total_x, digit_y),
        NSSize::new(total_width, 24.0),
    ));
    sync_active_count_marquee(handles);
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sync_active_count_marquee(handles: NativePanelHandles) {
    let active_count_clip = clip_view_from_ptr(handles.active_count_clip);
    let active_count = text_field_from_ptr(handles.active_count);
    let active_count_next = text_field_from_ptr(handles.active_count_next);
    let Some(source) = ACTIVE_COUNT_SCROLL_TEXT.get() else {
        active_count.setStringValue(&NSString::from_str("9"));
        active_count_next.setStringValue(&NSString::from_str("9"));
        return;
    };
    let value = source
        .lock()
        .ok()
        .map(|text| text.clone())
        .unwrap_or_else(|| "9".to_string());
    let chars = value.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        active_count.setHidden(false);
        active_count_next.setHidden(true);
        active_count.setStringValue(&NSString::from_str("0"));
        active_count_next.setStringValue(&NSString::from_str("0"));
        active_count_clip.scrollToPoint(NSPoint::new(0.0, 0.0));
        return;
    }

    let started = ACTIVE_COUNT_SCROLL_STARTED.get_or_init(Instant::now);
    let elapsed = started.elapsed().as_millis();
    let current = chars[0].to_string();
    let next = chars.get(1).copied().unwrap_or(chars[0]).to_string();

    let phase = if chars.len() < 2 {
        0.0
    } else {
        let step_elapsed = elapsed % ACTIVE_COUNT_SCROLL_STEP_MS;
        if step_elapsed < ACTIVE_COUNT_SCROLL_HOLD_MS {
            0.0
        } else if step_elapsed < ACTIVE_COUNT_SCROLL_HOLD_MS + ACTIVE_COUNT_SCROLL_MOVE_MS {
            ((step_elapsed - ACTIVE_COUNT_SCROLL_HOLD_MS) as f64
                / ACTIVE_COUNT_SCROLL_MOVE_MS as f64)
                .clamp(0.0, 1.0)
        } else {
            1.0
        }
    };
    let current_x = -(ACTIVE_COUNT_SCROLL_TRAVEL * phase).round();

    active_count.setAlignment(NSTextAlignment::Right);
    active_count_next.setAlignment(NSTextAlignment::Right);
    active_count.setHidden(false);
    active_count_next.setHidden(chars.len() < 2);
    active_count.setStringValue(&NSString::from_str(&current));
    active_count_next.setStringValue(&NSString::from_str(&next));
    active_count.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    active_count_next.setFrame(NSRect::new(
        NSPoint::new(ACTIVE_COUNT_SCROLL_TRAVEL + ACTIVE_COUNT_TEXT_OFFSET_X, 0.0),
        NSSize::new(ACTIVE_COUNT_TEXT_WIDTH, ACTIVE_COUNT_LABEL_HEIGHT),
    ));
    active_count_clip.scrollToPoint(NSPoint::new(-current_x, 0.0));
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
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_card_layer(view: &NSView, background: [f64; 4], border: [f64; 4]) {
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        let background = ns_color(background);
        let border = ns_color(border);
        layer.setCornerRadius(CARD_RADIUS);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&background.CGColor()));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&border.CGColor()));
        layer.setShadowColor(Some(&NSColor::blackColor().CGColor()));
        layer.setShadowOpacity(0.0);
        layer.setShadowRadius(0.0);
        layer.setShadowOffset(NSSize::new(0.0, 0.0));
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn make_label(
    mtm: MainThreadMarker,
    text: &str,
    frame: NSRect,
    font_size: f64,
    color: [f64; 4],
    centered: bool,
    single_line: bool,
) -> objc2::rc::Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    label.setFrame(frame);
    if centered {
        label.setAlignment(NSTextAlignment::Center);
    }
    label.setTextColor(Some(&ns_color(color)));
    label.setFont(Some(&NSFont::systemFontOfSize(font_size)));
    label.setDrawsBackground(false);
    label.setBezeled(false);
    label.setBordered(false);
    label.setEditable(false);
    label.setSelectable(false);
    if single_line {
        label.setUsesSingleLineMode(true);
    }
    label
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn make_badge_view(
    mtm: MainThreadMarker,
    text: &str,
    width: f64,
    background: [f64; 4],
    foreground: [f64; 4],
) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width.max(24.0), 22.0)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        let background = ns_color(background);
        layer.setCornerRadius(11.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&background.CGColor()));
    }

    let label = make_label(
        mtm,
        text,
        NSRect::new(
            NSPoint::new(7.0, 4.0),
            NSSize::new(width.max(24.0) - 14.0, 13.0),
        ),
        10.0,
        foreground,
        true,
        true,
    );
    label.setFont(Some(&NSFont::systemFontOfSize_weight(10.0, unsafe {
        objc2_app_kit::NSFontWeightSemibold
    })));
    view.addSubview(&label);
    view
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn make_live_tool_view(
    mtm: MainThreadMarker,
    tool_name: &str,
    description: Option<&str>,
    width: f64,
) -> objc2::rc::Retained<NSView> {
    let width = width.max(36.0);
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, 22.0)),
    );
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(5.0);
        layer.setMasksToBounds(true);
        layer.setBackgroundColor(Some(&ns_color([1.0, 1.0, 1.0, 0.04]).CGColor()));
        layer.setBorderWidth(1.0);
        layer.setBorderColor(Some(&ns_color([1.0, 1.0, 1.0, 0.06]).CGColor()));
    }

    let name_width = badge_width(tool_name, 9.0, 0.0).min((width - 14.0).max(0.0));
    let name_label = make_label(
        mtm,
        tool_name,
        NSRect::new(NSPoint::new(7.0, 5.0), NSSize::new(name_width, 11.0)),
        9.0,
        tool_tone_color(tool_name),
        false,
        true,
    );
    name_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        9.0,
        unsafe { objc2_app_kit::NSFontWeightBold },
    )));
    view.addSubview(&name_label);

    if let Some(description) = description.filter(|value| !value.trim().is_empty()) {
        let desc_x = 7.0 + name_width + 6.0;
        let desc_width = (width - desc_x - 7.0).max(0.0);
        let desc_label = make_label(
            mtm,
            description,
            NSRect::new(NSPoint::new(desc_x, 5.0), NSSize::new(desc_width, 11.0)),
            9.0,
            [1.0, 1.0, 1.0, 0.70],
            false,
            true,
        );
        desc_label.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            9.0,
            unsafe { objc2_app_kit::NSFontWeightRegular },
        )));
        view.addSubview(&desc_label);
    }

    view
}

#[cfg(target_os = "macos")]
fn tool_tone_color(tool: &str) -> [f64; 4] {
    match tool.to_ascii_lowercase().as_str() {
        "bash" => [0.49, 0.95, 0.64, 1.0],
        "edit" | "write" => [0.53, 0.67, 1.0, 1.0],
        "read" => [0.94, 0.82, 0.49, 1.0],
        "grep" | "glob" => [0.76, 0.63, 1.0, 1.0],
        "agent" => [1.0, 0.61, 0.40, 1.0],
        _ => [0.96, 0.97, 0.99, 0.86],
    }
}

#[cfg(target_os = "macos")]
fn badge_width(text: &str, font_size: f64, horizontal_padding: f64) -> f64 {
    (text.chars().count() as f64 * font_size * 0.58) + horizontal_padding
}

#[cfg(target_os = "macos")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn clear_subviews(view: &NSView) {
    let subviews = view.subviews();
    for index in 0..subviews.len() {
        subviews.objectAtIndex(index).removeFromSuperview();
    }
}

#[cfg(target_os = "macos")]
fn absolute_rect(panel_frame: NSRect, local_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            panel_frame.origin.x + local_frame.origin.x,
            panel_frame.origin.y + local_frame.origin.y,
        ),
        local_frame.size,
    )
}

#[cfg(target_os = "macos")]
fn compose_local_rect(parent_frame: NSRect, child_frame: NSRect) -> NSRect {
    NSRect::new(
        NSPoint::new(
            parent_frame.origin.x + child_frame.origin.x,
            parent_frame.origin.y + child_frame.origin.y,
        ),
        child_frame.size,
    )
}

#[cfg(target_os = "macos")]
fn point_in_rect(point: NSPoint, rect: NSRect) -> bool {
    point.x >= rect.origin.x
        && point.x <= rect.origin.x + rect.size.width
        && point.y >= rect.origin.y
        && point.y <= rect.origin.y + rect.size.height
}

#[cfg(target_os = "macos")]
fn resolve_screen_frame_for_panel(panel: &NSPanel) -> Option<NSRect> {
    if let Some(screen) = panel.screen() {
        return Some(screen.frame());
    }
    let mtm = MainThreadMarker::new()?;
    NSScreen::mainScreen(mtm)
        .or_else(|| {
            let screens = NSScreen::screens(mtm);
            if screens.is_empty() {
                None
            } else {
                Some(screens.objectAtIndex(0))
            }
        })
        .map(|screen| screen.frame())
}

#[cfg(target_os = "macos")]
fn ns_color(rgba: [f64; 4]) -> objc2::rc::Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(rgba[0], rgba[1], rgba[2], rgba[3])
}

#[cfg(target_os = "macos")]
fn native_panel_handles() -> Option<NativePanelHandles> {
    NATIVE_TEST_PANEL_HANDLES.get().copied()
}

#[cfg(target_os = "macos")]
fn native_panel_state() -> Option<&'static Mutex<NativePanelState>> {
    NATIVE_TEST_PANEL_STATE.get()
}

#[cfg(target_os = "macos")]
unsafe fn panel_from_ptr(ptr: usize) -> &'static NSPanel {
    unsafe { &*(ptr as *const NSPanel) }
}

#[cfg(target_os = "macos")]
unsafe fn text_field_from_ptr(ptr: usize) -> &'static NSTextField {
    unsafe { &*(ptr as *const NSTextField) }
}

#[cfg(target_os = "macos")]
unsafe fn clip_view_from_ptr(ptr: usize) -> &'static NSClipView {
    unsafe { &*(ptr as *const NSClipView) }
}

#[cfg(target_os = "macos")]
unsafe fn view_from_ptr(ptr: usize) -> &'static NSView {
    unsafe { &*(ptr as *const NSView) }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use std::time::{Duration, Instant};

    use chrono::Utc;
    use echoisland_runtime::{PendingPermissionView, RuntimeSnapshot, SessionSnapshotView};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    use super::{
        HOVER_DELAY_MS, NativeExpandedSurface, NativeHoverTransition, NativeMascotRuntime,
        NativePanelState, NativeStatusQueuePayload, PANEL_SURFACE_SWITCH_INITIAL_CARD_PROGRESS,
        card_content_visibility_phase, centered_top_frame, compact_active_count_text,
        panel_transition_canvas_height, shared_expanded_content_state, summarize_headline,
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
