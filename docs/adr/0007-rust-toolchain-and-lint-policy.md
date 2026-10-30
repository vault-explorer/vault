# 0007. Use Rust edition 2024, MSRV 1.90 and strict lints

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** project maintainer (made during repo scaffolding)

## Context

The plan says "Rust (stable)" but doesn't fix an edition, a minimum supported Rust version (MSRV), or lint levels. Cargo's resolver 3 picks dependency versions compatible with the declared `rust-version`. With an older MSRV (1.85), it resolved Tauri 2.11 and criterion 0.7 instead of the current Tauri 2.12 and criterion 0.8, which need Rust 1.90 and 1.86.

## Decision

- **Edition 2024**, **resolver 3**, **`rust-version = "1.90"`** for every crate (set once in `[workspace.package]`).
- Developers and CI use the latest **stable** toolchain; a dedicated CI job checks that the MSRV still builds.
- Raise the MSRV whenever a dependency we want needs it — it's a floor, not a promise to users (we ship binaries, not a library).
- Lints are configured once in `[workspace.lints]`: rustc `missing_docs` and `rust_2018_idioms`; clippy `all` + `pedantic`, plus `unwrap_used`, `todo` and `dbg_macro` as warnings. CI runs clippy with `-D warnings`, so every warning blocks a merge.
- `clippy.toml` allows `unwrap`/`expect` in tests and whitelists proper nouns (SQLite, WebView2) for `doc_markdown`.

## Alternatives considered

- **Pin an exact toolchain in `rust-toolchain.toml`** — reproducible, but forces a toolchain download on every contributor and needs manual bumps. Revisit if toolchain drift causes CI-only failures.
- **Lower MSRV (1.85)** — no benefit for an app, and it held back Tauri.
- **Clippy defaults only** — `pedantic` catches real issues (casts, needless clones) that matter on hot paths. Specific pedantic lints can be allowed in `[workspace.lints]` when they're noise.

## Consequences

- New crates must use `[lints] workspace = true` and the workspace package fields.
- Every public item needs a doc comment.
