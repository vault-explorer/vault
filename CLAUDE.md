# CLAUDE.md

Vault is an open-source, keyboard-first file explorer: opens instantly, finds anything in milliseconds, familiar on day one.
It is a Tauri 2 app — a Rust core (Tokio) owns all logic and disk access; a SolidJS + TypeScript UI renders it.
Windows 10 22H2+ / 11 is the MVP platform; macOS and Linux follow in Phase 2. Status: pre-alpha, Phase 0.

## Docs — read before changing anything

- [docs/PLAN.md](docs/PLAN.md) — index and decisions table (source of truth)
- [docs/STATUS.md](docs/STATUS.md) — current phase, done, next, open questions
- [docs/VISION.md](docs/VISION.md) · [docs/FEATURES.md](docs/FEATURES.md) · [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [docs/PERFORMANCE.md](docs/PERFORMANCE.md) · [docs/ROADMAP.md](docs/ROADMAP.md) · [docs/WINDOWS.md](docs/WINDOWS.md)
- [docs/backlog/](docs/backlog/) — work items for the active phase
- [docs/adr/](docs/adr/) — decision records · [docs/rfcs/](docs/rfcs/) — config/plugin-API proposals

## Non-negotiable rules

1. **The UI never touches the disk.** All file-system access happens in the Rust core; the UI only calls Tauri commands and listens to events. No Tauri fs/shell plugins in the webview.
2. **Every operation that can take more than 100 ms is a cancellable background task** with progress, on the core's prioritized scheduler. Nothing blocks the UI thread or an IPC command handler.
3. **Every destructive operation is journaled before it runs and is undoable.** Delete goes to the trash by default. No exceptions for the CLI or plugins — they use the same operations queue and undo log.
4. **Performance budgets in [docs/PERFORMANCE.md](docs/PERFORMANCE.md) must not regress.** A feature that breaks a budget doesn't ship. Cut features, never budgets.
5. **Platform code lives only in `crates/platform`, behind its traits** (`Watcher`, `Indexer`, `Thumbnailer`, `ShellIntegration`, `Trash`). No `#[cfg(target_os)]` anywhere else.

## Coding conventions

**Rust**
- Edition 2024, MSRV 1.90 (`rust-version` in the root `Cargo.toml`). Format with `rustfmt.toml`; clippy `all` + `pedantic` are on and CI denies warnings.
- Every crate starts with a `//!` doc comment describing its responsibility; every public item has a doc comment (`missing_docs` is on).
- No `unwrap()` outside tests. Library errors are typed enums implementing `std::error::Error`; return `Result`, don't panic.
- Front ends (`app/src-tauri`, `crates/cli`) hold no business logic — they translate input into `vault-core` calls.
- IPC payloads are DTOs in `app/src-tauri` with `#[serde(rename_all = "camelCase")]`; core types stay serialization-free unless there's a reason.
- Tests live next to the code (`#[cfg(test)] mod tests`); snapshot tests use `insta`; benches use criterion in `bench/`.

**TypeScript / UI**
- Strict TypeScript (see `app/ui/tsconfig.json`); ESLint (typescript-eslint strict type-checked + eslint-plugin-solid) with zero warnings; Prettier formatting.
- All Tauri `invoke` calls go through typed wrappers in `app/ui/src/ipc.ts`, mirroring the Rust DTOs.
- Lists are virtualized; never render one DOM node per item of an unbounded collection.

**General**
- Ask before adding a dependency not listed in the plan's tech stack ([docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#tech-stack)).
- Small, logical commits. Message: imperative summary ≤ 72 chars, optional area prefix (`feat:`, `fix:`, `docs:`, `chore:`, `perf:`, `test:`, `refactor:`), body explains why.
- Decisions not covered by the plan get an ADR in `docs/adr/`.

## Build, test, lint, bench

```sh
pnpm install                                         # once
pnpm dev                                             # run the app (Vite dev server + Tauri)
pnpm tauri build --no-bundle                         # release build, no installer

cargo test --workspace                               # all Rust tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm lint && pnpm format:check && pnpm typecheck     # UI checks

cargo bench -p vault-bench                           # benchmarks
```

`--workspace` includes the Tauri app crate, which needs WebView2 on Windows or webkit2gtk on Linux. Without those, plain `cargo test` / `cargo clippy` cover every crate except the app.

## Keep the docs true

- **At the end of every session, update [docs/STATUS.md](docs/STATUS.md)**: what changed, what's next, new open questions.
- **Whenever scope or design changes, update the relevant doc in the same change** (FEATURES, ARCHITECTURE, PERFORMANCE, ROADMAP, WINDOWS, backlog). Never let the code silently diverge from the docs.
- Record new decisions as ADRs; propose config/plugin-API changes as RFCs.
