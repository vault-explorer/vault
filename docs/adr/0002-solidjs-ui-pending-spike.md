# 0002. Use SolidJS for the UI, pending the Phase 0 spike

- **Status:** Accepted (provisional — confirmed or superseded by the Phase 0 virtual-list spike)
- **Date:** 2026-10-29
- **Deciders:** project maintainer

## Context

The UI must scroll a 100,000-item folder at 60 fps with at most 60 DOM rows, and return fuzzy results within 16 ms of a keystroke ([PERFORMANCE.md](../PERFORMANCE.md)). Row updates arrive as many small patches (metadata, git status, icons), so fine-grained updates matter more than raw render throughput.

## Decision

Build the UI in **TypeScript + SolidJS**, with Vite and pnpm. The virtual list is custom, starting from TanStack Virtual.

This is provisional. The Phase 0 spike builds the same 100k-row list in SolidJS and **Svelte 5** and measures fps on Windows 10 and 11. If Svelte 5 is clearly better against the budgets, a new ADR supersedes this one.

## Alternatives considered

- **Svelte 5** — runes give similar fine-grained reactivity; the main contender, evaluated in the spike.
- **React** — largest ecosystem, but virtual-DOM diffing on frequent row patches costs frame time.
- **Vanilla TS** — fastest possible, but slower to build a large UI with.

## Consequences

- The scaffold in `app/ui` uses SolidJS now. Keep it thin until the spike decides; avoid deep framework-specific investment.
- The spike results (fps, DOM rows, frame-time percentiles, hardware) must be written into the superseding or confirming ADR.
