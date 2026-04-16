# Current Implementation Architecture

## Purpose

This note describes only what is already implemented in the repository today.

Last reviewed: `2026-04-15`

## Workspace Layout

Current workspace members:

- `apps/desktop/src-tauri`: Tauri desktop host for windowing, tray, commands, scan loops, and platform service wiring
- `apps/desktop-host`: console host used for early integration and CLI-side validation
- `apps/hook-bridge`: converts external hook payloads into EchoIsland IPC events
- `crates/core`: event protocol, normalization, and session data model
- `crates/runtime`: runtime orchestration, snapshot aggregation, pending queues, and persistence sync
- `crates/ipc`: local TCP IPC, auth, and request transport
- `crates/persistence`: session save and restore
- `crates/paths`: shared rules for config, state, log, and hook paths
- `crates/adapters`: tool adapter status detection, install logic, and fallback scan logic

## Main Flows

The current system is not a single pipeline. It has three main flows feeding `runtime`:

```text
external tool hooks
    -> hook-bridge
    -> ipc
    -> runtime

local fallback scanning
    -> adapters
    -> desktop scan runner
    -> runtime

frontend UI / Tauri commands
    -> desktop commands/services
    -> runtime / platform services
```

## Current Responsibility Boundaries

### `core`

- defines protocol types and core state models
- does not own platform behavior
- does not contain UI logic

### `runtime`

- merges sessions from multiple sources
- produces frontend snapshots
- maintains pending permission and question queues
- drives persistence saves

### `ipc`

- exposes the default local address `127.0.0.1:37891`
- enforces payload limits and token auth
- delivers external events into the runtime

### `adapters`

- exposes tool-specific status entry points for Codex, Claude, OpenClaw, and future adapters
- holds hook install, status probing, and fallback scan logic
- does not directly manage desktop UI state

### `apps/desktop/src-tauri`

The desktop host is already split into focused services:

- `commands.rs`: Tauri command entry points
- `app_runtime.rs`: app-level runtime ownership and wiring
- `session_scan_runner.rs`: background scan loop and watcher debounce
- `terminal_focus_service.rs`: terminal focus and binding
- `window_surface_service.rs`: island sizing, stage transitions, and visibility
- `platform.rs` / `platform_stub.rs`: platform capabilities and stub behavior

### `apps/desktop/web`

The frontend is no longer a single script. It is now split across:

- `snapshot/`: refresh orchestration, status queue, completion tracking
- `renderers/`: cards, headlines, and panel rendering
- `actions/`: user actions
- `mascot/`: mascot state machine and drawing
- `panel-controller.js` / `snapshot-controller.js`: panel and snapshot coordination

## Current Architecture Assessment

The current structure is substantially better than the early version, but it is still best described as “Windows-first with cross-platform seams prepared”.

Key facts:

- the runtime core is already relatively platform-agnostic
- platform-specific work is still concentrated around windows, terminal focus, input capture, and tool wiring
- the frontend is already modular enough for continued refactoring

## Freshness Check

This revision removes older wording that no longer matched the codebase, such as:

- vague `desktop-ui` naming
- older architecture wording that did not reflect platform capabilities, platform stubs, and service-level splits

This note currently matches the repository structure closely enough to remain a valid short-term reference.
