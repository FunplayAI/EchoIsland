mod adapter;
pub mod claude;
pub mod codex;
mod install_support;
pub mod openclaw;
mod platform_support;

pub use adapter::{AdapterPath, AdapterStatus, InstallableAdapter, SessionScanningAdapter};
pub use claude::{
    ClaudeAdapter, ClaudePaths, ClaudeSessionScanner, ClaudeStatus,
    default_paths as claude_default_paths, get_claude_status, install_claude_adapter,
    scan_claude_sessions,
};
pub use codex::{
    CodexAdapter, CodexPaths, CodexSessionScanner, CodexStatus, default_paths, get_codex_status,
    install_codex_adapter, scan_codex_sessions,
};
pub use openclaw::{
    OpenClawAdapter, OpenClawPaths, OpenClawStatus, default_paths as openclaw_default_paths,
    get_openclaw_status, install_openclaw_adapter,
};
