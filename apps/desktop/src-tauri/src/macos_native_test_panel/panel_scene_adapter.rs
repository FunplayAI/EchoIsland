use echoisland_runtime::RuntimeSnapshot;

use super::{
    card_metrics::estimated_scene_content_height, panel_constants::EXPANDED_MAX_BODY_HEIGHT,
    panel_refs::native_panel_state, panel_runtime_input::native_panel_runtime_input_descriptor,
    panel_types::NativePanelState,
};
use crate::{
    native_panel_core::PanelRect,
    native_panel_renderer::{
        NativePanelCardStackCommand, NativePanelCompactBarCommand,
        NativePanelRuntimeInputDescriptor, NativePanelRuntimeSceneCacheKey,
        cached_render_command_bundle_for_key, cached_scene_for_key,
        native_panel_card_stack_command, native_panel_compact_bar_command,
        native_panel_runtime_scene_cache_key,
    },
    native_panel_scene::PanelScene,
};

#[derive(Clone, Debug)]
pub(super) struct NativePanelSnapshotRenderPlan {
    pub(super) scene: PanelScene,
    pub(super) expanded_content_height: f64,
    pub(super) expanded_body_height: f64,
    render_command_bundle: Option<crate::native_panel_renderer::NativePanelRenderCommandBundle>,
}

impl NativePanelSnapshotRenderPlan {
    pub(super) fn compact_bar_command(&self, frame: PanelRect) -> NativePanelCompactBarCommand {
        if let Some(bundle) = self.render_command_bundle.clone() {
            let mut command = bundle.compact_bar;
            command.frame = frame;
            return command;
        }

        native_panel_compact_bar_command(&self.scene, frame)
    }

    pub(super) fn card_stack_command(
        &self,
        frame: PanelRect,
        visible: bool,
    ) -> NativePanelCardStackCommand {
        if let Some(bundle) = self.render_command_bundle.clone() {
            let mut command = bundle.card_stack;
            command.frame = frame;
            command.visible = visible;
            return command;
        }

        native_panel_card_stack_command(&self.scene, frame, visible)
    }
}

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

pub(super) fn resolve_or_build_native_panel_scene(
    snapshot: &RuntimeSnapshot,
) -> crate::native_panel_scene::PanelScene {
    resolve_current_native_panel_scene_for_snapshot(snapshot)
        .unwrap_or_else(|| build_native_panel_scene(snapshot))
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

pub(super) fn resolve_current_native_panel_scene_for_snapshot(
    snapshot: &RuntimeSnapshot,
) -> Option<crate::native_panel_scene::PanelScene> {
    let input = native_panel_runtime_input_descriptor();
    native_panel_state()
        .and_then(|state| state.lock().ok())
        .and_then(|guard| {
            resolve_native_panel_scene_for_state_and_snapshot(&guard, snapshot, &input)
        })
}

pub(super) fn resolve_current_native_panel_render_command_bundle_for_snapshot(
    snapshot: &RuntimeSnapshot,
) -> Option<crate::native_panel_renderer::NativePanelRenderCommandBundle> {
    native_panel_state()
        .and_then(|state| state.lock().ok())
        .and_then(|guard| {
            resolve_native_panel_render_command_bundle_for_state_and_snapshot(&guard, snapshot)
        })
}

pub(super) fn resolve_current_native_panel_render_command_bundle(
    state: &NativePanelState,
) -> Option<crate::native_panel_renderer::NativePanelRenderCommandBundle> {
    state.last_snapshot.as_ref().and_then(|snapshot| {
        resolve_native_panel_render_command_bundle_for_state_and_snapshot(state, snapshot)
    })
}

pub(super) fn cache_native_panel_render_command_bundle_in_state(
    state: &mut NativePanelState,
    bundle: &crate::native_panel_renderer::NativePanelRenderCommandBundle,
) {
    let input = native_panel_runtime_input_descriptor();
    let cache_key = native_panel_runtime_scene_cache_key(&state.to_core_panel_state(), &input);
    crate::native_panel_renderer::cache_render_command_bundle_with_key(
        &mut state.scene_cache,
        Some(cache_key),
        bundle,
    );
    state.pointer_regions = bundle.pointer_regions.clone();
}

pub(super) fn resolve_snapshot_render_plan(
    snapshot: &RuntimeSnapshot,
) -> NativePanelSnapshotRenderPlan {
    let render_command_bundle =
        resolve_current_native_panel_render_command_bundle_for_snapshot(snapshot);
    let scene = render_command_bundle
        .as_ref()
        .map(|bundle| bundle.scene.clone())
        .unwrap_or_else(|| resolve_or_build_native_panel_scene(snapshot));
    render_plan_from_scene_and_bundle(scene, render_command_bundle)
}

pub(super) fn resolve_snapshot_compact_bar_command(
    snapshot: &RuntimeSnapshot,
    frame: PanelRect,
) -> NativePanelCompactBarCommand {
    resolve_snapshot_render_plan(snapshot).compact_bar_command(frame)
}

#[cfg(test)]
pub(super) fn resolve_snapshot_card_stack_command(
    snapshot: &RuntimeSnapshot,
    frame: PanelRect,
    visible: bool,
) -> NativePanelCardStackCommand {
    resolve_snapshot_render_plan(snapshot).card_stack_command(frame, visible)
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

fn scene_cache_key_for_state(
    state: &NativePanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelRuntimeSceneCacheKey {
    native_panel_runtime_scene_cache_key(&state.to_core_panel_state(), input)
}

fn snapshot_matches_current_state_or_cache(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> bool {
    state.last_snapshot.as_ref() == Some(snapshot)
        || state.scene_cache.last_snapshot.as_ref() == Some(snapshot)
}

fn build_scene_for_state_snapshot_with_input(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelScene {
    build_native_panel_scene_for_core_state_with_input(
        snapshot,
        &state.to_core_panel_state(),
        input,
    )
}

fn render_plan_from_scene_and_bundle(
    scene: PanelScene,
    render_command_bundle: Option<crate::native_panel_renderer::NativePanelRenderCommandBundle>,
) -> NativePanelSnapshotRenderPlan {
    let expanded_content_height = estimated_scene_content_height(&scene);

    NativePanelSnapshotRenderPlan {
        scene,
        expanded_content_height,
        expanded_body_height: expanded_content_height.min(EXPANDED_MAX_BODY_HEIGHT),
        render_command_bundle,
    }
}

pub(super) fn resolve_native_panel_scene_for_state_with_input(
    state: &NativePanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<crate::native_panel_scene::PanelScene> {
    let core_state = state.to_core_panel_state();
    let cache_key = scene_cache_key_for_state(state, input);
    cached_scene_for_key(&state.scene_cache, &cache_key).or_else(|| {
        state.last_snapshot.as_ref().map(|snapshot| {
            build_native_panel_scene_for_core_state_with_input(snapshot, &core_state, input)
        })
    })
}

pub(super) fn resolve_native_panel_scene_for_state_and_snapshot(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<crate::native_panel_scene::PanelScene> {
    let cache_key = scene_cache_key_for_state(state, input);
    if snapshot_matches_current_state_or_cache(state, snapshot) {
        if let Some(scene) = cached_scene_for_key(&state.scene_cache, &cache_key) {
            return Some(scene);
        }
    }

    Some(build_scene_for_state_snapshot_with_input(
        state, snapshot, input,
    ))
}

fn resolve_native_panel_render_command_bundle_for_state_and_snapshot_with_input(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> Option<crate::native_panel_renderer::NativePanelRenderCommandBundle> {
    let cache_key = scene_cache_key_for_state(state, input);
    snapshot_matches_current_state_or_cache(state, snapshot)
        .then(|| cached_render_command_bundle_for_key(&state.scene_cache, &cache_key))
        .flatten()
}

pub(super) fn resolve_native_panel_render_command_bundle_for_state_and_snapshot(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> Option<crate::native_panel_renderer::NativePanelRenderCommandBundle> {
    resolve_native_panel_render_command_bundle_for_state_and_snapshot_with_input(
        state,
        snapshot,
        &native_panel_runtime_input_descriptor(),
    )
}

#[cfg(test)]
fn current_snapshot_render_plan_for_state_and_snapshot(
    state: &NativePanelState,
    snapshot: &RuntimeSnapshot,
) -> NativePanelSnapshotRenderPlan {
    let input = native_panel_runtime_input_descriptor();
    let render_command_bundle =
        resolve_native_panel_render_command_bundle_for_state_and_snapshot_with_input(
            state, snapshot, &input,
        );
    let scene = render_command_bundle
        .as_ref()
        .map(|bundle| bundle.scene.clone())
        .or_else(|| resolve_native_panel_scene_for_state_and_snapshot(state, snapshot, &input))
        .expect("scene");
    render_plan_from_scene_and_bundle(scene, render_command_bundle)
}

#[cfg(test)]
pub(super) fn resolve_native_panel_runtime_render_state_for_state_with_input(
    state: &NativePanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> crate::native_panel_scene::PanelRuntimeRenderState {
    let core_state = state.to_core_panel_state();
    let cache_key = scene_cache_key_for_state(state, input);
    crate::native_panel_renderer::cached_runtime_render_state_for_key(
        &state.scene_cache,
        &cache_key,
    )
    .unwrap_or_else(|| {
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
        build_native_panel_scene_for_core_state_with_input,
        build_native_panel_scene_for_state_with_input,
        current_snapshot_render_plan_for_state_and_snapshot,
        resolve_native_panel_render_command_bundle_for_state_and_snapshot,
        resolve_native_panel_runtime_render_state_for_state_with_input,
        resolve_native_panel_scene_for_state_and_snapshot,
        resolve_native_panel_scene_for_state_with_input, resolve_or_build_native_panel_scene,
        resolve_snapshot_card_stack_command, resolve_snapshot_compact_bar_command,
        resolve_snapshot_render_plan,
    };
    use crate::{
        macos_native_test_panel::{
            mascot::NativeMascotRuntime, panel_types::NativeExpandedSurface,
        },
        native_panel_renderer::{
            NativePanelHostWindowDescriptor, NativePanelRuntimeInputDescriptor,
            NativePanelRuntimeSceneCache, cache_render_command_bundle,
            cache_render_command_bundle_with_key, cache_scene_runtime_with_key,
            native_panel_runtime_scene_cache_key, resolve_native_panel_render_command_bundle,
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

    fn cache_scene_for_state(
        state: &mut crate::macos_native_test_panel::panel_types::NativePanelState,
        input: &NativePanelRuntimeInputDescriptor,
        scene: crate::native_panel_scene::PanelScene,
        runtime_render_state: PanelRuntimeRenderState,
    ) {
        let cache_key = native_panel_runtime_scene_cache_key(&state.to_core_panel_state(), input);
        cache_scene_runtime_with_key(
            &mut state.scene_cache,
            Some(cache_key),
            scene,
            runtime_render_state,
        );
    }

    fn cache_bundle_for_state(
        state: &mut crate::macos_native_test_panel::panel_types::NativePanelState,
        input: &NativePanelRuntimeInputDescriptor,
        bundle: &crate::native_panel_renderer::NativePanelRenderCommandBundle,
    ) {
        let cache_key = native_panel_runtime_scene_cache_key(&state.to_core_panel_state(), input);
        cache_render_command_bundle_with_key(&mut state.scene_cache, Some(cache_key), bundle);
    }

    #[test]
    fn current_scene_resolution_prefers_shared_cache() {
        let mut state = panel_state();
        let input = runtime_input_descriptor();
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        cache_scene_for_state(
            &mut state,
            &input,
            cached_scene.clone(),
            PanelRuntimeRenderState::default(),
        );

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
        let input = runtime_input_descriptor();
        cache_scene_for_state(
            &mut state,
            &input,
            build_panel_scene(
                &crate::native_panel_core::PanelState::default(),
                &test_snapshot("cached"),
                &PanelSceneBuildInput::default(),
            ),
            PanelRuntimeRenderState {
                transitioning: true,
                ..PanelRuntimeRenderState::default()
            },
        );

        let resolved =
            resolve_native_panel_runtime_render_state_for_state_with_input(&state, &input);

        assert!(resolved.transitioning);
    }

    #[test]
    fn current_scene_resolution_prefers_render_command_bundle_cache() {
        let mut state = panel_state();
        let input = runtime_input_descriptor();
        let bundle = test_render_command_bundle("bundle", false);
        cache_bundle_for_state(&mut state, &input, &bundle);

        let resolved =
            resolve_native_panel_scene_for_state_with_input(&state, &input).expect("bundle scene");

        assert_eq!(
            resolved.compact_bar.headline.text,
            bundle.scene.compact_bar.headline.text
        );
    }

    #[test]
    fn current_runtime_state_resolution_prefers_render_command_bundle_cache() {
        let mut state = panel_state();
        let input = runtime_input_descriptor();
        let bundle = test_render_command_bundle("bundle", true);
        cache_bundle_for_state(&mut state, &input, &bundle);

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
        let input = runtime_input_descriptor();
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        let current_snapshot = test_snapshot("idle");
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_scene_for_state(
            &mut state,
            &input,
            cached_scene.clone(),
            PanelRuntimeRenderState::default(),
        );

        let resolved = if state.last_snapshot.as_ref() == Some(&current_snapshot) {
            resolve_native_panel_scene_for_state_with_input(&state, &input).expect("cached scene")
        } else {
            unreachable!("current snapshot should match state snapshot");
        };

        assert_eq!(
            resolved.compact_bar.headline.text,
            cached_scene.compact_bar.headline.text
        );
    }

    #[test]
    fn current_snapshot_scene_resolution_reuses_cached_scene() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("idle");
        let input = runtime_input_descriptor();
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_scene_for_state(
            &mut state,
            &input,
            cached_scene.clone(),
            PanelRuntimeRenderState::default(),
        );

        let resolved =
            resolve_native_panel_scene_for_state_and_snapshot(&state, &current_snapshot, &input)
                .expect("current snapshot scene");

        assert_eq!(
            resolved.compact_bar.headline.text,
            cached_scene.compact_bar.headline.text
        );
    }

    #[test]
    fn resolve_or_build_scene_uses_cached_current_snapshot() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("idle");
        let input = runtime_input_descriptor();
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_scene_for_state(
            &mut state,
            &input,
            cached_scene.clone(),
            PanelRuntimeRenderState::default(),
        );

        let resolved = resolve_or_build_native_panel_scene(&current_snapshot);

        assert_eq!(
            resolved.compact_bar.headline.text,
            cached_scene.compact_bar.headline.text
        );
    }

    #[test]
    fn mismatched_snapshot_scene_resolution_rebuilds_from_snapshot() {
        let mut state = panel_state();
        state.expanded = true;
        state.surface_mode = NativeExpandedSurface::Settings;
        state.last_snapshot = Some(test_snapshot("current"));
        state.scene_cache.last_snapshot = Some(test_snapshot("current"));
        let cached_scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot("cached"),
            &PanelSceneBuildInput::default(),
        );
        state.scene_cache.last_scene = Some(cached_scene.clone());
        let other_snapshot = test_snapshot("other");
        let expected = build_native_panel_scene_for_core_state_with_input(
            &other_snapshot,
            &state.to_core_panel_state(),
            &runtime_input_descriptor(),
        );

        let resolved = resolve_native_panel_scene_for_state_and_snapshot(
            &state,
            &other_snapshot,
            &runtime_input_descriptor(),
        )
        .expect("rebuilt scene");

        assert_eq!(
            resolved.compact_bar.headline.text,
            expected.compact_bar.headline.text
        );
        assert_eq!(
            resolved.surface,
            crate::native_panel_core::ExpandedSurface::Settings
        );
        assert_ne!(resolved.surface, cached_scene.surface);
    }

    #[test]
    fn current_snapshot_scene_resolution_ignores_stale_surface_cache() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("current");
        state.expanded = true;
        state.surface_mode = NativeExpandedSurface::Settings;
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_scene = Some(build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &current_snapshot,
            &PanelSceneBuildInput::default(),
        ));

        let resolved = resolve_native_panel_scene_for_state_and_snapshot(
            &state,
            &current_snapshot,
            &runtime_input_descriptor(),
        )
        .expect("rebuilt scene");

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
    fn render_command_bundle_resolution_reuses_current_snapshot_cache() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("idle");
        let input = runtime_input_descriptor();
        let bundle = test_render_command_bundle("bundle", true);
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_bundle_for_state(&mut state, &input, &bundle);

        let resolved = resolve_native_panel_render_command_bundle_for_state_and_snapshot(
            &state,
            &current_snapshot,
        )
        .expect("cached bundle");

        assert_eq!(
            resolved.compact_bar.headline.text,
            bundle.compact_bar.headline.text
        );
        assert!(resolved.runtime.transitioning);
    }

    #[test]
    fn render_command_bundle_resolution_skips_mismatched_snapshot_cache() {
        let mut state = panel_state();
        state.last_snapshot = Some(test_snapshot("current"));
        state.scene_cache.last_snapshot = Some(test_snapshot("current"));
        cache_render_command_bundle(
            &mut state.scene_cache,
            &test_render_command_bundle("bundle", true),
        );

        let resolved = resolve_native_panel_render_command_bundle_for_state_and_snapshot(
            &state,
            &test_snapshot("other"),
        );

        assert!(resolved.is_none());
    }

    #[test]
    fn snapshot_render_plan_ignores_stale_surface_bundle_cache() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("current");
        state.expanded = true;
        state.surface_mode = NativeExpandedSurface::Settings;
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_render_command_bundle(
            &mut state.scene_cache,
            &test_render_command_bundle("bundle", false),
        );

        let resolved =
            current_snapshot_render_plan_for_state_and_snapshot(&state, &current_snapshot);

        assert_eq!(
            resolved.scene.surface,
            crate::native_panel_core::ExpandedSurface::Settings
        );
        assert!(matches!(
            resolved.scene.cards.first(),
            Some(crate::native_panel_scene::SceneCard::Settings { .. })
        ));
        assert!(resolved.render_command_bundle.is_none());
    }

    #[test]
    fn snapshot_compact_bar_command_reuses_cached_bundle_and_overrides_frame() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("idle");
        let input = runtime_input_descriptor();
        let bundle = test_render_command_bundle("bundle", false);
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_bundle_for_state(&mut state, &input, &bundle);

        let frame = crate::native_panel_core::PanelRect {
            x: 11.0,
            y: 12.0,
            width: 13.0,
            height: 14.0,
        };

        let resolved = resolve_snapshot_compact_bar_command(&current_snapshot, frame);

        assert_eq!(resolved.headline.text, bundle.compact_bar.headline.text);
        assert_eq!(resolved.frame, frame);
    }

    #[test]
    fn snapshot_card_stack_command_reuses_cached_bundle_and_overrides_layout_inputs() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("idle");
        let input = runtime_input_descriptor();
        let bundle = test_render_command_bundle("bundle", false);
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_bundle_for_state(&mut state, &input, &bundle);

        let frame = crate::native_panel_core::PanelRect {
            x: 21.0,
            y: 22.0,
            width: 23.0,
            height: 24.0,
        };

        let resolved = resolve_snapshot_card_stack_command(&current_snapshot, frame, false);

        assert_eq!(resolved.cards.len(), bundle.card_stack.cards.len());
        assert_eq!(resolved.frame, frame);
        assert!(!resolved.visible);
    }

    #[test]
    fn snapshot_render_plan_reuses_cached_bundle_scene_and_body_height() {
        let mut state = panel_state();
        let current_snapshot = test_snapshot("idle");
        let input = runtime_input_descriptor();
        let bundle = test_render_command_bundle("bundle", false);
        state.last_snapshot = Some(current_snapshot.clone());
        state.scene_cache.last_snapshot = Some(current_snapshot.clone());
        cache_bundle_for_state(&mut state, &input, &bundle);

        let resolved = resolve_snapshot_render_plan(&current_snapshot);

        assert_eq!(
            resolved.scene.compact_bar.headline.text,
            bundle.scene.compact_bar.headline.text
        );
        assert!(resolved.expanded_body_height <= resolved.expanded_content_height);
    }

    fn test_render_command_bundle(
        status: &str,
        transitioning: bool,
    ) -> crate::native_panel_renderer::NativePanelRenderCommandBundle {
        let scene = build_panel_scene(
            &crate::native_panel_core::PanelState::default(),
            &test_snapshot(status),
            &PanelSceneBuildInput::default(),
        );
        let layout = crate::native_panel_core::resolve_panel_layout(
            crate::native_panel_core::PanelLayoutInput {
                screen_frame: crate::native_panel_core::PanelRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1440.0,
                    height: 900.0,
                },
                metrics: crate::native_panel_core::PanelGeometryMetrics {
                    compact_height: crate::native_panel_core::DEFAULT_COMPACT_PILL_HEIGHT,
                    compact_width: crate::native_panel_core::DEFAULT_COMPACT_PILL_WIDTH,
                    expanded_width: crate::native_panel_core::DEFAULT_EXPANDED_PILL_WIDTH,
                    panel_width: crate::native_panel_core::DEFAULT_PANEL_CANVAS_WIDTH,
                },
                canvas_height: 180.0,
                visible_height: 180.0,
                bar_progress: 1.0,
                height_progress: 1.0,
                drop_progress: 1.0,
                content_visibility: 1.0,
                collapsed_height: crate::native_panel_core::COLLAPSED_PANEL_HEIGHT,
                drop_distance: crate::native_panel_core::PANEL_DROP_DISTANCE,
                content_top_gap: crate::native_panel_core::EXPANDED_CONTENT_TOP_GAP,
                content_bottom_inset: crate::native_panel_core::EXPANDED_CONTENT_BOTTOM_INSET,
                cards_side_inset: crate::native_panel_core::EXPANDED_CARDS_SIDE_INSET,
                shoulder_size: crate::native_panel_core::COMPACT_SHOULDER_SIZE,
                separator_side_inset: crate::native_panel_core::EXPANDED_SEPARATOR_SIDE_INSET,
            },
        );
        let runtime = PanelRuntimeRenderState {
            transitioning,
            ..PanelRuntimeRenderState::default()
        };
        let render_state = crate::native_panel_core::resolve_panel_render_state(
            crate::native_panel_core::PanelRenderStateInput {
                shared_expanded_enabled: false,
                shell_visible: layout.shell_visible,
                separator_visibility: layout.separator_visibility,
                bar_progress: 1.0,
                height_progress: 1.0,
                cards_height: layout.cards_frame.height,
                status_surface_active: false,
                content_visibility: 1.0,
                transitioning,
                headline_emphasized: scene.compact_bar.headline.emphasized,
                edge_actions_visible: scene.compact_bar.actions_visible,
            },
        );

        resolve_native_panel_render_command_bundle(layout, &scene, runtime, render_state, None)
    }
}
