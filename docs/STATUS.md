# Status

> Update this file at the end of every working session (see [CLAUDE.md](../CLAUDE.md)).

**Last updated:** 2026-10-29
**Current phase:** 0 · Spikes ([ROADMAP.md](ROADMAP.md))
**Phase 0 gate:** 100k rows scroll at 60 fps on Windows 10 and 11 — *not yet measured*

## Done

- **2026-10-29 — Repository scaffold**
  - Plan split into focused docs; ADRs 0001–0006 record the plan's decisions; ADRs 0007–0012 record scaffolding decisions; RFC process documented.
  - Cargo workspace (`vault-core`, `vault-index`, `vault-platform`, `vault-git`, `vault-plugins`, `vault-cli` → `vlt`, `vault-bench`) and pnpm workspace (`vault-ui`).
  - `vault-platform` traits (`Watcher`, `Indexer`, `Thumbnailer`, `ShellIntegration`, `Trash`) with Windows/macOS/Linux stubs.
  - Tauri 2 app opens a window titled "Vault"; example `ping` command round-trips UI → Tauri → `vault-core`. Verified on Windows (release build, 4.2 MB exe).
  - Tooling: rustfmt, clippy (pedantic, `-D warnings` in CI), ESLint + Prettier, EditorConfig. CI workflow for Windows, macOS/Linux check, MSRV and a placeholder bench job.
  - Community files: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, licenses, issue/PR templates.

## Next

Work through [backlog/phase-0.md](backlog/phase-0.md), roughly in this order:

1. Benchmark harness — enough to measure the spikes consistently.
2. 100k-row virtual list spike (SolidJS vs Svelte 5) — decides the UI framework and the phase gate.
3. Tauri IPC streaming test.
4. MFT read prototype.

Also: push to GitHub and confirm the CI workflow passes there (it has not run on GitHub yet).

## Open questions

From the plan:

- [ ] SolidJS vs Svelte 5 — decided by the Phase 0 100k-row list benchmark
- [ ] Lua-only plugins for 1.0, or WASM from the start
- [ ] Final license confirmation (MIT / Apache-2.0)
- [ ] Domain, GitHub org, package-name availability for "Vault" and `vlt`; logo

Raised during scaffolding:

- [ ] **Bundle identifier** — `dev.vaultexplorer.vault` is a placeholder ([ADR 0008](adr/0008-workspace-layout-and-naming.md)); settle before any public build, since it fixes the app-data path.
- [ ] **Code of conduct contact** and **`OWNER`** in `.github/ISSUE_TEMPLATE/config.yml` — need the GitHub org / a project contact ([ADR 0012](adr/0012-project-process-defaults.md)).
- [ ] **Reference hardware** for performance budgets — which Windows 10 and 11 machines count ([PERFORMANCE.md](PERFORMANCE.md)).
- [ ] **Bench runner** — GitHub-hosted runners may be too noisy for a 10% regression gate; a self-hosted Windows runner may be needed.
- [ ] **TypeScript 7** — upgrade when typescript-eslint supports it ([ADR 0011](adr/0011-ui-toolchain.md)).
- [ ] **App icon** — current icon is a generated placeholder.

## Known gaps

- The CI workflow has been validated locally only (same commands on Windows and Linux), not yet on GitHub Actions.
- macOS has not been compiled at all, and the Linux app crate hasn't been compiled locally (no webkit2gtk); the `check` CI job covers both.
- The MSRV (1.90) job hasn't run; the lockfile was resolved for 1.90-compatible versions.
