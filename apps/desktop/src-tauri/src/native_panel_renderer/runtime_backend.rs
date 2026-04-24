use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

use crate::{
    native_panel_core::{PanelSnapshotSyncResult, PanelState},
    native_panel_scene::{PanelRuntimeSceneBundle, build_panel_runtime_scene_bundle},
};

use super::{
    NativePanelRuntimeInputDescriptor, NativePanelRuntimeSceneCache, NativePanelSceneHost,
    apply_runtime_scene_bundle_to_host, cache_runtime_scene,
};

#[derive(Clone, Debug)]
pub(crate) struct NativePanelRuntimeSceneSyncResult {
    pub(crate) snapshot_sync: PanelSnapshotSyncResult,
    pub(crate) bundle: PanelRuntimeSceneBundle,
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

    NativePanelRuntimeSceneSyncResult {
        snapshot_sync,
        bundle,
    }
}

pub(crate) fn cache_runtime_scene_sync_result(
    cache: &mut NativePanelRuntimeSceneCache,
    sync_result: NativePanelRuntimeSceneSyncResult,
) -> PanelSnapshotSyncResult {
    cache_runtime_scene(
        cache,
        sync_result.bundle.displayed_snapshot,
        sync_result.bundle.scene,
        sync_result.bundle.runtime_render_state,
    );
    sync_result.snapshot_sync
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
    let snapshot_sync = sync_result.snapshot_sync;
    apply_runtime_scene_bundle_to_host(
        host,
        cache,
        sync_result.bundle,
        input.selected_display_index(),
        input.screen_frame,
    )?;
    Ok(snapshot_sync)
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

pub(crate) trait NativePanelRuntimeBackend {
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

#[cfg(target_os = "macos")]
pub(crate) struct CurrentNativePanelRuntimeBackend;

#[cfg(target_os = "windows")]
pub(crate) struct CurrentNativePanelRuntimeBackend;

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
pub(crate) struct CurrentNativePanelRuntimeBackend;

#[cfg(target_os = "macos")]
fn current_platform_native_panel_runtime_backend()
-> crate::macos_native_test_panel::MacosNativePanelRuntimeBackendFacade {
    crate::macos_native_test_panel::current_macos_native_panel_runtime_backend()
}

#[cfg(target_os = "windows")]
fn current_platform_native_panel_runtime_backend()
-> crate::windows_native_panel::WindowsNativePanelRuntimeBackendFacade {
    crate::windows_native_panel::current_windows_native_panel_runtime_backend()
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl NativePanelRuntimeBackend for CurrentNativePanelRuntimeBackend {
    fn native_ui_enabled(&self) -> bool {
        current_platform_native_panel_runtime_backend().native_ui_enabled()
    }

    fn create_panel(&self) -> Result<(), String> {
        current_platform_native_panel_runtime_backend().create_panel()
    }

    fn hide_main_webview_window<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        current_platform_native_panel_runtime_backend().hide_main_webview_window(app)
    }

    fn spawn_platform_loops<R: tauri::Runtime + 'static>(&self, app: AppHandle<R>) {
        current_platform_native_panel_runtime_backend().spawn_platform_loops(app);
    }

    fn update_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), String> {
        current_platform_native_panel_runtime_backend().update_snapshot(app, snapshot)
    }

    fn hide_panel<R: tauri::Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        current_platform_native_panel_runtime_backend().hide_panel(app)
    }

    fn refresh_from_last_snapshot<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        current_platform_native_panel_runtime_backend().refresh_from_last_snapshot(app)
    }

    fn reposition_to_selected_display<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), String> {
        current_platform_native_panel_runtime_backend().reposition_to_selected_display(app)
    }

    fn set_shared_expanded_body_height<R: tauri::Runtime>(
        &self,
        app: &AppHandle<R>,
        body_height: f64,
    ) -> Result<(), String> {
        current_platform_native_panel_runtime_backend()
            .set_shared_expanded_body_height(app, body_height)
    }
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
impl NativePanelRuntimeBackend for CurrentNativePanelRuntimeBackend {
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

pub(crate) fn current_native_panel_runtime_backend() -> CurrentNativePanelRuntimeBackend {
    CurrentNativePanelRuntimeBackend
}

#[cfg(test)]
mod tests {
    use super::{
        cache_runtime_scene_sync_result, sync_and_apply_runtime_scene_from_input_descriptor,
        sync_runtime_scene_bundle_from_input_descriptor,
    };
    use crate::{
        native_panel_core::{PanelRect, PanelState},
        native_panel_renderer::{
            NativePanelHost, NativePanelHostWindowDescriptor, NativePanelHostWindowState,
            NativePanelRenderer, NativePanelRuntimeInputDescriptor, NativePanelRuntimeSceneCache,
            NativePanelSceneHost,
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

        assert!(result.snapshot_sync.play_message_card_sound);
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
                display_count: 2,
                settings: crate::native_panel_core::PanelSettingsState {
                    selected_display_index: 1,
                    ..Default::default()
                },
                app_version: env!("CARGO_PKG_VERSION").to_string(),
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
    }
}
