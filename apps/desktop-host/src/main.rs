use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, bail};
use async_trait::async_trait;
use echoisland_adapters::{
    ClaudeAdapter, CodexAdapter, InstallableAdapter, OpenClawAdapter, SessionScanningAdapter,
};
use echoisland_core::{EventEnvelope, ResponseEnvelope};
use echoisland_ipc::{DEFAULT_ADDR, EventHandler, send_raw, serve_tcp};
use echoisland_paths::bridge_binary_name;
use echoisland_runtime::SharedRuntime;
use tokio::fs;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug)]
struct HostRuntime {
    runtime: SharedRuntime,
}

impl Default for HostRuntime {
    fn default() -> Self {
        Self {
            runtime: SharedRuntime::new(),
        }
    }
}

#[async_trait]
impl EventHandler for HostRuntime {
    async fn handle_event(&self, event: EventEnvelope) -> ResponseEnvelope {
        self.runtime.handle_event(event).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("serve") => {
            let addr = args.next().unwrap_or_else(|| DEFAULT_ADDR.to_string());
            serve_tcp(&addr, Arc::new(HostRuntime::default())).await
        }
        Some("send") => {
            let mut addr = DEFAULT_ADDR.to_string();
            let mut file: Option<PathBuf> = None;
            let mut from_stdin = false;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--addr" => {
                        addr = args.next().context("missing value after --addr")?;
                    }
                    "--file" => {
                        file = Some(PathBuf::from(
                            args.next().context("missing value after --file")?,
                        ));
                    }
                    "--stdin" => from_stdin = true,
                    other => bail!("unknown arg: {other}"),
                }
            }

            let payload = if let Some(path) = file {
                fs::read(path).await?
            } else if from_stdin {
                read_stdin().await?
            } else {
                bail!("use --file <path> or --stdin")
            };

            let response = send_raw(&addr, &payload).await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
            Ok(())
        }
        Some("snapshot") => {
            let runtime = SharedRuntime::new();
            let snapshot = runtime.snapshot().await;
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
            Ok(())
        }
        Some("codex-status") => {
            let status = CodexAdapter::with_default_paths().status()?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Some("claude-status") => {
            let status = ClaudeAdapter::with_default_paths().status()?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Some("openclaw-status") => {
            let status = OpenClawAdapter::with_default_paths().status()?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Some("codex-scan") => {
            let sessions = CodexAdapter::with_default_paths().scan_sessions()?;
            println!("{}", serde_json::to_string_pretty(&sessions)?);
            Ok(())
        }
        Some("claude-scan") => {
            let sessions = ClaudeAdapter::with_default_paths().scan_sessions()?;
            println!("{}", serde_json::to_string_pretty(&sessions)?);
            Ok(())
        }
        Some("install-codex") => {
            let mut bridge_path: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--bridge" => {
                        bridge_path = Some(PathBuf::from(
                            args.next().context("missing value after --bridge")?,
                        ));
                    }
                    other => bail!("unknown arg: {other}"),
                }
            }

            let bridge_path = bridge_path.unwrap_or_else(default_bridge_path);
            if !bridge_path.exists() {
                bail!(
                    "bridge binary not found at {}. build it first with `cargo build -p echoisland-hook-bridge` or pass --bridge <path>",
                    bridge_path.display()
                );
            }

            let status = CodexAdapter::with_default_paths().install(&bridge_path)?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Some("install-claude") => {
            let mut bridge_path: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--bridge" => {
                        bridge_path = Some(PathBuf::from(
                            args.next().context("missing value after --bridge")?,
                        ));
                    }
                    other => bail!("unknown arg: {other}"),
                }
            }

            let bridge_path = bridge_path.unwrap_or_else(default_bridge_path);
            if !bridge_path.exists() {
                bail!(
                    "bridge binary not found at {}. build it first with `cargo build -p echoisland-hook-bridge` or pass --bridge <path>",
                    bridge_path.display()
                );
            }

            let status = ClaudeAdapter::with_default_paths().install(&bridge_path)?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Some("install-openclaw") => {
            let status =
                OpenClawAdapter::with_default_paths().install(std::path::Path::new("."))?;
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        Some(other) => bail!("unknown command: {other}"),
    }
}

fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}

async fn read_stdin() -> anyhow::Result<Vec<u8>> {
    use tokio::io::{self, AsyncReadExt};

    let mut stdin = io::stdin();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

fn default_bridge_path() -> PathBuf {
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from) {
        return target_dir.join("debug").join(bridge_binary_name());
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    root.join("target").join("debug").join(bridge_binary_name())
}
