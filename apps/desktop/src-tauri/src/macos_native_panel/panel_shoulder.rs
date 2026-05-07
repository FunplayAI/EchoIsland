use objc2_app_kit::{NSColor, NSView};
use objc2_core_graphics::{CGAffineTransformMakeScale, CGMutablePath, CGPath};
use objc2_foundation::NSRect;
use objc2_quartz_core::CAShapeLayer;

use super::panel_constants::{COMPACT_SHOULDER_SIZE, SHOULDER_CURVE_FACTOR};

pub(super) fn apply_shoulder_layer(view: &NSView, background: &NSColor, align_right: bool) {
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

pub(super) fn apply_shoulder_path_scale_x(
    shoulder: &NSView,
    frame: NSRect,
    progress: f64,
    anchor_on_right: bool,
) {
    shoulder.setFrame(frame);
    shoulder.setAlphaValue(1.0);

    let scale_x = (1.0 - progress.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    shoulder.setHidden(scale_x <= 0.001);

    let Some(layer) = shoulder.layer() else {
        return;
    };
    let Ok(shape_layer) = layer.downcast::<CAShapeLayer>() else {
        return;
    };

    shape_layer.setAffineTransform(CGAffineTransformMakeScale(1.0, 1.0));
    set_shoulder_path_for_scale_x(&shape_layer, scale_x, anchor_on_right);
}

fn set_shoulder_path_for_scale_x(shape_layer: &CAShapeLayer, scale_x: f64, anchor_on_right: bool) {
    let size = COMPACT_SHOULDER_SIZE;
    let control = size * SHOULDER_CURVE_FACTOR;
    let scale_x = scale_x.clamp(0.0, 1.0);
    let path = CGMutablePath::new();

    unsafe {
        if anchor_on_right {
            let left = size * (1.0 - scale_x);
            let right = size;
            let control_x = size - ((size - control) * scale_x);
            CGMutablePath::move_to_point(Some(path.as_ref()), std::ptr::null(), left, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), right, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), right, 0.0);
            CGMutablePath::add_curve_to_point(
                Some(path.as_ref()),
                std::ptr::null(),
                right,
                control,
                control_x,
                size,
                left,
                size,
            );
            CGMutablePath::close_subpath(Some(path.as_ref()));
        } else {
            let left = 0.0;
            let right = size * scale_x;
            let control_x = control * scale_x;
            CGMutablePath::move_to_point(Some(path.as_ref()), std::ptr::null(), right, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), left, size);
            CGMutablePath::add_line_to_point(Some(path.as_ref()), std::ptr::null(), left, 0.0);
            CGMutablePath::add_curve_to_point(
                Some(path.as_ref()),
                std::ptr::null(),
                left,
                control,
                control_x,
                size,
                right,
                size,
            );
            CGMutablePath::close_subpath(Some(path.as_ref()));
        }
    }

    if let Some(immutable_path) = CGPath::new_copy(Some(path.as_ref())) {
        shape_layer.setPath(Some(immutable_path.as_ref()));
    }
}
