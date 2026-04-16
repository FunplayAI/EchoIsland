# Technical Notes

This directory stores implementation-oriented reference notes for the current codebase.

Split versus `docs/`:

- `technical-notes/`: implementation notes, runtime behavior, parameters, module relations
- `docs/`: plans, roadmaps, checklists, proposals, risks, and execution documents

## Current Documents

- `architecture-current.zh-CN.md`
- `architecture-current.en.md`
- `current-low-overhead-scan.zh-CN.md`
- `current-low-overhead-scan.en.md`
- `module-dependency-map.zh-CN.md`
- `module-dependency-map.en.md`
- `windows-terminal-tab-focus.zh-CN.md`
- `windows-terminal-tab-focus.en.md`
- `window-surface-behavior.zh-CN.md`
- `window-surface-behavior.en.md`
- `status-queue-timing.zh-CN.md`
- `status-queue-timing.en.md`

## Naming Convention

Use the following rules for future notes:

- bilingual content files: `<topic>.zh-CN.md` + `<topic>.en.md`
- keep `topic` in English kebab-case
- prefer implementation-oriented topic names, not task titles or meeting-style notes
- keep `README.md` as the neutral entry, with language indexes in `README.zh-CN.md` and `README.en.md`

Recommended examples:

- `session-lifecycle.zh-CN.md`
- `session-lifecycle.en.md`
- `platform-capabilities.zh-CN.md`
- `platform-capabilities.en.md`

Avoid:

- `会话生命周期说明.md`
- `platform-notes-final.md`
- `some-scan-thoughts.md`

Extra convention:

- if a note temporarily exists in only one language, add the matching pair soon
- even before the pair is complete, keep the language suffix to avoid later rename churn

## Freshness Review

Last manual review: `2026-04-15`

Confirmed and updated in this pass:

- legacy single-language notes were replaced with bilingual pairs
- architecture wording now matches the current workspace and service split
- low-overhead scan notes now match the current watcher + incremental scan flow
- Windows Terminal focus notes now match the current `terminal_focus` module layout

No obvious stale implementation claims were found in the current `technical-notes/` set after this review.

Caveat:

- these notes describe the current implementation, not a long-term contract
- review them again after future cross-platform refactors, native hook integration, or major UI restructuring
