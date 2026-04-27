use crate::native_panel_renderer::facade::visual::{
    NativePanelVisualColor, NativePanelVisualPlan, NativePanelVisualPrimitive,
    resolve_native_panel_visual_plan,
};

use crate::native_panel_core::{PanelPoint, PanelRect};
use crate::native_panel_scene::SceneMascotPose;

use super::window_shell::WindowsNativePanelShellPaintJob;

pub(super) const WINDOWS_NATIVE_PANEL_TRANSPARENT_COLOR_KEY: u32 = 0x00FF00FF;

pub(super) type WindowsNativePanelPaintColor = NativePanelVisualColor;
pub(super) type WindowsNativePanelPaintPlan = NativePanelVisualPlan;
pub(super) type WindowsNativePanelPaintPrimitive = NativePanelVisualPrimitive;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum WindowsNativePanelPaintOperation {
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
    },
    FillMascotDot {
        frame: PanelRect,
        radius: f64,
        pose: SceneMascotPose,
        color: WindowsNativePanelPaintColor,
    },
}

pub(super) fn resolve_windows_native_panel_paint_plan(
    job: &WindowsNativePanelShellPaintJob,
) -> WindowsNativePanelPaintPlan {
    resolve_native_panel_visual_plan(job)
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

fn windows_native_panel_paint_operation_from_primitive(
    primitive: &WindowsNativePanelPaintPrimitive,
) -> WindowsNativePanelPaintOperation {
    match primitive {
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
        } => WindowsNativePanelPaintOperation::DrawText {
            origin: *origin,
            max_width: *max_width,
            text: text.clone(),
            color: *color,
            size: *size,
        },
        WindowsNativePanelPaintPrimitive::MascotDot {
            center,
            radius,
            pose,
        } => WindowsNativePanelPaintOperation::FillMascotDot {
            frame: PanelRect {
                x: center.x - radius,
                y: center.y - radius,
                width: radius * 2.0,
                height: radius * 2.0,
            },
            radius: *radius,
            pose: *pose,
            color: WindowsNativePanelPaintColor {
                r: 245,
                g: 207,
                b: 74,
            },
        },
    }
}

#[cfg(windows)]
pub(super) fn paint_windows_native_panel_job(
    raw_window_handle: Option<isize>,
    job: &WindowsNativePanelShellPaintJob,
) -> Result<WindowsNativePanelPaintPlan, String> {
    use std::iter;
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{
            CreatePen, CreateSolidBrush, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DeleteObject,
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
                        DT_SINGLELINE | DT_LEFT | DT_END_ELLIPSIS,
                    );
                }
                WindowsNativePanelPaintOperation::FillMascotDot {
                    frame,
                    radius,
                    color,
                    ..
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
            }
        }

        let _ = ReleaseDC(hwnd as _, hdc);
    }

    Ok(plan)
}

#[cfg(not(windows))]
pub(super) fn paint_windows_native_panel_job(
    _raw_window_handle: Option<isize>,
    job: &WindowsNativePanelShellPaintJob,
) -> Result<WindowsNativePanelPaintPlan, String> {
    Ok(resolve_windows_native_panel_paint_plan(job))
}

#[cfg(windows)]
fn color_ref(color: WindowsNativePanelPaintColor) -> u32 {
    color.r as u32 | ((color.g as u32) << 8) | ((color.b as u32) << 16)
}

#[cfg(windows)]
fn rect_from_panel_rect(rect: PanelRect) -> windows_sys::Win32::Foundation::RECT {
    windows_sys::Win32::Foundation::RECT {
        left: rect.x.round() as i32,
        top: rect.y.round() as i32,
        right: (rect.x + rect.width).round() as i32,
        bottom: (rect.y + rect.height).round() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WindowsNativePanelPaintOperation, WindowsNativePanelPaintPrimitive,
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
                x: 40.0,
                y: 12.0,
                width: 240.0,
                height: 36.0,
            },
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            },
            shell_frame: PanelRect {
                x: 20.0,
                y: 0.0,
                width: 280.0,
                height: 150.0,
            },
            headline_text: "Codex ready".to_string(),
            headline_emphasized: false,
            separator_visibility: 0.5,
            cards_visible: true,
            card_count: 2,
            cards: vec![
                NativePanelVisualCardInput {
                    title: "Settings".to_string(),
                    subtitle: Some("EchoIsland v0.2.0".to_string()),
                    body: None,
                    badge: None,
                    rows: vec![NativePanelVisualCardRowInput {
                        title: "Mute Sound".to_string(),
                        value: "Off".to_string(),
                        active: true,
                    }],
                },
                NativePanelVisualCardInput {
                    title: "Done".to_string(),
                    subtitle: Some("#abcdef · now".to_string()),
                    body: Some("Task complete".to_string()),
                    badge: Some(NativePanelVisualCardBadgeInput {
                        text: "Done".to_string(),
                        emphasized: true,
                    }),
                    rows: Vec::new(),
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
            mascot_pose: SceneMascotPose::Complete,
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
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            WindowsNativePanelPaintPrimitive::StrokeLine { .. }
        )));
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
            WindowsNativePanelPaintOperation::StrokeLine { .. }
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
}
