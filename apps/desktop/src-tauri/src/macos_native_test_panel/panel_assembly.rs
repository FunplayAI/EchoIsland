use super::*;

pub(super) struct NativePanelAssemblyViews<'a> {
    pub(super) content_view: &'a NSView,
    pub(super) left_shoulder: &'a NSView,
    pub(super) right_shoulder: &'a NSView,
    pub(super) pill_view: &'a NSView,
    pub(super) expanded_container: &'a NSView,
    pub(super) completion_glow: &'a NSView,
    pub(super) top_highlight: &'a NSView,
    pub(super) body_separator: &'a NSView,
    pub(super) settings_button: &'a NSView,
    pub(super) quit_button: &'a NSView,
    pub(super) mascot_shell: &'a NSView,
    pub(super) headline: &'a NSTextField,
    pub(super) active_count_clip: &'a NSClipView,
    pub(super) slash: &'a NSTextField,
    pub(super) total_count: &'a NSTextField,
}

pub(super) fn assemble_native_panel_views(views: NativePanelAssemblyViews<'_>) {
    views.pill_view.addSubview(views.completion_glow);
    views.pill_view.addSubview(views.top_highlight);
    views.pill_view.addSubview(views.mascot_shell);
    views.pill_view.addSubview(views.headline);
    views.pill_view.addSubview(views.active_count_clip);
    views.pill_view.addSubview(views.slash);
    views.pill_view.addSubview(views.total_count);
    views.pill_view.addSubview(views.settings_button);
    views.pill_view.addSubview(views.quit_button);
    views.content_view.addSubview(views.expanded_container);
    views.expanded_container.addSubview(views.body_separator);
    views.content_view.addSubview(views.left_shoulder);
    views.content_view.addSubview(views.right_shoulder);
    views.content_view.addSubview(views.pill_view);
}
