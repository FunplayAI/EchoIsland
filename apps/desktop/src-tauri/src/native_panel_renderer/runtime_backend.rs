use echoisland_runtime::RuntimeSnapshot;
use std::marker::PhantomData;
use tauri::{AppHandle, Manager};

use crate::{
    constants::MAIN_WINDOW_LABEL,
    native_panel_core::{PanelSnapshotSyncResult, PanelState},
    native_panel_scene::{PanelRuntimeSceneBundle, build_panel_runtime_scene_bundle},
};

use super::descriptors::NativePanelRuntimeInputDescriptor;
use super::host_runtime_facade::native_panel_host_display_reposition_from_input_descriptor;
use super::runtime_scene_cache::{
    NativePanelRuntimeSceneCache, NativePanelRuntimeSceneCacheKey, cache_runtime_scene_with_key,
    native_panel_runtime_scene_cache_key,
};
use super::traits::NativePanelSceneHost;

pub(crate) fn hide_main_webview_window_when_native_ui_enabled<R: tauri::Runtime>(
    app: &AppHandle<R>,
    native_ui_enabled: impl FnOnce() -> bool,
) -> Result<(), String> {
    if !native_ui_enabled() {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn reposition_native_panel_to_selected_display_then_refresh<R: tauri::Runtime>(
    app: &AppHandle<R>,
    reposition: impl FnOnce(&AppHandle<R>) -> Result<(), String>,
    refresh: impl FnOnce(&AppHandle<R>) -> Result<(), String>,
) -> Result<(), String> {
    reposition(app)?;
    refresh(app)
}

#[derive(Clone, Debug)]
pub(crate) struct NativePanelRuntimeSceneSyncResult {
    pub(crate) snapshot_sync: PanelSnapshotSyncResult,
    pub(crate) bundle: PanelRuntimeSceneBundle,
    pub(crate) cache_key: NativePanelRuntimeSceneCacheKey,
}

pub(crate) fn sync_runtime_scene_bundle_from_input_descriptor(
    panel_state: &mut PanelState,
    raw_snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
    now: chrono::DateTime<chrono::Utc>,
) -> NativePanelRuntimeSceneSyncResult {
    let snapshot_sync =
        crate::native_panel_core::sync_panel_snapshot_state(panel_state, raw_snapshot, now);
    let bundle = build_panel_runtime_scene_bundle(
        panel_state,
        &snapshot_sync.displayed_snapshot,
        &input.scene_input,
    );
    let cache_key = native_panel_runtime_scene_cache_key(panel_state, input);

    NativePanelRuntimeSceneSyncResult {
        snapshot_sync,
        bundle,
        cache_key,
    }
}

pub(crate) fn rebuild_runtime_scene_sync_result_from_snapshot_input(
    panel_state: &mut PanelState,
    snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
) -> NativePanelRuntimeSceneSyncResult {
    sync_runtime_scene_bundle_from_input_descriptor(
        panel_state,
        snapshot,
        input,
        chrono::Utc::now(),
    )
}

pub(crate) fn cache_runtime_scene_sync_result(
    cache: &mut NativePanelRuntimeSceneCache,
    sync_result: NativePanelRuntimeSceneSyncResult,
) -> PanelSnapshotSyncResult {
    cache_runtime_scene_with_key(
        cache,
        sync_result.bundle.displayed_snapshot,
        Some(sync_result.cache_key),
        sync_result.bundle.scene,
        sync_result.bundle.runtime_render_state,
    );
    sync_result.snapshot_sync
}

pub(crate) fn apply_runtime_scene_sync_result_to_host<H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    sync_result: NativePanelRuntimeSceneSyncResult,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<PanelSnapshotSyncResult, H::Error>
where
    H: NativePanelSceneHost,
{
    let NativePanelRuntimeSceneSyncResult {
        snapshot_sync,
        bundle,
        cache_key,
    } = sync_result;
    host.sync_scene_with_payload(
        &bundle.scene,
        bundle.runtime_render_state,
        native_panel_host_display_reposition_from_input_descriptor(input),
    )?;
    cache_runtime_scene_with_key(
        cache,
        bundle.displayed_snapshot,
        Some(cache_key),
        bundle.scene,
        bundle.runtime_render_state,
    );
    Ok(snapshot_sync)
}

pub(crate) fn sync_and_apply_runtime_scene_from_input_descriptor<H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    panel_state: &mut PanelState,
    raw_snapshot: &RuntimeSnapshot,
    input: &NativePanelRuntimeInputDescriptor,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<PanelSnapshotSyncResult, H::Error>
where
    H: NativePanelSceneHost,
{
    let sync_result =
        sync_runtime_scene_bundle_from_input_descriptor(panel_state, raw_snapshot, input, now);
    apply_runtime_scene_sync_result_to_host(host, cache, sync_result, input)
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_with_input_descriptor<H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    input: &NativePanelRuntimeInputDescriptor,
    rebuild_sync_result: impl FnOnce(&RuntimeSnapshot) -> NativePanelRuntimeSceneSyncResult,
) -> Result<bool, H::Error>
where
    H: NativePanelSceneHost,
{
    let Some(snapshot) = cache.last_snapshot.clone() else {
        return Ok(false);
    };
    let sync_result = rebuild_sync_result(&snapshot);
    apply_runtime_scene_sync_result_to_host(host, cache, sync_result, input)?;
    Ok(true)
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_for_panel_state_with_input_descriptor<H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    panel_state: &mut PanelState,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<bool, H::Error>
where
    H: NativePanelSceneHost,
{
    rerender_runtime_scene_sync_result_to_host_with_input_descriptor(
        host,
        cache,
        input,
        |snapshot| {
            rebuild_runtime_scene_sync_result_from_snapshot_input(panel_state, snapshot, input)
        },
    )
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_on_transition_with_input_descriptor<H, T>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    transition: Option<T>,
    input: &NativePanelRuntimeInputDescriptor,
    rebuild_sync_result: impl FnOnce(&RuntimeSnapshot) -> NativePanelRuntimeSceneSyncResult,
) -> Result<Option<T>, H::Error>
where
    H: NativePanelSceneHost,
{
    if transition.is_some() {
        rerender_runtime_scene_sync_result_to_host_with_input_descriptor(
            host,
            cache,
            input,
            rebuild_sync_result,
        )?;
    }
    Ok(transition)
}

pub(crate) fn rerender_runtime_scene_sync_result_to_host_on_transition_for_panel_state_with_input_descriptor<
    H,
    T,
>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    panel_state: &mut PanelState,
    transition: Option<T>,
    input: &NativePanelRuntimeInputDescriptor,
) -> Result<Option<T>, H::Error>
where
    H: NativePanelSceneHost,
{
    rerender_runtime_scene_sync_result_to_host_on_transition_with_input_descriptor(
        host,
        cache,
        transition,
        input,
        |snapshot| {
            rebuild_runtime_scene_sync_result_from_snapshot_input(panel_state, snapshot, input)
        },
    )
}

pub(crate) fn mutate_panel_state_and_rerender_runtime_scene_sync_result_with_input_descriptor<H>(
    host: &mut H,
    cache: &mut NativePanelRuntimeSceneCache,
    panel_state: &mut PanelState,
    input: &NativePanelRuntimeInputDescriptor,
    mutate: impl FnOnce(&mut PanelState) -> bool,
) -> Result<bool, H::Error>
where
    H: NativePanelSceneHost,
{
    if !mutate(panel_state) {
        return Ok(false);
    }
    let _ = rerender_runtime_scene_sync_result_to_host_for_panel_state_with_input_descriptor(
        host,
        cache,
        panel_state,
        input,
    )?;
    Ok(true)
}

pub(crate) trait NativePanelPlatformRuntimeBackend {
    fn native_ui_enabled(&self) -> bool;

    fn create_panel(&self) -> Result<(), String>;

    fn hide_main_webview_window<R: tauri::Runtime>(&self, app: &AppHandle<R>)
    -> Result<(), String>;

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, app: AppHandle<R>);

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String>;

    fn hide_panel<R: tauri::Runtime>(&self, app: &AppHandle<R>) -> Result<(), String>;

    fn refresh_from_last_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String>;

    fn reposition_to_selected_display<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String>;

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String>;
}

pub(crate) trait NativePanelPlatformRuntimeFacadeApi {
    fn native_ui_enabled() -> bool;

    fn create_panel() -> Result<(), String>;

    fn hide_main_webview_window<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        hide_main_webview_window_when_native_ui_enabled(app, Self::native_ui_enabled)
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(app: AppHandle<R>);

    fn update_snapshot<R: tauri::Runtime>(
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String>;

    fn hide_panel<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String>;

    fn refresh_from_last_snapshot<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String>;

    fn reposition_to_selected_display<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String>;

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NativePanelPlatformRuntimeBackendFacade<Api> {
    marker: PhantomData<fn() -> Api>,
}

impl<Api> NativePanelPlatformRuntimeBackendFacade<Api> {
    pub(crate) fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<Api> NativePanelPlatformRuntimeBackend for NativePanelPlatformRuntimeBackendFacade<Api>
where
    Api: NativePanelPlatformRuntimeFacadeApi,
{
    fn native_ui_enabled(&self) -> bool {
        Api::native_ui_enabled()
    }

    fn create_panel(&self) -> Result<(), String> {
        Api::create_panel()
    }

    fn hide_main_webview_window<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        Api::hide_main_webview_window(app)
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, app: AppHandle<R>) {
        Api::spawn_platform_loops(app);
    }

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        Api::update_snapshot(app, snapshot)
    }

    fn hide_panel<R: tauri::Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        Api::hide_panel(app)
    }

    fn refresh_from_last_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        Api::refresh_from_last_snapshot(app)
    }

    fn reposition_to_selected_display<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        Api::reposition_to_selected_display(app)
    }

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String> {
        Api::set_shared_expanded_body_height(app, body_height)
    }
}

pub(crate) trait NativePanelRuntimeBackend: NativePanelPlatformRuntimeBackend {
    fn native_ui_enabled(&self) -> bool {
        NativePanelPlatformRuntimeBackend::native_ui_enabled(self)
    }

    fn create_panel(&self) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::create_panel(self)
    }

    fn hide_main_webview_window<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::hide_main_webview_window(self, app)
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, app: AppHandle<R>) {
        NativePanelPlatformRuntimeBackend::spawn_platform_loops(self, app);
    }

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::update_snapshot(self, app, snapshot)
    }

    fn hide_panel<R: tauri::Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::hide_panel(self, app)
    }

    fn refresh_from_last_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::refresh_from_last_snapshot(self, app)
    }

    fn reposition_to_selected_display<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::reposition_to_selected_display(self, app)
    }

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::set_shared_expanded_body_height(self, app, body_height)
    }
}

impl<T> NativePanelRuntimeBackend for T where T: NativePanelPlatformRuntimeBackend {}

#[cfg(target_os = "macos")]
pub(crate) type CurrentNativePanelRuntimeBackend =
    crate::macos_native_panel::MacosNativePanelRuntimeBackendFacade;

#[cfg(target_os = "windows")]
pub(crate) type CurrentNativePanelRuntimeBackend =
    crate::windows_native_panel::WindowsNativePanelRuntimeBackendFacade;

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
pub(crate) struct CurrentNativePanelRuntimeBackend;

#[cfg(target_os = "macos")]
pub(crate) fn current_native_panel_runtime_backend() -> CurrentNativePanelRuntimeBackend {
    crate::macos_native_panel::current_macos_native_panel_runtime_backend()
}

#[cfg(target_os = "windows")]
pub(crate) fn current_native_panel_runtime_backend() -> CurrentNativePanelRuntimeBackend {
    crate::windows_native_panel::current_windows_native_panel_runtime_backend()
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
impl NativePanelPlatformRuntimeBackend for CurrentNativePanelRuntimeBackend {
    fn native_ui_enabled(&self) -> bool {
        false
    }

    fn create_panel(&self) -> Result<(), String> {
        Ok(())
    }

    fn hide_main_webview_window<R: tauri::Runtime>(&self, _: &AppHandle<R>) -> Result<(), String> {
        Ok(())
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, _: AppHandle<R>) {}

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        _: &AppHandle<R>,
        _: &RuntimeSnapshot,
    ) -> Result<(), String> {
        Ok(())
    }

    fn hide_panel<R: tauri::Runtime>(&self, _: &AppHandle<R>) -> Result<(), String> {
        Ok(())
    }

    fn refresh_from_last_snapshot<R: tauri::Runtime>(
        &self,
        _: &AppHandle<R>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn reposition_to_selected_display<R: tauri::Runtime>(
        &self,
        _: &AppHandle<R>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        &self,
        _: &AppHandle<R>,
        _: f64,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
pub(crate) fn current_native_panel_runtime_backend() -> CurrentNativePanelRuntimeBackend {
    CurrentNativePanelRuntimeBackend
}

#[cfg(test)]
mod tests {
    use super::{
        cache_runtime_scene_sync_result,
        rerender_runtime_scene_sync_result_to_host_with_input_descriptor,
        sync_and_apply_runtime_scene_from_input_descriptor,
        sync_runtime_scene_bundle_from_input_descriptor,
    };
    use crate::{
        native_panel_core::{ExpandedSurface, PanelRect, PanelState},
        native_panel_renderer::{
            descriptors::{
                NativePanelHostWindowDescriptor, NativePanelHostWindowState,
                NativePanelRuntimeInputDescriptor,
            },
            runtime_scene_cache::{
                NativePanelRuntimeSceneCache, native_panel_runtime_scene_cache_key,
            },
            traits::{NativePanelHost, NativePanelRenderer, NativePanelSceneHost},
        },
        native_panel_scene::{PanelRuntimeRenderState, PanelScene, PanelSceneBuildInput},
    };
    use chrono::Utc;
    use echoisland_runtime::{RuntimeSnapshot, SessionSnapshotView};

    fn runtime_snapshot(status: &str, session_status: &str) -> RuntimeSnapshot {
        RuntimeSnapshot {
            status: status.to_string(),
            primary_source: "codex".to_string(),
            active_session_count: 1,
            total_session_count: 1,
            pending_permission_count: 0,
            pending_question_count: 0,
            pending_permission: None,
            pending_question: None,
            pending_permissions: vec![],
            pending_questions: vec![],
            sessions: vec![SessionSnapshotView {
                session_id: "session-1".to_string(),
                source: "codex".to_string(),
                project_name: None,
                cwd: None,
                model: None,
                terminal_app: None,
                terminal_bundle: None,
                host_app: None,
                window_title: None,
                tty: None,
                terminal_pid: None,
                cli_pid: None,
                iterm_session_id: None,
                kitty_window_id: None,
                tmux_env: None,
                tmux_pane: None,
                tmux_client_tty: None,
                status: session_status.to_string(),
                current_tool: None,
                tool_description: None,
                last_user_prompt: None,
                last_assistant_message: Some("done".to_string()),
                tool_history_count: 0,
                tool_history: vec![],
                last_activity: Utc::now(),
            }],
        }
    }

    #[test]
    fn runtime_scene_bundle_sync_returns_core_sync_and_bundle() {
        let mut panel_state = PanelState::default();
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };

        let result = sync_runtime_scene_bundle_from_input_descriptor(
            &mut panel_state,
            &runtime_snapshot("idle", "Running"),
            &descriptor,
            Utc::now(),
        );

        assert_eq!(result.snapshot_sync.displayed_snapshot.status, "idle");
        assert_eq!(result.bundle.displayed_snapshot.status, "idle");
        assert_eq!(result.bundle.scene.compact_bar.active_count, "1");
        assert_eq!(result.bundle.scene.compact_bar.total_count, "1");
    }

    #[test]
    fn runtime_scene_bundle_sync_preserves_completion_side_effects() {
        let mut panel_state = PanelState::default();
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };

        let previous = runtime_snapshot("running", "Running");
        let current = runtime_snapshot("idle", "Idle");
        let _ = sync_runtime_scene_bundle_from_input_descriptor(
            &mut panel_state,
            &previous,
            &descriptor,
            Utc::now(),
        );
        let result = sync_runtime_scene_bundle_from_input_descriptor(
            &mut panel_state,
            &current,
            &descriptor,
            Utc::now(),
        );

        assert!(result.snapshot_sync.reminder.play_sound);
        assert!(!panel_state.completion_badge_items.is_empty());
    }

    #[derive(Default)]
    struct TestRenderer;

    impl NativePanelRenderer for TestRenderer {
        type Error = String;

        fn render_scene(
            &mut self,
            _scene: &PanelScene,
            _runtime: PanelRuntimeRenderState,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn set_visible(&mut self, _visible: bool) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct TestHost {
        renderer: TestRenderer,
        descriptor: NativePanelHostWindowDescriptor,
        synced_display_index: Option<usize>,
        synced_screen_frame: Option<Option<PanelRect>>,
        synced_scene: Option<PanelScene>,
    }

    impl NativePanelHost for TestHost {
        type Error = String;
        type Renderer = TestRenderer;

        fn renderer(&mut self) -> &mut Self::Renderer {
            &mut self.renderer
        }

        fn host_window_descriptor(&self) -> NativePanelHostWindowDescriptor {
            self.descriptor
        }

        fn host_window_descriptor_mut(&mut self) -> &mut NativePanelHostWindowDescriptor {
            &mut self.descriptor
        }

        fn window_state(&self) -> NativePanelHostWindowState {
            NativePanelHostWindowState::default()
        }

        fn show(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn hide(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl NativePanelSceneHost for TestHost {
        fn sync_scene(
            &mut self,
            scene: &PanelScene,
            _runtime: PanelRuntimeRenderState,
            preferred_display_index: usize,
            screen_frame: Option<PanelRect>,
        ) -> Result<(), Self::Error> {
            self.synced_scene = Some(scene.clone());
            self.synced_display_index = Some(preferred_display_index);
            self.synced_screen_frame = Some(screen_frame);
            Ok(())
        }
    }

    #[test]
    fn runtime_scene_sync_result_can_update_shared_cache_without_host_apply() {
        let mut panel_state = PanelState::default();
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        let sync_result = sync_runtime_scene_bundle_from_input_descriptor(
            &mut panel_state,
            &runtime_snapshot("idle", "Running"),
            &descriptor,
            Utc::now(),
        );
        let mut cache = NativePanelRuntimeSceneCache::default();

        let snapshot_sync = cache_runtime_scene_sync_result(&mut cache, sync_result);

        assert_eq!(snapshot_sync.displayed_snapshot.status, "idle");
        assert_eq!(
            cache
                .last_snapshot
                .as_ref()
                .map(|snapshot| snapshot.status.as_str()),
            Some("idle")
        );
        assert!(cache.last_scene.is_some());
        assert!(cache.last_runtime_render_state.is_some());
    }

    #[test]
    fn runtime_scene_controller_syncs_applies_and_updates_cache() {
        let mut panel_state = PanelState::default();
        let mut host = TestHost::default();
        let mut cache = NativePanelRuntimeSceneCache::default();
        let screen_frame = Some(PanelRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 120.0,
        });
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput {
                display_options: vec![
                    crate::native_panel_scene::panel_display_option_state(
                        0,
                        "display-1",
                        "Built-in",
                        3024,
                        1964,
                    ),
                    crate::native_panel_scene::panel_display_option_state(
                        1,
                        "display-2",
                        "Studio Display",
                        2560,
                        1440,
                    ),
                ],
                settings: crate::native_panel_core::PanelSettingsState {
                    selected_display_index: 1,
                    ..Default::default()
                },
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                update_status: crate::updater_service::AppUpdateStatus::idle(),
            },
            screen_frame,
        };

        let snapshot_sync = sync_and_apply_runtime_scene_from_input_descriptor(
            &mut host,
            &mut cache,
            &mut panel_state,
            &runtime_snapshot("idle", "Running"),
            &descriptor,
            Utc::now(),
        )
        .expect("sync and apply runtime scene");

        assert_eq!(snapshot_sync.displayed_snapshot.status, "idle");
        assert_eq!(host.synced_display_index, Some(1));
        assert_eq!(host.synced_screen_frame, Some(screen_frame));
        assert!(host.synced_scene.is_some());
        assert!(cache.last_snapshot.is_some());
        assert!(cache.last_scene.is_some());
        assert!(cache.last_runtime_render_state.is_some());
        assert_eq!(
            cache.last_cache_key,
            Some(native_panel_runtime_scene_cache_key(
                &panel_state,
                &descriptor
            ))
        );
    }

    #[test]
    fn rerender_scene_sync_result_updates_cache_key_for_surface_change() {
        let mut panel_state = PanelState::default();
        let mut host = TestHost::default();
        let mut cache = NativePanelRuntimeSceneCache::default();
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        sync_and_apply_runtime_scene_from_input_descriptor(
            &mut host,
            &mut cache,
            &mut panel_state,
            &runtime_snapshot("idle", "Running"),
            &descriptor,
            Utc::now(),
        )
        .expect("seed sync and apply runtime scene");

        panel_state.expanded = true;
        panel_state.surface_mode = ExpandedSurface::Settings;

        let updated = rerender_runtime_scene_sync_result_to_host_with_input_descriptor(
            &mut host,
            &mut cache,
            &descriptor,
            |snapshot| {
                sync_runtime_scene_bundle_from_input_descriptor(
                    &mut panel_state,
                    snapshot,
                    &descriptor,
                    Utc::now(),
                )
            },
        )
        .expect("rerender keyed sync result");

        assert!(updated);
        assert_eq!(
            cache.last_cache_key,
            Some(native_panel_runtime_scene_cache_key(
                &panel_state,
                &descriptor
            ))
        );
        assert_eq!(
            host.synced_scene.as_ref().map(|scene| scene.surface),
            Some(ExpandedSurface::Settings)
        );
    }

    #[test]
    fn rerender_scene_sync_result_for_panel_state_uses_shared_rebuild_path() {
        let mut panel_state = PanelState::default();
        let mut host = TestHost::default();
        let mut cache = NativePanelRuntimeSceneCache::default();
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        sync_and_apply_runtime_scene_from_input_descriptor(
            &mut host,
            &mut cache,
            &mut panel_state,
            &runtime_snapshot("idle", "Running"),
            &descriptor,
            Utc::now(),
        )
        .expect("seed sync and apply runtime scene");

        panel_state.expanded = true;
        panel_state.surface_mode = ExpandedSurface::Settings;

        let updated =
            super::rerender_runtime_scene_sync_result_to_host_for_panel_state_with_input_descriptor(
                &mut host,
                &mut cache,
                &mut panel_state,
                &descriptor,
            )
            .expect("rerender shared panel state path");

        assert!(updated);
        assert_eq!(
            cache.last_cache_key,
            Some(native_panel_runtime_scene_cache_key(
                &panel_state,
                &descriptor
            ))
        );
        assert_eq!(
            host.synced_scene.as_ref().map(|scene| scene.surface),
            Some(ExpandedSurface::Settings)
        );
    }

    #[test]
    fn mutate_panel_state_and_rerender_returns_false_without_change() {
        let mut panel_state = PanelState::default();
        let mut host = TestHost::default();
        let mut cache = NativePanelRuntimeSceneCache::default();
        let descriptor = NativePanelRuntimeInputDescriptor {
            scene_input: PanelSceneBuildInput::default(),
            screen_frame: None,
        };
        sync_and_apply_runtime_scene_from_input_descriptor(
            &mut host,
            &mut cache,
            &mut panel_state,
            &runtime_snapshot("idle", "Running"),
            &descriptor,
            Utc::now(),
        )
        .expect("seed sync and apply runtime scene");
        let seeded_surface = cache.last_scene.as_ref().map(|scene| scene.surface);

        let updated =
            super::mutate_panel_state_and_rerender_runtime_scene_sync_result_with_input_descriptor(
                &mut host,
                &mut cache,
                &mut panel_state,
                &descriptor,
                |_| false,
            )
            .expect("mutate without change");

        assert!(!updated);
        assert_eq!(
            cache.last_scene.as_ref().map(|scene| scene.surface),
            seeded_surface
        );
        assert_eq!(
            cache.last_cache_key,
            Some(native_panel_runtime_scene_cache_key(
                &panel_state,
                &descriptor
            ))
        );
    }
}
