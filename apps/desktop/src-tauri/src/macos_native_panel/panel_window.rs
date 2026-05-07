use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSPanel, NSView, NSWindowAnimationBehavior,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::NSRect;

pub(super) fn create_native_panel_window(
    mtm: MainThreadMarker,
    frame: NSRect,
) -> Retained<NSPanel> {
    let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
    NSPanel::initWithContentRect_styleMask_backing_defer(
        NSPanel::alloc(mtm),
        frame,
        style,
        NSBackingStoreType::Buffered,
        false,
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn configure_native_panel_window(
    panel: &NSPanel,
    content_view: &NSView,
    frame: NSRect,
) {
    unsafe {
        panel.setReleasedWhenClosed(false);
    }
    panel.setFloatingPanel(true);
    panel.setBecomesKeyOnlyIfNeeded(false);
    panel.setWorksWhenModal(true);
    panel.setLevel(26);
    panel.setBackgroundColor(Some(&NSColor::clearColor()));
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
    panel.setContentView(Some(content_view));
    panel.setFrame_display(frame, true);
    panel.orderFront(None);
    panel.displayIfNeeded();
}
