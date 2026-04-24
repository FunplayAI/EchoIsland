use echoisland_runtime::RuntimeSnapshot;

use super::{
    panel_refs::native_panel_state, panel_runtime_input::native_panel_runtime_input_descriptor,
    panel_types::NativePanelState,
};
use crate::native_panel_renderer::NativePanelRuntimeInputDescriptor;

pub(super) fn build_native_panel_scene(
    snapshot: &RuntimeSnapshot,
) -> crate::native_panel_scene::PanelScene {
    let input = native_panel_runtime_input_descriptor();
    native_panel_state()
        .and_then(|state| state.lock().ok())
        .map(|guard| build_native_panel_scene_for_state_with_input(&guard, snapshot, &input))
        .unwrap_or_else(|| {
            build_native_panel_scene_for_core_state_with_input(
                snapshot,
                &crate::native_panel_core::PanelState::default(),
                &input,
            )
        })
}

pub(super) fn build_native_panel_scene_for_state_with_input(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelScene {
    let core = state.to_core_panel_state();
    build_native_panel_scene_for_core_state_with_input(snapshot, &core, input)
}

pub(super) fn build_native_panel_scene_for_core_state_with_input(
    snapshot: &RuntimeSnapshot,
    state: &crate::native_panel_core::PanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelScene {
    crate::native_panel_scene::build_panel_scene(state, snapshot, &input.scene_input)
}

pub(super) fn resolve_native_panel_runtime_render_state_with_input(
    snapshot: Option<&RuntimeSnapshot>,
    state: &crate::native_panel_core::PanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelRuntimeRenderState {
    crate::native_panel_scene::resolve_panel_runtime_render_state(
        state,
        snapshot,
        &input.scene_input,
    )
}

pub(super) fn resolve_current_native_panel_runtime_render_state()
-> crate::native_panel_scene::PanelRuntimeRenderState {
    let input = native_panel_runtime_input_descriptor();
    native_panel_state()
        .and_then(|state| state.lock().ok())
        .map(|guard| build_native_panel_runtime_render_state_for_state_with_input(&guard, &input))
        .unwrap_or_default()
}

pub(super) fn resolve_current_native_panel_scene() -> Option<crate::native_panel_scene::PanelScene>
{
    let input = native_panel_runtime_input_descriptor();
    native_panel_state()
        .and_then(|state| state.lock().ok())
        .and_then(|guard| resolve_native_panel_scene_for_state_with_input(&guard, &input))
}

pub(super) fn build_native_panel_runtime_render_state_for_state_with_input(
    state: &NativePanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelRuntimeRenderState {
    let core_state = state.to_core_panel_state();
    resolve_native_panel_runtime_render_state_with_input(
        state.last_snapshot.as_ref(),
        &core_state,
        input,
    )
}

pub(super) fn resolve_native_panel_scene_for_state_with_input(
    state: &NativePanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<crate::native_panel_scene::PanelScene> {
    state.scene_cache.last_scene.clone().or_else(|| {
        let core_state = state.to_core_panel_state();
        state.last_snapshot.as_ref().map(|snapshot| {
            build_native_panel_scene_for_core_state_with_input(snapshot, &core_state, input)
        })
    })
}

#[cfg(test)]
pub(super) fn resolve_native_panel_runtime_render_state_for_state_with_input(
    state: &NativePanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelRuntimeRenderState {
    state
        .scene_cache
        .last_runtime_render_state
        .unwrap_or_else(|| {
            let core_state = state.to_core_panel_state();
            resolve_native_panel_runtime_render_state_with_input(
                state.last_snapshot.as_ref(),
                &core_state,
                input,
            )
        })
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use echoisland_runtime::RuntimeSnapshot;

    use super::{
        build_native_panel_runtime_render_state_for_state_with_input,
        build_native_panel_scene_for_state_with_input,
        resolve_native_panel_runtime_render_state_for_state_with_input,
        resolve_native_panel_scene_for_state_with_input,
    };
    use crate::{
        macos_native_test_panel::{
            mascot::NativeMascotRuntime, panel_types::NativeExpandedSurface,
        },
        native_panel_renderer::{
            NativePanelHostWindowDescriptor, NativePanelRuntimeInputDescriptor,
            NativePanelRuntimeSceneCache,
        },
        native_panel_scene::{PanelRuntimeRenderState, PanelSceneBuildInput, build_panel_scene},
    };

    fn test_snapshot(status: &str) -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: status.to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 0,
            total_session_count: 0,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![],
        }
    }

    fn panel_state() -> crate::macos_native_test_panel::panel_types::NativePanelState {
        crate::macos_native_test_panel::panel_types::NativePanelState {
            expanded: false,
            transitioning: false,
            transition_cards_progress: 0.0,
            transition_cards_entering: false,
            skip_next_close_card_exit: false,
            last_raw_snapshot: None,
            last_snapshot: Some(test_snapshot("idle")),
            scene_cache: NativePanelRuntimeSceneCache::default(),
            status_queue: Vec::new(),
            completion_badge_items: Vec::new(),
            pending_permission_card: None,
            pending_question_card: None,
            status_auto_expanded: false,
            surface_mode: NativeExpandedSurface::Default,
            shared_body_height: None,
            host_window_descriptor: NativePanelHostWindowDescriptor::default(),
            pointer_inside_since: None,
            pointer_outside_since: None,
            primary_mouse_down: false,
            last_focus_click: None,
            pointer_regions: Vec::new(),
            mascot_runtime: NativeMascotRuntime::new(Instant::now()),
        }
    }

    fn runtime_input_descriptor() -> NativePanelRuntimeInputDescriptor {
        NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        }
    }

    #[test]
    fn current_scene_resolution_prefers_shared_cache() {
        let mut state = panel_state();
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        state.scene_cache.last_scene = Some(cached_scene.clone());

        let input = runtime_input_descriptor();
        let resolved =
            resolve_native_panel_scene_for_state_with_input(&state, &input).expect("cached scene");

        assert_eq!(
            resolved.compact_bar.headline.text,
            cached_scene.compact_bar.headline.text
        );
    }

    #[test]
    fn current_runtime_state_resolution_prefers_shared_cache() {
        let mut state = panel_state();
        state.scene_cache.last_runtime_render_state = Some(PanelRuntimeRenderState {
            transitioning: true,
            ..PanelRuntimeRenderState::default()
        });

        let input = runtime_input_descriptor();
        let resolved =
            resolve_native_panel_runtime_render_state_for_state_with_input(&state, &input);

        assert!(resolved.transitioning);
    }

    #[test]
    fn visual_scene_build_ignores_stale_shared_cache() {
        let mut state = panel_state();
        state.expanded = true;
        state.surface_mode = NativeExpandedSurface::Settings;
        state.scene_cache.last_scene = Some(build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        ));

        let input = runtime_input_descriptor();
        let resolved = build_native_panel_scene_for_state_with_input(
            &state,
            state.last_snapshot.as_ref().expect("snapshot"),
            &input,
        );

        assert_eq!(
            resolved.surface,
            crate::native_panel_core::ExpandedSurface::Settings
        );
        assert!(matches!(
            resolved.cards.first(),
            Some(crate::native_panel_scene::SceneCard::Settings { .. })
        ));
    }

    #[test]
    fn visual_runtime_state_build_ignores_stale_shared_cache() {
        let mut state = panel_state();
        state.expanded = true;
        state.scene_cache.last_runtime_render_state = Some(PanelRuntimeRenderState {
            transitioning: true,
            shell_scene: crate::native_panel_scene::PanelShellSceneState {
                edge_actions_visible: false,
                ..crate::native_panel_scene::PanelShellSceneState::default()
            },
        });

        let input = runtime_input_descriptor();
        let resolved = build_native_panel_runtime_render_state_for_state_with_input(&state, &input);

        assert!(!resolved.transitioning);
        assert!(resolved.shell_scene.edge_actions_visible);
    }

    #[test]
    fn explicit_scene_build_reuses_shared_cache_for_current_snapshot() {
        let mut state = panel_state();
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        let current_snapshot = test_snapshot("idle");
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_scene = Some(cached_scene.clone());

        let resolved = if state.last_snapshot.as_ref() == Some(&current_snapshot) {
            let input = runtime_input_descriptor();
            resolve_native_panel_scene_for_state_with_input(&state, &input).expect("cached scene")
        } else {
            unreachable!("current snapshot should match state snapshot");
        };

        assert_eq!(
            resolved.compact_bar.headline.text,
            cached_scene.compact_bar.headline.text
        );
    }
}
