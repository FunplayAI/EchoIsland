use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use anyhow::Context;
use echoisland_core::{EventEnvelope, ResponseEnvelope};
use echoisland_ipc::{IpcAuth, default_token_path};
use echoisland_runtime::SharedRuntime;
use serde::Serialize;
use tauri::Emitter;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, tcp::OwnedReadHalf},
};
use tracing::warn;

use crate::app_runtime::maybe_refresh_native_ui_for_event;

pub const DEFAULT_HTTP_RECEIVER_ADDR: &str = "127.0.0.1:37892";
const MAX_HTTP_REQUEST_BYTES: usize = 1_048_576;
type DisconnectMonitor = Pin<Box<dyn Future<Output = bool> + Send>>;

#[derive(Debug, Clone, Serialize)]
pub struct HttpReceiverStatus {
    pub addr: String,
    pub event_path: String,
    pub token_path: String,
}

pub fn default_http_receiver_status() -> HttpReceiverStatus {
    HttpReceiverStatus {
        addr: DEFAULT_HTTP_RECEIVER_ADDR.to_string(),
        event_path: "/event".to_string(),
        token_path: default_token_path().display().to_string(),
    }
}

pub fn spawn_http_receiver<R: tauri::Runtime + 'static>(
    app_handle: tauri::AppHandle<R>,
    runtime: Arc<SharedRuntime>,
) {
    tauri::async_runtime::spawn(async move {
        let auth = match IpcAuth::from_default_storage() {
            Ok(auth) => auth,
            Err(error) => {
                let _ = app_handle.emit(
                    "ipc-error",
                    format!("http receiver auth init failed: {error}"),
                );
                return;
            }
        };

        if let Err(error) = serve_http(
            DEFAULT_HTTP_RECEIVER_ADDR,
            app_handle.clone(),
            runtime,
            auth,
        )
        .await
        {
            let _ = app_handle.emit("ipc-error", format!("http receiver failed: {error}"));
        }
    });
}

async fn serve_http<R: tauri::Runtime + 'static>(
    addr: &str,
    app_handle: tauri::AppHandle<R>,
    runtime: Arc<SharedRuntime>,
    auth: IpcAuth,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind http receiver on {addr}"))?;
    tracing::info!("http receiver listening on {addr}");

    loop {
        let (socket, peer) = listener.accept().await?;
        let app_handle = app_handle.clone();
        let runtime = runtime.clone();
        let auth = auth.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(socket, app_handle, runtime, auth).await {
                warn!("http receiver connection {peer:?} failed: {error:#}");
            }
        });
    }
}

async fn handle_connection<R: tauri::Runtime + 'static>(
    mut socket: TcpStream,
    app_handle: tauri::AppHandle<R>,
    runtime: Arc<SharedRuntime>,
    auth: IpcAuth,
) -> anyhow::Result<()> {
    let request = read_http_request(&mut socket).await?;
    match decode_http_request(&request, &auth) {
        DecodedHttpRequest::Immediate(response) => {
            socket.write_all(&response).await?;
            socket.shutdown().await?;
        }
        DecodedHttpRequest::Event(event_request) => {
            maybe_refresh_native_ui_for_event(
                app_handle.clone(),
                runtime.clone(),
                &event_request.normalized,
            );
            let (read_half, mut write_half) = socket.into_split();
            match await_http_event_response(
                runtime.clone(),
                event_request.event,
                if is_blocking_event_name(&event_request.normalized) {
                    Some(Box::pin(wait_for_http_disconnect(read_half)) as DisconnectMonitor)
                } else {
                    None
                },
            )
            .await
            {
                EventExecutionResult::Response(response) => {
                    if is_blocking_event_name(&event_request.normalized) {
                        let snapshot = runtime.snapshot().await;
                        warn!(
                            event_name = %event_request.normalized,
                            pending_permission_count = snapshot.pending_permission_count,
                            pending_question_count = snapshot.pending_question_count,
                            active_session_count = snapshot.active_session_count,
                            "http receiver applied pending event"
                        );
                    }
                    maybe_refresh_native_ui_for_event(
                        app_handle,
                        runtime,
                        &event_request.normalized,
                    );
                    let encoded = json_http_response(200, &response);
                    write_half.write_all(&encoded).await?;
                    write_half.shutdown().await?;
                }
                EventExecutionResult::Disconnected => {
                    maybe_refresh_native_ui_for_event(app_handle, runtime, "PeerDisconnect");
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
struct HttpEventRequest {
    event: EventEnvelope,
    normalized: String,
}

enum DecodedHttpRequest {
    Immediate(Vec<u8>),
    Event(HttpEventRequest),
}

fn decode_http_request(request_bytes: &[u8], auth: &IpcAuth) -> DecodedHttpRequest {
    let request = match parse_http_request(request_bytes) {
        Ok(request) => request,
        Err(error) => {
            return DecodedHttpRequest::Immediate(json_http_response(
                400,
                &ResponseEnvelope::error(format!("bad_request:{error}")),
            ));
        }
    };

    if request.method == "GET" && request.path == "/health" {
        return DecodedHttpRequest::Immediate(ok_json_bytes(serde_json::json!({
            "ok": true,
            "service": "codeisland-http-receiver",
            "event_path": "/event"
        })));
    }

    if request.method != "POST" || request.path != "/event" {
        return DecodedHttpRequest::Immediate(json_http_response(
            404,
            &ResponseEnvelope::error("not_found"),
        ));
    }

    let provided_token = request
        .headers
        .get("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_string)
        .or_else(|| request.headers.get("x-codeisland-token").cloned())
        .or_else(|| request.headers.get("x-echoisland-token").cloned())
        .or_else(|| {
            serde_json::from_slice::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|value| {
                    value
                        .get("token")
                        .and_then(|token| token.as_str())
                        .map(str::to_string)
                })
        });

    if provided_token.as_deref() != Some(auth.token()) {
        return DecodedHttpRequest::Immediate(json_http_response(
            401,
            &ResponseEnvelope::error("unauthorized"),
        ));
    }

    let event = match parse_event_body(&request.body) {
        Ok(event) => event,
        Err(response) => return DecodedHttpRequest::Immediate(json_http_response(400, &response)),
    };

    let normalized = event.normalized_event_name();
    warn!(
        method = %request.method,
        path = %request.path,
        event_name = %normalized,
        body_len = request.body.len(),
        "http receiver received event"
    );
    DecodedHttpRequest::Event(HttpEventRequest { event, normalized })
}

enum EventExecutionResult {
    Response(ResponseEnvelope),
    Disconnected,
}

async fn await_http_event_response(
    runtime: Arc<SharedRuntime>,
    event: EventEnvelope,
    disconnect_monitor: Option<DisconnectMonitor>,
) -> EventExecutionResult {
    if let Some(disconnect_monitor) = disconnect_monitor {
        let session_id = event.session_id.clone();
        let mut response_task = tokio::spawn({
            let runtime = runtime.clone();
            async move { runtime.handle_event(event).await }
        });
        tokio::select! {
            response = &mut response_task => EventExecutionResult::Response(
                response.unwrap_or_else(|_| ResponseEnvelope::error("handler_cancelled"))
            ),
            disconnected = disconnect_monitor => {
                if disconnected {
                    runtime.handle_peer_disconnect(&session_id).await;
                    let _ = response_task.await;
                    EventExecutionResult::Disconnected
                } else {
                    EventExecutionResult::Response(
                        response_task
                            .await
                            .unwrap_or_else(|_| ResponseEnvelope::error("handler_cancelled"))
                    )
                }
            }
        }
    } else {
        EventExecutionResult::Response(runtime.handle_event(event).await)
    }
}

fn is_blocking_event_name(normalized: &str) -> bool {
    matches!(normalized, "PermissionRequest" | "AskUserQuestion")
}

fn parse_event_body(body: &[u8]) -> Result<EventEnvelope, ResponseEnvelope> {
    let value = serde_json::from_slice::<serde_json::Value>(body)
        .map_err(|_| ResponseEnvelope::parse_failed())?;
    let event_value = value.get("event").cloned().unwrap_or(value);
    let event: EventEnvelope =
        serde_json::from_value(event_value).map_err(|_| ResponseEnvelope::parse_failed())?;
    event
        .validate()
        .map_err(|_| ResponseEnvelope::invalid_payload())?;
    Ok(event)
}

struct ParsedHttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn parse_http_request(bytes: &[u8]) -> anyhow::Result<ParsedHttpRequest> {
    let separator = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("missing header separator"))?;
    let (header_bytes, body_bytes) = bytes.split_at(separator + 4);
    let header_text =
        String::from_utf8(header_bytes[..separator].to_vec()).context("headers are not utf-8")?;
    let mut lines = header_text.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing request line"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing method"))?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing path"))?
        .to_string();

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let declared_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(body_bytes.len());
    if body_bytes.len() < declared_length {
        anyhow::bail!("incomplete request body");
    }

    Ok(ParsedHttpRequest {
        method,
        path,
        headers,
        body: body_bytes[..declared_length].to_vec(),
    })
}

async fn read_http_request(socket: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    let mut header_end = None;
    let mut expected_total_len = None;

    loop {
        let read = socket.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.len() > MAX_HTTP_REQUEST_BYTES {
            anyhow::bail!("http request exceeds {MAX_HTTP_REQUEST_BYTES} bytes");
        }

        if header_end.is_none() {
            header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|index| index + 4);
            if let Some(header_end_pos) = header_end {
                let header_text = String::from_utf8_lossy(&buffer[..header_end_pos]);
                let content_length = header_text
                    .lines()
                    .find_map(|line| {
                        line.split_once(':').and_then(|(name, value)| {
                            if name.trim().eq_ignore_ascii_case("content-length") {
                                value.trim().parse::<usize>().ok()
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or(0);
                expected_total_len = Some(header_end_pos + content_length);
            }
        }

        if let Some(expected_total_len) = expected_total_len {
            if buffer.len() >= expected_total_len {
                break;
            }
        }
    }

    Ok(buffer)
}

async fn wait_for_http_disconnect(mut reader: OwnedReadHalf) -> bool {
    let mut probe = [0_u8; 1];
    match reader.read(&mut probe).await {
        Ok(0) => true,
        Ok(_) => true,
        Err(error) => {
            warn!(error = %error, "http disconnect monitor ended with read error");
            true
        }
    }
}

fn ok_json_bytes(value: serde_json::Value) -> Vec<u8> {
    let body = serde_json::to_vec(&value).unwrap_or_else(|_| b"{\"ok\":false}".to_vec());
    http_response(200, "application/json", &body)
}

fn json_http_response(status: u16, response: &ResponseEnvelope) -> Vec<u8> {
    let body = serde_json::to_vec(response).unwrap_or_else(|_| b"{\"ok\":false}".to_vec());
    http_response(status, "application/json", &body)
}

fn http_response(status: u16, content_type: &str, body: &[u8]) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "Error",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use echoisland_core::{EventEnvelope, PROTOCOL_VERSION};
    use echoisland_runtime::SharedRuntime;
    use serde_json::json;
    use tokio::sync::oneshot;

    use super::{
        DecodedHttpRequest, EventExecutionResult, IpcAuth, await_http_event_response,
        decode_http_request, parse_http_request,
    };

    #[test]
    fn parses_http_request_with_headers() {
        let request = b"POST /event HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\nX-CodeIsland-Token: secret\r\n\r\n{}";
        let parsed = parse_http_request(request).unwrap();
        assert_eq!(parsed.method, "POST");
        assert_eq!(parsed.path, "/event");
        assert_eq!(
            parsed.headers.get("x-codeisland-token").map(String::as_str),
            Some("secret")
        );
        assert_eq!(parsed.body, b"{}");
    }

    #[tokio::test]
    async fn accepts_authenticated_http_event() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let storage_path = std::env::temp_dir().join(format!("echoisland-http-test-{suffix}.json"));
        let runtime = Arc::new(SharedRuntime::with_storage_path(storage_path.clone()));
        let auth = IpcAuth::from_token("secret");
        let body = serde_json::to_string(&json!({
            "protocol_version": PROTOCOL_VERSION,
            "hook_event_name": "SessionStart",
            "source": "opencode",
            "session_id": "open-1",
            "timestamp": chrono::Utc::now(),
        }))
        .unwrap();
        let request = format!(
            "POST /event HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let decoded = decode_http_request(request.as_bytes(), &auth);
        let event = match decoded {
            DecodedHttpRequest::Event(request) => request.event,
            DecodedHttpRequest::Immediate(_) => panic!("expected event request"),
        };
        let response = await_http_event_response(runtime.clone(), event, None).await;
        let response = match response {
            EventExecutionResult::Response(response) => response,
            EventExecutionResult::Disconnected => panic!("request should not disconnect"),
        };
        assert!(response.ok);
        let snapshot = runtime.snapshot().await;
        assert_eq!(snapshot.total_session_count, 1);
        let _ = std::fs::remove_file(storage_path);
    }

    #[tokio::test]
    async fn blocking_http_request_is_cleared_when_client_disconnects() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let storage_path =
            std::env::temp_dir().join(format!("echoisland-http-disconnect-test-{suffix}.json"));
        let runtime = Arc::new(SharedRuntime::with_storage_path(storage_path.clone()));
        let event = EventEnvelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            hook_event_name: "PermissionRequest".to_string(),
            session_id: "http-session".to_string(),
            source: "claude".to_string(),
            timestamp: chrono::Utc::now(),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(json!({ "command": "ls" })),
            cwd: Some("/tmp".to_string()),
            model: None,
            message: None,
            agent_id: None,
            metadata: None,
            question: None,
        };
        let (disconnect_tx, disconnect_rx) = oneshot::channel::<()>();
        let response_task = tokio::spawn({
            let runtime = runtime.clone();
            async move {
                await_http_event_response(
                    runtime,
                    event,
                    Some(Box::pin(async move {
                        let _ = disconnect_rx.await;
                        true
                    })),
                )
                .await
            }
        });

        for _ in 0..50 {
            if runtime.snapshot().await.pending_permission_count == 1 {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        assert_eq!(runtime.snapshot().await.pending_permission_count, 1);
        let _ = disconnect_tx.send(());

        let result = response_task.await.unwrap();
        assert!(matches!(result, EventExecutionResult::Disconnected));

        let snapshot = runtime.snapshot().await;
        assert_eq!(snapshot.pending_permission_count, 0);
        assert_eq!(snapshot.pending_question_count, 0);
        assert_eq!(snapshot.sessions[0].status, "Processing");
        let _ = std::fs::remove_file(storage_path);
    }
}
