# 0011. Use TypeScript 6.0, Vite and strict type-checked ESLint for the UI

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer (made during repo scaffolding)

## Context

The plan names SolidJS, TypeScript, Vite, pnpm, ESLint and Prettier but no versions or rule sets. At scaffolding time TypeScript 7 (the native port) was the latest release, but `typescript-eslint` 8 supports only TypeScript < 6.1.

## Decision

- **TypeScript `~6.0`** until typescript-eslint supports 7; then upgrade in its own PR.
- **Vite** with `vite-plugin-solid`; fixed dev port 5173 (`strictPort`) as Tauri expects; build target `chrome120` (WebView2 is evergreen Chromium).
- **ESLint flat config** using `defineConfig` from `eslint/config`: `@eslint/js` recommended, `typescript-eslint` **strictTypeChecked**, `eslint-plugin-solid` (TypeScript config) for `src/`, and `eslint-config-prettier` last. CI runs ESLint with `--max-warnings 0`. JS config files skip type-checked rules.
- **Prettier** defaults with `printWidth: 100` and trailing commas, scoped to `app/ui`.
- `tsconfig` is strict, plus `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` and `verbatimModuleSyntax`.
- No `@types/node`: config files avoid Node globals.
- These dev dependencies were approved during scaffolding: `@tauri-apps/api`, `@tauri-apps/cli`, `vite-plugin-solid`, `@eslint/js`, `typescript-eslint`, `eslint-plugin-solid`, `eslint-config-prettier`.

## Alternatives considered

- **TypeScript 7 without type-aware linting** — loses the strict type-checked rules, which catch real bugs in async IPC code.
- **Biome instead of ESLint + Prettier** — faster, but the plan names ESLint + Prettier and Biome lacks Solid-specific rules.

## Consequences

- If the Phase 0 spike picks Svelte 5 ([ADR 0002](0002-solidjs-ui-pending-spike.md)), the Solid-specific pieces (`vite-plugin-solid`, `eslint-plugin-solid`) are replaced; the rest stays.
