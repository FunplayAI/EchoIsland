use crate::native_panel_renderer::facade::visual::{
    NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
    NativePanelVisualTextAlignment, NativePanelVisualTextWeight, resolve_native_panel_visual_plan,
};

use crate::native_panel_core::{PanelPoint, PanelRect};
use crate::native_panel_scene::SceneMascotPose;

use super::{
    d2d_painter::WindowsNativePanelPainter, window_shell::WindowsNativePanelShellPaintJob,
};

#[cfg(all(windows, not(test)))]
thread_local! {
    static DIRECT2D_WINDOWS_NATIVE_PANEL_PAINTER:
        std::cell::RefCell<Option<super::d2d_painter::Direct2DWindowsNativePanelPainter>> =
            const { std::cell::RefCell::new(None) };
}

pub(super) const WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY: u32 = 0x00FF00FF;

pub(super) type WindowsNativePanelPaintColor = NativePanelVisualColor;
pub(super) type WindowsNativePanelPaintPlan = NativePanelVisualPlan;
pub(super) type WindowsNativePanelPaintPrimitive = NativePanelVisualPrimitive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowsNativePanelPainterBackend {
    Direct2D,
    GdiFallback,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum WindowsNativePanelPaintOperation {
    DrawCompletionGlowImage {
        frame: PanelRect,
        opacity: f64,
    },
    FillHitTestBlocker {
        frame: PanelRect,
        alpha: u8,
    },
    FillRoundRect {
        frame: PanelRect,
        radius: f64,
        color: WindowsNativePanelPaintColor,
    },
    FillRect {
        frame: PanelRect,
        color: WindowsNativePanelPaintColor,
    },
    FillEllipse {
        frame: PanelRect,
        color: WindowsNativePanelPaintColor,
    },
    StrokeLine {
        from: PanelPoint,
        to: PanelPoint,
        color: WindowsNativePanelPaintColor,
        width: i32,
    },
    DrawText {
        origin: PanelPoint,
        max_width: f64,
        text: String,
        color: WindowsNativePanelPaintColor,
        size: i32,
        weight: NativePanelVisualTextWeight,
        alignment: NativePanelVisualTextAlignment,
    },
    FillMascotDot {
        frame: PanelRect,
        radius: f64,
        pose: SceneMascotPose,
        color: WindowsNativePanelPaintColor,
        stroke_color: WindowsNativePanelPaintColor,
        stroke_width: f64,
    },
    FillCompactShoulder {
        frame: PanelRect,
        side: crate::native_panel_renderer::facade::visual::NativePanelVisualShoulderSide,
        progress: f64,
        fill: WindowsNativePanelPaintColor,
        border: WindowsNativePanelPaintColor,
    },
}

const WINDOWS_NATIVE_PANEL_HIT_TEST_BLOCKER_ALPHA: u8 = 1;

pub(super) fn resolve_windows_native_panel_paint_plan(
    job: &WindowsNativePanelShellPaintJob,
) -> WindowsNativePanelPaintPlan {
    resolve_native_panel_visual_plan(job)
}

pub(super) fn windows_native_panel_preferred_painter_backend() -> WindowsNativePanelPainterBackend {
    WindowsNativePanelPainterBackend::Direct2D
}

pub(super) fn windows_native_panel_composition_mode_for_preferred_painter()
-> super::layered_window::WindowsLayeredWindowCompositionMode {
    match windows_native_panel_preferred_painter_backend() {
        WindowsNativePanelPainterBackend::Direct2D => {
            super::layered_window::WindowsLayeredWindowCompositionMode::PerPixelAlpha
        }
        WindowsNativePanelPainterBackend::GdiFallback => {
            super::layered_window::WindowsLayeredWindowCompositionMode::GdiColorKeyFallback
        }
    }
}

pub(super) fn resolve_windows_native_panel_paint_operations(
    plan: &WindowsNativePanelPaintPlan,
) -> Vec<WindowsNativePanelPaintOperation> {
    if plan.hidden {
        return Vec::new();
    }

    plan.primitives
        .iter()
        .map(windows_native_panel_paint_operation_from_primitive)
        .collect()
}

pub(super) fn resolve_windows_native_panel_hit_test_blocker_operations(
    job: &WindowsNativePanelShellPaintJob,
) -> Vec<WindowsNativePanelPaintOperation> {
    if job.display_mode
        != crate::native_panel_renderer::facade::presentation::NativePanelVisualDisplayMode::Expanded
    {
        return Vec::new();
    }

    let mut operations = Vec::new();
    push_hit_test_blocker_operation(&mut operations, job.shell_frame);

    let content_top = job.content_frame.y + job.content_frame.height;
    let gap_y = job.shell_frame.y + job.shell_frame.height;
    push_hit_test_blocker_operation(
        &mut operations,
        PanelRect {
            x: job.shell_frame.x,
            y: gap_y,
            width: job.shell_frame.width,
            height: (content_top - gap_y).max(0.0),
        },
    );

    operations
}

fn push_hit_test_blocker_operation(
    operations: &mut Vec<WindowsNativePanelPaintOperation>,
    frame: PanelRect,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }
    operations.push(WindowsNativePanelPaintOperation::FillHitTestBlocker {
        frame,
        alpha: WINDOWS_NATIVE_PANEL_HIT_TEST_BLOCKER_ALPHA,
    });
}

fn windows_native_panel_paint_operation_from_primitive(
    primitive: &WindowsNativePanelPaintPrimitive,
) -> WindowsNativePanelPaintOperation {
    match primitive {
        WindowsNativePanelPaintPrimitive::CompletionGlow { frame, opacity } => {
            WindowsNativePanelPaintOperation::DrawCompletionGlowImage {
                frame: *frame,
                opacity: *opacity,
            }
        }
        WindowsNativePanelPaintPrimitive::RoundRect {
            frame,
            radius,
            color,
        } => WindowsNativePanelPaintOperation::FillRoundRect {
            frame: *frame,
            radius: *radius,
            color: *color,
        },
        WindowsNativePanelPaintPrimitive::Rect { frame, color } => {
            WindowsNativePanelPaintOperation::FillRect {
                frame: *frame,
                color: *color,
            }
        }
        WindowsNativePanelPaintPrimitive::Ellipse { frame, color } => {
            WindowsNativePanelPaintOperation::FillEllipse {
                frame: *frame,
                color: *color,
            }
        }
        WindowsNativePanelPaintPrimitive::StrokeLine {
            from,
            to,
            color,
            width,
        } => WindowsNativePanelPaintOperation::StrokeLine {
            from: *from,
            to: *to,
            color: *color,
            width: *width,
        },
        WindowsNativePanelPaintPrimitive::Text {
            origin,
            max_width,
            text,
            color,
            size,
            weight,
            alignment,
        } => WindowsNativePanelPaintOperation::DrawText {
            origin: *origin,
            max_width: *max_width,
            text: text.clone(),
            color: *color,
            size: *size,
            weight: *weight,
            alignment: *alignment,
        },
        WindowsNativePanelPaintPrimitive::MascotDot {
            center,
            radius,
            scale_x,
            scale_y,
            pose,
            debug_mode_enabled,
        } => {
            let body_width = radius * (24.0 / 11.0) * scale_x;
            let body_height = radius * (20.0 / 11.0) * scale_y;
            let color = if *debug_mode_enabled {
                WindowsNativePanelPaintColor {
                    r: 255,
                    g: 255,
                    b: 255,
                }
            } else if *pose == SceneMascotPose::Sleepy {
                WindowsNativePanelPaintColor { r: 3, g: 3, b: 3 }
            } else {
                WindowsNativePanelPaintColor { r: 5, g: 5, b: 5 }
            };
            WindowsNativePanelPaintOperation::FillMascotDot {
                frame: PanelRect {
                    x: center.x - body_width / 2.0,
                    y: center.y - body_height / 2.0,
                    width: body_width,
                    height: body_height,
                },
                radius: radius * (6.0 / 11.0),
                pose: *pose,
                color,
                stroke_color: if *debug_mode_enabled {
                    WindowsNativePanelPaintColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    }
                } else {
                    WindowsNativePanelPaintColor {
                        r: 255,
                        g: 122,
                        b: 36,
                    }
                },
                stroke_width: 2.2,
            }
        }
        WindowsNativePanelPaintPrimitive::CompactShoulder {
            frame,
            side,
            progress,
            fill,
            border,
        } => WindowsNativePanelPaintOperation::FillCompactShoulder {
            frame: *frame,
            side: *side,
            progress: *progress,
            fill: *fill,
            border: *border,
        },
    }
}

pub(super) fn paint_windows_native_panel_job(
    raw_window_handle: Option<isize>,
    job: &WindowsNativePanelShellPaintJob,
) -> Result<WindowsNativePanelPaintPlan, String> {
    #[cfg(all(windows, not(test)))]
    {
        return match windows_native_panel_preferred_painter_backend() {
            WindowsNativePanelPainterBackend::Direct2D => {
                paint_windows_native_panel_job_with_direct2d(raw_window_handle, job)
            }
            WindowsNativePanelPainterBackend::GdiFallback => {
                let mut painter =
                    super::d2d_painter::GdiWindowsNativePanelPainter::new(raw_window_handle);
                painter.paint(job)
            }
        };
    }

    #[cfg(any(not(windows), test))]
    {
        let _ = raw_window_handle;
        let mut painter = super::d2d_painter::PlanOnlyWindowsNativePanelPainter;
        painter.paint(job)
    }
}

#[cfg(all(windows, not(test)))]
fn paint_windows_native_panel_job_with_direct2d(
    raw_window_handle: Option<isize>,
    job: &WindowsNativePanelShellPaintJob,
) -> Result<WindowsNativePanelPaintPlan, String> {
    DIRECT2D_WINDOWS_NATIVE_PANEL_PAINTER.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(super::d2d_painter::Direct2DWindowsNativePanelPainter::new(
                raw_window_handle,
            )?);
        }
        let painter = slot
            .as_mut()
            .expect("Direct2D painter initialized when slot is Some");
        painter.set_raw_window_handle(raw_window_handle);
        painter.paint(job)
    })
}

#[cfg(all(windows, not(test)))]
pub(super) fn paint_windows_native_panel_job_with_gdi(
    raw_window_handle: Option<isize>,
    job: &WindowsNativePanelShellPaintJob,
) -> Result<WindowsNativePanelPaintPlan, String> {
    use std::iter;
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{
            CreatePen, CreateSolidBrush, DT_END_ELLIPSIS, DT_SINGLELINE, DT_VCENTER, DeleteObject,
            DrawTextW, Ellipse, FillRect, GetDC, GetStockObject, LineTo, MoveToEx, NULL_PEN,
            PS_SOLID, ReleaseDC, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
        },
        UI::WindowsAndMessaging::GetClientRect,
    };

    let plan = resolve_windows_native_panel_paint_plan(job);
    let operations = resolve_windows_native_panel_paint_operations(&plan);
    let Some(hwnd) = raw_window_handle else {
        return Ok(plan);
    };

    unsafe {
        let hdc = GetDC(hwnd as _);
        if hdc.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }

        let clear_brush = CreateSolidBrush(WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY);
        let mut client_rect = std::mem::zeroed::<RECT>();
        if GetClientRect(hwnd as _, &mut client_rect) != 0 {
            let _ = FillRect(hdc, &client_rect, clear_brush);
        }
        let _ = DeleteObject(clear_brush as _);

        for operation in &operations {
            match operation {
                WindowsNativePanelPaintOperation::DrawCompletionGlowImage { .. } => {}
                WindowsNativePanelPaintOperation::FillHitTestBlocker { .. } => {}
                WindowsNativePanelPaintOperation::FillRoundRect {
                    frame,
                    radius,
                    color,
                } => {
                    let brush = CreateSolidBrush(color_ref(*color));
                    let pen = GetStockObject(NULL_PEN);
                    let previous = SelectObject(hdc, brush as _);
                    let previous_pen = SelectObject(hdc, pen);
                    let _ = RoundRect(
                        hdc,
                        frame.x.round() as i32,
                        frame.y.round() as i32,
                        (frame.x + frame.width).round() as i32,
                        (frame.y + frame.height).round() as i32,
                        (radius * 2.0).round() as i32,
                        (radius * 2.0).round() as i32,
                    );
                    let _ = SelectObject(hdc, previous_pen);
                    let _ = SelectObject(hdc, previous);
                    let _ = DeleteObject(brush as _);
                }
                WindowsNativePanelPaintOperation::FillRect { frame, color } => {
                    let brush = CreateSolidBrush(color_ref(*color));
                    let rect = rect_from_panel_rect(*frame);
                    let _ = FillRect(hdc, &rect, brush);
                    let _ = DeleteObject(brush as _);
                }
                WindowsNativePanelPaintOperation::FillEllipse { frame, color } => {
                    let brush = CreateSolidBrush(color_ref(*color));
                    let pen = GetStockObject(NULL_PEN);
                    let previous = SelectObject(hdc, brush as _);
                    let previous_pen = SelectObject(hdc, pen);
                    let _ = Ellipse(
                        hdc,
                        frame.x.round() as i32,
                        frame.y.round() as i32,
                        (frame.x + frame.width).round() as i32,
                        (frame.y + frame.height).round() as i32,
                    );
                    let _ = SelectObject(hdc, previous_pen);
                    let _ = SelectObject(hdc, previous);
                    let _ = DeleteObject(brush as _);
                }
                WindowsNativePanelPaintOperation::StrokeLine {
                    from,
                    to,
                    color,
                    width,
                } => {
                    let pen = CreatePen(PS_SOLID, *width, color_ref(*color));
                    let previous = SelectObject(hdc, pen as _);
                    let _ = MoveToEx(
                        hdc,
                        from.x.round() as i32,
                        from.y.round() as i32,
                        std::ptr::null_mut(),
                    );
                    let _ = LineTo(hdc, to.x.round() as i32, to.y.round() as i32);
                    let _ = SelectObject(hdc, previous);
                    let _ = DeleteObject(pen as _);
                }
                WindowsNativePanelPaintOperation::DrawText {
                    origin,
                    max_width,
                    text,
                    color,
                    size,
                    alignment,
                    ..
                } => {
                    let _ = SetBkMode(hdc, TRANSPARENT as i32);
                    let _ = SetTextColor(hdc, color_ref(*color));
                    let mut wide: Vec<u16> = text.encode_utf16().chain(iter::once(0)).collect();
                    let mut rect = RECT {
                        left: origin.x.round() as i32,
                        top: origin.y.round() as i32,
                        right: (origin.x + max_width).round() as i32,
                        bottom: (origin.y + *size as f64 + 6.0).round() as i32,
                    };
                    let _ = DrawTextW(
                        hdc,
                        wide.as_mut_ptr(),
                        -1,
                        &mut rect,
                        DT_SINGLELINE
                            | DT_VCENTER
                            | gdi_text_alignment(*alignment)
                            | DT_END_ELLIPSIS,
                    );
                }
                WindowsNativePanelPaintOperation::FillMascotDot {
                    frame,
                    radius,
                    color,
                    stroke_color,
                    stroke_width,
                    ..
                } => {
                    let brush = CreateSolidBrush(color_ref(*color));
                    let pen = CreatePen(
                        PS_SOLID,
                        stroke_width.round().max(1.0) as i32,
                        color_ref(*stroke_color),
                    );
                    let previous = SelectObject(hdc, brush as _);
                    let previous_pen = SelectObject(hdc, pen as _);
                    let _ = RoundRect(
                        hdc,
                        frame.x.round() as i32,
                        frame.y.round() as i32,
                        (frame.x + frame.width).round() as i32,
                        (frame.y + frame.height).round() as i32,
                        (radius * 2.0).round() as i32,
                        (radius * 2.0).round() as i32,
                    );
                    let _ = SelectObject(hdc, previous_pen);
                    let _ = SelectObject(hdc, previous);
                    let _ = DeleteObject(pen as _);
                    let _ = DeleteObject(brush as _);
                }
                WindowsNativePanelPaintOperation::FillCompactShoulder { frame, fill, .. } => {
                    let brush = CreateSolidBrush(color_ref(*fill));
                    let rect = rect_from_panel_rect(*frame);
                    let _ = FillRect(hdc, &rect, brush);
                    let _ = DeleteObject(brush as _);
                }
            }
        }

        let _ = ReleaseDC(hwnd as _, hdc);
    }

    Ok(plan)
}

#[cfg(all(windows, not(test)))]
fn color_ref(color: WindowsNativePanelPaintColor) -> u32 {
    color.r as u32 | ((color.g as u32) << 8) | ((color.b as u32) << 16)
}

#[cfg(all(windows, not(test)))]
fn rect_from_panel_rect(rect: PanelRect) -> windows_sys::Win32::Foundation::RECT {
    windows_sys::Win32::Foundation::RECT {
        left: rect.x.round() as i32,
        top: rect.y.round() as i32,
        right: (rect.x + rect.width).round() as i32,
        bottom: (rect.y + rect.height).round() as i32,
    }
}

#[cfg(all(windows, not(test)))]
fn gdi_text_alignment(alignment: NativePanelVisualTextAlignment) -> u32 {
    use windows_sys::Win32::Graphics::Gdi::{DT_CENTER, DT_LEFT, DT_RIGHT};

    match alignment {
        NativePanelVisualTextAlignment::Left => DT_LEFT,
        NativePanelVisualTextAlignment::Center => DT_CENTER,
        NativePanelVisualTextAlignment::Right => DT_RIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WindowsNativePanelPaintColor, WindowsNativePanelPaintOperation,
        WindowsNativePanelPaintPrimitive, resolve_windows_native_panel_hit_test_blocker_operations,
        resolve_windows_native_panel_paint_operations, resolve_windows_native_panel_paint_plan,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelRect},
        native_panel_renderer::facade::{
            descriptor::{NativePanelEdgeAction, NativePanelHostWindowState},
            presentation::{
                NativePanelVisualCardBadgeInput, NativePanelVisualCardInput,
                NativePanelVisualCardRowInput, NativePanelVisualDisplayMode,
            },
        },
        native_panel_scene::SceneMascotPose,
        windows_native_panel::window_shell::{
            WindowsNativePanelShellActionButtonPaintInput, WindowsNativePanelShellPaintJob,
        },
    };

    fn paint_job(display_mode: NativePanelVisualDisplayMode) -> WindowsNativePanelShellPaintJob {
        let compact_bar_width = if display_mode == NativePanelVisualDisplayMode::Expanded {
            crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH
        } else {
            240.0
        };
        WindowsNativePanelShellPaintJob {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 20.0,
                    width: 320.0,
                    height: 160.0,
                }),
                visible: display_mode != NativePanelVisualDisplayMode::Hidden,
                preferred_display_index: 0,
            },
            display_mode,
            surface: ExpandedSurface::Default,
            panel_frame: PanelRect {
                x: 100.0,
                y: 20.0,
                width: 320.0,
                height: 160.0,
            },
            compact_bar_frame: PanelRect {
                x: (320.0 - compact_bar_width) / 2.0,
                y: 12.0,
                width: compact_bar_width,
                height: 36.0,
            },
            left_shoulder_frame: PanelRect {
                x: 34.0,
                y: 42.0,
                width: 6.0,
                height: 6.0,
            },
            right_shoulder_frame: PanelRect {
                x: 280.0,
                y: 42.0,
                width: 6.0,
                height: 6.0,
            },
            shoulder_progress: 0.0,
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            card_stack_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            card_stack_content_height: 160.0,
            shell_frame: PanelRect {
                x: 20.0,
                y: 0.0,
                width: 280.0,
                height: 150.0,
            },
            headline_text: "Codex ready".to_string(),
            headline_emphasized: false,
            active_count: "1".to_string(),
            active_count_elapsed_ms: 0,
            total_count: "3".to_string(),
            separator_visibility: 0.5,
            cards_visible: true,
            card_count: 2,
            cards: vec![
                NativePanelVisualCardInput {
                    style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Settings,
                    title: "Settings".to_string(),
                    subtitle: Some("EchoIsland v0.5.0".to_string()),
                    body: None,
                    badge: None,
                    source_badge: None,
                    body_prefix: None,
                    body_lines: Vec::new(),
                    action_hint: None,
                    rows: vec![NativePanelVisualCardRowInput {
                        title: "Mute Sound".to_string(),
                        value: "Off".to_string(),
                        active: true,
                    }],
                    height: 92.0,
                    collapsed_height: 64.0,
                    compact: false,
                    removing: false,
                },
                NativePanelVisualCardInput {
                    style: crate::native_panel_renderer::facade::presentation::NativePanelVisualCardStyle::Completion,
                    title: "Done".to_string(),
                    subtitle: Some("#abcdef 璺?now".to_string()),
                    body: Some("Task complete".to_string()),
                    badge: Some(NativePanelVisualCardBadgeInput {
                        text: "Done".to_string(),
                        emphasized: true,
                    }),
                    source_badge: Some(NativePanelVisualCardBadgeInput {
                        text: "Codex".to_string(),
                        emphasized: false,
                    }),
                    body_prefix: Some("$".to_string()),
                    body_lines: Vec::new(),
                    action_hint: None,
                    rows: Vec::new(),
                    height: 76.0,
                    collapsed_height: 52.0,
                    compact: false,
                    removing: false,
                },
            ],
            glow_visible: true,
            action_buttons_visible: true,
            action_buttons: vec![
                WindowsNativePanelShellActionButtonPaintInput {
                    action: NativePanelEdgeAction::Settings,
                    frame: PanelRect {
                        x: 250.0,
                        y: 20.0,
                        width: 18.0,
                        height: 18.0,
                    },
                },
                WindowsNativePanelShellActionButtonPaintInput {
                    action: NativePanelEdgeAction::Quit,
                    frame: PanelRect {
                        x: 280.0,
                        y: 20.0,
                        width: 18.0,
                        height: 18.0,
                    },
                },
            ],
            completion_count: 2,
            mascot_elapsed_ms: 0,
            mascot_pose: SceneMascotPose::Complete,
            mascot_debug_mode_enabled: false,
        }
    }

    #[test]
    fn paint_plan_is_empty_when_hidden() {
        let plan = resolve_windows_native_panel_paint_plan(&paint_job(
            NativePanelVisualDisplayMode::Hidden,
        ));

        assert!(plan.hidden);
        assert!(plan.primitives.is_empty());
    }

    #[test]
    fn paint_plan_contains_visible_panel_primitives() {
        let plan = resolve_windows_native_panel_paint_plan(&paint_job(
            NativePanelVisualDisplayMode::Expanded,
        ));

        assert!(!plan.hidden);
        assert!(
            plan.primitives
                .iter()
                .any(|primitive| matches!(primitive, WindowsNativePanelPaintPrimitive::Text { text, .. } if text == "Codex ready"))
        );
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            WindowsNativePanelPaintPrimitive::MascotDot { .. }
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            WindowsNativePanelPaintPrimitive::RoundRect { .. }
        )));
        assert!(
            plan.primitives
                .iter()
                .any(|primitive| matches!(primitive, WindowsNativePanelPaintPrimitive::Text { text, .. } if text == "\u{E713}"))
        );
        assert!(
            plan.primitives
                .iter()
                .any(|primitive| matches!(primitive, WindowsNativePanelPaintPrimitive::Text { text, .. } if text == "⏻"))
        );
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            WindowsNativePanelPaintPrimitive::Ellipse { .. }
        )));
        assert!(
            plan.primitives
                .iter()
                .any(|primitive| matches!(primitive, WindowsNativePanelPaintPrimitive::Text { text, .. } if text == "Mute Sound"))
        );
    }

    #[test]
    fn paint_operations_cover_visual_primitives_without_rebuilding_scene_semantics() {
        let plan = resolve_windows_native_panel_paint_plan(&paint_job(
            NativePanelVisualDisplayMode::Expanded,
        ));

        let operations = resolve_windows_native_panel_paint_operations(&plan);

        assert_eq!(operations.len(), plan.primitives.len());
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::DrawCompletionGlowImage { opacity, .. }
                if *opacity > 0.0
        )));
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::DrawText { text, .. } if text == "Codex ready"
        )));
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::FillMascotDot {
                pose: SceneMascotPose::Complete,
                ..
            }
        )));
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::FillRoundRect { .. }
        )));
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::DrawText { text, .. } if text == "\u{E713}"
        )));
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::DrawText { text, .. } if text == "⏻"
        )));
    }

    #[test]
    fn expanded_paint_adds_nearly_transparent_hit_test_blockers_for_alpha_window_gaps() {
        let job = paint_job(NativePanelVisualDisplayMode::Expanded);

        let operations = resolve_windows_native_panel_hit_test_blocker_operations(&job);

        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::FillHitTestBlocker {
                frame,
                alpha: 1,
            } if *frame == job.shell_frame
        )));
        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::FillHitTestBlocker {
                frame,
                alpha: 1,
            } if *frame == PanelRect {
                x: job.shell_frame.x,
                y: job.shell_frame.y + job.shell_frame.height,
                width: job.shell_frame.width,
                height: job.content_frame.height - job.shell_frame.height,
            }
        )));
    }

    #[test]
    fn compact_paint_does_not_add_alpha_window_gap_blockers() {
        let operations = resolve_windows_native_panel_hit_test_blocker_operations(&paint_job(
            NativePanelVisualDisplayMode::Compact,
        ));

        assert!(operations.is_empty());
    }

    #[test]
    fn mascot_paint_operation_uses_mac_body_fill_color() {
        let plan = resolve_windows_native_panel_paint_plan(&paint_job(
            NativePanelVisualDisplayMode::Compact,
        ));
        let operations = resolve_windows_native_panel_paint_operations(&plan);

        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::FillMascotDot {
                color: WindowsNativePanelPaintColor { r: 5, g: 5, b: 5 },
                ..
            }
        )));
    }

    #[test]
    fn mascot_paint_operation_uses_mac_body_shape() {
        let plan = resolve_windows_native_panel_paint_plan(&paint_job(
            NativePanelVisualDisplayMode::Compact,
        ));
        let operations = resolve_windows_native_panel_paint_operations(&plan);

        assert!(operations.iter().any(|operation| matches!(
            operation,
            WindowsNativePanelPaintOperation::FillMascotDot {
                frame,
                radius,
                ..
            } if (frame.width - 24.0).abs() < 0.5
                && (frame.height - 20.0).abs() < 0.5
                && (*radius - 6.0).abs() < 0.5
        )));
    }

    #[test]
    fn hidden_paint_plan_has_no_backend_operations() {
        let plan = resolve_windows_native_panel_paint_plan(&paint_job(
            NativePanelVisualDisplayMode::Hidden,
        ));

        let operations = resolve_windows_native_panel_paint_operations(&plan);

        assert!(operations.is_empty());
    }

    #[test]
    fn production_painter_prefers_direct2d_per_pixel_alpha_backend() {
        assert_eq!(
            super::windows_native_panel_preferred_painter_backend(),
            super::WindowsNativePanelPainterBackend::Direct2D
        );
        assert_eq!(
            super::windows_native_panel_composition_mode_for_preferred_painter(),
            crate::windows_native_panel::layered_window::WindowsLayeredWindowCompositionMode::PerPixelAlpha
        );
    }
}
