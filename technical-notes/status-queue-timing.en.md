# Status Queue Timing Parameters

## Purpose

This note explains one thing:

- what each status-queue-related timing parameter in `apps/desktop/web/ui-context.js` controls

The goal is to make future tuning easier without re-reading the implementation.

---

## Location

The parameters live in:

- `apps/desktop/web/ui-context.js`

They are grouped into:

- `timings.statusQueue`
- `timings.pendingCard`
- `timings.interaction`

---

## `timings.statusQueue`

### `completionMs`

- Meaning: base lifetime for completion cards in the status queue
- Current role: controls when a `completion` card starts its exit phase
- Increase it: completion cards stay visible longer
- Decrease it: completion cards start disappearing sooner

### `approvalMs`

- Meaning: base lifetime for approval cards in the status queue
- Current role: controls when an `approval` card starts its exit phase
- Increase it: approval cards remain available longer
- Decrease it: approval cards auto-expire sooner

### `exitMinMs`

- Meaning: minimum duration baseline for a single status card exit
- Current role: combined with `cardExitDurationMs` to decide how long the `removing` phase lasts
- Increase it: card exit feels slower and more visible
- Decrease it: card exit feels faster

### `exitExtraMs`

- Meaning: extra buffer added on top of the shared card exit animation duration
- Current role: prevents a card from being removed before its exit animation has visually completed
- Increase it: safer exit timing, but slower pacing
- Decrease it: snappier pacing, but too small may feel abrupt

### `refreshLeadMs`

- Meaning: timing compensation added to the exact refresh target
- Current role: helps trigger the next refresh around the animation completion point
- Increase it: more conservative refresh timing
- Decrease it: closer to the exact target time

### `refreshMinDelayMs`

- Meaning: minimum delay allowed for exact status queue refreshes
- Current role: prevents overly dense near-immediate refresh loops
- Increase it: more stable but less responsive
- Decrease it: more responsive but potentially more frequent

### `autoCloseHoverSuppressMs`

- Meaning: short hover-expand suppression after the status queue auto-closes
- Current role: prevents the island from immediately reopening because the pointer is still nearby
- Increase it: less chance of accidental reopen
- Decrease it: hover can reopen the island sooner

---

## `timings.pendingCard`

### `minVisibleMs`

- Meaning: minimum visible duration for pending cards
- Current role: prevents a real pending card from appearing and disappearing too quickly during state jitter
- Increase it: pending cards feel more stable
- Decrease it: UI responds faster but may flicker more

### `releaseGraceMs`

- Meaning: extra hold time after the real pending source disappears
- Current role: gives the UI a smooth release window
- Increase it: pending cards exit more gently
- Decrease it: pending cards leave faster

---

## `timings.interaction`

### `compactActionHoverSuppressMs`

- Meaning: hover-expand suppression after clicking the compact island to focus a session
- Current role: prevents the island from expanding immediately after a click-to-focus action
- Increase it: post-click UI feels more stable
- Decrease it: hover behavior recovers sooner

---

## Tuning Guide

### 1. Tune how long status cards stay visible

Start with:

- `statusQueue.completionMs`
- `statusQueue.approvalMs`

### 2. Tune card exit smoothness

Start with:

- `statusQueue.exitMinMs`
- `statusQueue.exitExtraMs`

### 3. Tune accidental reopen after close

Start with:

- `statusQueue.autoCloseHoverSuppressMs`
- `interaction.compactActionHoverSuppressMs`

### 4. Tune pending-card flicker

Start with:

- `pendingCard.minVisibleMs`
- `pendingCard.releaseGraceMs`

---

## One-Line Summary

- `statusQueue` controls how status cards live, exit, and schedule exact refreshes
- `pendingCard` controls pending-card stability
- `interaction` controls hover suppression after user actions
