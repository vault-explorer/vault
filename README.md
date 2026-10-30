# Vault

**A cross-platform, open-source file explorer that opens instantly, finds anything in milliseconds, and gets out of your way — with a keyboard-first power layer for developers one keystroke below the surface.**

- **Fast is a feature.** Every interaction has a time budget, enforced in CI: cold start < 400 ms, 100k-item folders scroll at 60 fps, whole-disk filename search < 50 ms.
- **Familiar on day one.** Sidebar, tabs, previews, undo for every file operation.
- **Keyboard-first when you want it.** Command palette, fuzzy jump, Vim keymap, git status badges, integrated terminal, `vlt .` from any shell.
- **Never lose a file.** Every destructive operation is journaled and undoable.

> **Status: pre-alpha.** Nothing to use yet. We're in Phase 0 (technical spikes); Windows 10 22H2+ / 11 is the first target, macOS and Linux follow. See [docs/STATUS.md](docs/STATUS.md) and [docs/ROADMAP.md](docs/ROADMAP.md).

## Documentation

Start with [docs/PLAN.md](docs/PLAN.md) — it links to the vision, features, architecture, performance budgets, roadmap and decision records.

## Building from source

### Prerequisites

| | Version | Notes |
| --- | --- | --- |
| Rust | 1.90+ (stable) | via [rustup](https://rustup.rs) |
| Node.js | 22+ | |
| pnpm | 11 | `corepack enable` picks up the pinned version from `package.json` |
| **Windows** | 10 22H2+ or 11 | [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++"); WebView2 (preinstalled) |
| Linux (crates only during Phase 0–1) | | to build the app: `libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libsoup-3.0-dev` |
| macOS (crates only during Phase 0–1) | 13+ | Xcode Command Line Tools |

### Commands

```sh
pnpm install                 # JS dependencies (UI + Tauri CLI)
pnpm dev                     # run the app with hot reload
pnpm build                   # release build + installers (MSI/NSIS on Windows)
pnpm tauri build --no-bundle # release build without installers

cargo test --workspace       # all Rust tests (`cargo test` alone skips the app crate)
cargo run -p vault-cli -- --help   # the `vlt` CLI
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for lint, format and benchmark commands.

## Repository layout

```
crates/      core, index, platform, git, plugins, cli (vlt)
app/         src-tauri (Tauri 2 shell) and ui (SolidJS)
plugins/     first-party example plugins
bench/       performance benchmarks run in CI
docs/        plan, ADRs, RFCs, backlog
```

## Contributing

Contributions are welcome — read [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md). Report security issues privately as described in [SECURITY.md](SECURITY.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Vault by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
