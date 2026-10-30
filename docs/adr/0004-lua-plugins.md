# 0004. Use Lua 5.4 via mlua for 1.0 plugins

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer

## Context

The core stays small; niche features are plugins ([VISION.md](../VISION.md)). Plugins provide previewers, columns, actions, sidebar panels and themes, and must go through the same core APIs, operations queue and undo log as everything else ([ARCHITECTURE.md](../ARCHITECTURE.md)). The plugin API freezes at the Phase 3 gate.

## Decision

For 1.0, plugins are written in **Lua 5.4**, hosted in-process via the **`mlua`** crate. WASM plugins are a post-1.0 goal.

## Alternatives considered

- **WASM (wasmtime + WIT) from the start** — strong sandboxing and any source language, but heavier runtime, larger installer and slower to build a good API around. Still an open question whether to start here ([STATUS.md](../STATUS.md)).
- **JavaScript in the webview** — would let plugins bypass the "UI never touches the disk" rule.
- **Native dynamic libraries** — unsafe, ABI-fragile, and a crash takes down the app.

## Consequences

- `crates/plugins` owns the Lua host and the plugin API surface. Lua code gets no direct file-system access; it calls core APIs.
- Plugin API changes go through the [RFC process](../rfcs/README.md).
- If the open question resolves to "WASM from the start", a new ADR supersedes this one before the Phase 3 API freeze.
