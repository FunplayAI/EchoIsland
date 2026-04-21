use super::*;

pub(super) fn build_native_panel_scene(
    snapshot: &RuntimeSnapshot,
) -> crate::native_panel_scene::PanelScene {
    let core = native_panel_state()
        .and_then(|state| state.lock().ok().map(|state| state.to_core_panel_state()))
        .unwrap_or_default();
    build_native_panel_scene_for_core_state(snapshot, &core)
}

pub(super) fn build_native_panel_scene_for_core_state(
    snapshot: &RuntimeSnapshot,
    state: &crate::native_panel_core::PanelState,
) -> crate::native_panel_scene::PanelScene {
    let input = native_panel_scene_build_input();
    crate::native_panel_scene::build_panel_scene(state, snapshot, &input)
}

pub(super) fn resolve_native_panel_runtime_render_state(
    snapshot: Option<&RuntimeSnapshot>,
    state: &crate::native_panel_core::PanelState,
) -> crate::native_panel_scene::PanelRuntimeRenderState {
    let input = native_panel_scene_build_input();
    crate::native_panel_scene::resolve_panel_runtime_render_state(state, snapshot, &input)
}

pub(super) fn resolve_current_native_panel_runtime_render_state()
-> crate::native_panel_scene::PanelRuntimeRenderState {
    native_panel_state()
        .and_then(|state| state.lock().ok())
        .map(|guard| {
            let core_state = guard.to_core_panel_state();
            resolve_native_panel_runtime_render_state(guard.last_snapshot.as_ref(), &core_state)
        })
        .unwrap_or_default()
}

fn native_panel_scene_build_input() -> crate::native_panel_scene::PanelSceneBuildInput {
    crate::native_panel_scene::PanelSceneBuildInput {
        display_count: scene_display_count(),
        settings: scene_settings_state(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn scene_settings_state() -> crate::native_panel_core::PanelSettingsState {
    let settings = crate::app_settings::current_app_settings();
    crate::native_panel_core::PanelSettingsState {
        selected_display_index: settings.preferred_display_index,
        completion_sound_enabled: settings.completion_sound_enabled,
        mascot_enabled: settings.mascot_enabled,
    }
}

fn scene_display_count() -> usize {
    let Some(mtm) = MainThreadMarker::new() else {
        return 1;
    };
    NSScreen::screens(mtm).len()
}
