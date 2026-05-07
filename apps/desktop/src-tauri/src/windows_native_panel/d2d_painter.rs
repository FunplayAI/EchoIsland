use super::{
    direct2d::WindowsDirect2DFactory,
    directwrite::WindowsDirectWriteFactory,
    directwrite::WindowsDirectWriteTextLayoutRequest,
    dpi::{WindowsDpiScale, WindowsPhysicalRect},
    paint_backend::{WindowsNativePanelPaintPlan, resolve_windows_native_panel_paint_plan},
    window_shell::WindowsNativePanelShellPaintJob,
};
use crate::native_panel_core::{PanelPoint, PanelRect};
use crate::native_panel_renderer::facade::visual::NativePanelVisualShoulderSide;

#[cfg(all(windows, not(test)))]
use super::{
    dpi::resolve_windows_dpi_scale_for_window,
    paint_backend::{
        WindowsNativePanelPaintColor, WindowsNativePanelPaintOperation,
        resolve_windows_native_panel_hit_test_blocker_operations,
        resolve_windows_native_panel_paint_operations,
    },
};

#[cfg(all(windows, not(test)))]
const COMPLETION_GLOW_IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/island-completion-inner-glow-9slice.png"
));

pub(super) fn directwrite_text_requests_from_paint_plan(
    plan: &WindowsNativePanelPaintPlan,
) -> Vec<WindowsDirectWriteTextLayoutRequest> {
    if plan.hidden {
        return Vec::new();
    }
    plan.primitives
        .iter()
        .filter_map(|primitive| {
            match primitive {
            crate::native_panel_renderer::facade::visual::NativePanelVisualPrimitive::Text {
                text,
                max_width,
                size,
                weight,
                alignment,
                ..
            }
            | crate::native_panel_renderer::facade::visual::NativePanelVisualPrimitive::MascotText {
                text,
                max_width,
                size,
                weight,
                alignment,
                ..
            } => Some(WindowsDirectWriteTextLayoutRequest::new(
                text.clone(),
                *max_width,
                *size,
                *weight,
                *alignment,
            )),
            _ => None,
        }
        })
        .collect()
}

pub(super) trait WindowsNativePanelPainter {
    fn paint(
        &mut self,
        job: &WindowsNativePanelShellPaintJob,
    ) -> Result<WindowsNativePanelPaintPlan, String>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WindowsDirect2DResourceKey {
    physical_rect: WindowsPhysicalRect,
    dpi_scale_millis: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct WindowsDirect2DResourceCacheState {
    current_key: Option<WindowsDirect2DResourceKey>,
    rebuild_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowsDirect2DCoordinateSpace {
    surface_height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowsCompactShoulderPath {
    start: PanelPoint,
    line_to_top_edge: PanelPoint,
    line_to_outer_edge: PanelPoint,
    curve_control_1: PanelPoint,
    curve_control_2: PanelPoint,
    curve_end: PanelPoint,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WindowsCompactPillPath {
    start: PanelPoint,
    top_right: PanelPoint,
    right_edge_bottom_arc_start: PanelPoint,
    bottom_right_control_1: PanelPoint,
    bottom_right_control_2: PanelPoint,
    bottom_right_arc_end: PanelPoint,
    bottom_left_arc_start: PanelPoint,
    bottom_left_control_1: PanelPoint,
    bottom_left_control_2: PanelPoint,
    bottom_left_arc_end: PanelPoint,
}

impl WindowsDirect2DResourceKey {
    pub(super) fn new(physical_rect: WindowsPhysicalRect, dpi_scale: WindowsDpiScale) -> Self {
        Self {
            physical_rect,
            dpi_scale_millis: (dpi_scale.scale * 1000.0).round() as i32,
        }
    }
}

impl WindowsDirect2DResourceCacheState {
    pub(super) fn sync(&mut self, key: WindowsDirect2DResourceKey) -> bool {
        if self.current_key == Some(key) {
            return false;
        }
        self.current_key = Some(key);
        self.rebuild_count += 1;
        true
    }

    pub(super) fn rebuild_count(&self) -> usize {
        self.rebuild_count
    }
}

impl WindowsDirect2DCoordinateSpace {
    pub(super) fn new(surface_height: f64) -> Self {
        Self {
            surface_height: surface_height.max(1.0),
        }
    }

    pub(super) fn rect(self, rect: PanelRect) -> PanelRect {
        PanelRect {
            x: rect.x,
            y: self.surface_height - rect.y - rect.height,
            width: rect.width,
            height: rect.height,
        }
    }

    pub(super) fn point(self, point: PanelPoint) -> PanelPoint {
        PanelPoint {
            x: point.x,
            y: self.surface_height - point.y,
        }
    }

    pub(super) fn text_rect(
        self,
        origin: PanelPoint,
        max_width: f64,
        text_height: f64,
    ) -> PanelRect {
        PanelRect {
            x: origin.x,
            y: self.surface_height - origin.y - text_height,
            width: max_width,
            height: text_height,
        }
    }
}

impl WindowsCompactShoulderPath {
    pub(super) fn resolve(
        frame: PanelRect,
        side: NativePanelVisualShoulderSide,
        progress: f64,
    ) -> Option<Self> {
        let scale_x = (1.0 - progress.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        if frame.width <= 0.0 || frame.height <= 0.0 || scale_x <= 0.001 {
            return None;
        }

        let control_y = frame.y
            + frame.height * (1.0 - crate::native_panel_core::COMPACT_SHOULDER_CURVE_FACTOR);

        match side {
            NativePanelVisualShoulderSide::Left => {
                let left = frame.x + frame.width * (1.0 - scale_x);
                let right = frame.x + frame.width;
                let control_x = right
                    - frame.width
                        * (1.0 - crate::native_panel_core::COMPACT_SHOULDER_CURVE_FACTOR)
                        * scale_x;
                Some(Self {
                    start: PanelPoint {
                        x: left,
                        y: frame.y,
                    },
                    line_to_top_edge: PanelPoint {
                        x: right,
                        y: frame.y,
                    },
                    line_to_outer_edge: PanelPoint {
                        x: right,
                        y: frame.y + frame.height,
                    },
                    curve_control_1: PanelPoint {
                        x: right,
                        y: control_y,
                    },
                    curve_control_2: PanelPoint {
                        x: control_x,
                        y: frame.y,
                    },
                    curve_end: PanelPoint {
                        x: left,
                        y: frame.y,
                    },
                })
            }
            NativePanelVisualShoulderSide::Right => {
                let left = frame.x;
                let right = frame.x + frame.width * scale_x;
                let control_x = frame.x
                    + frame.width
                        * crate::native_panel_core::COMPACT_SHOULDER_CURVE_FACTOR
                        * scale_x;
                Some(Self {
                    start: PanelPoint {
                        x: right,
                        y: frame.y,
                    },
                    line_to_top_edge: PanelPoint {
                        x: left,
                        y: frame.y,
                    },
                    line_to_outer_edge: PanelPoint {
                        x: left,
                        y: frame.y + frame.height,
                    },
                    curve_control_1: PanelPoint {
                        x: left,
                        y: control_y,
                    },
                    curve_control_2: PanelPoint {
                        x: control_x,
                        y: frame.y,
                    },
                    curve_end: PanelPoint {
                        x: right,
                        y: frame.y,
                    },
                })
            }
        }
    }
}

impl WindowsCompactPillPath {
    pub(super) fn resolve(frame: PanelRect, radius: f64) -> Self {
        const ARC_CONTROL_FACTOR: f64 = 0.552_284_749_830_793_6;

        let radius = radius
            .max(0.0)
            .min(frame.width / 2.0)
            .min(frame.height.max(0.0));
        let control = radius * ARC_CONTROL_FACTOR;
        let left = frame.x;
        let right = frame.x + frame.width;
        let top = frame.y;
        let bottom = frame.y + frame.height;

        Self {
            start: PanelPoint { x: left, y: top },
            top_right: PanelPoint { x: right, y: top },
            right_edge_bottom_arc_start: PanelPoint {
                x: right,
                y: bottom - radius,
            },
            bottom_right_control_1: PanelPoint {
                x: right,
                y: bottom - radius + control,
            },
            bottom_right_control_2: PanelPoint {
                x: right - radius + control,
                y: bottom,
            },
            bottom_right_arc_end: PanelPoint {
                x: right - radius,
                y: bottom,
            },
            bottom_left_arc_start: PanelPoint {
                x: left + radius,
                y: bottom,
            },
            bottom_left_control_1: PanelPoint {
                x: left + radius - control,
                y: bottom,
            },
            bottom_left_control_2: PanelPoint {
                x: left,
                y: bottom - radius + control,
            },
            bottom_left_arc_end: PanelPoint {
                x: left,
                y: bottom - radius,
            },
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Direct2DWindowsNativePanelPainter {
    raw_window_handle: Option<isize>,
    direct2d: WindowsDirect2DFactory,
    directwrite: WindowsDirectWriteFactory,
    resource_cache: WindowsDirect2DResourceCacheState,
    #[cfg(all(windows, not(test)))]
    surface: Option<WindowsDirect2DPaintSurface>,
    #[cfg(all(windows, not(test)))]
    completion_glow_bitmap: Option<windows::Win32::Graphics::Direct2D::ID2D1Bitmap>,
}

impl Direct2DWindowsNativePanelPainter {
    pub(super) fn new(raw_window_handle: Option<isize>) -> Result<Self, String> {
        Ok(Self {
            raw_window_handle,
            direct2d: WindowsDirect2DFactory::shared()?,
            directwrite: WindowsDirectWriteFactory::shared()?,
            resource_cache: WindowsDirect2DResourceCacheState::default(),
            #[cfg(all(windows, not(test)))]
            surface: None,
            #[cfg(all(windows, not(test)))]
            completion_glow_bitmap: None,
        })
    }

    pub(super) fn set_raw_window_handle(&mut self, raw_window_handle: Option<isize>) {
        self.raw_window_handle = raw_window_handle;
    }

    pub(super) fn is_per_pixel_alpha_ready(&self) -> bool {
        self.direct2d.is_initialized() && self.directwrite.is_initialized()
    }

    pub(super) fn resource_rebuild_count(&self) -> usize {
        self.resource_cache.rebuild_count()
    }
}

impl WindowsNativePanelPainter for Direct2DWindowsNativePanelPainter {
    fn paint(
        &mut self,
        job: &WindowsNativePanelShellPaintJob,
    ) -> Result<WindowsNativePanelPaintPlan, String> {
        let plan = resolve_windows_native_panel_paint_plan(job);
        #[cfg(all(windows, not(test)))]
        self.paint_plan_to_layered_window(job, &plan)?;
        #[cfg(any(not(windows), test))]
        let _ = (self.raw_window_handle, job);
        Ok(plan)
    }
}

#[cfg(all(windows, not(test)))]
impl Direct2DWindowsNativePanelPainter {
    fn paint_plan_to_layered_window(
        &mut self,
        job: &WindowsNativePanelShellPaintJob,
        plan: &WindowsNativePanelPaintPlan,
    ) -> Result<(), String> {
        use windows::Win32::Foundation::RECT;
        use windows::Win32::Graphics::{
            Direct2D::{
                Common::D2D1_COLOR_F, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                D2D1_DRAW_TEXT_OPTIONS_CLIP, D2D1_ROUNDED_RECT, D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
            },
            DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
            Gdi::HDC,
        };

        if plan.hidden {
            return Ok(());
        }
        let Some(hwnd) = self.raw_window_handle else {
            return Ok(());
        };
        let Some(frame) = job.window_state.frame else {
            return Ok(());
        };
        let dpi_scale = resolve_windows_dpi_scale_for_window(self.raw_window_handle);
        let physical_rect = dpi_scale.rect_to_physical(frame);
        let resource_key = WindowsDirect2DResourceKey::new(physical_rect, dpi_scale);
        self.ensure_surface(resource_key, dpi_scale)?;
        let coordinate_space = WindowsDirect2DCoordinateSpace::new(frame.height);
        let surface = self
            .surface
            .as_ref()
            .ok_or_else(|| "Direct2D surface was not initialized".to_string())?;
        let bind_rect = RECT {
            left: 0,
            top: 0,
            right: physical_rect.width,
            bottom: physical_rect.height,
        };
        unsafe {
            surface
                .target
                .BindDC(HDC(surface.dib.hdc), &bind_rect)
                .map_err(|error| error.to_string())?;
            surface
                .target
                .SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            surface
                .target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            surface.target.BeginDraw();
            surface.target.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            let mut operations = resolve_windows_native_panel_hit_test_blocker_operations(job);
            operations.extend(resolve_windows_native_panel_paint_operations(plan));
            for operation in operations {
                match operation {
                    WindowsNativePanelPaintOperation::DrawCompletionGlowImage {
                        frame,
                        opacity,
                    } => {
                        let target = &surface.target;
                        let Ok(bitmap) = ensure_completion_glow_bitmap_for_target(
                            target,
                            &mut self.completion_glow_bitmap,
                        ) else {
                            continue;
                        };
                        draw_completion_glow_image(
                            target,
                            bitmap,
                            coordinate_space,
                            frame,
                            opacity,
                        );
                    }
                    WindowsNativePanelPaintOperation::FillHitTestBlocker { frame, alpha } => {
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_alpha_color(alpha), None)
                            .map_err(|error| error.to_string())?;
                        surface
                            .target
                            .FillRectangle(&d2d_rect(coordinate_space.rect(frame)), &brush);
                    }
                    WindowsNativePanelPaintOperation::FillRoundRect {
                        frame,
                        radius,
                        color,
                    } => {
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(color), None)
                            .map_err(|error| error.to_string())?;
                        if job.display_mode
                            == crate::native_panel_renderer::facade::presentation::NativePanelVisualDisplayMode::Compact
                            && frame == job.compact_bar_frame
                        {
                            let geometry = d2d_compact_pill_geometry(
                                self.direct2d
                                    .factory()
                                    .ok_or_else(|| "Direct2D factory is not initialized".to_string())?,
                                WindowsCompactPillPath::resolve(
                                    coordinate_space.rect(frame),
                                    radius,
                                ),
                            )?;
                            surface.target.FillGeometry(&geometry, &brush, None);
                        } else {
                            surface.target.FillRoundedRectangle(
                                &D2D1_ROUNDED_RECT {
                                    rect: d2d_rect(coordinate_space.rect(frame)),
                                    radiusX: radius as f32,
                                    radiusY: radius as f32,
                                },
                                &brush,
                            );
                        }
                    }
                    WindowsNativePanelPaintOperation::FillRect { frame, color } => {
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(color), None)
                            .map_err(|error| error.to_string())?;
                        surface
                            .target
                            .FillRectangle(&d2d_rect(coordinate_space.rect(frame)), &brush);
                    }
                    WindowsNativePanelPaintOperation::FillEllipse { frame, color } => {
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(color), None)
                            .map_err(|error| error.to_string())?;
                        surface
                            .target
                            .FillEllipse(&d2d_ellipse(coordinate_space.rect(frame)), &brush);
                    }
                    WindowsNativePanelPaintOperation::StrokeLine {
                        from,
                        to,
                        color,
                        width,
                    } => {
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(color), None)
                            .map_err(|error| error.to_string())?;
                        surface.target.DrawLine(
                            d2d_point(coordinate_space.point(from)),
                            d2d_point(coordinate_space.point(to)),
                            &brush,
                            width.max(1) as f32,
                            None,
                        );
                    }
                    WindowsNativePanelPaintOperation::DrawText {
                        origin,
                        max_width,
                        text,
                        color,
                        size,
                        weight,
                        alignment,
                    } => {
                        let request = WindowsDirectWriteTextLayoutRequest::new(
                            text.clone(),
                            max_width,
                            size,
                            weight,
                            alignment,
                        );
                        let text_format = self.directwrite.create_text_format(
                            request.fonts,
                            request.size,
                            request.weight,
                            request.alignment,
                        )?;
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(color), None)
                            .map_err(|error| error.to_string())?;
                        let text_rect = d2d_rect(coordinate_space.text_rect(
                            origin,
                            max_width,
                            windows_directwrite_text_box_height(&text, size),
                        ));
                        let wide: Vec<u16> = text.encode_utf16().collect();
                        surface.target.DrawText(
                            &wide,
                            &text_format,
                            &text_rect,
                            &brush,
                            D2D1_DRAW_TEXT_OPTIONS_CLIP,
                            DWRITE_MEASURING_MODE_NATURAL,
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
                        let fill_brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(color), None)
                            .map_err(|error| error.to_string())?;
                        let stroke_brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(stroke_color), None)
                            .map_err(|error| error.to_string())?;
                        let rect = D2D1_ROUNDED_RECT {
                            rect: d2d_rect(coordinate_space.rect(frame)),
                            radiusX: radius as f32,
                            radiusY: radius as f32,
                        };
                        surface.target.FillRoundedRectangle(&rect, &fill_brush);
                        surface.target.DrawRoundedRectangle(
                            &rect,
                            &stroke_brush,
                            stroke_width.max(1.0) as f32,
                            None,
                        );
                    }
                    WindowsNativePanelPaintOperation::FillCompactShoulder {
                        frame,
                        side,
                        progress,
                        fill,
                        ..
                    } => {
                        let Some(path) = WindowsCompactShoulderPath::resolve(
                            coordinate_space.rect(frame),
                            side,
                            progress,
                        ) else {
                            continue;
                        };
                        let brush = surface
                            .target
                            .CreateSolidColorBrush(&d2d_color(fill), None)
                            .map_err(|error| error.to_string())?;
                        let geometry = d2d_compact_shoulder_geometry(
                            self.direct2d
                                .factory()
                                .ok_or_else(|| "Direct2D factory is not initialized".to_string())?,
                            path,
                        )?;
                        surface.target.FillGeometry(&geometry, &brush, None);
                    }
                }
            }

            surface
                .target
                .EndDraw(None, None)
                .map_err(|error| error.to_string())?;
        }

        surface
            .dib
            .update_layered_window(hwnd, physical_rect.x, physical_rect.y)
    }

    fn ensure_surface(
        &mut self,
        key: WindowsDirect2DResourceKey,
        dpi_scale: WindowsDpiScale,
    ) -> Result<(), String> {
        if !self.resource_cache.sync(key) && self.surface.is_some() {
            return Ok(());
        }
        let factory = self
            .direct2d
            .factory()
            .ok_or_else(|| "Direct2D factory is not initialized".to_string())?;
        self.surface = Some(WindowsDirect2DPaintSurface::new(factory, key, dpi_scale)?);
        self.completion_glow_bitmap = None;
        Ok(())
    }
}

#[cfg(all(windows, not(test)))]
fn ensure_completion_glow_bitmap_for_target<'a>(
    target: &windows::Win32::Graphics::Direct2D::ID2D1DCRenderTarget,
    slot: &'a mut Option<windows::Win32::Graphics::Direct2D::ID2D1Bitmap>,
) -> Result<&'a windows::Win32::Graphics::Direct2D::ID2D1Bitmap, String> {
    if slot.is_none() {
        *slot = Some(create_completion_glow_bitmap(target)?);
    }
    slot.as_ref()
        .ok_or_else(|| "completion glow bitmap was not initialized".to_string())
}

#[cfg(all(windows, not(test)))]
fn create_completion_glow_bitmap(
    target: &windows::Win32::Graphics::Direct2D::ID2D1DCRenderTarget,
) -> Result<windows::Win32::Graphics::Direct2D::ID2D1Bitmap, String> {
    use windows::Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        Graphics::Imaging::{
            CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICBitmapSource,
            IWICImagingFactory, WICBitmapDitherTypeNone, WICBitmapPaletteTypeMedianCut,
            WICDecodeMetadataCacheOnLoad,
        },
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        },
    };

    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .or_else(|error| {
            if error.code() == RPC_E_CHANGED_MODE {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|error| error.to_string())?;

    let factory: IWICImagingFactory =
        unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| error.to_string())?;
    let stream = unsafe { factory.CreateStream() }.map_err(|error| error.to_string())?;
    unsafe { stream.InitializeFromMemory(COMPLETION_GLOW_IMAGE_BYTES) }
        .map_err(|error| error.to_string())?;
    let decoder = unsafe {
        factory.CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnLoad)
    }
    .map_err(|error| error.to_string())?;
    let frame = unsafe { decoder.GetFrame(0) }.map_err(|error| error.to_string())?;
    let converter =
        unsafe { factory.CreateFormatConverter() }.map_err(|error| error.to_string())?;
    unsafe {
        converter.Initialize(
            &frame,
            &GUID_WICPixelFormat32bppPBGRA,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeMedianCut,
        )
    }
    .map_err(|error| error.to_string())?;
    let source: IWICBitmapSource = converter.into();
    unsafe { target.CreateBitmapFromWicBitmap(&source, None) }.map_err(|error| error.to_string())
}

#[cfg(all(windows, not(test)))]
fn draw_completion_glow_image(
    target: &windows::Win32::Graphics::Direct2D::ID2D1DCRenderTarget,
    bitmap: &windows::Win32::Graphics::Direct2D::ID2D1Bitmap,
    coordinate_space: WindowsDirect2DCoordinateSpace,
    frame: PanelRect,
    opacity: f64,
) {
    use windows::Win32::Graphics::Direct2D::D2D1_BITMAP_INTERPOLATION_MODE_LINEAR;

    for slice in
        crate::native_panel_renderer::facade::presentation::resolve_completion_glow_image_slices(
            frame,
        )
    {
        let dest = slice.dest;
        let source = slice.source;
        if dest.width <= 0.0 || dest.height <= 0.0 || source.width <= 0.0 || source.height <= 0.0 {
            continue;
        }
        let dest = d2d_rect(coordinate_space.rect(dest));
        let source = d2d_rect(source);
        unsafe {
            target.DrawBitmap(
                bitmap,
                Some(&dest),
                opacity.clamp(0.0, 1.0) as f32,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                Some(&source),
            );
        }
    }
}

#[cfg(all(windows, not(test)))]
fn d2d_color(
    color: WindowsNativePanelPaintColor,
) -> windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
    windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: 1.0,
    }
}

#[cfg(all(windows, not(test)))]
fn d2d_alpha_color(alpha: u8) -> windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
    windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: alpha as f32 / 255.0,
    }
}

#[cfg(all(windows, not(test)))]
fn d2d_rect(rect: PanelRect) -> windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F {
    windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F {
        left: rect.x as f32,
        top: rect.y as f32,
        right: (rect.x + rect.width) as f32,
        bottom: (rect.y + rect.height) as f32,
    }
}

#[cfg(all(windows, not(test)))]
fn d2d_ellipse(rect: PanelRect) -> windows::Win32::Graphics::Direct2D::D2D1_ELLIPSE {
    windows::Win32::Graphics::Direct2D::D2D1_ELLIPSE {
        point: windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F {
            x: (rect.x + rect.width / 2.0) as f32,
            y: (rect.y + rect.height / 2.0) as f32,
        },
        radiusX: (rect.width / 2.0) as f32,
        radiusY: (rect.height / 2.0) as f32,
    }
}

#[cfg(all(windows, not(test)))]
fn d2d_point(point: PanelPoint) -> windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F {
    windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F {
        x: point.x as f32,
        y: point.y as f32,
    }
}

#[cfg(all(windows, not(test)))]
fn d2d_compact_shoulder_geometry(
    factory: &windows::Win32::Graphics::Direct2D::ID2D1Factory,
    path: WindowsCompactShoulderPath,
) -> Result<windows::Win32::Graphics::Direct2D::ID2D1PathGeometry, String> {
    use windows::Win32::Graphics::Direct2D::Common::{
        D2D1_BEZIER_SEGMENT, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED,
    };

    let geometry = unsafe { factory.CreatePathGeometry() }.map_err(|error| error.to_string())?;
    let sink = unsafe { geometry.Open() }.map_err(|error| error.to_string())?;
    unsafe {
        sink.BeginFigure(d2d_point(path.start), D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(d2d_point(path.line_to_top_edge));
        sink.AddLine(d2d_point(path.line_to_outer_edge));
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: d2d_point(path.curve_control_1),
            point2: d2d_point(path.curve_control_2),
            point3: d2d_point(path.curve_end),
        });
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close().map_err(|error| error.to_string())?;
    }
    Ok(geometry)
}

#[cfg(all(windows, not(test)))]
fn d2d_compact_pill_geometry(
    factory: &windows::Win32::Graphics::Direct2D::ID2D1Factory,
    path: WindowsCompactPillPath,
) -> Result<windows::Win32::Graphics::Direct2D::ID2D1PathGeometry, String> {
    use windows::Win32::Graphics::Direct2D::Common::{
        D2D1_BEZIER_SEGMENT, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED,
    };

    let geometry = unsafe { factory.CreatePathGeometry() }.map_err(|error| error.to_string())?;
    let sink = unsafe { geometry.Open() }.map_err(|error| error.to_string())?;
    unsafe {
        sink.BeginFigure(d2d_point(path.start), D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(d2d_point(path.top_right));
        sink.AddLine(d2d_point(path.right_edge_bottom_arc_start));
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: d2d_point(path.bottom_right_control_1),
            point2: d2d_point(path.bottom_right_control_2),
            point3: d2d_point(path.bottom_right_arc_end),
        });
        sink.AddLine(d2d_point(path.bottom_left_arc_start));
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: d2d_point(path.bottom_left_control_1),
            point2: d2d_point(path.bottom_left_control_2),
            point3: d2d_point(path.bottom_left_arc_end),
        });
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close().map_err(|error| error.to_string())?;
    }
    Ok(geometry)
}

fn windows_directwrite_text_box_height(text: &str, size: i32) -> f64 {
    let line_count = text.lines().count().max(1) as f64;
    let line_height = if size >= 13 { 24.0 } else { size as f64 + 8.0 };
    line_count * line_height
}

#[cfg(all(windows, not(test)))]
#[derive(Debug)]
struct WindowsDirect2DLayeredDib {
    hdc: windows_sys::Win32::Graphics::Gdi::HDC,
    bitmap: windows_sys::Win32::Graphics::Gdi::HBITMAP,
    previous: windows_sys::Win32::Graphics::Gdi::HGDIOBJ,
    width: i32,
    height: i32,
}

#[cfg(all(windows, not(test)))]
#[derive(Debug)]
struct WindowsDirect2DPaintSurface {
    _key: WindowsDirect2DResourceKey,
    dib: WindowsDirect2DLayeredDib,
    target: windows::Win32::Graphics::Direct2D::ID2D1DCRenderTarget,
}

#[cfg(all(windows, not(test)))]
impl WindowsDirect2DPaintSurface {
    fn new(
        factory: &windows::Win32::Graphics::Direct2D::ID2D1Factory,
        key: WindowsDirect2DResourceKey,
        dpi_scale: WindowsDpiScale,
    ) -> Result<Self, String> {
        use windows::Win32::Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT},
                D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES,
                D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
            },
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
        };

        let dib =
            WindowsDirect2DLayeredDib::new(key.physical_rect.width, key.physical_rect.height)?;
        let target_props = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: (96.0 * dpi_scale.scale) as f32,
            dpiY: (96.0 * dpi_scale.scale) as f32,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        let target = unsafe { factory.CreateDCRenderTarget(&target_props) }
            .map_err(|error| error.to_string())?;
        Ok(Self {
            _key: key,
            dib,
            target,
        })
    }
}

#[cfg(all(windows, not(test)))]
impl WindowsDirect2DLayeredDib {
    fn new(width: i32, height: i32) -> Result<Self, String> {
        use std::ptr;
        use windows_sys::Win32::Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection,
            DIB_RGB_COLORS, RGBQUAD, SelectObject,
        };

        let width = width.max(1);
        let height = height.max(1);
        unsafe {
            let hdc = CreateCompatibleDC(ptr::null_mut());
            if hdc.is_null() {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    biSizeImage: (width * height * 4) as u32,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };
            let mut bits = ptr::null_mut();
            let bitmap = CreateDIBSection(
                hdc,
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut bits,
                ptr::null_mut(),
                0,
            );
            if bitmap.is_null() || bits.is_null() {
                let _ = windows_sys::Win32::Graphics::Gdi::DeleteDC(hdc);
                return Err(std::io::Error::last_os_error().to_string());
            }
            let previous = SelectObject(hdc, bitmap as _);
            if previous.is_null() {
                let _ = windows_sys::Win32::Graphics::Gdi::DeleteObject(bitmap as _);
                let _ = windows_sys::Win32::Graphics::Gdi::DeleteDC(hdc);
                return Err(std::io::Error::last_os_error().to_string());
            }
            Ok(Self {
                hdc,
                bitmap,
                previous,
                width,
                height,
            })
        }
    }

    fn update_layered_window(&self, hwnd: isize, x: i32, y: i32) -> Result<(), String> {
        use windows_sys::Win32::{
            Foundation::{POINT, SIZE},
            Graphics::Gdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION},
            UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow},
        };

        let destination = POINT { x, y };
        let size = SIZE {
            cx: self.width,
            cy: self.height,
        };
        let source = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let ok = unsafe {
            UpdateLayeredWindow(
                hwnd as _,
                std::ptr::null_mut(),
                &destination,
                &size,
                self.hdc,
                &source,
                0,
                &blend,
                ULW_ALPHA,
            )
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(())
    }
}

#[cfg(all(windows, not(test)))]
impl Drop for WindowsDirect2DLayeredDib {
    fn drop(&mut self) {
        unsafe {
            let _ = windows_sys::Win32::Graphics::Gdi::SelectObject(self.hdc, self.previous);
            let _ = windows_sys::Win32::Graphics::Gdi::DeleteObject(self.bitmap as _);
            let _ = windows_sys::Win32::Graphics::Gdi::DeleteDC(self.hdc);
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct PlanOnlyWindowsNativePanelPainter;

impl WindowsNativePanelPainter for PlanOnlyWindowsNativePanelPainter {
    fn paint(
        &mut self,
        job: &WindowsNativePanelShellPaintJob,
    ) -> Result<WindowsNativePanelPaintPlan, String> {
        Ok(resolve_windows_native_panel_paint_plan(job))
    }
}

#[cfg(all(windows, not(test)))]
#[derive(Debug)]
pub(super) struct GdiWindowsNativePanelPainter {
    raw_window_handle: Option<isize>,
}

#[cfg(all(windows, not(test)))]
impl GdiWindowsNativePanelPainter {
    pub(super) fn new(raw_window_handle: Option<isize>) -> Self {
        Self { raw_window_handle }
    }
}

#[cfg(all(windows, not(test)))]
impl WindowsNativePanelPainter for GdiWindowsNativePanelPainter {
    fn paint(
        &mut self,
        job: &WindowsNativePanelShellPaintJob,
    ) -> Result<WindowsNativePanelPaintPlan, String> {
        super::paint_backend::paint_windows_native_panel_job_with_gdi(self.raw_window_handle, job)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Direct2DWindowsNativePanelPainter, PlanOnlyWindowsNativePanelPainter,
        WindowsCompactPillPath, WindowsCompactShoulderPath, WindowsDirect2DCoordinateSpace,
        WindowsDirect2DResourceCacheState, WindowsDirect2DResourceKey, WindowsNativePanelPainter,
        windows_directwrite_text_box_height,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelPoint, PanelRect},
        native_panel_renderer::facade::{
            descriptor::NativePanelEdgeAction,
            descriptor::NativePanelHostWindowState,
            presentation::{
                NativePanelVisualActionButtonInput, NativePanelVisualCardBadgeInput,
                NativePanelVisualCardInput, NativePanelVisualCardRowInput,
                NativePanelVisualDisplayMode,
            },
        },
        native_panel_scene::SceneMascotPose,
        windows_native_panel::window_shell::WindowsNativePanelShellPaintJob,
    };

    fn compact_paint_job() -> WindowsNativePanelShellPaintJob {
        WindowsNativePanelShellPaintJob {
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 20.0,
                    width: 320.0,
                    height: 80.0,
                }),
                visible: true,
                preferred_display_index: 0,
            },
            display_mode: NativePanelVisualDisplayMode::Compact,
            surface: ExpandedSurface::Default,
            panel_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 80.0,
            },
            compact_bar_frame: PanelRect {
                x: 32.0,
                y: 12.0,
                width: 253.0,
                height: 40.0,
            },
            left_shoulder_frame: PanelRect {
                x: 26.0,
                y: 46.0,
                width: 6.0,
                height: 6.0,
            },
            right_shoulder_frame: PanelRect {
                x: 285.0,
                y: 46.0,
                width: 6.0,
                height: 6.0,
            },
            shoulder_progress: 0.0,
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 80.0,
            },
            card_stack_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 80.0,
            },
            card_stack_content_height: 80.0,
            shell_frame: PanelRect {
                x: 32.0,
                y: 0.0,
                width: 253.0,
                height: 80.0,
            },
            headline_text: "Codex ready".to_string(),
            headline_emphasized: false,
            active_count: "1".to_string(),
            active_count_elapsed_ms: 0,
            total_count: "3".to_string(),
            separator_visibility: 0.0,
            cards_visible: false,
            card_count: 0,
            cards: Vec::new(),
            glow_visible: false,
            glow_opacity: 0.0,
            action_buttons_visible: false,
            action_buttons: Vec::new(),
            completion_count: 1,
            mascot_elapsed_ms: 0,
            mascot_motion_frame: None,
            mascot_pose: SceneMascotPose::Idle,
            mascot_debug_mode_enabled: false,
        }
    }

    fn expanded_paint_job() -> WindowsNativePanelShellPaintJob {
        WindowsNativePanelShellPaintJob {
            display_mode: NativePanelVisualDisplayMode::Expanded,
            window_state: NativePanelHostWindowState {
                frame: Some(PanelRect {
                    x: 100.0,
                    y: 20.0,
                    width: 420.0,
                    height: 180.0,
                }),
                visible: true,
                preferred_display_index: 0,
            },
            panel_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 420.0,
                height: 180.0,
            },
            content_frame: PanelRect {
                x: 0.0,
                y: 0.0,
                width: 420.0,
                height: 180.0,
            },
            card_stack_frame: PanelRect {
                x: 68.5,
                y: 34.0,
                width: 283.0,
                height: 180.0,
            },
            card_stack_content_height: 180.0,
            compact_bar_frame: PanelRect {
                x: 83.5,
                y: 143.0,
                width: 253.0,
                height: 37.0,
            },
            left_shoulder_frame: PanelRect {
                x: 77.5,
                y: 174.0,
                width: 6.0,
                height: 6.0,
            },
            right_shoulder_frame: PanelRect {
                x: 336.5,
                y: 174.0,
                width: 6.0,
                height: 6.0,
            },
            shoulder_progress: 1.0,
            shell_frame: PanelRect {
                x: 68.5,
                y: 34.0,
                width: 283.0,
                height: 146.0,
            },
            surface: ExpandedSurface::Settings,
            headline_text: "Settings".to_string(),
            headline_emphasized: true,
            active_count: "2".to_string(),
            active_count_elapsed_ms: 0,
            total_count: "4".to_string(),
            separator_visibility: 0.8,
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
                    subtitle: Some("#abcdef now".to_string()),
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
            glow_opacity: 0.78,
            action_buttons_visible: true,
            action_buttons: vec![NativePanelVisualActionButtonInput {
                action: NativePanelEdgeAction::Settings,
                frame: PanelRect {
                    x: 300.0,
                    y: 152.0,
                    width: 18.0,
                    height: 18.0,
                },
            }],
            completion_count: 0,
            mascot_elapsed_ms: 0,
            mascot_motion_frame: None,
            mascot_pose: SceneMascotPose::Complete,
            mascot_debug_mode_enabled: false,
        }
    }

    #[test]
    fn plan_only_painter_preserves_text_primitives_for_tests() {
        let mut painter = PlanOnlyWindowsNativePanelPainter;
        let plan = painter.paint(&compact_paint_job()).expect("paint plan");

        assert!(!plan.hidden);
        assert!(
            plan.primitives
                .iter()
                .any(|primitive| matches!(
                    primitive,
                    crate::windows_native_panel::paint_backend::WindowsNativePanelPaintPrimitive::Text { text, .. }
                    if text == "Codex ready"
                ))
        );
    }

    #[test]
    fn direct2d_painter_skeleton_consumes_shared_visual_plan() {
        let mut painter = Direct2DWindowsNativePanelPainter::default();
        let plan = painter.paint(&compact_paint_job()).expect("paint plan");

        assert!(!plan.hidden);
        assert!(!plan.primitives.is_empty());
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            crate::windows_native_panel::paint_backend::WindowsNativePanelPaintPrimitive::CompactShoulder {
                side: crate::native_panel_renderer::facade::visual::NativePanelVisualShoulderSide::Left,
                ..
            }
        )));
        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            crate::windows_native_panel::paint_backend::WindowsNativePanelPaintPrimitive::CompactShoulder {
                side: crate::native_panel_renderer::facade::visual::NativePanelVisualShoulderSide::Right,
                ..
            }
        )));
    }

    #[test]
    fn direct2d_painter_skeleton_routes_compact_text_to_directwrite_requests() {
        let mut job = compact_paint_job();
        job.completion_count = 0;
        let mut painter = Direct2DWindowsNativePanelPainter::default();
        let plan = painter.paint(&job).expect("paint plan");

        let requests = super::directwrite_text_requests_from_paint_plan(&plan);

        assert!(requests.iter().any(|request| request.text == "Codex ready"));
        assert!(requests.iter().any(|request| request.text == "1"));
        assert!(requests.iter().any(|request| request.text == "/"));
        assert!(requests.iter().any(|request| request.text == "3"));
        assert!(requests.iter().any(|request| {
            request.text == "Codex ready"
                && request.weight
                    == crate::native_panel_renderer::facade::visual::NativePanelVisualTextWeight::Semibold
                && request.alignment
                    == crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Center
        }));
        assert!(requests.iter().any(|request| {
            request.text == "1"
                && request.alignment
                    == crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Right
        }));
        assert!(requests.iter().any(|request| {
            request.text == "/"
                && request.alignment
                    == crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Center
        }));
        assert!(requests.iter().any(|request| {
            request.text == "3"
                && request.alignment
                    == crate::native_panel_renderer::facade::visual::NativePanelVisualTextAlignment::Left
        }));
        assert!(requests.iter().all(|request| {
            request.fonts.primary == "Noto Sans SC" && request.fonts.fallback == "Segoe UI Variable"
        }));
    }

    #[test]
    fn direct2d_painter_skeleton_routes_expanded_cards_to_directwrite_requests() {
        let mut painter = Direct2DWindowsNativePanelPainter::default();
        let plan = painter.paint(&expanded_paint_job()).expect("paint plan");
        let requests = super::directwrite_text_requests_from_paint_plan(&plan);

        assert!(plan.primitives.iter().any(|primitive| matches!(
            primitive,
            crate::windows_native_panel::paint_backend::WindowsNativePanelPaintPrimitive::RoundRect {
                frame,
                ..
            } if (frame.width - 283.0).abs() < 0.001
                && (frame.height - 146.0).abs() < 0.001
        )));
        assert!(requests.iter().any(|request| request.text == "Settings"));
        assert!(requests.iter().any(|request| request.text == "Done"));
        assert!(
            requests
                .iter()
                .any(|request| request.text == "Task complete")
        );
        assert!(requests.iter().any(|request| request.text == "Codex"));
    }

    #[cfg(windows)]
    #[test]
    fn direct2d_painter_initializes_native_factories_for_alpha_rendering() {
        let painter = Direct2DWindowsNativePanelPainter::new(None).expect("create painter");

        assert!(painter.is_per_pixel_alpha_ready());
    }

    #[test]
    fn direct2d_resource_cache_reuses_same_physical_rect_and_dpi_key() {
        let key = WindowsDirect2DResourceKey::new(
            crate::windows_native_panel::dpi::WindowsPhysicalRect {
                x: 10,
                y: 20,
                width: 253,
                height: 80,
            },
            crate::windows_native_panel::dpi::WindowsDpiScale::from_scale(1.0),
        );
        let mut cache = WindowsDirect2DResourceCacheState::default();

        assert!(cache.sync(key));
        assert!(!cache.sync(key));
        assert_eq!(cache.rebuild_count(), 1);
    }

    #[test]
    fn direct2d_resource_cache_rebuilds_when_dpi_changes_without_size_change() {
        let rect = crate::windows_native_panel::dpi::WindowsPhysicalRect {
            x: 10,
            y: 20,
            width: 253,
            height: 80,
        };
        let mut cache = WindowsDirect2DResourceCacheState::default();

        assert!(cache.sync(WindowsDirect2DResourceKey::new(
            rect,
            crate::windows_native_panel::dpi::WindowsDpiScale::from_scale(1.0)
        )));
        assert!(cache.sync(WindowsDirect2DResourceKey::new(
            rect,
            crate::windows_native_panel::dpi::WindowsDpiScale::from_scale(1.25)
        )));
        assert_eq!(cache.rebuild_count(), 2);
    }

    #[test]
    fn direct2d_coordinate_space_flips_shared_mac_style_rects_to_windows_top_left() {
        let coordinates = WindowsDirect2DCoordinateSpace::new(80.0);

        assert_eq!(
            coordinates.rect(PanelRect {
                x: 83.5,
                y: 43.0,
                width: 253.0,
                height: 37.0,
            }),
            PanelRect {
                x: 83.5,
                y: 0.0,
                width: 253.0,
                height: 37.0,
            }
        );
    }

    #[test]
    fn direct2d_coordinate_space_flips_text_origin_with_text_height() {
        let coordinates = WindowsDirect2DCoordinateSpace::new(80.0);

        assert_eq!(
            coordinates.text_rect(PanelPoint { x: 139.5, y: 53.5 }, 129.0, 22.0),
            PanelRect {
                x: 139.5,
                y: 4.5,
                width: 129.0,
                height: 22.0,
            }
        );
    }

    #[test]
    fn direct2d_text_box_height_matches_compact_label_metrics() {
        assert_eq!(windows_directwrite_text_box_height("EchoIsland", 13), 24.0);
        assert_eq!(windows_directwrite_text_box_height("1", 15), 24.0);
        assert_eq!(windows_directwrite_text_box_height("2", 8), 16.0);
        assert_eq!(
            windows_directwrite_text_box_height("line one\nline two", 10),
            36.0
        );
    }

    #[test]
    fn compact_shoulder_path_matches_mac_style_corner_curve() {
        let path = WindowsCompactShoulderPath::resolve(
            PanelRect {
                x: 26.0,
                y: 0.0,
                width: 6.0,
                height: 6.0,
            },
            crate::native_panel_renderer::facade::visual::NativePanelVisualShoulderSide::Left,
            0.0,
        )
        .expect("visible shoulder path");

        assert_point_near(path.start, PanelPoint { x: 26.0, y: 0.0 });
        assert_point_near(path.line_to_top_edge, PanelPoint { x: 32.0, y: 0.0 });
        assert_point_near(path.line_to_outer_edge, PanelPoint { x: 32.0, y: 6.0 });
        assert_point_near(path.curve_control_1, PanelPoint { x: 32.0, y: 2.28 });
        assert_point_near(path.curve_control_2, PanelPoint { x: 29.72, y: 0.0 });
        assert_point_near(path.curve_end, PanelPoint { x: 26.0, y: 0.0 });
    }

    #[test]
    fn compact_pill_path_keeps_top_edge_flat_and_rounds_bottom_corners() {
        let path = WindowsCompactPillPath::resolve(
            PanelRect {
                x: 32.0,
                y: 0.0,
                width: 253.0,
                height: 37.0,
            },
            12.5,
        );

        assert_point_near(path.start, PanelPoint { x: 32.0, y: 0.0 });
        assert_point_near(path.top_right, PanelPoint { x: 285.0, y: 0.0 });
        assert_point_near(
            path.right_edge_bottom_arc_start,
            PanelPoint { x: 285.0, y: 24.5 },
        );
        assert_point_near(path.bottom_right_arc_end, PanelPoint { x: 272.5, y: 37.0 });
        assert_point_near(path.bottom_left_arc_start, PanelPoint { x: 44.5, y: 37.0 });
        assert_point_near(path.bottom_left_arc_end, PanelPoint { x: 32.0, y: 24.5 });
    }

    fn assert_point_near(actual: PanelPoint, expected: PanelPoint) {
        assert!(
            (actual.x - expected.x).abs() < 0.001,
            "expected x {} got {}",
            expected.x,
            actual.x
        );
        assert!(
            (actual.y - expected.y).abs() < 0.001,
            "expected y {} got {}",
            expected.y,
            actual.y
        );
    }
}
