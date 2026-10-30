# 0001. Use Tauri 2 with a Rust core

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer

## Context

Vault must open instantly, stay under 150 MB idle and ship as an installer under 15 MB ([PERFORMANCE.md](../PERFORMANCE.md)), while running on Windows first and macOS/Linux later. File-system work (listing, indexing, MFT reads, copies) must be fast and must never block the UI.

## Decision

- The shell is **Tauri 2**, using the OS webview (WebView2 on Windows).
- All logic and all disk access live in a **Rust (stable) core** using **Tokio** for async. The UI talks to it only over Tauri IPC: commands down, an event stream up.

## Alternatives considered

- **Electron** — bundles Chromium and Node: installer and idle-memory budgets are out of reach.
- **Native UI per platform (WinUI, AppKit, GTK)** — best fidelity, but three UIs for a part-time project.
- **Rust-native GUI (egui, Slint, iced)** — small and fast, but text rendering, accessibility and theming are less mature than a webview for a file manager's needs.

## Consequences

- The UI is swappable: since it holds no logic, a later UI rewrite doesn't touch the core.
- IPC throughput is a risk for huge folders; Phase 0 must prove batched streaming fits the budgets ([backlog](../backlog/phase-0.md)).
- WebView2 must be present; the installer bootstraps it if missing ([WINDOWS.md](../WINDOWS.md)).
- Revisit if the IPC streaming spike cannot meet "first rows < 50 ms" for 10k items.
