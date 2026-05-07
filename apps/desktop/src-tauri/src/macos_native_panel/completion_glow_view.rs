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
use crate::native_panel_renderer::facade::presentation::{
    COMPLETION_GLOW_IMAGE_HEIGHT, COMPLETION_GLOW_IMAGE_WIDTH, CompletionGlowImageSliceSpec,
    resolve_completion_glow_image_slices,
};

const COMPLETION_GLOW_IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/island-completion-inner-glow-9slice.png"
));
const COMPLETION_GLOW_IMAGE_FILE_NAME: &str = "echoisland-island-completion-inner-glow.png";
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
    let slices = resolve_completion_glow_image_slices(panel_rect_from_ns_size(bar_size));
    layout_completion_glow_slice_layers(&left_layer, &center_layer, &right_layer, &slices);
}

pub(super) fn update_completion_glow_layout_from_slices(
    completion_glow: &NSView,
    slices: &[CompletionGlowImageSliceSpec; 3],
) {
    let Some(root_layer) = completion_glow.layer() else {
        return;
    };
    let Some(sublayers) = (unsafe { root_layer.sublayers() }) else {
        return;
    };
    if sublayers.len() < 3 {
        return;
    }
    let left_layer = sublayers.objectAtIndex(0);
    let center_layer = sublayers.objectAtIndex(1);
    let right_layer = sublayers.objectAtIndex(2);
    layout_completion_glow_slice_layers(&left_layer, &center_layer, &right_layer, slices);
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

    let initial_slices = resolve_completion_glow_image_slices(panel_rect_from_ns_size(pill_size));
    let left_layer = create_completion_glow_slice_layer(
        AsRef::<AnyObject>::as_ref(&cg_image),
        COMPLETION_GLOW_LEFT_LAYER_NAME,
        initial_slices[0].source,
    );
    let center_layer = create_completion_glow_slice_layer(
        AsRef::<AnyObject>::as_ref(&cg_image),
        COMPLETION_GLOW_CENTER_LAYER_NAME,
        initial_slices[1].source,
    );
    let right_layer = create_completion_glow_slice_layer(
        AsRef::<AnyObject>::as_ref(&cg_image),
        COMPLETION_GLOW_RIGHT_LAYER_NAME,
        initial_slices[2].source,
    );
    layout_completion_glow_slice_layers(&left_layer, &center_layer, &right_layer, &initial_slices);
    root_layer.addSublayer(&left_layer);
    root_layer.addSublayer(&center_layer);
    root_layer.addSublayer(&right_layer);
}

fn create_completion_glow_slice_layer(
    cg_image: &AnyObject,
    name: &str,
    source: crate::native_panel_core::PanelRect,
) -> Retained<CALayer> {
    let layer = CALayer::layer();
    layer.setName(Some(&NSString::from_str(name)));
    unsafe {
        layer.setContents(Some(cg_image));
        layer.setContentsGravity(kCAGravityResize);
    }
    set_completion_glow_contents_rect(&layer, source);
    layer
}

fn layout_completion_glow_slice_layers(
    left_layer: &CALayer,
    center_layer: &CALayer,
    right_layer: &CALayer,
    slices: &[CompletionGlowImageSliceSpec; 3],
) {
    apply_completion_glow_slice_layout(left_layer, slices[0]);
    apply_completion_glow_slice_layout(center_layer, slices[1]);
    apply_completion_glow_slice_layout(right_layer, slices[2]);
}

fn apply_completion_glow_slice_layout(layer: &CALayer, slice: CompletionGlowImageSliceSpec) {
    layer.setFrame(NSRect::new(
        NSPoint::new(slice.dest.x, slice.dest.y),
        NSSize::new(slice.dest.width, slice.dest.height),
    ));
    set_completion_glow_contents_rect(layer, slice.source);
}

fn set_completion_glow_contents_rect(layer: &CALayer, source: crate::native_panel_core::PanelRect) {
    layer.setContentsRect(NSRect::new(
        NSPoint::new(
            source.x / COMPLETION_GLOW_IMAGE_WIDTH,
            source.y / COMPLETION_GLOW_IMAGE_HEIGHT,
        ),
        NSSize::new(
            source.width / COMPLETION_GLOW_IMAGE_WIDTH,
            source.height / COMPLETION_GLOW_IMAGE_HEIGHT,
        ),
    ));
}

fn panel_rect_from_ns_size(size: NSSize) -> crate::native_panel_core::PanelRect {
    crate::native_panel_core::PanelRect {
        x: 0.0,
        y: 0.0,
        width: size.width.max(0.0),
        height: size.height.max(0.0),
    }
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
