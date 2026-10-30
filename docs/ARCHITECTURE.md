# Architecture

> Part of the [Vault plan](PLAN.md). Decisions behind this design are recorded in [adr/](adr/). Changes to the plugin API or config format go through an [RFC](rfcs/README.md).

## Layers

Three layers. **The UI never touches the disk.**

```
UI (TypeScript + SolidJS in WebView2)
  File views · Command palette · Preview pane · Terminal pane
        │  Tauri IPC: commands down, event stream up
Rust core (Tokio; owns all logic and disk access)
  Directory service · Index & search · Operations queue
  Git service · Plugin host (Lua) · Preview service
        │  shared platform traits: Watcher, Indexer, Thumbnailer, ShellIntegration, Trash
Platform & storage
  Windows (MFT/USN, Shell APIs) · macOS (later) · Linux (later) · SQLite + thumbnail cache
```

## Opening a folder

1. UI sends `open_dir(path)`, shows cached listing if any.
2. Directory service streams entries in batches (~500); UI appends to a virtualized list.
3. Metadata, git status, icons arrive as separate events and patch rows.
4. A watcher attaches; later changes are diff events, never full reloads.
5. Preview service preloads thumbnails for visible rows and selection neighbors.

## Key rules

- **Operations journal:** every file op is journaled before running → crash recovery + undo across restarts.
- **One scheduler:** everything is a task on one prioritized scheduler (visible folder > selection preview > background index). Anything > 100 ms is a cancellable background task with progress.
- **Virtual file systems** sit behind one trait (local first; archives, SFTP, S3 later).
- **Network paths get timeouts** so a dead share can't freeze the app.
- **One API surface:** the `vlt` CLI and plugins go through the same core APIs, operations queue and undo log.
- **Platform isolation:** OS-specific code lives only in `crates/platform`, behind the shared traits. Phase 2 ports add modules; they don't touch callers.

## Tech stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri 2 ([ADR 0001](adr/0001-tauri-2-and-rust-core.md)) |
| UI | SolidJS + TypeScript, Vite, pnpm ([ADR 0002](adr/0002-solidjs-ui-pending-spike.md)) |
| Virtual list | Custom, starting from TanStack Virtual |
| Async | Tokio |
| Watching | `notify` + Windows `ReadDirectoryChangesW`/USN fast path |
| Windows index | MFT + USN journal via `windows` crate (elevated helper optional; walk fallback) |
| Content search | `grep-searcher`, `ignore` |
| Fuzzy matching | `nucleo` |
| Index store | SQLite (`rusqlite`) + in-memory FST for names |
| Git | `gix` (fallback `git2`) |
| Previews | Windows thumbnail APIs (IShellItemImageFactory) first; `image`, `pdfium`, `syntect` fallbacks |
| Terminal | xterm.js + `portable-pty` |
| Plugins | Lua 5.4 via `mlua` ([ADR 0004](adr/0004-lua-plugins.md)) |
| Config | TOML + JSON Schema |
| Trash | `trash` crate |
| Tests | `cargo test`, `insta`, Playwright (UI), criterion benches |
| Packaging | MSI/MSIX via Tauri bundler, code-signed; Winget. Later: DMG, AppImage/deb/rpm/Flatpak, Homebrew, AUR |
| Telemetry | None by default; opt-in crash reports only |

## Repository layout

```
/crates
  core/        file model, operations queue, journal, undo log
  index/       indexer, search, query-language parser
  platform/    windows/ (MVP), macos/ and linux/ stubs behind shared traits
  git/         gix wrapper, status cache
  plugins/     Lua host, plugin API
  cli/         `vlt` companion binary
/app
  src-tauri/   Tauri commands, IPC, window management
  ui/          SolidJS app, components, themes
/plugins       first-party example plugins
/bench         performance benchmarks run in CI
/docs          user docs, plugin API reference, ADRs, RFCs, this plan
```

### Crate dependency direction

```
app/src-tauri ──┐
crates/cli ─────┼──▶ vault-core ──▶ vault-platform
                │        ▲
                ├──▶ vault-index ──▶ vault-platform
                ├──▶ vault-git
                └──▶ vault-plugins ──▶ vault-core
```

- `vault-platform` depends on no other Vault crate.
- Front ends (`app/src-tauri`, `crates/cli`) hold no business logic; they translate IPC/CLI input into core calls.
