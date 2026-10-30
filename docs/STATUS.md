# Status

> Update this file at the end of every working session (see [CLAUDE.md](../CLAUDE.md)).

**Last updated:** 2026-10-30
**Current phase:** 0 · Spikes ([ROADMAP.md](ROADMAP.md))
**Phase 0 gate:** 100k rows scroll at 60 fps on Windows 10 and 11 — *not yet measured*

## Done

- **2026-10-30 — GitHub and CI**
  - Repo published at [vault-explorer/vault](https://github.com/vault-explorer/vault). The first CI run on `main` was green on all five jobs: Windows build/test/lint, macOS and Ubuntu `cargo check`, MSRV 1.90 and the bench placeholder. Nothing needed fixing.
  - Opened for review:
    - [#1](https://github.com/vault-explorer/vault/pull/1) — upload Windows exe + MSI + NSIS installer as a CI artifact on every PR, with build sizes in the job summary. First run: `vault.exe` 4.0 MB, MSI 1.9 MB, NSIS installer 1.3 MB (budget < 15 MB).
    - [#2](https://github.com/vault-explorer/vault/pull/2) — bundle identifier `io.github.vault-explorer.vault` ([ADR 0013](adr/0013-bundle-identifier.md)), issue-template links, Code of Conduct contact, `repository` field in manifests.
    - [#3](https://github.com/vault-explorer/vault/pull/3) — Dependabot (cargo, npm, GitHub Actions; weekly; minor/patch grouped; TypeScript majors ignored per ADR 0011).
  - Private vulnerability reporting is enabled on the repo.
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

Before that: review and merge #1–#3, then merge the first Dependabot PRs (expected: `pnpm/action-setup` v4 → v6 to clear the Node 20 deprecation warning).

## Open questions

From the plan:

- [ ] SolidJS vs Svelte 5 — decided by the Phase 0 100k-row list benchmark
- [ ] Lua-only plugins for 1.0, or WASM from the start
- [ ] Final license confirmation (MIT / Apache-2.0)
- [x] ~~GitHub org~~ — `vault-explorer` (2026-10-30)
- [ ] Domain, package-name availability for "Vault" and `vlt`; logo

Raised during scaffolding:

- [x] ~~**Bundle identifier**~~ — `io.github.vault-explorer.vault` ([ADR 0013](adr/0013-bundle-identifier.md); lands with #2)
- [x] ~~**Code of conduct contact** and **`OWNER`** in issue templates~~ — resolved in #2
- [ ] **Installer size enforcement** — sizes are reported in CI from #1 on, but not yet checked against the 15 MB budget; part of the benchmark harness.
- [ ] **Code signing** — CI artifacts are unsigned (SmartScreen warns); needs a certificate before any public build ([WINDOWS.md](WINDOWS.md)).
- [ ] **Reference hardware** for performance budgets — which Windows 10 and 11 machines count ([PERFORMANCE.md](PERFORMANCE.md)).
- [ ] **Bench runner** — GitHub-hosted runners may be too noisy for a 10% regression gate; a self-hosted Windows runner may be needed.
- [ ] **TypeScript 7** — upgrade when typescript-eslint supports it ([ADR 0011](adr/0011-ui-toolchain.md)).
- [ ] **App icon** — current icon is a generated placeholder.

## Known gaps

- macOS and Linux are compile-checked in CI only, never run (expected until Phase 2).
- The CI artifact upload (#1) and Dependabot config (#3) are only validated once their PRs run or merge.
