# Changelog

All notable changes to EchoIsland are documented here.

## v0.5.0 - 2026-04-30

### Highlights

- Bumped desktop, Tauri, and Rust workspace package versions to `0.5.0`.
- Updated Windows release metadata and user-visible version text to `EchoIsland v0.5.0`.
- Added `v0.5.0` release metadata to `llms.txt`.
- Improved the Windows native Dynamic Island panel UI, hover-out close behavior, card layout, animation timing, and action button behavior.
- Continued unifying Windows and macOS native panel rendering/runtime paths through shared visual and interaction planning.

### Windows Packages

- NSIS installer: `EchoIsland_0.5.0_x64-setup.exe`
- MSI installer: `EchoIsland_0.5.0_x64_en-US.msi`
- Portable executable: `EchoIsland.exe`

### Notes

- Git tag: `v0.5.0`
- GitHub Release assets still need to be uploaded manually or through GitHub CLI/API.
- Windows native panel is enabled by default.
- macOS native panel work remains in active migration and should be validated on macOS before treating it as release-ready.
