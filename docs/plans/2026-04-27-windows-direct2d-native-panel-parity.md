# Windows Direct2D Native Panel Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the current Windows GDI native panel painter with a Direct2D/DirectWrite backend that keeps Windows on native rendering while matching the macOS / legacy Windows Web island visual behavior as closely as practical.

**Architecture:** Keep the existing shared Rust core, scene, layout, presentation, runtime, shell command, and pointer-event pipeline. Replace only the Windows native window composition and paint backend so it consumes the shared presentation output but renders with Direct2D/DirectWrite, per-pixel alpha, DPI-aware coordinates, and platform-specific shape/hit-test handling.

**Tech Stack:** Rust, Tauri 2, `windows-sys` for existing Win32 shell plumbing, `windows` or expanded `windows-sys` bindings for Direct2D/DirectWrite/DXGI/WIC/DPI APIs, existing `native_panel_core`, `native_panel_scene`, and `native_panel_renderer`.

---

## Progress Update - 2026-04-27

Current status:

- New checkpoint after `0ae927a`: Windows production paint routing now prefers `Direct2DWindowsNativePanelPainter` instead of the previous hard-coded GDI painter. Window creation derives its layered-window composition mode from the selected painter and now selects per-pixel alpha for the Direct2D path.
- The Direct2D painter now initializes shared Direct2D and DirectWrite factories, renders the shared visual primitives into a top-down 32-bit DIB-backed Direct2D DC render target, and uploads the result through `UpdateLayeredWindow(ULW_ALPHA)`.
- DirectWrite text rendering is connected for shared text primitives with the Windows native font fallback (`Segoe UI Variable` -> `Segoe UI`) and no-wrap text format.
- GDI remains available as an explicit fallback backend, but it is no longer the preferred production paint path.
- This completes the first M1-level native paint path checkpoint. It still needs manual visual validation and follow-up parity work for exact compact shoulder geometry, expanded card polish, and resource reuse across frame revisions.
- Direct2D painter lifetime now reuses a thread-local painter instance, and DIB/DC/render-target resources are keyed by physical window rect plus DPI scale so unchanged frames can reuse the existing surface while monitor/DPI/size changes still force rebuild.
- Compact visual planning now keeps expanded-only primitives out of compact mode: cards, action buttons, and completion glow no longer contribute large panel blocks to the compact paint plan.
- Windows host-window placement and Direct2D primitive drawing now account for the coordinate-system boundary: shared/mac layout remains bottom-left based, while Windows screen placement and Direct2D painting convert to top-left coordinates at the platform edge.
- Tasks 1-8 are completed and committed through compact/expanded shared paint-model parity.
- Task 9 is completed in `60ea215`: Windows native animation frames are sampled from the shared timeline and redraw is scheduled only while animation is active.
- Task 10 is completed in `5850555`: Windows native hit testing uses shared pointer regions, transparent margins pass through via `WM_NCHITTEST`, and whole-window `WS_EX_TRANSPARENT` passthrough is no longer used for the island.
- Task 11 is in progress. First checkpoint is committed in `589171b`: platform loop records physical window rect and surface resource revision. Current local work extends that boundary so DPI scale changes also advance the surface revision, paint dispatch records the revision used for resource rebuilds, and tests cover display repositioning, negative monitor origins, and DPI scale changes.
- Direct2D/DirectWrite factory smoke initialization is implemented: Windows tests create real `ID2D1Factory` and `IDWriteFactory` instances successfully.
- Windows native UI is now enabled by default on Windows. Use `ECHOISLAND_WINDOWS_NATIVE_UI=0/off/false` to temporarily disable it during regression.
- Direct2D/DirectWrite bindings, factory initialization, painter abstractions, and the first real layered-window Direct2D paint path exist. Do not treat visual parity as complete yet.

Last verified commands:

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo test -p echoisland-desktop native_panel_renderer
cargo test -p echoisland-desktop native_panel_core
cargo check -p echoisland-desktop
cargo fmt --check
```

---

## Scope

### In Scope

- Windows native renderer remains supported and becomes the target path.
- macOS native panel behavior remains unchanged.
- Shared core/scene/layout/presentation code remains the source of truth.
- Current GDI painter is replaced or demoted to fallback/test-only.
- Compact island visual parity is the first milestone.
- Expanded cards/settings visual parity follows after compact parity is stable.
- DPI, multi-monitor, transparent composition, and hit regions are handled explicitly.

### Out of Scope

- Rewriting the Web frontend.
- Replacing macOS AppKit/CALayer rendering.
- Supporting Windows 7/8.
- Perfect pixel parity across Windows and macOS font rendering.
- Adding new product behavior while renderer parity is in progress.

## Current Problem

The current Windows native backend renders through `apps/desktop/src-tauri/src/windows_native_panel/paint_backend.rs` with GDI primitives. It can show a window, but the visual output is not equivalent to macOS or the legacy Web island:

- GDI round rects and text do not match AppKit/CALayer or CSS antialiasing.
- The current primitive model omits key island details: top shoulder cutouts, corner masks, layered glow, gradient/border composition, and accurate clipping.
- The window uses a color-key transparency model, which produces rough edges and does not support high-quality per-pixel alpha.
- DPI conversion is not consistently owned by the native painter/window layer.
- Compact and expanded hit regions are not guaranteed to match the drawn shape.

## Target Design

## Cross-Platform Constraints

The long-term goal is to keep platform-specific code as small as possible. Windows Direct2D must be a renderer/backend, not a second product implementation.

Hard constraints:

- Do not put product state decisions in `windows_native_panel`.
- Do not duplicate queue priority, card selection, settings semantics, hover state, surface switching, or animation timing in Windows-specific code.
- Do not let Windows decide what the settings rows, cards, badges, or compact headline mean.
- Do not fork mac and Windows scene/layout behavior to solve a visual bug.
- If Windows needs richer drawing data, extend a shared model such as `NativePanelPresentationModel`, `NativePanelRenderCommandBundle`, or a new platform-neutral `NativePanelDrawModel`.
- Platform renderers may differ in how they draw, but not in what they draw.
- Shared tests must cover scene, layout, presentation, hit targets, and settings semantics before platform renderers consume them.
- Windows-specific tests should focus on DPI conversion, Win32 window state, Direct2D resource lifecycle, hit-test translation, and pixel/paint command mapping.

Allowed platform-specific responsibilities:

- Window creation, topmost/no-activate behavior, transparency, and composition.
- DPI and logical-to-physical pixel conversion.
- Direct2D/DirectWrite resource creation and drawing commands.
- Native font fallback and rasterization differences.
- Platform hit-test / region / mouse passthrough integration.
- System integrations such as tray, focus, installer, and OS-specific capabilities.

Target ownership boundary:

```text
Shared:   state -> queue -> scene -> layout -> animation -> presentation -> semantic hit targets
Platform: window -> composition -> DPI -> draw API -> system input/output
```

Review rule:

- Any new `windows_native_panel` change that introduces business branching must be rejected or moved into shared code.
- Any visual parity fix that requires extra semantic data must first add that data to the shared presentation/draw model, then consume it from Windows and mac/Web as appropriate.

### Layering

```text
RuntimeSnapshot
  -> native_panel_core
  -> native_panel_scene
  -> native_panel_renderer presentation / render command bundle
  -> windows_native_panel Direct2D renderer
  -> Win32 transparent topmost native window
```

### Rendering Model

- Keep layout units in logical pixels in shared core.
- Convert logical pixels to physical pixels only at the Windows platform boundary.
- Use Direct2D for:
  - antialiased rounded rectangles
  - gradients
  - border strokes
  - shoulder geometry
  - alpha blending
  - mascot primitives
  - glow primitives
- Use DirectWrite for:
  - compact headline text
  - counts/badges
  - card text
  - settings row labels/values
- Use per-pixel alpha composition instead of magenta color-key transparency.

### Window Model

- Keep the existing Win32 topmost, no-activate, toolwindow behavior.
- Add DPI-aware sizing and positioning.
- Apply shape/hit-test behavior separately from visual painting:
  - compact mode: only the island bar/shoulder area should receive pointer input
  - expanded mode: expanded panel and card areas should receive pointer input
  - transparent areas should not block normal desktop interaction

### Rollout Model

- Keep `ECHOISLAND_WINDOWS_NATIVE_UI` as the explicit switch while the renderer is under development.
- Once compact and expanded parity pass manual verification on Windows 10/11 at multiple DPI scales, flip Windows native back to default enabled.
- Keep Web fallback available during rollout.

---

## Task 1: Freeze Current Windows Native Runtime Behavior

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Verify: `apps/desktop/src-tauri/src/windows_native_panel/host_runtime.rs`
- Verify: `apps/desktop/src-tauri/src/windows_native_panel/renderer.rs`
- Verify: `apps/desktop/src-tauri/src/windows_native_panel/window_shell.rs`
- Verify: `apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs`

**Step 1: Write runtime lifecycle tests**

Add tests that assert the Windows native runtime still:

- creates and shows the host through `runtime.create_panel()`
- produces a visible first snapshot window state
- pumps shell commands into the platform loop
- queues redraw after consuming a presenter frame
- preserves pointer regions after rendering a scene

**Step 2: Run focused tests**

Run:

```powershell
cargo test -p echoisland-desktop windows_runtime_first_snapshot_renders_without_seeded_animation_descriptor
cargo test -p echoisland-desktop windows_native_default_enable_preflight_uses_shared_runtime_pipeline
cargo test -p echoisland-desktop windows_host_lifecycle_tracks_create_show_hide
```

Expected: PASS before renderer replacement starts.

**Step 3: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "test: lock windows native panel runtime behavior"
```

---

## Task 2: Add Direct2D/DirectWrite Dependencies Behind Windows cfg

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel.rs`
- Create: `apps/desktop/src-tauri/src/windows_native_panel/direct2d.rs`
- Create: `apps/desktop/src-tauri/src/windows_native_panel/directwrite.rs`

**Step 1: Choose binding strategy**

Preferred: add the `windows` crate for Direct2D/DirectWrite COM ergonomics while keeping `windows-sys` for existing low-level Win32 calls.

Add target Windows dependency similar to:

```toml
[target.'cfg(windows)'.dependencies.windows]
version = "0.58"
features = [
  "Win32_Foundation",
  "Win32_Graphics_Direct2D",
  "Win32_Graphics_Direct2D_Common",
  "Win32_Graphics_DirectWrite",
  "Win32_Graphics_Dxgi_Common",
  "Win32_Graphics_Gdi",
  "Win32_System_Com",
  "Win32_UI_HiDpi",
  "Win32_UI_WindowsAndMessaging",
]
```

If dependency policy prefers no `windows` crate, expand `windows-sys` features instead, but expect more unsafe COM plumbing.

**Step 2: Add smoke tests**

Add test-only constructors or smoke helpers that validate:

- Direct2D factory can be represented by a wrapper type.
- DirectWrite factory can be represented by a wrapper type.
- Non-Windows builds keep stubs/no-op modules.

**Step 3: Run checks**

Run:

```powershell
cargo check -p echoisland-desktop
```

Expected: PASS on Windows.

**Step 4: Commit**

```powershell
git add apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/src/windows_native_panel.rs apps/desktop/src-tauri/src/windows_native_panel/direct2d.rs apps/desktop/src-tauri/src/windows_native_panel/directwrite.rs
git commit -m "build: add windows direct2d directwrite renderer bindings"
```

---

## Task 3: Introduce DPI-Aware Windows Drawing Units

**Files:**
- Create: `apps/desktop/src-tauri/src/windows_native_panel/dpi.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/host_window.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`

**Step 1: Write tests for logical-to-physical conversion**

Add tests for:

- 100% scale: `253x80` logical maps to `253x80` physical.
- 125% scale: `253x80` logical maps to rounded physical pixels.
- 150% scale: hit region and window region use the same conversion.
- negative monitor origins are preserved.

**Step 2: Implement DPI helpers**

Create helpers:

```rust
pub(super) struct WindowsDpiScale {
    pub(super) scale: f64,
}

impl WindowsDpiScale {
    pub(super) fn logical_to_physical(self, value: f64) -> i32;
    pub(super) fn rect_to_physical(self, rect: PanelRect) -> PhysicalRect;
}
```

On Windows, resolve scale from the target monitor/window using `GetDpiForWindow`, `GetDpiForMonitor`, or a safe fallback to 96 DPI.

**Step 3: Route platform window sizing through DPI helpers**

Update `apply_windows_native_window_state` so `SetWindowPos` receives physical pixels derived from shared logical layout.

**Step 4: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel::tests
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/dpi.rs apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs apps/desktop/src-tauri/src/windows_native_panel/host_window.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "refactor: make windows native panel dpi aware"
```

---

## Task 4: Replace Color-Key Transparency With Per-Pixel Alpha Composition

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs`
- Create: `apps/desktop/src-tauri/src/windows_native_panel/layered_window.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`

**Step 1: Write window composition tests**

Add tests around platform-state command behavior:

- create window records a raw handle
- resize state can be applied before/after show
- redraw command stores pending paint state
- destroy clears raw handle

For actual per-pixel alpha, rely on Windows-only smoke/manual verification.

**Step 2: Implement layered composition boundary**

Create a platform helper that owns:

- memory bitmap / pixel buffer allocation
- alpha-enabled bitmap metadata
- `UpdateLayeredWindow` or equivalent per-pixel alpha upload
- size changes when window frame changes

Do not keep using `LWA_COLORKEY` as the primary transparency path.

**Step 3: Preserve fallback**

Keep the current GDI/color-key path behind a fallback flag or internal helper until Direct2D is stable.

**Step 4: Run checks**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 5: Manual verification**

Run:

```powershell
$env:ECHOISLAND_WINDOWS_NATIVE_UI="1"
npm.cmd run desktop:dev
```

Verify:

- no magenta edge artifacts
- transparent areas do not show colored pixels
- window remains topmost
- window does not steal focus

**Step 6: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs apps/desktop/src-tauri/src/windows_native_panel/layered_window.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "feat: add per-pixel alpha windows native panel window"
```

---

## Task 5: Build Direct2D Painter Skeleton

**Files:**
- Create: `apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/paint_backend.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/runtime_traits.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`

**Step 1: Write paint-operation tests**

Add platform-neutral tests that verify:

- hidden paint job emits no work
- compact paint job emits a compact background primitive
- expanded paint job emits shell/card primitives
- text primitives are preserved for DirectWrite

**Step 2: Implement `WindowsNativePanelPainter` trait**

Introduce:

```rust
pub(super) trait WindowsNativePanelPainter {
    fn paint(&mut self, job: &WindowsNativePanelShellPaintJob) -> Result<WindowsNativePanelPaintPlan, String>;
}
```

Provide:

- `GdiWindowsNativePanelPainter` as temporary fallback
- `Direct2DWindowsNativePanelPainter` as the target implementation

**Step 3: Route runtime paint through painter abstraction**

Update `paint_windows_native_panel_job` or replace it with a painter-owned entrypoint.

**Step 4: Run checks**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs apps/desktop/src-tauri/src/windows_native_panel/paint_backend.rs apps/desktop/src-tauri/src/windows_native_panel/runtime_traits.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "refactor: introduce windows native panel painter abstraction"
```

---

## Task 6: Match Compact Island Geometry

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/host_window.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/renderer.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Verify: `apps/desktop/src-tauri/src/native_panel_core/constants/geometry.rs`
- Verify: `apps/desktop/src-tauri/src/island_window.rs`
- Verify: `apps/desktop/web/styles.css`

**Step 1: Write compact geometry tests**

Assert target values:

- stage/canvas: `420x80`
- compact pill width: shared `DEFAULT_COMPACT_PILL_WIDTH`
- compact pill height: shared `DEFAULT_COMPACT_PILL_HEIGHT`
- compact hit width: legacy Web `265`
- shoulder size: shared `COMPACT_SHOULDER_SIZE`
- top corners use compact mask semantics

**Step 2: Implement compact frame mapping**

Ensure Direct2D compact mode draws only the island bar and shoulders, not a full black `420x80` rectangle.

**Step 3: Implement shoulder drawing**

Draw left/right top shoulders equivalent to CSS:

- transparent outside shoulder curve
- black fill inside shoulder curve
- subtle border line
- progress-controlled hide/show for open/close animation

**Step 4: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 5: Manual verification**

Run:

```powershell
$env:ECHOISLAND_WINDOWS_NATIVE_UI="1"
npm.cmd run desktop:dev
```

Verify compact state against mac/legacy Web:

- centered at top of selected display
- black island shape, not a rounded rectangle block
- top edge attaches visually to screen top
- shoulders visible in compact mode
- no colored/aliased transparent edge

**Step 6: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs apps/desktop/src-tauri/src/windows_native_panel/host_window.rs apps/desktop/src-tauri/src/windows_native_panel/renderer.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "feat: match windows native compact island geometry"
```

---

## Task 7: Match Compact Content Rendering

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/directwrite.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Verify: `apps/desktop/src-tauri/src/native_panel_renderer/presentation_model.rs`
- Verify: `apps/desktop/web/styles.css`

**Step 1: Write compact content tests**

Assert paint model includes:

- mascot position near left side of pill
- headline text origin and max width
- active/completion count/badge placement
- action button visibility only near expanded progress

**Step 2: Implement DirectWrite text rendering**

Use DirectWrite for:

- headline
- count/badge text
- settings/card text later

Pick Windows-native font fallback:

- primary: `Segoe UI Variable` where available
- fallback: `Segoe UI`

**Step 3: Implement mascot compact drawing**

Use the shared scene pose but draw with Direct2D antialiasing. Do not attempt pixel-perfect mac mascot in this task; target placement and visual weight first.

**Step 4: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 5: Manual verification**

Verify:

- text alignment matches old Web/mac reasonably
- no clipped text at 100/125/150% DPI
- mascot does not shift when switching display scale

**Step 6: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs apps/desktop/src-tauri/src/windows_native_panel/directwrite.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "feat: render windows native compact island content with directwrite"
```

---

## Task 8: Match Expanded Panel and Card Rendering

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/directwrite.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Verify: `apps/desktop/src-tauri/src/native_panel_renderer/render_commands.rs`
- Verify: `apps/desktop/src-tauri/src/native_panel_renderer/presentation_model.rs`
- Verify: `apps/desktop/src-tauri/src/native_panel_core/constants/geometry.rs`

**Step 1: Write expanded paint tests**

Cover:

- expanded shell background
- separator visibility
- card stack frame
- settings rows
- pending permission/question card
- completion card
- empty state card

**Step 2: Implement shell and card backgrounds**

Use Direct2D:

- rounded expanded background
- subtle border/highlight
- card rounded rectangles
- completion glow approximation

**Step 3: Implement text and badges**

Use DirectWrite:

- card title/subtitle/body
- status badges
- settings row value pills
- empty state text

**Step 4: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 5: Manual verification**

Verify settings page:

- clicking settings replaces content like mac
- rows match mac visual hierarchy
- display/sound/mascot/release rows remain clickable

**Step 6: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/d2d_painter.rs apps/desktop/src-tauri/src/windows_native_panel/directwrite.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "feat: render windows native expanded panel with direct2d"
```

---

## Task 9: Align Animation Timing and Redraw Scheduling

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/renderer.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/runtime_traits.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Verify: `apps/desktop/src-tauri/src/native_panel_core/transitions.rs`

**Step 1: Write animation scheduler tests**

Cover:

- opening transition schedules frame redraws while active
- closing transition stops redraws when complete
- surface switch redraws only while transition active
- idle state does not redraw continuously

**Step 2: Implement redraw scheduling**

Use the existing transition descriptors and runtime state. Ensure Windows native only redraws when:

- a snapshot changes
- pointer/hover changes
- an animation frame is due
- display/DPI changes

**Step 3: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 4: Manual performance check**

Verify:

- idle CPU stays near zero
- animation is smooth enough at 60 FPS
- no constant redraw loop after animation finishes

**Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/renderer.rs apps/desktop/src-tauri/src/windows_native_panel/runtime_traits.rs apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "perf: schedule windows native panel redraws only when needed"
```

---

## Task 10: Align Hit Testing and Pointer Passthrough

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/window_shell.rs`
- Create: `apps/desktop/src-tauri/src/windows_native_panel/hit_region.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Verify: `apps/desktop/src-tauri/src/island_window.rs`

**Step 1: Write hit-region tests**

Cover:

- compact hit area matches drawn island shape
- transparent margins pass through
- expanded shell accepts input
- cards/settings rows accept input
- action buttons accept input only when visible

**Step 2: Implement native hit-test/region sync**

Use one of:

- `WM_NCHITTEST` for point-level hit testing
- `SetWindowRgn` for coarse region clipping
- mouse passthrough via extended style for fully non-interactive transparent regions

Prefer point-level hit testing if region updates become too coarse for animated shapes.

**Step 3: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 4: Manual verification**

Verify:

- transparent stage area does not block clicks
- compact bar hover expands consistently
- settings and quit buttons are clickable
- card focus clicks still route to terminal focus

**Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs apps/desktop/src-tauri/src/windows_native_panel/window_shell.rs apps/desktop/src-tauri/src/windows_native_panel/hit_region.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "feat: align windows native panel hit testing with drawn shape"
```

---

## Task 11: Multi-Monitor and DPI Validation

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/dpi.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/host_window.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/runtime_traits.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`

**Step 1: Write display-selection tests**

Cover:

- selected display index updates window state
- monitor frame with negative origin is supported
- DPI scale changes recompute physical frame
- moving between monitors recomputes painter resources

**Step 2: Implement resource recreation on DPI/size changes**

Ensure Direct2D resources dependent on pixel size are recreated when:

- selected display changes
- DPI scale changes
- window frame changes

**Step 3: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 4: Manual matrix**

Verify:

- Windows 10, 100% scale
- Windows 11, 100% scale
- Windows 11, 125% scale
- Windows 11, 150% scale
- two monitors with different scale factors
- selected display cycling from Settings

**Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/platform_loop.rs apps/desktop/src-tauri/src/windows_native_panel/runtime_traits.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs
git commit -m "fix: handle windows native panel multi-monitor dpi changes"
```

---

## Task 12: Flip Windows Native Renderer Back to Default Enabled

**Files:**
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/facade.rs`
- Modify: `apps/desktop/src-tauri/src/windows_native_panel/tests.rs`
- Modify: `docs/plans/2026-04-27-windows-direct2d-native-panel-parity.md`

**Step 1: Write enablement tests**

Update tests so:

- Windows native UI is enabled by default on Windows.
- `ECHOISLAND_WINDOWS_NATIVE_UI=0` disables native renderer.
- `ECHOISLAND_WINDOWS_NATIVE_UI=1` explicitly enables native renderer.
- non-Windows remains disabled for this backend.

**Step 2: Flip default**

Current expected default:

```rust
windows_native_ui_enabled_from_env(true, ...)
```

only after parity and validation are complete.

**Step 3: Run tests**

```powershell
cargo test -p echoisland-desktop windows_native_ui
cargo check -p echoisland-desktop
```

Expected: PASS.

**Step 4: Manual startup verification**

Run:

```powershell
Remove-Item Env:ECHOISLAND_WINDOWS_NATIVE_UI -ErrorAction SilentlyContinue
npm.cmd run desktop:dev
```

Expected:

- native backend initializes
- webview window surface initialization is skipped
- Windows panel appears visually aligned with mac/legacy Web
- no PowerShell helper window loops

**Step 5: Commit**

```powershell
git add apps/desktop/src-tauri/src/windows_native_panel/facade.rs apps/desktop/src-tauri/src/windows_native_panel/tests.rs docs/plans/2026-04-27-windows-direct2d-native-panel-parity.md
git commit -m "feat: enable direct2d windows native panel by default"
```

---

## Final Verification Checklist

Run automated checks:

```powershell
cargo test -p echoisland-desktop windows_native_panel
cargo test -p echoisland-desktop native_panel_core
cargo test -p echoisland-desktop native_panel_renderer
cargo check -p echoisland-desktop
npm.cmd run desktop:dev
```

Manual visual checks:

- Compact mode looks like the mac/legacy Web island, not a generic rounded rectangle.
- Top shoulders are visible and correctly clipped.
- Transparent edges are antialiased.
- No magenta or color-key artifacts.
- Text is crisp at 100/125/150% DPI.
- Expanded cards match layout and spacing.
- Settings surface replaces content like mac.
- Completion glow is present and not overdrawn.
- Hover expand/collapse feels aligned with mac timing.
- Transparent areas do not block mouse input.
- App does not steal focus when appearing.
- Idle CPU remains near zero after animations finish.

Manual packaging checks:

```powershell
npm.cmd run desktop:build:portable
npm.cmd run desktop:build
```

Expected:

- portable exe launches without console/powershell refresh loop
- installer app has icon
- native panel renders consistently after install

---

## Risks and Mitigations

- **Risk:** Direct2D + layered window composition has unsafe Win32/COM complexity.
  **Mitigation:** isolate in `direct2d.rs`, `directwrite.rs`, and `layered_window.rs`; keep tests around pure conversion/state logic.

- **Risk:** Multi-monitor DPI causes frame and paint buffer mismatch.
  **Mitigation:** centralize logical/physical conversion in `dpi.rs`; test 100/125/150% and mixed-DPI monitors.

- **Risk:** Windows and mac text rendering cannot be pixel identical.
  **Mitigation:** target layout/weight parity, use Segoe UI Variable/Segoe UI instead of forcing mac fonts.

- **Risk:** Per-pixel alpha behaves differently under RDP or older drivers.
  **Mitigation:** keep Web fallback and temporary GDI fallback switch during rollout.

- **Risk:** Shared visual primitive model is too coarse for mac-level parity.
  **Mitigation:** let Windows painter consume `NativePanelPresentationModel` or render command bundle directly for high-fidelity details instead of forcing all details into generic primitives.

---

## Suggested Milestones

1. **M1: Native window + Direct2D smoke render**
   - Direct2D initializes, paints a transparent antialiased shape, no color-key artifacts.

2. **M2: Compact island parity**
   - Compact shape, shoulder, text, mascot, and hit region match mac/legacy Web closely.

3. **M3: Expanded/settings parity**
   - Cards, settings surface, badges, and click behavior match mac interaction.

4. **M4: DPI and multi-monitor readiness**
   - Verified on 100/125/150% scale and mixed-DPI monitors.

5. **M5: Default enablement**
   - Windows native renderer becomes default; Web remains fallback.
