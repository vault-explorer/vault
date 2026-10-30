# Contributing to Vault

Thanks for helping! Vault is pre-alpha, so the most useful contributions right now are the Phase 0 items in [docs/backlog/phase-0.md](docs/backlog/phase-0.md), benchmarks, and reviews of the design docs.

## Before you start

1. Read [docs/PLAN.md](docs/PLAN.md) and the doc for the area you're touching.
2. Read the **non-negotiable rules** and **coding conventions** in [CLAUDE.md](CLAUDE.md#non-negotiable-rules). They apply to human and AI contributors alike.
3. For anything bigger than a bug fix, open an issue or Discussion first.
   - Changing the **config format, plugin API or `vlt` CLI interface**? Write an [RFC](docs/rfcs/README.md).
   - Making an **architecture decision** (new framework or hard-to-swap crate, layering change, platform/licensing change)? Add an [ADR](docs/adr/README.md).

## Development setup

Follow [Building from source](README.md#building-from-source) in the README.

## Checks to run before opening a PR

CI runs all of these on Windows; please run them locally first.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint
pnpm format:check
pnpm typecheck
```

On Linux or macOS without the Tauri system libraries, drop `--workspace` to skip the app crate.

Benchmarks: `cargo bench -p vault-bench`. If your change touches a hot path, include before/after numbers and your hardware in the PR.

## Pull requests

- Keep PRs small and focused; one logical change per commit.
- Commit messages: imperative summary ≤ 72 chars, optionally prefixed with the area (`feat:`, `fix:`, `docs:`, `chore:`, `perf:`, `test:`, `refactor:`), then a body explaining *why*.
- Update [docs/STATUS.md](docs/STATUS.md) and any doc your change makes outdated, in the same PR.
- Fill in the PR template checklist.

## Data-loss bugs

If you find a bug that loses or damages files, label it `data-loss`. These block every release.

## License

By contributing, you agree that your contributions are dual-licensed under [MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE), without any additional terms or conditions.
