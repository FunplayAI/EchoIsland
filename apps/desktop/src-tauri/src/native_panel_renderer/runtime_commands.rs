use echoisland_runtime::RuntimeSnapshot;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tracing::warn;

use crate::{
    app_runtime::AppRuntime,
    app_settings::{
        current_app_settings, update_completion_sound_enabled, update_mascot_enabled,
        update_preferred_display_selection,
    },
    display_settings::list_available_displays,
    native_panel_core::PanelInteractionCommand,
    native_panel_scene_input::resolve_next_display_selection_update_from_display_options,
    terminal_focus_service::spawn_runtime_focus_session,
};

use super::descriptors::{
    NativePanelPlatformEvent, NativePanelPointerInput, NativePanelPointerInputOutcome,
    NativePanelQueuedRuntimeCommandHandler, NativePanelRuntimeCommandCapability,
    dispatch_native_panel_platform_events,
};
use super::host_runtime_facade::NativePanelRuntimeDispatchMode;
use super::runtime_backend::{
    NativePanelPlatformRuntimeBackend, current_native_panel_runtime_backend,
};
use super::runtime_click::dispatch_native_panel_click_command_with_handler;
use super::runtime_interaction::NativePanelSettingsSurfaceSnapshotUpdate;
use super::transition_controller::NativePanelTransitionRequest;

pub(crate) fn execute_native_panel_focus_session_command<R: tauri::Runtime + 'static>(
    app: &AppHandle<R>,
    session_id: String,
) {
    let runtime = app.state::<AppRuntime>().inner().clone();
    spawn_runtime_focus_session(runtime, session_id);
}

pub(crate) fn execute_native_panel_quit_application_command<R: tauri::Runtime>(app: &AppHandle<R>) {
    app.exit(0);
}

pub(crate) fn execute_native_panel_cycle_display_command<R: tauri::Runtime>(
    app: &AppHandle<R>,
    reposition: impl FnOnce(&AppHandle<R>) -> Result<(), String>,
) -> Result<(), String> {
    let displays = list_available_displays(app)?;
    let settings = current_app_settings();
    let Some(next_selection) =
        resolve_next_display_selection_update_from_display_options(&displays, &settings)
    else {
        return Ok(());
    };
    update_preferred_display_selection(
        next_selection.selected_display_index,
        Some(next_selection.selected_display_key),
    )
    .map_err(|error| error.to_string())?;
    reposition(app)
}

pub(crate) fn execute_native_panel_toggle_completion_sound_command(
    refresh: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let next_enabled = !current_app_settings().completion_sound_enabled;
    update_completion_sound_enabled(next_enabled).map_err(|error| error.to_string())?;
    refresh()
}

pub(crate) fn execute_native_panel_toggle_mascot_command(
    refresh: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let next_enabled = !current_app_settings().mascot_enabled;
    update_mascot_enabled(next_enabled).map_err(|error| error.to_string())?;
    refresh()
}

pub(crate) fn execute_native_panel_open_settings_location_command() -> Result<(), String> {
    crate::commands::open_settings_location()
}

pub(crate) fn execute_native_panel_open_release_page_command() -> Result<(), String> {
    crate::commands::open_release_page()
}

pub(crate) fn execute_native_panel_settings_surface_command(
    sync_update: impl FnOnce() -> Result<Option<NativePanelSettingsSurfaceSnapshotUpdate>, String>,
    dispatch_update: impl FnOnce(
        Option<NativePanelTransitionRequest>,
        Option<RuntimeSnapshot>,
    ) -> Result<(), String>,
) -> Result<bool, String> {
    let Some(update) = sync_update()? else {
        return Ok(false);
    };
    dispatch_update(update.transition_request, update.snapshot)?;
    Ok(true)
}

pub(crate) fn dispatch_native_panel_click_command_with_app_handle<R>(
    app: AppHandle<R>,
    command: PanelInteractionCommand,
    toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    dispatch_mode: NativePanelRuntimeDispatchMode,
) -> Result<Option<NativePanelPlatformEvent>, String>
where
    R: tauri::Runtime + 'static,
{
    let mut executor = native_panel_app_handle_runtime_command_executor(
        app,
        toggle_settings_surface,
        dispatch_mode,
    );
    dispatch_native_panel_click_command_with_handler(&mut executor, command)
}

pub(crate) fn dispatch_native_panel_platform_events_with_app_handle<R>(
    app: AppHandle<R>,
    events: impl IntoIterator<Item = NativePanelPlatformEvent>,
    toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    dispatch_mode: NativePanelRuntimeDispatchMode,
) -> Result<(), String>
where
    R: tauri::Runtime + 'static,
{
    let mut executor = native_panel_app_handle_runtime_command_executor(
        app,
        toggle_settings_surface,
        dispatch_mode,
    );
    dispatch_native_panel_platform_events(&mut executor, events)
}

pub(crate) fn run_native_panel_runtime_with_queued_command_dispatch<R, T>(
    app: &AppHandle<R>,
    run: impl FnOnce(&mut NativePanelQueuedRuntimeCommandHandler) -> Result<T, String>,
    toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    dispatch_mode: NativePanelRuntimeDispatchMode,
) -> Result<T, String>
where
    R: tauri::Runtime + 'static,
{
    let mut handler = NativePanelQueuedRuntimeCommandHandler::default();
    let value = run(&mut handler)?;
    dispatch_native_panel_platform_events_with_app_handle(
        app.clone(),
        handler.take_events(),
        toggle_settings_surface,
        dispatch_mode,
    )?;
    Ok(value)
}

pub(crate) fn run_native_panel_pointer_input_with_queued_command_dispatch<R>(
    app: &AppHandle<R>,
    input_event: NativePanelPointerInput,
    now: std::time::Instant,
    run: impl FnOnce(
        NativePanelPointerInput,
        std::time::Instant,
        &mut NativePanelQueuedRuntimeCommandHandler,
    ) -> Result<NativePanelPointerInputOutcome, String>,
    toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    dispatch_mode: NativePanelRuntimeDispatchMode,
) -> Result<NativePanelPointerInputOutcome, String>
where
    R: tauri::Runtime + 'static,
{
    run_native_panel_runtime_with_queued_command_dispatch(
        app,
        |handler| run(input_event, now, handler),
        toggle_settings_surface,
        dispatch_mode,
    )
}

pub(crate) fn dispatch_drained_native_panel_platform_events_with_app_handle<R>(
    app: AppHandle<R>,
    drain_events: fn() -> Result<Vec<NativePanelPlatformEvent>, String>,
    toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    dispatch_mode: NativePanelRuntimeDispatchMode,
) -> Result<(), String>
where
    R: tauri::Runtime + 'static,
{
    let events = drain_events()?;
    if events.is_empty() {
        return Ok(());
    }
    dispatch_native_panel_platform_events_with_app_handle(
        app,
        events,
        toggle_settings_surface,
        dispatch_mode,
    )
}

pub(crate) fn spawn_native_panel_platform_event_dispatch_loop<R>(
    loop_started: &'static AtomicBool,
    app: AppHandle<R>,
    notifier: Arc<Notify>,
    dispatch: fn(AppHandle<R>) -> Result<(), String>,
    error_message: &'static str,
) where
    R: tauri::Runtime + 'static,
{
    if loop_started.swap(true, Ordering::SeqCst) {
        return;
    }

    let loop_notifier = notifier.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            loop_notifier.notified().await;
            if let Err(error) = dispatch(app.clone()) {
                warn!(error = %error, "{error_message}");
            }
        }
    });
    notifier.notify_one();
}

pub(crate) fn spawn_native_panel_platform_loops_with_event_dispatch<R>(
    app: AppHandle<R>,
    spawn_platform_loops: impl FnOnce(),
    spawn_event_dispatch_loop: impl FnOnce(AppHandle<R>),
) where
    R: tauri::Runtime + 'static,
{
    spawn_platform_loops();
    spawn_event_dispatch_loop(app);
}

pub(crate) fn dispatch_native_panel_app_command<R, F>(
    app: &AppHandle<R>,
    command: F,
) -> Result<(), String>
where
    R: tauri::Runtime + 'static,
    F: FnOnce(AppHandle<R>) -> Result<(), String> + Send + 'static,
{
    command(app.clone())
}

pub(crate) fn spawn_native_panel_app_command<R, F>(
    app: AppHandle<R>,
    command: F,
    error_message: &'static str,
) where
    R: tauri::Runtime + 'static,
    F: FnOnce(AppHandle<R>) -> Result<(), String> + Send + 'static,
{
    tauri::async_runtime::spawn(async move {
        if let Err(error) = command(app) {
            warn!(error = %error, "{error_message}");
        }
    });
}

pub(crate) fn dispatch_native_panel_command(
    command: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
    command()
}

pub(crate) fn spawn_native_panel_command(
    command: impl FnOnce() -> Result<(), String> + Send + 'static,
    error_message: &'static str,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = command() {
            warn!(error = %error, "{error_message}");
        }
    });
}

pub(crate) struct NativePanelAppHandleRuntimeCommandExecutor<R: tauri::Runtime + 'static> {
    pub(crate) app: AppHandle<R>,
    pub(crate) toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    pub(crate) dispatch_mode: NativePanelRuntimeDispatchMode,
}

pub(crate) fn native_panel_app_handle_runtime_command_executor<R: tauri::Runtime + 'static>(
    app: AppHandle<R>,
    toggle_settings_surface: fn(&AppHandle<R>) -> Result<(), String>,
    dispatch_mode: NativePanelRuntimeDispatchMode,
) -> NativePanelAppHandleRuntimeCommandExecutor<R> {
    NativePanelAppHandleRuntimeCommandExecutor {
        app,
        toggle_settings_surface,
        dispatch_mode,
    }
}

pub(crate) trait NativePanelAppHandleRuntimeCommandBackend {
    type Runtime: tauri::Runtime + 'static;

    fn app_handle(&self) -> &AppHandle<Self::Runtime>;

    fn dispatch_app_command(
        &mut self,
        command: impl FnOnce(AppHandle<Self::Runtime>) -> Result<(), String> + Send + 'static,
        _error_message: &'static str,
    ) -> Result<(), String> {
        dispatch_native_panel_app_command(self.app_handle(), command)
    }

    fn dispatch_command(
        &mut self,
        command: impl FnOnce() -> Result<(), String> + Send + 'static,
        _error_message: &'static str,
    ) -> Result<(), String> {
        dispatch_native_panel_command(command)
    }

    fn refresh_from_last_snapshot_with_app(app: &AppHandle<Self::Runtime>) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::refresh_from_last_snapshot(
            &current_native_panel_runtime_backend(),
            app,
        )
    }

    fn reposition_to_selected_display_with_app(
        app: &AppHandle<Self::Runtime>,
    ) -> Result<(), String> {
        NativePanelPlatformRuntimeBackend::reposition_to_selected_display(
            &current_native_panel_runtime_backend(),
            app,
        )
    }

    fn focus_session_command(&mut self, session_id: String) -> Result<(), String> {
        self.dispatch_app_command(
            move |app| {
                execute_native_panel_focus_session_command(&app, session_id);
                Ok(())
            },
            "failed to focus session from native panel",
        )
    }

    fn toggle_settings_surface_command(&mut self) -> Result<(), String>;

    fn quit_application_command(&mut self) -> Result<(), String> {
        self.dispatch_app_command(
            move |app| {
                execute_native_panel_quit_application_command(&app);
                Ok(())
            },
            "failed to quit application from native panel",
        )
    }

    fn cycle_display_command(&mut self) -> Result<(), String> {
        self.dispatch_app_command(
            move |app| {
                execute_native_panel_cycle_display_command(&app, |app| {
                    Self::reposition_to_selected_display_with_app(app)
                })
            },
            "failed to update preferred display",
        )
    }

    fn toggle_completion_sound_command(&mut self) -> Result<(), String> {
        self.dispatch_app_command(
            move |app| {
                execute_native_panel_toggle_completion_sound_command(|| {
                    Self::refresh_from_last_snapshot_with_app(&app)
                })
            },
            "failed to update completion sound setting",
        )
    }

    fn toggle_mascot_command(&mut self) -> Result<(), String> {
        self.dispatch_app_command(
            move |app| {
                execute_native_panel_toggle_mascot_command(|| {
                    Self::refresh_from_last_snapshot_with_app(&app)
                })
            },
            "failed to update mascot setting",
        )
    }

    fn open_settings_location_command(&mut self) -> Result<(), String> {
        self.dispatch_command(
            execute_native_panel_open_settings_location_command,
            "failed to open settings folder",
        )
    }

    fn open_release_page_command(&mut self) -> Result<(), String> {
        self.dispatch_command(
            execute_native_panel_open_release_page_command,
            "failed to open release page",
        )
    }
}

impl<T> NativePanelRuntimeCommandCapability for T
where
    T: NativePanelAppHandleRuntimeCommandBackend,
{
    type Error = String;

    fn focus_session(&mut self, session_id: String) -> Result<(), Self::Error> {
        self.focus_session_command(session_id)
    }

    fn toggle_settings_surface(&mut self) -> Result<(), Self::Error> {
        self.toggle_settings_surface_command()
    }

    fn quit_application(&mut self) -> Result<(), Self::Error> {
        self.quit_application_command()
    }

    fn cycle_display(&mut self) -> Result<(), Self::Error> {
        self.cycle_display_command()
    }

    fn toggle_completion_sound(&mut self) -> Result<(), Self::Error> {
        self.toggle_completion_sound_command()
    }

    fn toggle_mascot(&mut self) -> Result<(), Self::Error> {
        self.toggle_mascot_command()
    }

    fn open_settings_location(&mut self) -> Result<(), Self::Error> {
        self.open_settings_location_command()
    }

    fn open_release_page(&mut self) -> Result<(), Self::Error> {
        self.open_release_page_command()
    }
}

impl<R: tauri::Runtime + 'static> NativePanelAppHandleRuntimeCommandBackend
    for NativePanelAppHandleRuntimeCommandExecutor<R>
{
    type Runtime = R;

    fn app_handle(&self) -> &AppHandle<Self::Runtime> {
        &self.app
    }

    fn dispatch_app_command(
        &mut self,
        command: impl FnOnce(AppHandle<Self::Runtime>) -> Result<(), String> + Send + 'static,
        error_message: &'static str,
    ) -> Result<(), String> {
        match self.dispatch_mode {
            NativePanelRuntimeDispatchMode::Immediate => {
                dispatch_native_panel_app_command(self.app_handle(), command)
            }
            NativePanelRuntimeDispatchMode::Scheduled => {
                spawn_native_panel_app_command(self.app.clone(), command, error_message);
                Ok(())
            }
        }
    }

    fn dispatch_command(
        &mut self,
        command: impl FnOnce() -> Result<(), String> + Send + 'static,
        error_message: &'static str,
    ) -> Result<(), String> {
        match self.dispatch_mode {
            NativePanelRuntimeDispatchMode::Immediate => dispatch_native_panel_command(command),
            NativePanelRuntimeDispatchMode::Scheduled => {
                spawn_native_panel_command(command, error_message);
                Ok(())
            }
        }
    }

    fn toggle_settings_surface_command(&mut self) -> Result<(), String> {
        (self.toggle_settings_surface)(&self.app)
    }
}

#[cfg(test)]
mod tests {
    use super::execute_native_panel_settings_surface_command;
    use crate::native_panel_renderer::{
        runtime_interaction::NativePanelSettingsSurfaceSnapshotUpdate,
        transition_controller::NativePanelTransitionRequest,
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
    fn settings_surface_command_dispatches_synced_update() {
        let mut dispatched = None;

        let changed = execute_native_panel_settings_surface_command(
            || {
                Ok(Some(NativePanelSettingsSurfaceSnapshotUpdate {
                    transition_request: Some(NativePanelTransitionRequest::SurfaceSwitch),
                    snapshot: Some(runtime_snapshot("idle", "Running")),
                }))
            },
            |request, snapshot| {
                dispatched = Some((request, snapshot.map(|snapshot| snapshot.status)));
                Ok(())
            },
        )
        .expect("dispatch settings surface update");

        assert!(changed);
        assert_eq!(
            dispatched,
            Some((
                Some(NativePanelTransitionRequest::SurfaceSwitch),
                Some("idle".to_string()),
            ))
        );
    }

    #[test]
    fn settings_surface_command_skips_dispatch_without_update() {
        let mut dispatched = false;

        let changed = execute_native_panel_settings_surface_command(
            || Ok(None),
            |_, _| {
                dispatched = true;
                Ok(())
            },
        )
        .expect("skip settings surface dispatch");

        assert!(!changed);
        assert!(!dispatched);
    }

    #[test]
    fn settings_surface_command_propagates_sync_error() {
        let error = execute_native_panel_settings_surface_command(
            || Err("sync failed".to_string()),
            |_, _| Ok(()),
        )
        .expect_err("surface sync errors should propagate");

        assert_eq!(error, "sync failed");
    }

    #[test]
    fn settings_surface_command_propagates_dispatch_error() {
        let error = execute_native_panel_settings_surface_command(
            || {
                Ok(Some(NativePanelSettingsSurfaceSnapshotUpdate {
                    transition_request: Some(NativePanelTransitionRequest::SurfaceSwitch),
                    snapshot: Some(runtime_snapshot("idle", "Running")),
                }))
            },
            |_, _| Err("dispatch failed".to_string()),
        )
        .expect_err("surface dispatch errors should propagate");

        assert_eq!(error, "dispatch failed");
    }
}
