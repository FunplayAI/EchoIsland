use super::{
    draw_presenter::WindowsNativePanelDrawPresenter, host_window::WindowsNativePanelDrawFrame,
};
use crate::{
    native_panel_core::{
        MASCOT_IDLE_LONG_SECONDS, MASCOT_WAKE_ANGRY_SECONDS, PanelPoint, PanelRect,
    },
    native_panel_renderer::facade::{
        descriptor::{
            NativePanelHostWindowState, NativePanelInteractionPlan, NativePanelPointerInput,
            NativePanelPointerPointState, NativePanelPointerRegion,
        },
        interaction::{
            NativePanelHoverFallbackFrames, NativePanelPollingHostFacts,
            resolve_native_panel_hover_fallback_frames,
        },
        presentation::{
            NativePanelPresentationModel, NativePanelVisualActionButtonInput,
            NativePanelVisualDisplayMode, NativePanelVisualPlanInput,
            native_panel_visual_display_mode_from_presentation,
            native_panel_visual_plan_input_from_presentation,
        },
        shell::{
            NativePanelHostShellCommand, NativePanelHostShellLifecycle, NativePanelHostShellState,
            NativePanelPlatformWindowHandleAdapter, native_panel_has_raw_window_handle,
            sync_native_panel_raw_window_handle,
        },
    },
};

pub(super) const WINDOWS_WM_MOUSEMOVE: u32 = 0x0200;
pub(super) const WINDOWS_WM_LBUTTONUP: u32 = 0x0202;
pub(super) const WINDOWS_WM_MOUSELEAVE: u32 = 0x02A3;
pub(super) const WINDOWS_WM_PAINT: u32 = 0x000F;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WindowsNativePanelWindowHandle {
    pub(super) hwnd: Option<isize>,
}

impl NativePanelPlatformWindowHandleAdapter for WindowsNativePanelWindowHandle {
    type RawHandle = isize;

    fn raw_window_handle(&self) -> Option<Self::RawHandle> {
        self.hwnd
    }

    fn set_raw_window_handle(&mut self, handle: Option<Self::RawHandle>) {
        self.hwnd = handle;
    }
}

pub(super) type WindowsNativePanelShellCommand = NativePanelHostShellCommand;

pub(super) type WindowsNativePanelShellPaintJob = NativePanelVisualPlanInput;

pub(super) type WindowsNativePanelShellActionButtonPaintInput = NativePanelVisualActionButtonInput;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WindowsNativePanelShellDisplaySnapshot {
    pub(super) display_mode: NativePanelVisualDisplayMode,
    pub(super) visual_input: NativePanelVisualPlanInput,
    pub(super) shared_visible: bool,
    pub(super) pointer_region_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WindowsNativePanelShellPresentResult {
    pub(super) display_updated: bool,
    pub(super) paint_queued: bool,
    pub(super) redraw_requested: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct WindowsNativePanelWindowShell {
    shell_state: NativePanelHostShellState,
    handle: WindowsNativePanelWindowHandle,
    last_frame: Option<WindowsNativePanelDrawFrame>,
    pending_paint_job: Option<WindowsNativePanelShellPaintJob>,
    last_painted_job: Option<WindowsNativePanelShellPaintJob>,
    paint_pass_count: usize,
    display_snapshot: Option<WindowsNativePanelShellDisplaySnapshot>,
    last_pointer_input: Option<NativePanelPointerInput>,
    mascot_idle_started_elapsed_ms: Option<u128>,
    mascot_wake_started_elapsed_ms: Option<u128>,
    mascot_wake_return_pose: Option<crate::native_panel_scene::SceneMascotPose>,
    last_mascot_visual_pose: Option<crate::native_panel_scene::SceneMascotPose>,
}

impl WindowsNativePanelWindowShell {
    pub(super) fn raw_window_handle(&self) -> Option<isize> {
        self.handle.raw_window_handle()
    }

    pub(super) fn set_raw_window_handle(&mut self, hwnd: Option<isize>) {
        sync_native_panel_raw_window_handle(&mut self.handle, hwnd);
    }

    pub(super) fn has_raw_window_handle(&self) -> bool {
        native_panel_has_raw_window_handle(&self.handle)
    }

    pub(super) fn lifecycle(&self) -> NativePanelHostShellLifecycle {
        self.shell_state.lifecycle()
    }

    pub(super) fn redraw_requests(&self) -> usize {
        self.shell_state.redraw_requests()
    }

    pub(super) fn last_window_state(&self) -> Option<NativePanelHostWindowState> {
        self.shell_state.last_window_state()
    }

    pub(super) fn last_ignores_mouse_events(&self) -> Option<bool> {
        self.shell_state.last_ignores_mouse_events()
    }

    pub(super) fn last_frame(&self) -> Option<&WindowsNativePanelDrawFrame> {
        self.last_frame.as_ref()
    }

    pub(super) fn display_snapshot(&self) -> Option<&WindowsNativePanelShellDisplaySnapshot> {
        self.display_snapshot.as_ref()
    }

    pub(super) fn pending_paint_job(&self) -> Option<&WindowsNativePanelShellPaintJob> {
        self.pending_paint_job.as_ref()
    }

    pub(super) fn last_painted_job(&self) -> Option<&WindowsNativePanelShellPaintJob> {
        self.last_painted_job.as_ref()
    }

    pub(super) fn paint_pass_count(&self) -> usize {
        self.paint_pass_count
    }

    pub(super) fn last_pointer_input(&self) -> Option<NativePanelPointerInput> {
        self.last_pointer_input
    }

    pub(super) fn active_count_marquee_needs_refresh(&self) -> bool {
        self.lightweight_refresh_visible()
            && self
                .display_snapshot
                .as_ref()
                .is_some_and(|display| display.visual_input.active_count.chars().count() > 1)
    }

    pub(super) fn refresh_active_count_marquee(&mut self, elapsed_ms: u128) -> bool {
        let Some(display) = self.display_snapshot.as_mut() else {
            return false;
        };
        if display.visual_input.active_count.chars().count() <= 1 {
            return false;
        }
        display.visual_input.active_count_elapsed_ms = elapsed_ms;
        self.pending_paint_job = Some(display.visual_input.clone());
        self.request_redraw();
        true
    }

    pub(super) fn mascot_animation_needs_refresh(&self) -> bool {
        self.lightweight_refresh_visible()
            && self.display_snapshot.as_ref().is_some_and(|display| {
                display.visual_input.mascot_pose
                    != crate::native_panel_scene::SceneMascotPose::Hidden
            })
    }

    pub(super) fn refresh_mascot_animation(&mut self, elapsed_ms: u128) -> bool {
        let Some(display) = self.display_snapshot.as_mut() else {
            return false;
        };
        if display.visual_input.mascot_pose == crate::native_panel_scene::SceneMascotPose::Hidden {
            return false;
        }
        let pose = resolve_windows_shell_mascot_timed_pose(
            &mut self.mascot_idle_started_elapsed_ms,
            &mut self.mascot_wake_started_elapsed_ms,
            &mut self.mascot_wake_return_pose,
            display.display_mode,
            display.visual_input.mascot_pose,
            elapsed_ms,
        );
        display.visual_input.mascot_pose = pose.pose;
        display.visual_input.mascot_elapsed_ms = pose.elapsed_ms;
        self.last_mascot_visual_pose = Some(pose.pose);
        self.pending_paint_job = Some(display.visual_input.clone());
        self.request_redraw();
        true
    }

    pub(super) fn pointer_regions(&self) -> &[NativePanelPointerRegion] {
        self.last_frame
            .as_ref()
            .map(|frame| frame.pointer_regions.as_slice())
            .unwrap_or(&[])
    }

    fn lightweight_refresh_visible(&self) -> bool {
        if matches!(
            self.lifecycle(),
            NativePanelHostShellLifecycle::Detached | NativePanelHostShellLifecycle::Hidden
        ) {
            return false;
        }

        self.last_window_state()
            .map(|window_state| window_state.visible)
            .unwrap_or_else(|| {
                self.display_snapshot
                    .as_ref()
                    .is_some_and(|display| display.visual_input.window_state.visible)
            })
    }

    pub(super) fn pointer_state_at_point(&self, point: PanelPoint) -> NativePanelPointerPointState {
        NativePanelInteractionPlan::from_pointer_regions(self.pointer_regions())
            .pointer_state_at_point(point)
    }

    pub(super) fn hover_inside_for_input(&self, input: NativePanelPointerInput) -> Option<bool> {
        NativePanelInteractionPlan::from_pointer_regions(self.pointer_regions())
            .inside_for_input(input)
    }

    pub(super) fn platform_loop_started(&self) -> bool {
        self.shell_state.platform_loop_started()
    }

    pub(super) fn platform_loop_spawn_count(&self) -> usize {
        self.shell_state.platform_loop_spawn_count()
    }

    pub(super) fn take_pending_commands(&mut self) -> Vec<WindowsNativePanelShellCommand> {
        self.shell_state.take_pending_commands()
    }

    pub(super) fn has_pending_destroy_command(&self) -> bool {
        self.shell_state.has_pending_destroy_command()
    }

    pub(super) fn create(&mut self) {
        self.shell_state.create();
    }

    pub(super) fn show(&mut self) {
        self.shell_state.show();
    }

    pub(super) fn hide(&mut self) {
        self.shell_state.hide();
    }

    pub(super) fn destroy(&mut self) {
        if self.shell_state.destroy() {
            self.last_frame = None;
            self.set_raw_window_handle(None);
        }
    }

    pub(super) fn sync_window_state(&mut self, window_state: NativePanelHostWindowState) {
        self.shell_state.sync_window_state(window_state);
    }

    pub(super) fn request_redraw(&mut self) {
        self.shell_state.request_redraw();
    }

    pub(super) fn sync_mouse_event_passthrough(&mut self, ignores_mouse_events: bool) {
        self.shell_state
            .sync_mouse_event_passthrough(ignores_mouse_events);
    }

    pub(super) fn paint_next_frame(&mut self) -> Option<WindowsNativePanelShellPaintJob> {
        let job = self.pending_paint_job.take()?;
        self.paint_pass_count += 1;
        self.last_painted_job = Some(job.clone());
        Some(job)
    }

    pub(super) fn hover_frames(&self) -> Option<NativePanelHoverFallbackFrames> {
        let display = self.display_snapshot.as_ref()?;
        Some(windows_client_hover_fallback_frames(
            &display.visual_input,
            resolve_native_panel_hover_fallback_frames(&display.visual_input),
        ))
    }

    pub(super) fn polling_host_facts<'a>(
        &'a self,
        pointer: PanelPoint,
        primary_mouse_down: bool,
        snapshot: Option<echoisland_runtime::RuntimeSnapshot>,
    ) -> Option<NativePanelPollingHostFacts<'a>> {
        Some(NativePanelPollingHostFacts {
            pointer,
            pointer_regions: self.pointer_regions(),
            hover_frames: self.hover_frames()?,
            primary_mouse_down,
            cards_visible: self
                .display_snapshot
                .as_ref()
                .map(|display| display.visual_input.cards_visible)
                .unwrap_or(false),
            snapshot,
        })
    }

    pub(super) fn record_platform_loop_spawn(&mut self) {
        self.shell_state.record_platform_loop_spawn();
    }

    pub(super) fn decode_window_message(
        &self,
        message_id: u32,
        lparam: isize,
    ) -> Option<NativePanelPointerInput> {
        match message_id {
            WINDOWS_WM_MOUSEMOVE => Some(NativePanelPointerInput::Move(
                panel_point_from_window_lparam(lparam),
            )),
            WINDOWS_WM_LBUTTONUP => Some(NativePanelPointerInput::Click(
                panel_point_from_window_lparam(lparam),
            )),
            WINDOWS_WM_MOUSELEAVE => Some(NativePanelPointerInput::Leave),
            _ => None,
        }
    }

    pub(super) fn record_pointer_input(&mut self, input: NativePanelPointerInput) {
        self.last_pointer_input = Some(input);
    }

    pub(super) fn consume_presenter(
        &mut self,
        presenter: &mut WindowsNativePanelDrawPresenter,
    ) -> WindowsNativePanelShellPresentResult {
        let Some(frame) = presenter.take_redraw_frame() else {
            return WindowsNativePanelShellPresentResult::default();
        };
        self.sync_window_state(frame.window_state);
        let mut display_snapshot = build_display_snapshot(&frame);
        let previous_mascot_elapsed_ms = self
            .display_snapshot
            .as_ref()
            .map(|display| display.visual_input.mascot_elapsed_ms);
        sync_presented_mascot_visual_state(
            &mut display_snapshot,
            &mut self.mascot_idle_started_elapsed_ms,
            &mut self.mascot_wake_started_elapsed_ms,
            &mut self.mascot_wake_return_pose,
            self.last_mascot_visual_pose,
            previous_mascot_elapsed_ms,
        );
        let paint_job = build_paint_job(&display_snapshot);
        self.last_frame = Some(frame);
        self.last_mascot_visual_pose = Some(display_snapshot.visual_input.mascot_pose);
        self.display_snapshot = Some(display_snapshot);
        self.pending_paint_job = Some(paint_job);
        self.request_redraw();

        WindowsNativePanelShellPresentResult {
            display_updated: true,
            paint_queued: true,
            redraw_requested: true,
        }
    }
}

fn panel_point_from_window_lparam(lparam: isize) -> PanelPoint {
    let x = (lparam as u32 & 0xFFFF) as u16 as i16 as f64;
    let y = ((lparam as u32 >> 16) & 0xFFFF) as u16 as i16 as f64;
    PanelPoint { x, y }
}

fn display_mode_for_presentation(
    window_state: NativePanelHostWindowState,
    presentation: Option<&NativePanelPresentationModel>,
) -> NativePanelVisualDisplayMode {
    native_panel_visual_display_mode_from_presentation(window_state, presentation)
}

fn build_display_snapshot(
    frame: &WindowsNativePanelDrawFrame,
) -> WindowsNativePanelShellDisplaySnapshot {
    let presentation = frame.presentation_model.as_ref();
    let display_mode = display_mode_for_presentation(frame.window_state, presentation);

    WindowsNativePanelShellDisplaySnapshot {
        display_mode,
        visual_input: native_panel_visual_plan_input_from_presentation(
            frame.window_state,
            display_mode,
            presentation,
        ),
        shared_visible: presentation
            .map(|presentation| presentation.shell.shared_visible)
            .unwrap_or(false),
        pointer_region_count: frame.pointer_regions.len(),
    }
}

fn build_paint_job(
    display: &WindowsNativePanelShellDisplaySnapshot,
) -> WindowsNativePanelShellPaintJob {
    display.visual_input.clone()
}

fn windows_client_hover_fallback_frames(
    input: &NativePanelVisualPlanInput,
    frames: NativePanelHoverFallbackFrames,
) -> NativePanelHoverFallbackFrames {
    let surface_height = input
        .window_state
        .frame
        .map(|frame| frame.height)
        .or_else(|| non_zero_rect(input.content_frame).map(|frame| frame.height))
        .unwrap_or_else(|| {
            frames
                .interactive_pill_frame
                .height
                .max(frames.hover_pill_frame.height)
        })
        .max(1.0);
    let interactive_pill_frame = windows_client_frame(
        surface_height,
        local_visual_frame(input, frames.interactive_pill_frame),
    );
    let hover_pill_frame = windows_client_frame(
        surface_height,
        stable_compact_hover_frame(local_visual_frame(input, frames.interactive_pill_frame)),
    );
    let interactive_expanded_frame = frames
        .interactive_expanded_frame
        .map(|frame| windows_client_frame(surface_height, local_visual_frame(input, frame)));

    NativePanelHoverFallbackFrames {
        interactive_pill_frame,
        hover_pill_frame,
        interactive_expanded_frame,
    }
}

fn local_visual_frame(input: &NativePanelVisualPlanInput, frame: PanelRect) -> PanelRect {
    let panel = input.panel_frame;
    let frame_is_absolute = frame.x >= panel.x
        && frame.x + frame.width <= panel.x + panel.width
        && frame.y >= panel.y
        && frame.y + frame.height <= panel.y + panel.height;

    if frame_is_absolute {
        return PanelRect {
            x: frame.x - panel.x,
            y: frame.y - panel.y,
            width: frame.width,
            height: frame.height,
        };
    }

    frame
}

fn stable_compact_hover_frame(compact: PanelRect) -> PanelRect {
    union_rect(
        compact,
        PanelRect {
            x: compact.x + 20.0,
            y: compact.y + compact.height - 3.0,
            width: 30.0,
            height: 18.0,
        },
    )
}

fn union_rect(left: PanelRect, right: PanelRect) -> PanelRect {
    let min_x = left.x.min(right.x);
    let min_y = left.y.min(right.y);
    let max_x = (left.x + left.width).max(right.x + right.width);
    let max_y = (left.y + left.height).max(right.y + right.height);
    PanelRect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(0.0),
        height: (max_y - min_y).max(0.0),
    }
}

fn windows_client_frame(surface_height: f64, frame: PanelRect) -> PanelRect {
    PanelRect {
        x: frame.x,
        y: surface_height - frame.y - frame.height,
        width: frame.width,
        height: frame.height,
    }
}

fn non_zero_rect(rect: PanelRect) -> Option<PanelRect> {
    (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WindowsShellTimedMascotPose {
    pose: crate::native_panel_scene::SceneMascotPose,
    elapsed_ms: u128,
}

fn sync_presented_mascot_visual_state(
    display: &mut WindowsNativePanelShellDisplaySnapshot,
    idle_started_elapsed_ms: &mut Option<u128>,
    wake_started_elapsed_ms: &mut Option<u128>,
    wake_return_pose: &mut Option<crate::native_panel_scene::SceneMascotPose>,
    last_visual_pose: Option<crate::native_panel_scene::SceneMascotPose>,
    previous_elapsed_ms: Option<u128>,
) {
    let base_pose = display.visual_input.mascot_pose;
    if base_pose == crate::native_panel_scene::SceneMascotPose::Hidden {
        *idle_started_elapsed_ms = None;
        *wake_started_elapsed_ms = None;
        *wake_return_pose = None;
        return;
    }

    let wakes_from_sleep = last_visual_pose
        == Some(crate::native_panel_scene::SceneMascotPose::Sleepy)
        && (base_pose != crate::native_panel_scene::SceneMascotPose::Idle
            || display.display_mode == NativePanelVisualDisplayMode::Expanded);
    if wakes_from_sleep {
        display.visual_input.mascot_pose = crate::native_panel_scene::SceneMascotPose::WakeAngry;
        display.visual_input.mascot_elapsed_ms = 0;
        *idle_started_elapsed_ms = None;
        *wake_started_elapsed_ms = None;
        *wake_return_pose = Some(base_pose);
        return;
    }

    if last_visual_pose == Some(base_pose) {
        display.visual_input.mascot_elapsed_ms =
            previous_elapsed_ms.unwrap_or(display.visual_input.mascot_elapsed_ms);
    }

    if base_pose != crate::native_panel_scene::SceneMascotPose::Idle
        || display.display_mode == NativePanelVisualDisplayMode::Expanded
    {
        *idle_started_elapsed_ms = None;
    }
}

fn resolve_windows_shell_mascot_timed_pose(
    idle_started_elapsed_ms: &mut Option<u128>,
    wake_started_elapsed_ms: &mut Option<u128>,
    wake_return_pose: &mut Option<crate::native_panel_scene::SceneMascotPose>,
    display_mode: NativePanelVisualDisplayMode,
    current_pose: crate::native_panel_scene::SceneMascotPose,
    elapsed_ms: u128,
) -> WindowsShellTimedMascotPose {
    if let Some(return_pose) = *wake_return_pose {
        let started_at = wake_started_elapsed_ms.get_or_insert(elapsed_ms);
        let wake_elapsed_ms = elapsed_ms.saturating_sub(*started_at);
        if wake_elapsed_ms < (MASCOT_WAKE_ANGRY_SECONDS * 1000.0) as u128 {
            return WindowsShellTimedMascotPose {
                pose: crate::native_panel_scene::SceneMascotPose::WakeAngry,
                elapsed_ms: wake_elapsed_ms,
            };
        }
        *wake_started_elapsed_ms = None;
        *wake_return_pose = None;
        return WindowsShellTimedMascotPose {
            pose: return_pose,
            elapsed_ms,
        };
    }

    if current_pose == crate::native_panel_scene::SceneMascotPose::Idle
        && display_mode != NativePanelVisualDisplayMode::Expanded
    {
        let started_at = idle_started_elapsed_ms.get_or_insert(elapsed_ms);
        if elapsed_ms.saturating_sub(*started_at) >= MASCOT_IDLE_LONG_SECONDS as u128 * 1000 {
            return WindowsShellTimedMascotPose {
                pose: crate::native_panel_scene::SceneMascotPose::Sleepy,
                elapsed_ms,
            };
        }
    } else {
        *idle_started_elapsed_ms = None;
    }

    WindowsShellTimedMascotPose {
        pose: current_pose,
        elapsed_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WINDOWS_WM_LBUTTONUP, WINDOWS_WM_MOUSELEAVE, WINDOWS_WM_MOUSEMOVE,
        WindowsNativePanelShellCommand, WindowsNativePanelWindowHandle,
        WindowsNativePanelWindowShell, windows_client_hover_fallback_frames,
    };
    use crate::native_panel_renderer::facade::shell::NativePanelPlatformWindowHandleAdapter;
    use crate::{
        native_panel_core::{PanelPoint, PanelRect},
        native_panel_renderer::facade::{
            descriptor::{
                NativePanelHostWindowState, NativePanelPointerInput, NativePanelPointerRegion,
                NativePanelPointerRegionKind,
            },
            presentation::{
                NativePanelActionButtonsPresentation, NativePanelCardStackPresentation,
                NativePanelCompactBarPresentation, NativePanelGlowPresentation,
                NativePanelMascotPresentation, NativePanelPresentationMetrics,
                NativePanelPresentationModel, NativePanelShellPresentation,
                NativePanelVisualDisplayMode, NativePanelVisualPlanInput,
            },
            shell::NativePanelHostShellLifecycle,
        },
        windows_native_panel::{
            draw_presenter::WindowsNativePanelDrawPresenter,
            host_window::WindowsNativePanelDrawFrame,
        },
    };

    fn visible_window_state() -> NativePanelHostWindowState {
        NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 80.0,
            }),
            visible: true,
            preferred_display_index: 0,
        }
    }

    fn presentation_with_mascot(
        pose: crate::native_panel_scene::SceneMascotPose,
        shell_visible: bool,
    ) -> NativePanelPresentationModel {
        NativePanelPresentationModel {
            panel_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 80.0,
            },
            content_frame: PanelRect {
                x: 10.0,
                y: 40.0,
                width: 300.0,
                height: 30.0,
            },
            shell: NativePanelShellPresentation {
                surface: crate::native_panel_core::ExpandedSurface::Default,
                frame: PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 320.0,
                    height: 80.0,
                },
                visible: shell_visible,
                separator_visibility: if shell_visible { 1.0 } else { 0.0 },
                shared_visible: true,
            },
            compact_bar: NativePanelCompactBarPresentation {
                frame: PanelRect {
                    x: 10.0,
                    y: 0.0,
                    width: 300.0,
                    height: 37.0,
                },
                left_shoulder_frame: PanelRect {
                    x: 4.0,
                    y: 30.0,
                    width: 6.0,
                    height: 6.0,
                },
                right_shoulder_frame: PanelRect {
                    x: 310.0,
                    y: 30.0,
                    width: 6.0,
                    height: 6.0,
                },
                shoulder_progress: 0.0,
                headline: crate::native_panel_scene::SceneText {
                    text: "Codex ready".to_string(),
                    emphasized: false,
                },
                active_count: "1".to_string(),
                total_count: "1".to_string(),
                completion_count: 0,
                headline_emphasized: false,
                actions_visible: false,
            },
            card_stack: NativePanelCardStackPresentation {
                frame: PanelRect {
                    x: 10.0,
                    y: 40.0,
                    width: 300.0,
                    height: 30.0,
                },
                surface: crate::native_panel_core::ExpandedSurface::Default,
                cards: Vec::new(),
                content_height: 0.0,
                body_height: 0.0,
                visible: false,
            },
            mascot: NativePanelMascotPresentation { pose },
            glow: None,
            action_buttons: NativePanelActionButtonsPresentation {
                visible: false,
                buttons: Vec::new(),
            },
            metrics: NativePanelPresentationMetrics {
                expanded_content_height: 0.0,
                expanded_body_height: 0.0,
            },
        }
    }

    #[test]
    fn shell_window_handle_helpers_roundtrip_handle_presence() {
        let mut handle = WindowsNativePanelWindowHandle::default();

        assert_eq!(handle.raw_window_handle(), None);

        handle.set_raw_window_handle(Some(99));

        assert_eq!(handle.raw_window_handle(), Some(99));
    }

    #[test]
    fn shell_proxies_raw_window_handle_adapter() {
        let mut shell = WindowsNativePanelWindowShell::default();

        assert!(!shell.has_raw_window_handle());

        shell.set_raw_window_handle(Some(123));

        assert_eq!(shell.raw_window_handle(), Some(123));
        assert!(shell.has_raw_window_handle());
    }

    #[test]
    fn shell_decodes_pointer_move_message_from_lparam() {
        let shell = WindowsNativePanelWindowShell::default();
        let message = shell.decode_window_message(WINDOWS_WM_MOUSEMOVE, 0x001E_000Aisize);

        assert_eq!(
            message,
            Some(NativePanelPointerInput::Move(PanelPoint {
                x: 10.0,
                y: 30.0,
            }))
        );
    }

    #[test]
    fn shell_decodes_pointer_click_message_from_signed_lparam() {
        let shell = WindowsNativePanelWindowShell::default();
        let message = shell.decode_window_message(WINDOWS_WM_LBUTTONUP, 0xFFEC_FFF6u32 as isize);

        assert_eq!(
            message,
            Some(NativePanelPointerInput::Click(PanelPoint {
                x: -10.0,
                y: -20.0,
            }))
        );
    }

    #[test]
    fn shell_decodes_pointer_leave_message() {
        let shell = WindowsNativePanelWindowShell::default();
        let message = shell.decode_window_message(WINDOWS_WM_MOUSELEAVE, 0x0000_0000isize);

        assert_eq!(message, Some(NativePanelPointerInput::Leave));
    }

    #[test]
    fn shell_records_last_pointer_input() {
        let mut shell = WindowsNativePanelWindowShell::default();

        shell.record_pointer_input(NativePanelPointerInput::Leave);

        assert_eq!(
            shell.last_pointer_input(),
            Some(NativePanelPointerInput::Leave)
        );
    }

    #[test]
    fn shell_consumes_presenter_redraw_frame() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 320.0,
                    height: 80.0,
                }),
                visible: true,
                preferred_display_index: 0,
            },
            pointer_regions: Vec::new(),
            presentation_model: None,
        });

        let result = shell.consume_presenter(&mut presenter);

        assert!(result.display_updated);
        assert!(result.paint_queued);
        assert!(result.redraw_requested);
        assert_eq!(shell.lifecycle(), NativePanelHostShellLifecycle::Created);
        assert_eq!(shell.redraw_requests(), 1);
        assert_eq!(
            shell
                .last_frame()
                .and_then(|frame| frame.window_state.frame)
                .map(|frame| frame.width),
            Some(320.0)
        );
        let paint_job = shell.paint_next_frame().expect("paint job");
        assert_eq!(
            paint_job.display_mode,
            NativePanelVisualDisplayMode::Compact
        );
        assert_eq!(shell.consume_presenter(&mut presenter), Default::default());
    }

    #[test]
    fn shell_lifecycle_tracks_create_show_hide_destroy() {
        let mut shell = WindowsNativePanelWindowShell::default();

        shell.create();
        assert_eq!(shell.lifecycle(), NativePanelHostShellLifecycle::Created);

        shell.show();
        assert_eq!(shell.lifecycle(), NativePanelHostShellLifecycle::Visible);

        shell.hide();
        assert_eq!(shell.lifecycle(), NativePanelHostShellLifecycle::Hidden);

        shell.destroy();
        assert_eq!(shell.lifecycle(), NativePanelHostShellLifecycle::Detached);
    }

    #[test]
    fn shell_tracks_window_state_and_platform_loop() {
        let mut shell = WindowsNativePanelWindowShell::default();
        let state = NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 8.0,
                y: 16.0,
                width: 320.0,
                height: 96.0,
            }),
            visible: true,
            preferred_display_index: 2,
        };

        shell.sync_window_state(state);
        shell.record_platform_loop_spawn();

        assert_eq!(shell.last_window_state(), Some(state));
        assert!(shell.platform_loop_started());
        assert_eq!(shell.platform_loop_spawn_count(), 1);
    }

    #[test]
    fn shell_emits_lifecycle_and_redraw_commands() {
        let mut shell = WindowsNativePanelWindowShell::default();
        let state = NativePanelHostWindowState {
            frame: Some(PanelRect {
                x: 4.0,
                y: 6.0,
                width: 220.0,
                height: 72.0,
            }),
            visible: true,
            preferred_display_index: 1,
        };

        shell.create();
        shell.show();
        shell.sync_window_state(state);
        shell.request_redraw();
        shell.hide();

        assert_eq!(
            shell.take_pending_commands(),
            vec![
                WindowsNativePanelShellCommand::Create,
                WindowsNativePanelShellCommand::Show,
                WindowsNativePanelShellCommand::SyncWindowState(state),
                WindowsNativePanelShellCommand::RequestRedraw,
                WindowsNativePanelShellCommand::Hide,
            ]
        );
    }

    #[test]
    fn shell_emits_mouse_passthrough_command_only_when_state_changes() {
        let mut shell = WindowsNativePanelWindowShell::default();

        shell.sync_mouse_event_passthrough(true);
        shell.sync_mouse_event_passthrough(true);
        shell.sync_mouse_event_passthrough(false);

        assert_eq!(shell.last_ignores_mouse_events(), Some(false));
        assert_eq!(
            shell.take_pending_commands(),
            vec![
                WindowsNativePanelShellCommand::SyncMouseEventPassthrough(true),
                WindowsNativePanelShellCommand::SyncMouseEventPassthrough(false),
            ]
        );
    }

    #[test]
    fn shell_builds_display_snapshot_from_presenter_frame() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 50.0,
                    width: 320.0,
                    height: 120.0,
                }),
                visible: true,
                preferred_display_index: 0,
            },
            pointer_regions: Vec::new(),
            presentation_model: Some(NativePanelPresentationModel {
                panel_frame: PanelRect {
                    x: 100.0,
                    y: 50.0,
                    width: 320.0,
                    height: 120.0,
                },
                content_frame: PanelRect {
                    x: 110.0,
                    y: 90.0,
                    width: 300.0,
                    height: 70.0,
                },
                shell: NativePanelShellPresentation {
                    surface: crate::native_panel_core::ExpandedSurface::Status,
                    frame: PanelRect {
                        x: 100.0,
                        y: 70.0,
                        width: 320.0,
                        height: 100.0,
                    },
                    visible: true,
                    separator_visibility: 0.8,
                    shared_visible: true,
                },
                compact_bar: NativePanelCompactBarPresentation {
                    frame: PanelRect {
                        x: 110.0,
                        y: 60.0,
                        width: 300.0,
                        height: 24.0,
                    },
                    left_shoulder_frame: PanelRect {
                        x: 104.0,
                        y: 78.0,
                        width: 6.0,
                        height: 6.0,
                    },
                    right_shoulder_frame: PanelRect {
                        x: 410.0,
                        y: 78.0,
                        width: 6.0,
                        height: 6.0,
                    },
                    shoulder_progress: 0.0,
                    headline: crate::native_panel_scene::SceneText {
                        text: "Approval waiting".to_string(),
                        emphasized: true,
                    },
                    active_count: "23".to_string(),
                    total_count: "2".to_string(),
                    completion_count: 3,
                    headline_emphasized: true,
                    actions_visible: true,
                },
                card_stack: NativePanelCardStackPresentation {
                    frame: PanelRect {
                        x: 110.0,
                        y: 90.0,
                        width: 300.0,
                        height: 70.0,
                    },
                    surface: crate::native_panel_core::ExpandedSurface::Status,
                    cards: vec![crate::native_panel_scene::SceneCard::Empty],
                    content_height: 70.0,
                    body_height: 70.0,
                    visible: true,
                },
                mascot: NativePanelMascotPresentation {
                    pose: crate::native_panel_scene::SceneMascotPose::Complete,
                },
                glow: Some(NativePanelGlowPresentation {
                    glow: crate::native_panel_scene::SceneGlow {
                        style: crate::native_panel_scene::SceneGlowStyle::Completion,
                        opacity: 0.8,
                    },
                }),
                action_buttons: NativePanelActionButtonsPresentation {
                    visible: true,
                    buttons: Vec::new(),
                },
                metrics: NativePanelPresentationMetrics {
                    expanded_content_height: 70.0,
                    expanded_body_height: 70.0,
                },
            }),
        });

        let result = shell.consume_presenter(&mut presenter);

        assert!(result.redraw_requested);
        let snapshot = shell.display_snapshot().expect("display snapshot");
        assert_eq!(
            snapshot.display_mode,
            NativePanelVisualDisplayMode::Expanded
        );
        assert_eq!(
            snapshot.visual_input.surface,
            crate::native_panel_core::ExpandedSurface::Status
        );
        assert_eq!(snapshot.visual_input.headline_text, "Approval waiting");
        assert!(snapshot.visual_input.headline_emphasized);
        assert!(snapshot.visual_input.cards_visible);
        assert_eq!(snapshot.visual_input.card_count, 1);
        assert_eq!(snapshot.visual_input.cards[0].title, "No active sessions");
        assert!(snapshot.visual_input.glow_visible);
        assert!(snapshot.visual_input.action_buttons_visible);
        assert_eq!(snapshot.visual_input.completion_count, 3);
        assert_eq!(
            snapshot.visual_input.mascot_pose,
            crate::native_panel_scene::SceneMascotPose::Complete
        );

        assert!(shell.active_count_marquee_needs_refresh());
        assert!(shell.mascot_animation_needs_refresh());

        shell.hide();

        assert!(!shell.active_count_marquee_needs_refresh());
        assert!(!shell.mascot_animation_needs_refresh());
    }

    #[test]
    fn shell_refreshes_idle_mascot_into_sleepy_after_shared_delay() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: None,
        });
        shell.consume_presenter(&mut presenter);

        assert!(shell.refresh_mascot_animation(0));
        assert!(shell.refresh_mascot_animation(
            crate::native_panel_core::MASCOT_IDLE_LONG_SECONDS as u128 * 1000 + 1
        ));

        let paint_job = shell.paint_next_frame().expect("paint job");
        assert_eq!(
            paint_job.mascot_pose,
            crate::native_panel_scene::SceneMascotPose::Sleepy
        );
    }

    #[test]
    fn shell_plays_wake_angry_after_sleepy_when_mascot_becomes_active() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: None,
        });
        shell.consume_presenter(&mut presenter);
        shell.refresh_mascot_animation(0);
        shell.refresh_mascot_animation(
            crate::native_panel_core::MASCOT_IDLE_LONG_SECONDS as u128 * 1000 + 1,
        );

        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: Some(presentation_with_mascot(
                crate::native_panel_scene::SceneMascotPose::Running,
                false,
            )),
        });
        shell.consume_presenter(&mut presenter);

        assert_eq!(
            shell
                .display_snapshot()
                .expect("display snapshot")
                .visual_input
                .mascot_pose,
            crate::native_panel_scene::SceneMascotPose::WakeAngry
        );
        assert!(shell.refresh_mascot_animation(130_000));
        assert_eq!(
            shell.paint_next_frame().expect("wake paint").mascot_pose,
            crate::native_panel_scene::SceneMascotPose::WakeAngry
        );
        assert!(shell.refresh_mascot_animation(131_000));
        assert_eq!(
            shell
                .paint_next_frame()
                .expect("returned paint")
                .mascot_pose,
            crate::native_panel_scene::SceneMascotPose::Running
        );
    }

    #[test]
    fn shell_keeps_sleepy_when_idle_compact_presenter_refreshes() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: None,
        });
        shell.consume_presenter(&mut presenter);
        shell.refresh_mascot_animation(0);
        shell.refresh_mascot_animation(
            crate::native_panel_core::MASCOT_IDLE_LONG_SECONDS as u128 * 1000 + 1,
        );

        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: None,
        });
        shell.consume_presenter(&mut presenter);

        assert!(shell.refresh_mascot_animation(
            crate::native_panel_core::MASCOT_IDLE_LONG_SECONDS as u128 * 1000 + 16
        ));
        assert_eq!(
            shell.paint_next_frame().expect("sleepy paint").mascot_pose,
            crate::native_panel_scene::SceneMascotPose::Sleepy
        );
    }

    #[test]
    fn shell_preserves_mascot_elapsed_time_when_expanding_same_pose() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: Some(presentation_with_mascot(
                crate::native_panel_scene::SceneMascotPose::MessageBubble,
                false,
            )),
        });
        shell.consume_presenter(&mut presenter);
        assert!(shell.refresh_mascot_animation(500));

        presenter.present(WindowsNativePanelDrawFrame {
            window_state: visible_window_state(),
            pointer_regions: Vec::new(),
            presentation_model: Some(presentation_with_mascot(
                crate::native_panel_scene::SceneMascotPose::MessageBubble,
                true,
            )),
        });
        shell.consume_presenter(&mut presenter);

        assert_eq!(
            shell
                .display_snapshot()
                .expect("display snapshot")
                .visual_input
                .mascot_elapsed_ms,
            500
        );
    }

    #[test]
    fn shell_pointer_and_hover_facts_follow_cached_frame() {
        let mut presenter = WindowsNativePanelDrawPresenter::default();
        let mut shell = WindowsNativePanelWindowShell::default();
        presenter.present(WindowsNativePanelDrawFrame {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 50.0,
                    width: 320.0,
                    height: 120.0,
                }),
                visible: true,
                preferred_display_index: 0,
            },
            pointer_regions: vec![NativePanelPointerRegion {
                frame: PanelRect {
                    x: 110.0,
                    y: 60.0,
                    width: 100.0,
                    height: 30.0,
                },
                kind: NativePanelPointerRegionKind::CompactBar,
            }],
            presentation_model: None,
        });

        shell.consume_presenter(&mut presenter);

        let pointer_state = shell.pointer_state_at_point(PanelPoint { x: 120.0, y: 70.0 });
        assert!(pointer_state.inside);
        assert_eq!(
            shell.hover_inside_for_input(NativePanelPointerInput::Leave),
            Some(false)
        );
        assert_eq!(
            shell.hover_inside_for_input(NativePanelPointerInput::Move(PanelPoint {
                x: 120.0,
                y: 70.0
            })),
            Some(true)
        );

        let hover_frames = shell.hover_frames().expect("hover frames");
        assert!(hover_frames.interactive_pill_frame.width > 0.0);
        assert!(hover_frames.hover_pill_frame.width > 0.0);

        let facts = shell
            .polling_host_facts(PanelPoint { x: 120.0, y: 70.0 }, false, None)
            .expect("polling facts");
        assert_eq!(facts.pointer_regions.len(), 1);
    }

    #[test]
    fn shell_hover_frames_use_windows_client_coordinates_and_stable_bubble_hover() {
        let zero = PanelRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        let input = NativePanelVisualPlanInput {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 510.0,
                    y: 0.0,
                    width: 420.0,
                    height: 80.0,
                }),
                visible: true,
                preferred_display_index: 0,
            },
            display_mode: NativePanelVisualDisplayMode::Compact,
            surface: crate::native_panel_core::ExpandedSurface::Default,
            panel_frame: PanelRect {
                x: 510.0,
                y: 0.0,
                width: 420.0,
                height: 80.0,
            },
            compact_bar_frame: PanelRect {
                x: 83.5,
                y: 43.0,
                width: 253.0,
                height: 37.0,
            },
            left_shoulder_frame: zero,
            right_shoulder_frame: zero,
            shoulder_progress: 0.0,
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 420.0,
                height: 80.0,
            },
            card_stack_frame: zero,
            card_stack_content_height: 0.0,
            shell_frame: zero,
            headline_text: String::new(),
            headline_emphasized: false,
            active_count: String::new(),
            active_count_elapsed_ms: 0,
            total_count: String::new(),
            separator_visibility: 0.0,
            cards_visible: false,
            card_count: 0,
            cards: Vec::new(),
            glow_visible: false,
            action_buttons_visible: false,
            action_buttons: Vec::new(),
            completion_count: 0,
            mascot_elapsed_ms: 0,
            mascot_pose: crate::native_panel_scene::SceneMascotPose::Idle,
        };

        let frames = windows_client_hover_fallback_frames(
            &input,
            crate::native_panel_renderer::facade::interaction::resolve_native_panel_hover_fallback_frames(
                &input,
            ),
        );

        assert_eq!(
            frames.interactive_pill_frame,
            PanelRect {
                x: 83.5,
                y: 0.0,
                width: 253.0,
                height: 37.0,
            }
        );
        assert_eq!(frames.hover_pill_frame.x, 83.5);
        assert!(frames.hover_pill_frame.y < frames.interactive_pill_frame.y);
        assert!(frames.hover_pill_frame.height > frames.interactive_pill_frame.height);
    }
}
