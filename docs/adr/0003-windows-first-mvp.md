# 0003. Target Windows 10 22H2+ and Windows 11 for the MVP

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer

## Context

Vault is a part-time project. Shipping on three OSes at once would triple platform work before anything is usable. Windows has the largest desktop audience and the weakest built-in explorer for developers, and its MFT/USN journal enables the headline whole-disk search.

## Decision

- The MVP (Phases 0–1) targets **Windows 10 22H2+ and Windows 11, x64**. ARM64 Windows comes later.
- **macOS 13+ and Ubuntu 22.04+** follow in Phase 2.
- All OS-specific code lives in `crates/platform` behind shared traits (`Watcher`, `Indexer`, `Thumbnailer`, `ShellIntegration`, `Trash`), so ports add modules without touching callers.
- CI runs `cargo check` on macOS and Linux from day one so the code stays portable.

## Alternatives considered

- **All three platforms from the start** — too much surface for one part-time maintainer.
- **Windows 11 only** — Windows 10 still has a large install base despite leaving mainstream support in October 2025.

## Consequences

- Test on both Windows 10 and 11 ([WINDOWS.md](../WINDOWS.md)).
- macOS and Linux stubs must compile and return `Unsupported` until Phase 2.
- Linux contributors can build the Rust crates but need the Tauri system libraries (webkit2gtk) to build the app.
