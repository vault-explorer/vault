# Status

> Update this file at the end of every working session (see [CLAUDE.md](../CLAUDE.md)).

**Last updated:** 2026-10-31
**Current phase:** 0 · Spikes ([ROADMAP.md](ROADMAP.md))
**Phase 0 gate:** 100k rows scroll at 60 fps on Windows 10 and 11 — *not yet measured*

## Done

- **2026-10-31 — Benchmark harness, first version** ([bench/README.md](../bench/README.md))
  - Deterministic fixture generator: `flat-10k`, `flat-100k`, opt-in `nested-1m`. Digests match on Windows and Linux.
  - Criterion benches for directory listing and a placeholder fuzzy bench, on new real APIs (`vault_core::dir`, `vault_index::fuzzy`).
  - App-level benches on Windows: cold start from an explicit ready signal, idle memory, and scroll frame times over raw CDP ([ADR 0015](adr/0015-app-benches-over-raw-cdp.md)).
  - Result JSON, `compare.mjs`, and a PR-only CI job that benches merge base and head in one job and reports without failing ([ADR 0014](adr/0014-report-only-bench-ci.md)).
  - **First numbers** (Windows 10 22H2, Ryzen 5 5600X, 16 GB, NVMe SSD, 239 Hz display, Defender on; medians, warm cache):

    | Metric | Result | Budget | |
    | --- | --- | --- | --- |
    | Cold start | 438–447 ms | < 400 ms | **Over budget** |
    | Idle memory | 76–77 MB | < 150 MB | Pass |
    | 10k listing, first 500 / full (core only) | 0.2 ms / 3.9 ms | < 50 ms / < 300 ms | Pass, partial |
    | 100k listing, first 500 / full (core only) | 0.2 ms / 40 ms | None | — |
    | Fuzzy, 100k names, one query (core only) | 4.5–4.7 ms | < 16 ms | Pass, partial |
    | Scroll probe: p95 frame / dropped / DOM rows | 4.3 ms / 0% / 44 | ≤ 16.7 ms / < 1% / ≤ 60 | Pass, probe list only |

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

1. 100k-row virtual list spike (SolidJS vs Svelte 5) — decides the UI framework and the phase gate. Replaces the scroll probe.
2. Tauri IPC streaming test — completes the open-folder budget.
3. MFT read prototype — first user of the `nested-1m` fixture.

Harness items still open are listed in [backlog/phase-0.md](backlog/phase-0.md#4-benchmark-harness--first-version-done).

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
- [ ] **Installer size enforcement** — sizes are reported in CI from #1 on, but not yet checked against the 15 MB budget; still open in the harness backlog.
- [ ] **Idle CPU** — budgeted but not measured yet.
- [ ] **Code signing** — CI artifacts are unsigned (SmartScreen warns); needs a certificate before any public build ([WINDOWS.md](WINDOWS.md)).
- [ ] **Reference hardware** for performance budgets — which Windows 10 and 11 machines count ([PERFORMANCE.md](PERFORMANCE.md)).
- [ ] **Cold start is over budget** — about 440 ms against 400 ms on the first machine measured, with an almost empty UI. About 350 ms passes before the page starts navigating (process launch and WebView2 startup) and about 30 ms in the UI. Needs investigating before more startup work lands.
- [ ] **Bench runner** — CI is report-only until there is a quiet runner and proven noise ([ADR 0014](adr/0014-report-only-bench-ci.md) lists the conditions).
- [ ] **Windows 11 numbers** — all local measurements so far are on Windows 10.
- [ ] **TypeScript 7** — upgrade when typescript-eslint supports it ([ADR 0011](adr/0011-ui-toolchain.md)).
- [ ] **App icon** — current icon is a generated placeholder.

## Known gaps

- `nested-1m` has only been generated on Linux (~19 GB). On NTFS, only the plan, not the tree on disk, has been tested.
- `pnpm tauri build` rewrites `tauri` and `tauri-build` in `app/src-tauri/Cargo.toml` to the table form, leaving the working tree dirty. Harmless; don't commit it.
- macOS and Linux are compile-checked in CI only, never run (expected until Phase 2).
- The CI artifact upload (#1) and Dependabot config (#3) are only validated once their PRs run or merge.
