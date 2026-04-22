use crate::native_panel_scene::PanelScene;
use echoisland_runtime::RuntimeSnapshot;
use objc2_foundation::NSString;
use objc2_quartz_core::CATransaction;

use super::compact_bar::compact_style_for_scene;
use super::compact_bar_layout::{compact_headline_should_hide, sync_active_count_marquee};
use super::panel_globals::ACTIVE_COUNT_SCROLL_TEXT;
use super::panel_helpers::ns_color;
use super::panel_refs::{NativePanelRefs, resolve_native_panel_refs};
use super::panel_scene_adapter::build_native_panel_scene;
use super::panel_types::NativePanelHandles;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn apply_snapshot_values_to_panel(
    handles: NativePanelHandles,
    snapshot: &RuntimeSnapshot,
) {
    let refs = resolve_native_panel_refs(handles);
    let scene = build_native_panel_scene(snapshot);
    apply_scene_to_panel_refs(&refs, &scene);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn apply_scene_to_panel_refs(refs: &NativePanelRefs, scene: &PanelScene) {
    let headline = refs.headline;
    let active_count = refs.active_count;
    let active_count_next = refs.active_count_next;
    let total_count = refs.total_count;

    let headline_value = NSString::from_str(&scene.compact_bar.headline.text);
    let active_count_text = scene.compact_bar.active_count.clone();
    let total_count_text = scene.compact_bar.total_count.clone();
    let active_count_value = NSString::from_str(&active_count_text);
    let total_count_value = NSString::from_str(&total_count_text);
    let style = compact_style_for_scene(scene);
    let headline_color = ns_color(style.headline_color);
    let active_count_color = ns_color(style.active_count_color);
    let total_count_color = ns_color(style.total_count_color);

    headline.setStringValue(&headline_value);
    headline.setTextColor(Some(&headline_color));
    headline.setHidden(compact_headline_should_hide(refs));
    active_count.setTextColor(Some(&active_count_color));
    active_count_next.setTextColor(Some(&active_count_color));
    total_count.setStringValue(&total_count_value);
    total_count.setTextColor(Some(&total_count_color));
    if let Some(source) = ACTIVE_COUNT_SCROLL_TEXT.get() {
        if let Ok(mut value) = source.lock() {
            *value = active_count_text;
        }
    }
    active_count.setStringValue(&active_count_value);
    sync_active_count_marquee(refs);

    headline.displayIfNeeded();
    refs.active_count_clip.displayIfNeeded();
    active_count.displayIfNeeded();
    active_count_next.displayIfNeeded();
    total_count.displayIfNeeded();
}

pub(super) fn with_disabled_layer_actions<T>(f: impl FnOnce() -> T) -> T {
    CATransaction::begin();
    CATransaction::setDisableActions(true);
    let result = f();
    CATransaction::commit();
    result
}
