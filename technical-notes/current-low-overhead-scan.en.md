# Current Low-Overhead Scan Design

## Purpose

This note explains how the current Windows Codex fallback scanner stays low overhead.

Last reviewed: `2026-04-15`

## Current State

The implementation has already moved away from “scan local files whenever the UI refreshes” to “scan in the background, let the UI read snapshots only”.

That means:

- frontend snapshot reads no longer trigger full disk scans
- scan frequency is decoupled from UI refresh frequency
- file watchers drive the fast path, while polling is only a fallback

## Current Flow

```text
Codex local file changes / fallback polling
        │
        ▼
desktop background scan loop
        │
        ▼
CodexSessionScanner
  ├─ incremental `history.jsonl` reading
  ├─ session file `mtime + size` checks
  ├─ bounded recent-session scanning
  └─ cached `SessionRecord` diffing
        │
        ▼
runtime.sync_source_sessions("codex", ...)
        │
        ▼
frontend `get_snapshot` only reads runtime snapshot state
```

## Where the Low Cost Comes From

### 1. A dedicated scan loop

The background loop in `apps/desktop/src-tauri/src/session_scan_runner.rs` now:

- reacts quickly to watcher events
- falls back to interval polling when idle
- debounces watcher bursts

### 2. Incremental `history.jsonl` reads

`crates/adapters/src/codex/scan.rs` keeps explicit history scan state:

- `size`
- `modified_at`
- `offset`
- `latest_prompt_by_session`

It continues from the last offset when possible instead of rereading the whole file each time.

### 3. Session files are reparsed only when changed

Session files are not fully reparsed every round. The scanner checks:

- file size
- modification time

Only changed files are reparsed.

### 4. Active and idle scan intervals are split

The current recommended intervals are:

- active: about `3s`
- idle: about `15s`

So the scanner stays responsive during activity and cheaper when idle.

### 5. The UI is snapshot-only

`get_snapshot` currently reads `runtime.snapshot()` and no longer piggybacks fallback scanning.

This is important because it avoids:

- extra scans caused by hover behavior
- scan cascades caused by frequent UI animation refreshes

## Current Scope

This note currently describes:

- the Windows Codex fallback scanner

It is not yet the final unified strategy for all tools, and it is not a replacement for future hook-first integrations.

## Freshness Check

No obvious stale claims were found in this note during the current review, but it should be revisited when:

- Codex Windows native hooks become practically usable and the flow changes to “hooks first, scan fallback”
- local scanning expands into a broader Claude or OpenClaw scanner strategy
