use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use objc2::MainThreadMarker;
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2::{AnyThread, runtime::AnyObject};
use objc2_app_kit::{NSColor, NSImage, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use objc2_quartz_core::{CALayer, kCAGravityResize};
use tracing::warn;

use super::panel_constants::COMPACT_PILL_RADIUS;

const COMPLETION_GLOW_IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../web/assets/island-completion-inner-glow-9slice.png"
));
const COMPLETION_GLOW_IMAGE_FILE_NAME: &str = "echoisland-island-completion-inner-glow.png";
const COMPLETION_GLOW_IMAGE_WIDTH: f64 = 480.0;
const COMPLETION_GLOW_IMAGE_HEIGHT: f64 = 160.0;
const COMPLETION_GLOW_IMAGE_RADIUS: f64 = 64.0;
const COMPLETION_GLOW_SLICE_LEFT: f64 = 160.0;
const COMPLETION_GLOW_SLICE_RIGHT: f64 = 160.0;
const COMPLETION_GLOW_LEFT_LAYER_NAME: &str = "completion-glow-left";
const COMPLETION_GLOW_CENTER_LAYER_NAME: &str = "completion-glow-center";
const COMPLETION_GLOW_RIGHT_LAYER_NAME: &str = "completion-glow-right";

static EMBEDDED_COMPLETION_GLOW_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

pub(super) fn create_completion_glow(mtm: MainThreadMarker, pill_size: NSSize) -> Retained<NSView> {
    let glow = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(pill_size.width.max(0.0), pill_size.height.max(0.0)),
        ),
    );
    glow.setWantsLayer(true);
    let layer = CALayer::layer();
    layer.setCornerRadius(COMPACT_PILL_RADIUS);
    layer.setMasksToBounds(true);
    layer.setBackgroundColor(Some(&NSColor::clearColor().CGColor()));
    layer.setBorderWidth(0.0);
    apply_completion_glow_image(&layer, pill_size);
    glow.setLayer(Some(&layer));
    glow.setHidden(true);
    glow.setAlphaValue(0.0);
    glow
}

pub(super) fn update_completion_glow_layout(completion_glow: &NSView, bar_size: NSSize) {
    let Some(root_layer) = completion_glow.layer() else {
        return;
    };
    root_layer.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), bar_size));

    let Some(sublayers) = (unsafe { root_layer.sublayers() }) else {
        return;
    };
    if sublayers.len() < 3 {
        return;
    }
    let left_layer = sublayers.objectAtIndex(0);
    let center_layer = sublayers.objectAtIndex(1);
    let right_layer = sublayers.objectAtIndex(2);
    layout_completion_glow_slice_layers(&left_layer, &center_layer, &right_layer, bar_size);
}

fn apply_completion_glow_image(root_layer: &CALayer, pill_size: NSSize) {
    let Some(image_path) = ensure_embedded_completion_glow_file() else {
        return;
    };
    let image_path = NSString::from_str(&image_path.to_string_lossy());
    let Some(image) = NSImage::initWithContentsOfFile(NSImage::alloc(), &image_path) else {
        warn!("failed to load completion glow image");
        return;
    };

    let mut proposed_rect = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(COMPLETION_GLOW_IMAGE_WIDTH, COMPLETION_GLOW_IMAGE_HEIGHT),
    );
    let Some(cg_image) =
        (unsafe { image.CGImageForProposedRect_context_hints(&mut proposed_rect, None, None) })
    else {
        warn!("failed to resolve completion glow CGImage");
        return;
    };

    let left_layer = create_completion_glow_slice_layer(
        AsRef::<AnyObject>::as_ref(&cg_image),
        COMPLETION_GLOW_LEFT_LAYER_NAME,
        0.0,
        COMPLETION_GLOW_SLICE_LEFT / COMPLETION_GLOW_IMAGE_WIDTH,
    );
    let center_layer = create_completion_glow_slice_layer(
        AsRef::<AnyObject>::as_ref(&cg_image),
        COMPLETION_GLOW_CENTER_LAYER_NAME,
        COMPLETION_GLOW_SLICE_LEFT / COMPLETION_GLOW_IMAGE_WIDTH,
        (COMPLETION_GLOW_IMAGE_WIDTH - COMPLETION_GLOW_SLICE_LEFT - COMPLETION_GLOW_SLICE_RIGHT)
            / COMPLETION_GLOW_IMAGE_WIDTH,
    );
    let right_layer = create_completion_glow_slice_layer(
        AsRef::<AnyObject>::as_ref(&cg_image),
        COMPLETION_GLOW_RIGHT_LAYER_NAME,
        (COMPLETION_GLOW_IMAGE_WIDTH - COMPLETION_GLOW_SLICE_RIGHT) / COMPLETION_GLOW_IMAGE_WIDTH,
        COMPLETION_GLOW_SLICE_RIGHT / COMPLETION_GLOW_IMAGE_WIDTH,
    );
    layout_completion_glow_slice_layers(&left_layer, &center_layer, &right_layer, pill_size);
    root_layer.addSublayer(&left_layer);
    root_layer.addSublayer(&center_layer);
    root_layer.addSublayer(&right_layer);
}

fn create_completion_glow_slice_layer(
    cg_image: &AnyObject,
    name: &str,
    rect_x: f64,
    rect_width: f64,
) -> Retained<CALayer> {
    let layer = CALayer::layer();
    layer.setName(Some(&NSString::from_str(name)));
    unsafe {
        layer.setContents(Some(cg_image));
        layer.setContentsGravity(kCAGravityResize);
    }
    layer.setContentsRect(NSRect::new(
        NSPoint::new(rect_x, 0.0),
        NSSize::new(rect_width, 1.0),
    ));
    layer
}

fn layout_completion_glow_slice_layers(
    left_layer: &CALayer,
    center_layer: &CALayer,
    right_layer: &CALayer,
    bar_size: NSSize,
) {
    let bar_width = bar_size.width.max(0.0);
    let display_scale = (COMPACT_PILL_RADIUS / COMPLETION_GLOW_IMAGE_RADIUS).max(0.0);
    let display_height = (COMPLETION_GLOW_IMAGE_HEIGHT * display_scale)
        .min(bar_size.height)
        .max(0.0);
    let mut left_width = COMPLETION_GLOW_SLICE_LEFT * display_scale;
    let mut right_width = COMPLETION_GLOW_SLICE_RIGHT * display_scale;

    let total_cap_width = left_width + right_width;
    if total_cap_width > bar_width && total_cap_width > 0.0 {
        let shrink = bar_width / total_cap_width;
        left_width *= shrink;
        right_width *= shrink;
    }

    let center_width = (bar_width - left_width - right_width).max(0.0);
    left_layer.setFrame(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(left_width, display_height),
    ));
    center_layer.setFrame(NSRect::new(
        NSPoint::new(left_width, 0.0),
        NSSize::new(center_width, display_height),
    ));
    right_layer.setFrame(NSRect::new(
        NSPoint::new((bar_width - right_width).max(0.0), 0.0),
        NSSize::new(right_width, display_height),
    ));
}

fn ensure_embedded_completion_glow_file() -> Option<&'static Path> {
    EMBEDDED_COMPLETION_GLOW_PATH
        .get_or_init(|| {
            let path = std::env::temp_dir().join(COMPLETION_GLOW_IMAGE_FILE_NAME);
            match fs::write(&path, COMPLETION_GLOW_IMAGE_BYTES) {
                Ok(()) => Some(path),
                Err(error) => {
                    warn!(error = %error, "failed to write embedded completion glow image");
                    None
                }
            }
        })
        .as_deref()
}
