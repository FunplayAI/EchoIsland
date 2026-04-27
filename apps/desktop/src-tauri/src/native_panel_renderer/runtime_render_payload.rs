use echoisland_runtime::RuntimeSnapshot;
use tauri::AppHandle;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativePanelRuntimeRenderPayloadState {
    pub(crate) expanded: bool,
    pub(crate) shared_body_height: Option<f64>,
    pub(crate) transitioning: bool,
    pub(crate) transition_cards_progress: f64,
    pub(crate) transition_cards_entering: bool,
}

pub(crate) trait NativePanelRuntimeRenderPayloadStateBridge {
    fn runtime_render_payload_snapshot(&self) -> Option<RuntimeSnapshot>;

    fn runtime_render_payload_state(&self) -> NativePanelRuntimeRenderPayloadState;
}

pub(crate) fn resolve_native_panel_runtime_render_payload_for_state<S, P>(
    state: &S,
    build: impl FnOnce(RuntimeSnapshot, NativePanelRuntimeRenderPayloadState) -> P,
) -> Option<P>
where
    S: NativePanelRuntimeRenderPayloadStateBridge,
{
    let snapshot = state.runtime_render_payload_snapshot()?;
    Some(build(snapshot, state.runtime_render_payload_state()))
}

pub(crate) fn dispatch_native_panel_runtime_render_payload_if_available<R, P>(
    app: &AppHandle<R>,
    payload: Option<P>,
    dispatch: impl FnOnce(&AppHandle<R>, P) -> Result<(), String>,
) -> Result<bool, String>
where
    R: tauri::Runtime,
{
    let Some(payload) = payload else {
        return Ok(false);
    };

    dispatch(app, payload)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{
        NativePanelRuntimeRenderPayloadState, NativePanelRuntimeRenderPayloadStateBridge,
        resolve_native_panel_runtime_render_payload_for_state,
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
    fn runtime_render_payload_helper_projects_snapshot_and_runtime_state() {
        struct TestRenderPayloadState {
            snapshot: Option<RuntimeSnapshot>,
            state: NativePanelRuntimeRenderPayloadState,
        }

        impl NativePanelRuntimeRenderPayloadStateBridge for TestRenderPayloadState {
            fn runtime_render_payload_snapshot(&self) -> Option<RuntimeSnapshot> {
                self.snapshot.clone()
            }

            fn runtime_render_payload_state(&self) -> NativePanelRuntimeRenderPayloadState {
                self.state
            }
        }

        let projected = resolve_native_panel_runtime_render_payload_for_state(
            &TestRenderPayloadState {
                snapshot: Some(runtime_snapshot("idle", "Idle")),
                state: NativePanelRuntimeRenderPayloadState {
                    expanded: true,
                    shared_body_height: Some(180.0),
                    transitioning: true,
                    transition_cards_progress: 0.42,
                    transition_cards_entering: true,
                },
            },
            |snapshot, state| (snapshot.status, state),
        )
        .expect("render payload should exist");

        assert_eq!(projected.0, "idle".to_string());
        assert_eq!(
            projected.1,
            NativePanelRuntimeRenderPayloadState {
                expanded: true,
                shared_body_height: Some(180.0),
                transitioning: true,
                transition_cards_progress: 0.42,
                transition_cards_entering: true,
            }
        );
    }
}
