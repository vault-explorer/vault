# 0010. Keep serialization at the IPC boundary and lock down the webview

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer (made during repo scaffolding)

## Context

The UI never touches the disk ([ARCHITECTURE.md](../ARCHITECTURE.md)). The plan doesn't say where IPC types live, how they're named on the wire, or what the webview is allowed to do.

## Decision

- **DTOs live in `app/src-tauri`** and convert from core types (`impl From<vault_core::Pong> for PongDto`). Core types stay free of serde until there's a reason (e.g. the operations journal needs persistence). `serde` and `serde_json` were approved as dependencies for this.
- Wire format is **camelCase** (`#[serde(rename_all = "camelCase")]`). The UI mirrors each DTO by hand in `app/ui/src/ipc.ts`, and every `invoke` goes through a typed wrapper there. Revisit with generated bindings (e.g. `specta`) once there are more than a handful of commands.
- **Content Security Policy** in production: `default-src 'self'`, no inline scripts or styles, IPC and asset protocols only. `devCsp` is disabled so Vite's dev server works.
- **Capabilities:** the main window gets `core:default` only. No Tauri `fs`, `shell` or `http` plugins are exposed to the webview.

## Alternatives considered

- **Derive `Serialize` on core types directly** — couples the core's internal model to the wire format.
- **Generated TypeScript bindings now** — another dependency for one command; premature.

## Consequences

- Each new command needs a DTO, a `From` impl and a TS mirror. Cheap now, and the cost is what pushes us toward codegen later.
- Any Tauri plugin that grants the webview OS access needs an ADR.
