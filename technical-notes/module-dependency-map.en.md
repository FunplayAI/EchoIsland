# Module Dependency and Responsibility Map

## Purpose

This note gives a quick view of both module dependencies and responsibility boundaries.

Last reviewed: `2026-04-15`

## Overview

The repository can currently be summarized as:

- a Rust core
- a Tauri desktop host
- tool adapters plus a hook bridge
- a web frontend presentation layer

A more accurate dependency picture today is:

```text
external tools / hooks / local scanning
        │
        ├─ hook -> hook-bridge -> ipc -> runtime
        └─ fallback scan -> adapters -> desktop -> runtime

runtime -> persistence

desktop -> runtime
desktop -> adapters
desktop -> ipc
desktop -> paths

desktop web -> tauri commands -> runtime / platform services
```

## Rust-side Modules

### `crates/core`

- protocol types
- session data structures
- normalization and core state foundations

### `crates/runtime`

- top-level runtime orchestration
- snapshot aggregation
- pending permission and question queues
- persistence synchronization

### `crates/persistence`

- session state save
- startup restore

### `crates/ipc`

- local TCP listen/send
- token auth
- payload size limits

### `crates/paths`

- user and app-state path rules
- hook config paths
- bridge log paths

### `crates/adapters`

- tool status queries
- hook install entry points
- fallback scan implementations

## Application-level Modules

### `apps/hook-bridge`

- receives external tool hook context
- converts it into normalized IPC events
- does not own UI behavior

### `apps/desktop/src-tauri`

- app startup
- command entry points
- scan loops
- terminal focus
- window surface control
- platform capability exposure

### `apps/desktop/web`

The frontend is already split by responsibility into:

- snapshot orchestration
- status queue behavior
- panel rendering
- action bindings
- mascot animation

## Current Dependency Assessment

From a cross-platform perspective, the most valuable separation to keep is:

- `core/runtime/persistence` as the platform-agnostic core
- `desktop/src-tauri` as the platform host and bridge layer
- `desktop/web` as a relatively independent UI layer

The most platform-sensitive areas are still:

- window behavior
- terminal focus and tab selection
- tool input capture and local integration

## Freshness Check

This revision corrects several older simplifications:

- not every input path goes through `adapters`
- `hook-bridge` is an independent ingress, not just an adapter detail
- the frontend is now split enough that a single `desktop-ui` label is too vague
