# 0009. Select the platform implementation at compile time

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer (made during repo scaffolding)

## Context

All OS-specific code lives in `crates/platform` behind the `Watcher`, `Indexer`, `Thumbnailer`, `ShellIntegration` and `Trash` traits ([ADR 0003](0003-windows-first-mvp.md)). The plan doesn't say how callers get an implementation, or what unimplemented platforms do.

## Decision

- Each OS has one module (`windows`, `macos`, `linux`) with one type implementing all five traits plus a `Platform` supertrait (which adds `name()`).
- Exactly one module is compiled, chosen by `#[cfg(target_os = …)]` inside `vault-platform`, and re-exported as `vault_platform::Native`. Callers use `vault_platform::native()` and the traits — static dispatch, no `cfg` outside this crate.
- Unimplemented operations return `PlatformError::Unsupported { operation, platform }`, never panic.
- The trait **method signatures are provisional** (synchronous, minimal). They will be redesigned, probably async and streaming, as the Phase 0 MFT and IPC spikes land. That redesign doesn't need a new ADR unless the dispatch model changes.
- No error-handling crate yet: `PlatformError` implements `std::error::Error` by hand.

## Alternatives considered

- **`Box<dyn Platform>` chosen at runtime** — only useful for mocking; tests can use generic parameters instead, and static dispatch has zero cost.
- **One crate per OS** — more manifests and CI wiring for no benefit at this size.
- **`thiserror`** — convenient, but not in the plan's dependency list. Revisit when error types multiply.

## Consequences

- The Windows module only compiles on Windows; Linux/macOS CI `cargo check` won't catch Windows-only errors, so the Windows CI job is required on every PR.
- Code that needs to be testable against fake platforms should be generic over the traits rather than call `native()` directly.
