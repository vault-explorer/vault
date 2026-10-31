# Performance budgets

> Part of the [Vault plan](PLAN.md). These budgets are non-negotiable: **a feature that breaks a budget doesn't ship.** If a phase runs long, cut features, never budgets.

## Budgets

The goal is CI enforcement, where a regression of **more than 10%** against `main` fails the build. For now CI only reports, because shared runners are too noisy to gate on ([ADR 0014](adr/0014-report-only-bench-ci.md)). The budgets themselves still apply.

| Interaction | Budget |
| --- | --- |
| Cold start to usable window | < 400 ms |
| Open folder with 10,000 items | first rows < 50 ms, full < 300 ms |
| Scroll 100,000-item folder | 60 fps, ≤ 60 DOM rows |
| Keystroke → fuzzy results | < 16 ms |
| Whole-disk filename search (warm) | < 50 ms |
| Initial index of 1M files (Windows/MFT) | < 60 s, background, pausable |
| Preview after selection (text/images) | < 100 ms |
| Idle memory | < 150 MB |
| Idle CPU | ~0% (event-driven, no polling) |
| Installer size | < 15 MB |

## Reference hardware

Budgets are measured on Windows 10 22H2 and Windows 11 (x64). Which machines count as reference hardware is still open — see [STATUS.md](STATUS.md). Until then, record CPU, RAM, disk type and OS build next to every number you report. `bench/scripts/run.mjs` adds them to each result file.

## How budgets are measured

- Rust-side budgets: criterion benches in [`/bench`](../bench/).
- App-level budgets: `bench/scripts/app-bench.mjs` launches release builds on Windows and drives WebView2 over the Chrome DevTools Protocol ([ADR 0015](adr/0015-app-benches-over-raw-cdp.md)).
- CI benches each PR's merge base and head in one job and posts the comparison to the job summary. It doesn't fail the build yet.
- [`bench/README.md`](../bench/README.md) explains how to run all of this and read the results.

### What is measured today

A budget counts as met only when its whole interaction is measured. "Partly" means a pass says nothing yet about the full budget.

| Interaction | Status | Measured by | Fully measurable |
| --- | --- | --- | --- |
| Cold start to usable window | **Measured** | `cold_start` | Now |
| Open folder with 10,000 items | **Partly:** core listing only, without IPC or paint | `dir_list.10k.*` | Phase 0, IPC streaming test |
| Scroll 100,000-item folder | **Partly:** harness works, but against a bench probe list, not the real list | `scroll.*` | Phase 0, 100k-row list spike |
| Keystroke → fuzzy results | **Partly:** placeholder that matches one query in the core, without UI or IPC | `fuzzy.100k.one_query` | Phase 2, command palette and fuzzy jump |
| Whole-disk filename search (warm) | Not measured | — | Phase 2, whole-disk index |
| Initial index of 1M files | Not measured; the `nested-1m` fixture is ready for it | — | Phase 0, MFT prototype |
| Preview after selection | Not measured | — | Phase 1, basic previews |
| Idle memory | **Measured** | `idle_memory.private_working_set` | Now |
| Idle CPU | Not measured | — | Could be added now; not built yet |
| Installer size | Reported by the CI `windows` job, not compared with the budget | — | Could be added now; not built yet |

### Benchmark metrics

`bench/budgets.json` is the machine-readable copy of this table. A `vault-bench` test fails if the two disagree.

| Metric | Limit | Source of the limit | Definition |
| --- | --- | --- | --- |
| `dir_list.10k.first_batch` | < 50 ms | Open folder: first rows | `vault_core::dir::list_dir`, first 500 entries |
| `dir_list.10k.full` | < 300 ms | Open folder: full | Same, whole listing |
| `dir_list.100k.first_batch` | — | No budget for this folder size | First 500 entries of 100,000 |
| `dir_list.100k.full` | — | No budget for this folder size | Whole listing of 100,000 |
| `fuzzy.100k.one_query` | < 16 ms | Keystroke → fuzzy results | `vault_index::fuzzy`, one query over 100,000 names |
| `cold_start` | < 400 ms | Cold start to usable window | From launching the process to first contentful paint with the core reachable; warm OS cache and WebView2 profile |
| `idle_memory.private_working_set` | < 150 MB | Idle memory | Private working set of the app and all WebView2 processes after 30 s idle, the same memory Task Manager shows (1 MB = 1024² bytes) |
| `idle_memory.working_set` | — | Context only | Full working set; counts memory shared between processes more than once |
| `scroll.frame_time.p50` | — | Context only | Median interval between `requestAnimationFrame` callbacks during a real scroll gesture |
| `scroll.frame_time.p95` | ≤ 16.7 ms | Scroll: 60 fps, as defined in [backlog/phase-0.md](backlog/phase-0.md) | 95th percentile of the same |
| `scroll.frame_time.p99` | — | Context only | 99th percentile of the same |
| `scroll.dropped_frames` | < 1 % | Scroll: 60 fps, as defined in the backlog | Share of intervals over 25 ms (1.5 frames at 60 fps), so the result doesn't depend on the monitor's refresh rate |
| `scroll.dom_rows_max` | ≤ 60 rows | Scroll: ≤ 60 DOM rows | Most rows mounted in any frame |

## Rules that keep us inside budget

- Anything that can take > 100 ms runs as a cancellable background task with progress ([ARCHITECTURE.md](ARCHITECTURE.md#key-rules)).
- Directory listings stream in batches (~500 entries); the UI never waits for a full listing.
- Lists are virtualized from day one.
- No polling. Idle work is driven by watcher and OS events.
