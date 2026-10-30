# Performance budgets

> Part of the [Vault plan](PLAN.md). These budgets are non-negotiable: **a feature that breaks a budget doesn't ship.** If a phase runs long, cut features, never budgets.

## Budgets

CI-enforced; a regression of **more than 10%** against `main` fails the build.

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

Budgets are measured on Windows 10 22H2 and Windows 11 (x64). Which machines count as reference hardware is still open — see [STATUS.md](STATUS.md). Until then, record CPU, RAM, disk type and OS build next to every number you report.

## How budgets are measured

- Rust-side budgets: criterion benches in [`/bench`](../bench/).
- UI-side budgets (fps, DOM rows, keystroke latency): Playwright-driven measurements against the built app (to be built in Phase 0 — see [backlog/phase-0.md](backlog/phase-0.md)).
- CI compares each PR's results against `main` and fails on > 10% regression. Until the harness lands, the `bench` CI job is a placeholder.

## Rules that keep us inside budget

- Anything that can take > 100 ms runs as a cancellable background task with progress ([ARCHITECTURE.md](ARCHITECTURE.md#key-rules)).
- Directory listings stream in batches (~500 entries); the UI never waits for a full listing.
- Lists are virtualized from day one.
- No polling. Idle work is driven by watcher and OS events.
