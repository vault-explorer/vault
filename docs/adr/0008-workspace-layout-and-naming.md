# 0008. Name crates `vault-*` and keep the app crate out of default members

- **Status:** Accepted — bundle identifier superseded by [0013](0013-bundle-identifier.md)
- **Date:** 2026-10-29
- **Deciders:** project maintainer (made during repo scaffolding)

## Context

The plan fixes the directory layout ([ARCHITECTURE.md](../ARCHITECTURE.md#repository-layout)) but not package names, how the Tauri app fits the Cargo workspace, or the app's bundle identifier. The Tauri crate needs webview system libraries (webkit2gtk) to build on Linux, which most Rust contributors won't have installed.

## Decision

- **Package names:** `vault-core`, `vault-index`, `vault-platform`, `vault-git`, `vault-plugins`, `vault-cli` (binary `vlt`), `vault-bench`, and `vault-app` for `app/src-tauri` (binary `vault`). Directory names stay as in the plan. Nothing is published to crates.io (`publish = false`).
- **One Cargo workspace** at the root containing all crates, the app and the bench crate. `default-members` excludes `vault-app`, so `cargo build` / `cargo test` work anywhere; `--workspace` includes the app.
- **One pnpm workspace** at the root containing `app/ui` (package `vault-ui`). The Tauri CLI is a root dev-dependency, so `pnpm dev` / `pnpm tauri …` run from the repo root.
- **Bundle identifier:** `dev.vaultexplorer.vault` — **provisional** until the domain / GitHub org question is settled ([STATUS.md](../STATUS.md)). Changing it later changes the app-data folder location, so settle it before any public build.
- **Benchmarks** live in the `vault-bench` crate under `bench/`, with spike code in `bench/spikes/`.

## Alternatives considered

- **Unprefixed crate names (`core`, `index`)** — `core` clashes with Rust's `core` crate.
- **Separate Cargo workspace for the app** — two lockfiles drift apart.
- **Exclude nothing from default members** — breaks `cargo test` on Linux machines without webkit2gtk.

## Consequences

- CI must pass `--workspace` explicitly to cover the app (it does).
- The `repository` field is omitted from manifests until the GitHub org exists; issue-template links use an `OWNER` placeholder.
