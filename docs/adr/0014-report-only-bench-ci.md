# 0014. Report benchmark comparisons in CI without failing the build

- **Status:** Proposed
- **Date:** 2026-10-31
- **Deciders:** project maintainer

## Context

[PERFORMANCE.md](../PERFORMANCE.md) calls for CI to fail any PR that regresses a budget by more than 10% against `main`. Our CI runs on GitHub's shared `windows-latest` runners, where other tenants on the host, varying hardware and image updates all move timings. A gate on those numbers would fail PRs for changes they didn't make, and people soon stop trusting a gate like that.

Storing `main`'s results and comparing later would make this worse, because the two numbers could come from different machines days apart.

**Measured noise.** Same commit, measured twice, comparing medians:

| Metric | Local desktop, 2 runs | `windows-latest`, 1 job |
| --- | --- | --- |
| `cold_start` | +2.1% | pending |
| `idle_memory.private_working_set` | -0.8% | pending |
| `dir_list.10k.full` | -1.4% | pending |
| `dir_list.100k.full` | +1.9% | pending |
| `fuzzy.100k.one_query` | -3.7% | pending |
| `scroll.frame_time.p95` | 0.0% | pending |

The local numbers are the upper bound of what a quiet machine can do. Two runs can't establish a noise distribution, so these numbers show what to expect, not a limit.

## Decision

The `bench` job runs on every PR and builds and benches the PR's merge base and head **in the same job**, so both see the same runner. It posts a table to the job summary and uploads the result files. **It can't fail the build:** the job and its bench steps use `continue-on-error`, and `compare.mjs` exits 0 for any valid input.

We compare with the merge base rather than the tip of `main`, so the table shows only what the PR changed. A base from before the harness has nothing to run, so the job measures head against itself and says so.

## Alternatives considered

- **Blocking now on hosted runners** — false failures on unrelated PRs.
- **Blocking with a much wider threshold** — a threshold wide enough to survive the noise would miss the regressions we care about.
- **Stored baselines from `main`** (the backlog's original idea) — different machines and times, so more noise. Base and head in one job costs more CI time but cancels most of it.
- **Self-hosted Windows runner now** — needs a dedicated machine and a security setup for running PR code, which isn't worth it for a project this early.

## Consequences

- A PR can regress a budget and still merge. Reviewers must read the table; `FAIL` and ⚠ don't block anything.
- PERFORMANCE.md's "regression fails the build" is a goal until the conditions below are met. The rule that a feature breaking a budget doesn't ship still applies, and is enforced by review.
- The job builds the app twice per commit and benches two commits, so it's the slowest job on a PR. Because it can't block, nobody needs to wait for it to merge.

### What it would take to make it blocking

All of these:

1. **A quiet runner.** A dedicated self-hosted Windows machine (physical, or a VM with no neighbours) with a real GPU or a fixed display setup. Fork PRs must not run on it unreviewed, because that runs untrusted code on our machine.
2. **Proven noise.** Same-commit runs (at least 20, across several days) where the 95th-percentile change for each metric is under 5%, half the 10% threshold. The check is per metric, so a noisy metric can stay report-only while the rest block.
3. **Stable measurement.** Reference hardware decided ([STATUS.md](../STATUS.md)), and the scroll metrics pointed at the real list instead of the probe.
4. **A demonstration.** A deliberately slow commit fails the job, and at least 20 ordinary PRs pass with no false failures while it runs in shadow mode.
5. **The change itself.** Remove `continue-on-error` and add a failure mode to `compare.mjs`, with an override label for accepted regressions that a maintainer has to apply.
6. **A decision on absolute budgets.** Decide separately whether going over budget blocks, as well as the 10% relative rule. Cold start is already over budget on the first machine we measured.
