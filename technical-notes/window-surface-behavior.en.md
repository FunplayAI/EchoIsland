# Island Window Behavior Rules

## Purpose

This note records the current behavior boundary between automatic island window updates and user-initiated window activation.

Last reviewed: `2026-04-15`

## Background

The island previously risked interrupting typing in a terminal or IDE when cards auto-popped.

The root cause was not the frontend animation itself. It was the window surface update path on Windows, where a topmost refresh path could also activate the window.

The behavior is now explicitly split into two modes:

- `passive`: system-driven window surface updates that must not interrupt current input focus
- `interactive`: user-initiated window reveal where focus activation is allowed

## Current Rules

### 1. `passive`

Applies to:

- automatic status-queue popup
- approval or question cards causing panel expansion
- auto-collapse and auto-height updates
- regular window surface synchronization

Behavior requirements:

- may update size, hit region, and topmost state
- must not intentionally steal the current input focus
- must not interrupt terminal or IDE typing

Current commands:

- `set_island_bar_stage_passive`
- `set_island_panel_stage_passive`
- `set_island_expanded_passive`

### 2. `interactive`

Applies to:

- explicit tray action to show the main window
- any future explicit “open main window” entry

Behavior requirements:

- may show and activate the main window
- may move focus back to EchoIsland

Current command:

- `show_main_window_interactive`

## Current Implementation Areas

Rust side:

- `apps/desktop/src-tauri/src/window_surface_service.rs`
- `apps/desktop/src-tauri/src/commands.rs`
- `apps/desktop/src-tauri/src/island_window.rs`

Frontend call sites:

- `apps/desktop/web/api.js`
- `apps/desktop/web/panel-controller.js`

## Key Windows Constraint

On Windows, the auto-popup path must not depend on a “disable topmost, then re-enable topmost” pattern just to refresh stacking order.

That pattern can have activation side effects and interrupt current typing.

The current strategy is:

- `passive` paths use non-activating topmost refresh behavior
- only explicit user interaction may use show-and-focus logic

## Practical Benefit

This split makes the semantics much clearer:

- automatic card popup = surface-only change
- explicit window reveal = activation allowed

That reduces the risk of mixing passive refresh behavior with interactive activation when continuing work on:

- approval cards
- message cards
- hover expansion
- tray-based reveal

## Future Rule

When adding new window-related commands later, decide first which class they belong to:

- UI-driven automatic change
- explicit user action

Only the second class should use `interactive` semantics.
