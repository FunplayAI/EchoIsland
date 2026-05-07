use objc2_foundation::{NSRect, NSSize};

use super::panel_geometry::absolute_rect;

pub(super) fn native_hover_pill_rect(panel_frame: NSRect, pill_frame: NSRect) -> NSRect {
    let top_gap =
        (panel_frame.size.height - (pill_frame.origin.y + pill_frame.size.height)).max(0.0);
    absolute_rect(
        panel_frame,
        NSRect::new(
            pill_frame.origin,
            NSSize::new(pill_frame.size.width, pill_frame.size.height + top_gap),
        ),
    )
}
