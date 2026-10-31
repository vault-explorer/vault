# Phase 0 · Spikes — backlog

> Timing: weeks 1–4 ([ROADMAP.md](../ROADMAP.md)). **Gate: 100k rows scroll at 60 fps on Windows 10 and 11.** If the phase runs long, cut scope — never relax the gate or a [budget](../PERFORMANCE.md).

Spikes are for learning, not shipping: keep spike code in `bench/spikes/<name>/` unless it's clearly the start of the real implementation. Every spike ends with written results — numbers, hardware (CPU, RAM, disk, OS build), and a decision recorded as an ADR.

**Measurement rules for every item:** release builds; Windows 10 22H2 **and** Windows 11; report median and p95 over at least 5 runs; note warm vs cold file-system cache.

---

## 1. 100k-row virtual list spike — SolidJS vs Svelte 5

Decides [ADR 0002](../adr/0002-solidjs-ui-pending-spike.md). Budget: *Scroll 100,000-item folder — 60 fps, ≤ 60 DOM rows.*

- [ ] Build the same Details-view list (name, size, modified, type columns; icons as placeholders) in **SolidJS** and **Svelte 5**, both inside the Tauri app on WebView2, both starting from TanStack Virtual.
- [ ] Feed both from the same in-memory 100,000-row synthetic dataset generated in Rust and sent once over IPC.
- [ ] Scripted scroll scenarios: constant-speed wheel scroll top→bottom, `PageDown` held, `End`/`Home` jumps, scrollbar drag.
- [ ] Measure with `requestAnimationFrame` timestamps and the WebView2 DevTools performance trace: fps, frame-time p50/p95/p99, dropped frames, DOM row count, JS heap.
- [ ] Measure row-patch cost: update metadata on 1,000 random rows while scrolling (simulates git/metadata events).
- [ ] Write results and the framework choice into a new ADR (confirming or superseding 0002).

**Acceptance criteria**

- [ ] Winning implementation sustains **60 fps** (p95 frame time ≤ 16.7 ms, < 1% dropped frames) in every scroll scenario, on Windows 10 and 11.
- [ ] **≤ 60 DOM rows** mounted at any time (asserted in the spike, not eyeballed).
- [ ] Patching 1,000 rows during scroll keeps p95 frame time ≤ 16.7 ms.
- [ ] Results table for both frameworks committed; decision ADR merged.

## 2. Tauri IPC streaming test — batched directory entries

Validates the "open a folder" flow in [ARCHITECTURE.md](../ARCHITECTURE.md#opening-a-folder). Budget: *Open folder with 10,000 items — first rows < 50 ms, full < 300 ms.*

- [ ] Rust command `open_dir(path)` that lists a directory on a Tokio task and streams entries to the UI in batches.
- [ ] Compare transports: Tauri 2 `ipc::Channel` vs global events; JSON vs a compact binary encoding.
- [ ] Sweep batch sizes: 100, 500, 1,000, 5,000 entries.
- [ ] Fixtures: generated folders with 10,000 and 100,000 files on a local NTFS SSD; one real large folder (e.g. a `node_modules`).
- [ ] Measure end to end, from `invoke` in the UI to rows painted: time to first rows, time to full listing, UI frame times while streaming.
- [ ] Prove cancellation: navigating away mid-stream stops the producer within one batch.
- [ ] Record the chosen transport, encoding and batch size in an ADR.

**Acceptance criteria**

- [ ] 10,000 items: **first rows painted < 50 ms**, **full listing < 300 ms** (warm cache, median and p95).
- [ ] Streaming 100,000 items keeps the UI at **60 fps** (p95 frame time ≤ 16.7 ms) and stays responsive to input.
- [ ] The listing work is a cancellable background task (rule: anything > 100 ms), verified by a test.
- [ ] No disk access from the UI; the webview receives only DTOs.

## 3. MFT read prototype

Feeds the `Indexer` trait in `crates/platform` ([WINDOWS.md](../WINDOWS.md)). Budgets: *Initial index of 1M files — < 60 s, background, pausable*; enables *whole-disk filename search (warm) < 50 ms* later.

- [ ] Enumerate an NTFS volume's MFT via `FSCTL_ENUM_USN_DATA` (`windows` crate), reconstruct full paths from file reference numbers.
- [ ] Tail the USN journal (`FSCTL_READ_USN_JOURNAL`) and print create/rename/delete changes.
- [ ] Run as a standalone elevated example (`bench/spikes/mft/`); **Vault itself must never require admin** — document how the optional elevated helper would hand results to the unelevated app.
- [ ] Implement the directory-walk fallback and compare file counts against the MFT result.
- [ ] Handle long paths (`\\?\`), reserved names, hard links, and non-NTFS volumes (ReFS, FAT32, exFAT → fallback).
- [ ] Measure on a volume with ≥ 1,000,000 files: wall time, peak memory, CPU.
- [ ] Record the approach (and helper-process design) in an ADR.

**Acceptance criteria**

- [ ] Enumerates **1,000,000 files in < 60 s** (median of 5 cold runs), in the background, and can pause/resume.
- [ ] MFT and walk results agree (same path set, differences explained).
- [ ] USN tailing reports changes made during the run.
- [ ] Peak memory recorded and consistent with the *idle memory < 150 MB* budget for the steady-state index (or a plan to get there).

## 4. Benchmark harness — first version done

Makes [PERFORMANCE.md](../PERFORMANCE.md) measurable. How to use it: [bench/README.md](../../bench/README.md). CI enforcement is deferred by [ADR 0014](../adr/0014-report-only-bench-ci.md), so the unticked items below stay open.

- [x] Criterion benches in `bench/` for Rust-side budgets, with shared fixtures (synthetic trees, name corpora) in `vault-bench`.
- [x] UI-side harness: drive the built app over WebView2's DevTools protocol to measure cold start and scroll frame times and DOM rows ([ADR 0015](../adr/0015-app-benches-over-raw-cdp.md)). Scroll runs against a bench probe list until item 1 lands.
- [ ] UI-side harness: keystroke → results latency and preview latency. Not measurable until those features exist.
- [x] Process metrics: idle memory (app + WebView2 processes).
- [ ] Process metrics: idle CPU after 60 s.
- [ ] Installer size check on the MSI/NSIS output. CI reports sizes but doesn't compare them with the budget.
- [x] Compare each PR against its base in the `bench` CI job, posting a summary. Replaces storing `main`'s results: both commits are built and benched in one job ([ADR 0014](../adr/0014-report-only-bench-ci.md)).
- [ ] Measure runner noise over more runs: run the same commit 5 times; if variance > 5%, move the bench job to a dedicated (self-hosted) Windows runner. First data point in ADR 0014.
- [ ] Windows 11 numbers. So far only Windows 10 22H2 locally, plus whatever image `windows-latest` uses.

**Acceptance criteria**

- [x] Every row in the budget table maps to a named benchmark, or is explicitly marked "not measurable yet" with the phase it becomes measurable. See [PERFORMANCE.md](../PERFORMANCE.md#what-is-measured-today).
- [ ] The CI `bench` job fails a PR that regresses any measured budget by **> 10%** against `main`. Deferred: report-only for now, with the conditions for blocking in ADR 0014.
- [ ] Cold start to usable window (< 400 ms), idle memory (< 150 MB) and installer size (< 15 MB) are measured on every PR. Cold start and idle memory are; installer size is reported only.
- [ ] Run-to-run noise on the CI runner is documented and below the regression threshold. See ADR 0014.

---

## Done in Phase 0

- [x] Monorepo scaffold: Cargo + pnpm workspaces, crates, Tauri 2 app, example IPC command
- [x] CI: Windows build/test/lint, macOS/Linux `cargo check`, MSRV check, placeholder bench job
- [x] Docs, ADRs 0001–0006, RFC process, community files
- [x] Benchmark harness, first version (item 4; open sub-items remain listed there)
