# Vault — Product & Technical Plan

> Source of truth for the Vault project. Agents and contributors: read this before making changes.
> If something here is wrong or outdated, update the relevant document in the same change — don't silently diverge.

The plan is split into focused documents:

| Document | What it covers |
| --- | --- |
| [VISION.md](VISION.md) | Pitch, audience, design principles, non-goals |
| [FEATURES.md](FEATURES.md) | Full feature list by area |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Layers, data flow, key rules, tech stack, repo layout |
| [PERFORMANCE.md](PERFORMANCE.md) | Performance budgets (CI-enforced) |
| [ROADMAP.md](ROADMAP.md) | Phases, gates, risks |
| [WINDOWS.md](WINDOWS.md) | Windows MVP platform notes |
| [STATUS.md](STATUS.md) | Current phase, done / next, **open questions** |
| [backlog/](backlog/) | Work items for the active phase |
| [adr/](adr/) | Architecture Decision Records — why things are the way they are |
| [rfcs/](rfcs/) | Proposals for config-format and plugin-API changes |

## Decisions made

Each decision has an ADR with context and consequences.

| Decision | Choice | ADR |
| --- | --- | --- |
| Shell + core | Tauri 2, Rust (stable) core on Tokio | [0001](adr/0001-tauri-2-and-rust-core.md) |
| UI | TypeScript + SolidJS (pending Phase 0 spike vs Svelte 5) | [0002](adr/0002-solidjs-ui-pending-spike.md) |
| MVP platform | Windows 10 22H2+ and Windows 11 (x64; ARM64 later); macOS 13+, Ubuntu 22.04+ in Phase 2 | [0003](adr/0003-windows-first-mvp.md) |
| Plugins | Lua 5.4 via `mlua` for 1.0; WASM later | [0004](adr/0004-lua-plugins.md) |
| License | MIT / Apache-2.0 dual license (confirm before first release) | [0005](adr/0005-dual-mit-apache-license.md) |
| Name | **Vault** (app), **`vlt`** (CLI) | [0006](adr/0006-cli-name-vlt.md) |
| Config | TOML + JSON Schema | — |
| Telemetry | None by default; opt-in crash reports only | — |

Decisions made while scaffolding the repo (toolchain, naming, platform dispatch, IPC boundary, UI tooling, project process) are ADRs [0007–0012](adr/README.md#index).

## Project process

- **CI (GitHub Actions):** build + test on Windows per PR (macOS/Linux compile checks from day one, full tests from Phase 2); clippy, rustfmt, eslint, prettier; benchmark comparison vs `main`; nightly builds; tagged releases publish installers + updater feed.
- **Community:** README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, issue/PR templates, `good first issue` labels, GitHub Projects roadmap, Discussions.
- **RFCs** (Markdown PR in [`rfcs/`](rfcs/)) for config-format or plugin-API changes.
- **ADRs** in [`adr/`](adr/) for architecture decisions.
