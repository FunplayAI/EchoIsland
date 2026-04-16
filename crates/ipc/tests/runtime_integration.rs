use std::{
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use chrono::Utc;
use echoisland_core::{EventEnvelope, PROTOCOL_VERSION, ResponseEnvelope};
use echoisland_ipc::{EventHandler, IpcAuth, send_raw_with_auth, serve_tcp_with_auth};
use echoisland_runtime::SharedRuntime;
use serde_json::json;
use tokio::time::{Duration, Instant, sleep};

struct RuntimeEventHandler {
    runtime: Arc<SharedRuntime>,
}

#[async_trait]
impl EventHandler for RuntimeEventHandler {
    async fn handle_event(&self, event: EventEnvelope) -> ResponseEnvelope {
        self.runtime.handle_event(event).await
    }

    async fn handle_disconnect(&self, session_id: &str) {
        let _ = self.runtime.handle_peer_disconnect(session_id).await;
    }
}

fn temp_state_path() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("echoisland-ipc-runtime-test-{suffix}.json"))
}

fn free_local_addr() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    drop(listener);
    addr
}

fn session_start_payload() -> Vec<u8> {
    let now = Utc::now().to_rfc3339();
    serde_json::to_vec(&json!({
        "protocol_version": PROTOCOL_VERSION,
        "hook_event_name": "SessionStart",
        "source": "codex",
        "session_id": "session-1",
        "timestamp": now,
        "cwd": "D:/AI Island/project",
        "model": "gpt-5.4"
    }))
    .unwrap()
}

fn permission_request_payload() -> Vec<u8> {
    let now = Utc::now().to_rfc3339();
    serde_json::to_vec(&json!({
        "protocol_version": PROTOCOL_VERSION,
        "hook_event_name": "PermissionRequest",
        "source": "claude",
        "session_id": "session-permission",
        "timestamp": now,
        "tool_name": "Bash",
        "tool_input": {
          "description": "Run formatter"
        }
    }))
    .unwrap()
}

fn question_request_payload() -> Vec<u8> {
    let now = Utc::now().to_rfc3339();
    serde_json::to_vec(&json!({
        "protocol_version": PROTOCOL_VERSION,
        "hook_event_name": "AskUserQuestion",
        "source": "claude",
        "session_id": "session-question",
        "timestamp": now,
        "question": {
          "header": "Environment",
          "text": "Where should I deploy?",
          "options": [
            {
              "label": "staging",
              "description": "Safer"
            }
          ]
        }
    }))
    .unwrap()
}

fn invalid_payload() -> Vec<u8> {
    let now = Utc::now().to_rfc3339();
    serde_json::to_vec(&json!({
        "protocol_version": "999",
        "hook_event_name": "SessionStart",
        "source": "codex",
        "session_id": "session-invalid",
        "timestamp": now
    }))
    .unwrap()
}

async fn wait_for_snapshot<F>(runtime: &SharedRuntime, predicate: F)
where
    F: Fn(&echoisland_runtime::RuntimeSnapshot) -> bool,
{
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let snapshot = runtime.snapshot().await;
        if predicate(&snapshot) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for runtime snapshot condition"
        );
        sleep(Duration::from_millis(20)).await;
    }
}

async fn start_server(
    runtime: Arc<SharedRuntime>,
    auth: IpcAuth,
) -> (String, tokio::task::JoinHandle<anyhow::Result<()>>) {
    let addr = free_local_addr();
    let handler = Arc::new(RuntimeEventHandler { runtime });
    let server_addr = addr.clone();
    let handle =
        tokio::spawn(async move { serve_tcp_with_auth(&server_addr, handler, auth).await });
    sleep(Duration::from_millis(80)).await;
    (addr, handle)
}

async fn wait_for_pending_to_clear(runtime: &SharedRuntime) {
    wait_for_snapshot(runtime, |snapshot| {
        snapshot.pending_permission_count == 0 && snapshot.pending_question_count == 0
    })
    .await;
}

#[tokio::test]
async fn tcp_event_updates_runtime_snapshot() {
    let path = temp_state_path();
    let runtime = Arc::new(SharedRuntime::with_storage_path(path.clone()));
    let auth = IpcAuth::from_token("secret");
    let (addr, server) = start_server(runtime.clone(), auth.clone()).await;

    let response = send_raw_with_auth(&addr, &session_start_payload(), &auth)
        .await
        .unwrap();
    assert!(response.ok);

    wait_for_snapshot(&runtime, |snapshot| {
        snapshot.total_session_count == 1
            && snapshot.active_session_count == 1
            && snapshot
                .sessions
                .iter()
                .any(|session| session.session_id == "session-1" && session.status == "Processing")
    })
    .await;

    server.abort();
    runtime.flush_persistence().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn tcp_permission_request_round_trip_completes_after_approval() {
    let path = temp_state_path();
    let runtime = Arc::new(SharedRuntime::with_storage_path(path.clone()));
    let auth = IpcAuth::from_token("secret");
    let (addr, server) = start_server(runtime.clone(), auth.clone()).await;

    let send_task = tokio::spawn({
        let auth = auth.clone();
        let addr = addr.clone();
        async move {
            send_raw_with_auth(&addr, &permission_request_payload(), &auth)
                .await
                .unwrap()
        }
    });

    wait_for_snapshot(&runtime, |snapshot| snapshot.pending_permission_count == 1).await;
    let request_id = runtime
        .snapshot()
        .await
        .pending_permission
        .map(|pending| pending.request_id)
        .expect("missing pending permission");

    runtime.approve_permission(&request_id).await.unwrap();

    let response = send_task.await.unwrap();
    assert!(response.ok);
    assert_eq!(
        response
            .decision
            .as_ref()
            .map(|value| value.behavior.as_str()),
        Some("allow")
    );

    wait_for_snapshot(&runtime, |snapshot| {
        snapshot.pending_permission_count == 0
            && snapshot.sessions.iter().any(|session| {
                session.session_id == "session-permission" && session.status == "Running"
            })
    })
    .await;

    server.abort();
    runtime.flush_persistence().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn tcp_question_request_round_trip_returns_answer() {
    let path = temp_state_path();
    let runtime = Arc::new(SharedRuntime::with_storage_path(path.clone()));
    let auth = IpcAuth::from_token("secret");
    let (addr, server) = start_server(runtime.clone(), auth.clone()).await;

    let send_task = tokio::spawn({
        let auth = auth.clone();
        let addr = addr.clone();
        async move {
            send_raw_with_auth(&addr, &question_request_payload(), &auth)
                .await
                .unwrap()
        }
    });

    wait_for_snapshot(&runtime, |snapshot| snapshot.pending_question_count == 1).await;
    let request_id = runtime
        .snapshot()
        .await
        .pending_question
        .map(|pending| pending.request_id)
        .expect("missing pending question");

    runtime
        .answer_question(&request_id, "staging")
        .await
        .unwrap();

    let response = send_task.await.unwrap();
    assert!(response.ok);
    assert_eq!(
        response
            .answer
            .as_ref()
            .and_then(|value| value.value.as_deref()),
        Some("staging")
    );
    assert_eq!(
        response.answer.as_ref().map(|value| value.skipped),
        Some(false)
    );

    wait_for_snapshot(&runtime, |snapshot| {
        snapshot.pending_question_count == 0
            && snapshot.sessions.iter().any(|session| {
                session.session_id == "session-question"
                    && session.status == "Processing"
                    && session.last_assistant_message.as_deref() == Some("staging")
            })
    })
    .await;

    server.abort();
    runtime.flush_persistence().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn tcp_permission_request_is_cleared_when_client_disconnects() {
    let path = temp_state_path();
    let runtime = Arc::new(SharedRuntime::with_storage_path(path.clone()));
    let auth = IpcAuth::from_token("secret");
    let (addr, server) = start_server(runtime.clone(), auth.clone()).await;

    let send_task = tokio::spawn({
        let auth = auth.clone();
        let addr = addr.clone();
        async move {
            let _ = send_raw_with_auth(&addr, &permission_request_payload(), &auth).await;
        }
    });

    wait_for_snapshot(&runtime, |snapshot| snapshot.pending_permission_count == 1).await;
    send_task.abort();

    wait_for_pending_to_clear(&runtime).await;
    wait_for_snapshot(&runtime, |snapshot| {
        snapshot.sessions.iter().any(|session| {
            session.session_id == "session-permission" && session.status == "Processing"
        })
    })
    .await;

    server.abort();
    runtime.flush_persistence().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn tcp_invalid_authenticated_payload_returns_invalid_payload() {
    let path = temp_state_path();
    let runtime = Arc::new(SharedRuntime::with_storage_path(path.clone()));
    let auth = IpcAuth::from_token("secret");
    let (addr, server) = start_server(runtime.clone(), auth.clone()).await;

    let response = send_raw_with_auth(&addr, &invalid_payload(), &auth)
        .await
        .unwrap();
    assert!(!response.ok);
    assert_eq!(response.error.as_deref(), Some("invalid_payload"));

    server.abort();
    runtime.flush_persistence().await;
    let _ = std::fs::remove_file(path);
}
