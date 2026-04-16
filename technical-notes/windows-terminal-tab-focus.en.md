# Windows Terminal Tab Focus Notes

## Purpose

This note records how the current Windows Terminal tab focus logic works.

Last reviewed: `2026-04-15`

Goal:

- click a session card in the island
- focus the correct terminal window
- switch to the correct `Windows Terminal` tab when possible

This is a note about the current implementation, not the final cross-platform design.

## Entry Points

Frontend entry:

- `apps/desktop/web/main.js`

Tauri command entry:

- `apps/desktop/src-tauri/src/commands.rs`
- command: `focus_session_terminal`
- bind command: `bind_session_terminal`

Main Rust implementation areas:

- `apps/desktop/src-tauri/src/terminal_focus_service.rs`
- `apps/desktop/src-tauri/src/terminal_focus/`
- `apps/desktop/src-tauri/src/focus_store.rs`

## Current Data Model

### `SessionFocusTarget`

This is the target description for “which session should be focused”, including:

- `source`
- `project_name`
- `cwd`
- `terminal_app`
- `host_app`
- `window_title`
- `terminal_pid`

### `SessionTabCache`

This stores cached Windows Terminal tab data:

- `terminal_pid`
- `window_hwnd`
- `runtime_id`
- `title`

It is persisted in the local focus store and reused for future clicks.

## Current Behavior

### 1. Explicit binding

The user can run `bind_session_terminal` for a session:

- read the current foreground `Windows Terminal` tab
- store that tab info as the session binding

This path is the most direct and most reliable.

### 2. Automatic learning

`terminal_focus/learning.rs` currently performs two observations:

- `observe_foreground_terminal_tab`: records the recent foreground tab
- `learn_newly_active_session_tabs`: when exactly one newly active candidate exists in the snapshot, try to learn a binding from the foreground tab

Automatic learning currently depends on:

- whether the session became active
- whether `last_user_prompt` changed
- whether `last_activity` advanced

If multiple candidates exist, learning is skipped on purpose to avoid wrong bindings.

### 3. Click-to-focus

When a card is clicked, `TerminalFocusService::focus_session`:

- reads the current session info
- builds a `SessionFocusTarget`
- prefers an existing cached tab binding
- then falls back to token matching and window focus logic

The Windows-specific implementation lives in:

- `apps/desktop/src-tauri/src/terminal_focus/windows.rs`

## Current Limits

The current solution is still strongly Windows-shaped:

- it mainly targets `Windows Terminal`
- automatic learning depends on foreground window observation
- tab matching is still a combined strategy of cache, token matching, and recent observation

That is why this should be treated as a stable current implementation, not the final cross-platform abstraction.

## Freshness Check

This review corrects older gaps in the note:

- the old note mentioned isolated functions but not the current `terminal_focus` submodule structure
- the old note did not clearly describe the three-layer approach of explicit binding, automatic learning, and recent-foreground fallback

This version now matches the current code structure.
