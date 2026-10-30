# Vault — Product & Technical Plan

> Source of truth for the Vault project. Agents and contributors: read this before making changes.
> If something here is wrong or outdated, update this file in the same change, don't silently diverge.

## 1. Vision

**Pitch:** A cross-platform, open-source file explorer that opens instantly, finds anything in milliseconds, and gets out of your way — with a keyboard-first power layer for developers one keystroke below the surface.

**Audience (priority order):**
1. Developers — git status, fuzzy jump, open in editor, no mouse required.
2. Power users — huge folders, fast search, bulk rename, previews, dual pane.
3. Everyone else — familiar on day one; never sees a feature they didn't ask for.

**Design principles (use these to settle every argument):**
- **Fast is a feature.** Every interaction has a time budget (section 6). A feature that breaks a budget doesn't ship.
- **Simple surface, deep core.** Default window looks familiar. Power lives in the command palette, keybindings and settings.
- **Keyboard and mouse are equals.** Everything clickable has a shortcut and vice versa.
- **Never lose a file.** Destructive actions are undoable or go to trash; long ops are resumable and transparent.
- **Plain-text config.** Settings, keymaps, themes are TOML files with a GUI on top.
- **Extensible, not bloated.** Core stays small; niche features are plugins.

**Non-goals for 1.0:** cloud sync, cross-device tag databases, mobile apps, AI features.

## 2. Decisions made

| Decision | Choice |
| --- | --- |
| Name | **Vault** (app), **`vlt`** (companion CLI — `vault` is taken by HashiCorp Vault) |
| License | Open source; MIT / Apache-2.0 dual license (recommended, confirm before first release) |
| MVP platform | **Windows 10 22H2+ and Windows 11** (x64; ARM64 later) |
| Later platforms | macOS 13+, Ubuntu 22.04+ (Phase 2) |
| Core | Rust (stable), Tokio |
| Shell | Tauri 2 |
| UI | TypeScript + SolidJS (pending Phase 0 spike vs Svelte 5) |
| Plugins | Lua 5.4 via `mlua` for 1.0; WASM later |
| Config | TOML + JSON Schema |
| Telemetry | None by default; opt-in crash reports only |

## 3. Open questions

- [ ] SolidJS vs Svelte 5 — decided by the Phase 0 100k-row list benchmark
- [ ] Lua-only plugins for 1.0, or WASM from the start
- [ ] Final license confirmation
- [ ] Domain, GitHub org, package-name availability for "Vault"; logo

## 4. Features

### 4.1 Core experience (everyone)
- Sidebar: Favorites, Recent, Drives, Network, Trash; drag folders in to pin.
- Smart path bar: breadcrumbs; `Ctrl+L` turns it into a text field with autocomplete; non-path text searches.
- Views: Details (sortable columns), Grid (thumbnails), Columns (Miller). Remembered per folder.
- Tabs with drag-reorder and session restore.
- Preview pane (`Space`): images, video, audio, PDF, Markdown, syntax-highlighted code, archive listings, folder summaries.
- Status bar: counts, selection size, free space, background-operation progress chip.
- File ops: copy, move, rename, delete, new file/folder — all undoable (`Ctrl+Z`). Delete → Recycle Bin; `Shift+Delete` permanent with confirm.
- Operations queue: background copies with speed, ETA, pause, cancel; one clear conflict dialog (replace / skip / keep both / apply to all).
- Archives: browse zip/7z/tar.gz as folders; compress/extract.
- Type-to-select, filter-as-you-type, native "Open with", light/dark/system theme, density setting.
- First-run 30-second tour with opt-in "Show me the power features".

### 4.2 Instant search (headline feature)
- Global fuzzy jump (`Ctrl+P`), frecency-ranked.
- Whole-disk filename search < 50 ms (Windows: MFT + USN journal; fallback: directory walk).
- Content search via ripgrep internals: regex, `.gitignore`-aware, streamed results.
- Query language: `ext:ts size:>1mb modified:<7d in:~/code -node_modules`. Saved searches as sidebar virtual folders.

### 4.3 Keyboard-first
- Command palette (`Ctrl+Shift+P`): every action, setting and plugin command with its shortcut.
- Keymap presets: Default, Vim, Emacs-ish; fully remappable in TOML.
- Quick-jump hints (label visible items with letters).
- Marks: `m a` to set, `' a` to jump.

### 4.4 Developer integrations
- Git: status badges, branch in path bar, dimmed ignored files, "changed files only" filter (via `gix`).
- Integrated terminal pane (xterm.js + `portable-pty`) following the current folder; "Open terminal here" for external terminals (Windows Terminal first).
- Open in editor: VS Code, Cursor, Zed, JetBrains, Neovim.
- Project detection (`package.json`, `Cargo.toml`, `pyproject.toml`, `.sln`) with runnable scripts.
- Copy path variants: absolute, repo-relative, `file://`, POSIX vs Windows style.
- Hidden/dotfile toggle; `.gitignore` view mode.
- `vlt .` CLI to open Vault from a terminal; file-picker mode for other tools.

### 4.5 Automation
- Bulk rename with live preview: find/replace, regex, numbering, case, or "edit names in your editor".
- Custom actions in config (`ffmpeg -i {file} {stem}.mp4`), scoped by file type, shown in context menu and palette.
- Lua plugins: previewers, columns, actions, sidebar panels, themes.
- Folder rules (post-1.0).

## 5. Architecture

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

**Opening a folder:**
1. UI sends `open_dir(path)`, shows cached listing if any.
2. Directory service streams entries in batches (~500); UI appends to a virtualized list.
3. Metadata, git status, icons arrive as separate events and patch rows.
4. A watcher attaches; later changes are diff events, never full reloads.
5. Preview service preloads thumbnails for visible rows and selection neighbors.

**Key rules:**
- Operations journal: every file op is journaled before running → crash recovery + undo across restarts.
- Everything is a task on one prioritized scheduler (visible folder > selection preview > background index). Anything > 100 ms is a cancellable background task with progress.
- Virtual file systems behind one trait (local first; archives, SFTP, S3 later).
- Network paths get timeouts so a dead share can't freeze the app.
- The `vlt` CLI and plugins go through the same core APIs, operations queue and undo log.

## 6. Performance budgets (CI-enforced; >10% regression fails the build)

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

## 7. Tech stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri 2 |
| UI | SolidJS + TypeScript, Vite, pnpm |
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
| Plugins | Lua 5.4 via `mlua` |
| Trash | `trash` crate |
| Tests | `cargo test`, `insta`, Playwright (UI), criterion benches |
| Packaging | MSI/MSIX via Tauri bundler, code-signed; Winget. Later: DMG, AppImage/deb/rpm/Flatpak, Homebrew, AUR |

## 8. Windows-specific notes (MVP)

- WebView2 runtime is preinstalled on Windows 10 22H2 / 11; installer should bootstrap it if missing.
- Windows 10 left mainstream support in October 2025 — test on both 10 and 11.
- Long paths (`\\?\`), UNC shares, drive letters, case-insensitive matching, reserved names (`CON`, `NUL`…).
- Raw MFT reads need elevation → optional helper service; never require admin to run Vault.
- Shell context menu extensions (IContextMenu) can crash hosts → load them out of process.
- Respect Windows shortcuts: `F2` rename, `Delete` to Recycle Bin, `Alt+Up` parent, `Ctrl+Shift+N` new folder.
- Code-sign builds early to build SmartScreen reputation.
- Keep all Windows code behind platform traits so macOS/Linux ports in Phase 2 only add modules.

## 9. Repository layout

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
/docs          user docs, plugin API reference, ADRs, this plan
/bench         performance benchmarks run in CI
```

## 10. Open source setup

- CI (GitHub Actions): build + test on Windows per PR (macOS/Linux compile checks from day one, full tests from Phase 2); clippy, rustfmt, eslint, prettier; benchmark comparison vs `main`; nightly builds; tagged releases publish installers + updater feed.
- Community: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, issue/PR templates, `good first issue` labels, GitHub Projects roadmap, Discussions.
- RFC process (Markdown PR in `docs/rfcs/`) for config-format or plugin-API changes.
- Architecture Decision Records in `docs/adr/`.

## 11. Roadmap (suggested pace: part-time solo)

Each phase ends with a pass/fail gate. If a phase runs long, move features out — never relax the gate.

| Phase | Timing | Scope | Gate |
| --- | --- | --- | --- |
| 0 · Spikes | Weeks 1–4 | 100k-row virtual list in SolidJS vs Svelte; Tauri IPC streaming test; MFT read prototype; monorepo, CI, benchmark harness | 100k rows scroll at 60 fps on Windows 10 and 11 |
| 1 · MVP daily driver (Windows) | Months 1–3 | Sidebar, smart path bar, details/grid/columns, tabs, filter; file ops with undo + operations queue; basic previews; TOML settings | You use it as your main explorer for two weeks |
| 2 · Developer alpha | Months 4–6 | Command palette, fuzzy jump, Vim keymap, git badges, open in editor, terminal pane; whole-disk index; `vlt` CLI; macOS + Linux ports; public installers | Public alpha on Windows, macOS, Linux |
| 3 · Power beta | Months 7–9 | Content search + query language, saved searches, bulk rename, custom actions, Lua plugin API, archives, dual pane, PDF/media previews | Plugin API frozen; zero open data-loss bugs |
| 4 · 1.0 | Months 10–12 | Accessibility, translations, onboarding tour, signed installers, auto-update, Winget/Homebrew/Flathub, docs site | Every performance budget green on all three OSes |

Post-1.0: WASM plugins, SFTP/S3 virtual file systems, folder rules, plugin registry.

## 12. Risks

| Risk | Mitigation |
| --- | --- |
| Scope creep | Phase gates; cut features, never budgets |
| Webview performance on huge folders | Virtualize from day one; CI benchmarks; logic in Rust so UI is swappable |
| MFT needs admin | Optional elevated helper; walk fallback |
| Data-loss bug | Journal, undo log, fuzz/property tests on copy/move, beta channel |
| Shell extension crashes | Out-of-process loading |
| Maintainer burnout | Ship small, recruit early, stable plugin API |