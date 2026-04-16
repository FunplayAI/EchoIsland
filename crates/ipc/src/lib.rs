use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use echoisland_core::{EventEnvelope, ResponseEnvelope};
use echoisland_paths::current_platform_paths;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, tcp::OwnedReadHalf},
};
use tracing::{debug, error, warn};
use uuid::Uuid;

pub const DEFAULT_ADDR: &str = "127.0.0.1:37891";
pub const MAX_PAYLOAD_BYTES: usize = 1_048_576;
const FRAME_MAGIC: &[u8; 4] = b"EIPC";

#[derive(Debug, Clone)]
pub struct IpcAuth {
    token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthenticatedEventEnvelope {
    token: String,
    event: EventEnvelope,
}

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn handle_event(&self, event: EventEnvelope) -> ResponseEnvelope;

    async fn handle_disconnect(&self, _session_id: &str) {}
}

pub async fn serve_tcp(addr: &str, handler: Arc<dyn EventHandler>) -> anyhow::Result<()> {
    let auth = IpcAuth::from_default_storage()?;
    serve_tcp_with_auth(addr, handler, auth).await
}

pub async fn serve_tcp_with_auth(
    addr: &str,
    handler: Arc<dyn EventHandler>,
    auth: IpcAuth,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!("listening on {addr}");

    loop {
        let (socket, peer) = listener.accept().await?;
        let handler = handler.clone();
        let auth = auth.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(socket, handler, auth).await {
                warn!("connection {peer:?} failed: {error:#}");
            }
        });
    }
}

pub async fn send_raw(addr: &str, payload: &[u8]) -> anyhow::Result<ResponseEnvelope> {
    let auth = IpcAuth::from_existing_storage()?;
    send_raw_with_auth(addr, payload, &auth).await
}

pub async fn send_raw_with_auth(
    addr: &str,
    payload: &[u8],
    auth: &IpcAuth,
) -> anyhow::Result<ResponseEnvelope> {
    let event = serde_json::from_slice::<EventEnvelope>(payload)
        .context("failed to parse event payload")?;
    let request = AuthenticatedEventEnvelope {
        token: auth.token.clone(),
        event,
    };
    let encoded = serde_json::to_vec(&request).context("failed to encode ipc request")?;
    let mut stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("failed to connect to {addr}"))?;
    if encoded.len() > MAX_PAYLOAD_BYTES {
        anyhow::bail!("payload exceeds {MAX_PAYLOAD_BYTES} bytes");
    }
    stream.write_all(FRAME_MAGIC).await?;
    stream.write_u32(encoded.len() as u32).await?;
    stream.write_all(&encoded).await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    let parsed = serde_json::from_slice::<ResponseEnvelope>(&response)
        .context("failed to parse response")?;
    Ok(parsed)
}

async fn handle_connection(
    socket: TcpStream,
    handler: Arc<dyn EventHandler>,
    auth: IpcAuth,
) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let request = read_request(reader).await?;
    let maybe_blocking_session_id = decode_authenticated_event(&request.payload, &auth)
        .ok()
        .and_then(|event| {
            if is_blocking_event(&event) {
                Some(event.session_id)
            } else {
                None
            }
        });

    let mut response_task = tokio::spawn({
        let handler = handler.clone();
        let auth = auth.clone();
        let payload = request.payload.clone();
        async move { process_payload(&payload, handler, &auth).await }
    });

    let response = if let (Some(session_id), Some(read_half)) =
        (maybe_blocking_session_id, request.disconnect_monitor)
    {
        tokio::select! {
            response = &mut response_task => Some(response.unwrap_or_else(|_| ResponseEnvelope::error("handler_cancelled"))),
            disconnected = wait_for_peer_disconnect(read_half) => {
                if disconnected {
                    handler.handle_disconnect(&session_id).await;
                }
                let _ = response_task.await;
                None
            }
        }
    } else {
        Some(
            response_task
                .await
                .unwrap_or_else(|_| ResponseEnvelope::error("handler_cancelled")),
        )
    };

    if let Some(response) = response {
        let encoded = serde_json::to_vec(&response)?;
        writer.write_all(&encoded).await?;
        writer.shutdown().await?;
    }
    Ok(())
}

async fn process_payload(
    payload: &[u8],
    handler: Arc<dyn EventHandler>,
    auth: &IpcAuth,
) -> ResponseEnvelope {
    match serde_json::from_slice::<AuthenticatedEventEnvelope>(payload) {
        Ok(request) => {
            if request.token != auth.token {
                return ResponseEnvelope::error("unauthorized");
            }
            match request.event.validate() {
                Ok(()) => handler.handle_event(request.event).await,
                Err(error) => {
                    debug!("invalid payload: {error}");
                    ResponseEnvelope::invalid_payload()
                }
            }
        }
        Err(wrapper_error) => match serde_json::from_slice::<EventEnvelope>(payload) {
            Ok(_) => {
                debug!("unauthorized legacy payload: {wrapper_error}");
                ResponseEnvelope::error("unauthorized")
            }
            Err(error) => {
                error!("parse failed: {error}");
                ResponseEnvelope::parse_failed()
            }
        },
    }
}

#[derive(Debug)]
struct IncomingRequest {
    payload: Vec<u8>,
    disconnect_monitor: Option<OwnedReadHalf>,
}

async fn read_request(mut reader: OwnedReadHalf) -> anyhow::Result<IncomingRequest> {
    let mut magic = [0_u8; 4];
    reader.read_exact(&mut magic).await?;
    if &magic == FRAME_MAGIC {
        let payload_len = reader.read_u32().await? as usize;
        if payload_len > MAX_PAYLOAD_BYTES {
            anyhow::bail!("payload exceeds {MAX_PAYLOAD_BYTES} bytes");
        }
        let mut payload = vec![0_u8; payload_len];
        reader.read_exact(&mut payload).await?;
        return Ok(IncomingRequest {
            payload,
            disconnect_monitor: Some(reader),
        });
    }

    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    buffer.extend_from_slice(&magic);
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.len() > MAX_PAYLOAD_BYTES {
            return Err(anyhow::anyhow!("payload exceeds {MAX_PAYLOAD_BYTES} bytes"));
        }
    }
    Ok(IncomingRequest {
        payload: buffer,
        disconnect_monitor: None,
    })
}

fn decode_authenticated_event(
    payload: &[u8],
    auth: &IpcAuth,
) -> Result<EventEnvelope, ResponseEnvelope> {
    match serde_json::from_slice::<AuthenticatedEventEnvelope>(payload) {
        Ok(request) => {
            if request.token != auth.token {
                return Err(ResponseEnvelope::error("unauthorized"));
            }
            request
                .event
                .validate()
                .map_err(|_| ResponseEnvelope::invalid_payload())?;
            Ok(request.event)
        }
        Err(wrapper_error) => match serde_json::from_slice::<EventEnvelope>(payload) {
            Ok(_) => {
                debug!("unauthorized legacy payload: {wrapper_error}");
                Err(ResponseEnvelope::error("unauthorized"))
            }
            Err(error) => {
                error!("parse failed: {error}");
                Err(ResponseEnvelope::parse_failed())
            }
        },
    }
}

fn is_blocking_event(event: &EventEnvelope) -> bool {
    matches!(
        event.normalized_event_name().as_str(),
        "PermissionRequest" | "AskUserQuestion"
    )
}

async fn wait_for_peer_disconnect(mut reader: OwnedReadHalf) -> bool {
    let mut probe = [0_u8; 1];
    match reader.read(&mut probe).await {
        Ok(0) => true,
        Ok(_) => true,
        Err(error) => {
            debug!(error = %error, "peer disconnect monitor ended with read error");
            true
        }
    }
}

impl IpcAuth {
    pub fn from_default_storage() -> anyhow::Result<Self> {
        let path = default_token_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let token = if path.exists() {
            let token = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let token = token.trim().to_string();
            if token.is_empty() {
                let token = Uuid::new_v4().to_string();
                fs::write(&path, &token)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                token
            } else {
                token
            }
        } else {
            let token = Uuid::new_v4().to_string();
            fs::write(&path, &token)
                .with_context(|| format!("failed to write {}", path.display()))?;
            token
        };

        Ok(Self { token })
    }

    pub fn from_existing_storage() -> anyhow::Result<Self> {
        let path = default_token_path();
        let token = fs::read_to_string(&path)
            .with_context(|| format!("failed to read ipc token from {}", path.display()))?;
        let token = token.trim().to_string();
        if token.is_empty() {
            anyhow::bail!("ipc token file is empty: {}", path.display());
        }
        Ok(Self { token })
    }

    pub fn from_token(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

pub fn default_token_path() -> PathBuf {
    current_platform_paths().ipc_token_path
}

#[cfg(test)]
mod tests {
    use super::{IpcAuth, process_payload};
    use std::sync::Arc;

    use async_trait::async_trait;
    use echoisland_core::{EventEnvelope, PROTOCOL_VERSION, ResponseEnvelope};
    use serde_json::json;

    struct EchoHandler;

    #[async_trait]
    impl super::EventHandler for EchoHandler {
        async fn handle_event(&self, _event: EventEnvelope) -> ResponseEnvelope {
            ResponseEnvelope::ok()
        }
    }

    fn sample_event() -> serde_json::Value {
        json!({
            "protocol_version": PROTOCOL_VERSION,
            "hook_event_name": "SessionStart",
            "source": "codex",
            "session_id": "session-1",
            "timestamp": "2026-04-10T12:00:00Z"
        })
    }

    #[tokio::test]
    async fn rejects_legacy_unauthenticated_payload() {
        let handler = Arc::new(EchoHandler);
        let auth = IpcAuth::from_token("secret");
        let payload = serde_json::to_vec(&sample_event()).unwrap();

        let response = process_payload(&payload, handler, &auth).await;

        assert!(!response.ok);
        assert_eq!(response.error.as_deref(), Some("unauthorized"));
    }

    #[tokio::test]
    async fn accepts_authenticated_payload() {
        let handler = Arc::new(EchoHandler);
        let auth = IpcAuth::from_token("secret");
        let payload = serde_json::to_vec(&json!({
            "token": "secret",
            "event": sample_event()
        }))
        .unwrap();

        let response = process_payload(&payload, handler, &auth).await;

        assert!(response.ok);
        assert_eq!(response.error, None);
    }
}
