use super::*;

#[derive(Clone, Copy)]
pub(super) struct NativePanelRefs {
    pub(super) panel: &'static NSPanel,
    pub(super) content_view: &'static NSView,
    pub(super) left_shoulder: &'static NSView,
    pub(super) right_shoulder: &'static NSView,
    pub(super) pill_view: &'static NSView,
    pub(super) expanded_container: &'static NSView,
    pub(super) cards_container: &'static NSView,
    pub(super) top_highlight: &'static NSView,
    pub(super) body_separator: &'static NSView,
    pub(super) settings_button: &'static NSView,
    pub(super) quit_button: &'static NSView,
    pub(super) mascot_shell: &'static NSView,
    pub(super) mascot_body: &'static NSView,
    pub(super) mascot_left_eye: &'static NSView,
    pub(super) mascot_right_eye: &'static NSView,
    pub(super) mascot_mouth: &'static NSView,
    pub(super) mascot_bubble: &'static NSView,
    pub(super) mascot_sleep_label: &'static NSTextField,
    pub(super) mascot_completion_badge: &'static NSView,
    pub(super) mascot_completion_badge_label: &'static NSTextField,
    pub(super) headline: &'static NSTextField,
    pub(super) active_count_clip: &'static NSClipView,
    pub(super) active_count: &'static NSTextField,
    pub(super) active_count_next: &'static NSTextField,
    pub(super) slash: &'static NSTextField,
    pub(super) total_count: &'static NSTextField,
}

pub(super) fn native_panel_handles() -> Option<NativePanelHandles> {
    NATIVE_TEST_PANEL_HANDLES.get().copied()
}

pub(super) fn native_panel_state() -> Option<&'static Mutex<NativePanelState>> {
    NATIVE_TEST_PANEL_STATE.get()
}

pub(super) unsafe fn panel_from_ptr(ptr: usize) -> &'static NSPanel {
    unsafe { &*(ptr as *const NSPanel) }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn resolve_native_panel_refs(handles: NativePanelHandles) -> NativePanelRefs {
    NativePanelRefs {
        panel: panel_from_ptr(handles.panel),
        content_view: view_from_ptr(handles.content_view),
        left_shoulder: view_from_ptr(handles.left_shoulder),
        right_shoulder: view_from_ptr(handles.right_shoulder),
        pill_view: view_from_ptr(handles.pill_view),
        expanded_container: view_from_ptr(handles.expanded_container),
        cards_container: view_from_ptr(handles.cards_container),
        top_highlight: view_from_ptr(handles.top_highlight),
        body_separator: view_from_ptr(handles.body_separator),
        settings_button: view_from_ptr(handles.settings_button),
        quit_button: view_from_ptr(handles.quit_button),
        mascot_shell: view_from_ptr(handles.mascot_shell),
        mascot_body: view_from_ptr(handles.mascot_body),
        mascot_left_eye: view_from_ptr(handles.mascot_left_eye),
        mascot_right_eye: view_from_ptr(handles.mascot_right_eye),
        mascot_mouth: view_from_ptr(handles.mascot_mouth),
        mascot_bubble: view_from_ptr(handles.mascot_bubble),
        mascot_sleep_label: text_field_from_ptr(handles.mascot_sleep_label),
        mascot_completion_badge: view_from_ptr(handles.mascot_completion_badge),
        mascot_completion_badge_label: text_field_from_ptr(handles.mascot_completion_badge_label),
        headline: text_field_from_ptr(handles.headline),
        active_count_clip: clip_view_from_ptr(handles.active_count_clip),
        active_count: text_field_from_ptr(handles.active_count),
        active_count_next: text_field_from_ptr(handles.active_count_next),
        slash: text_field_from_ptr(handles.slash),
        total_count: text_field_from_ptr(handles.total_count),
    }
}

pub(super) unsafe fn text_field_from_ptr(ptr: usize) -> &'static NSTextField {
    unsafe { &*(ptr as *const NSTextField) }
}

pub(super) unsafe fn clip_view_from_ptr(ptr: usize) -> &'static NSClipView {
    unsafe { &*(ptr as *const NSClipView) }
}

pub(super) unsafe fn view_from_ptr(ptr: usize) -> &'static NSView {
    unsafe { &*(ptr as *const NSView) }
}
