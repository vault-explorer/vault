# Features

> Part of the [Vault plan](PLAN.md). Which phase ships what is in [ROADMAP.md](ROADMAP.md). Every feature must fit the [performance budgets](PERFORMANCE.md).

## 1. Core experience (everyone)

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

## 2. Instant search (headline feature)

- Global fuzzy jump (`Ctrl+P`), frecency-ranked.
- Whole-disk filename search < 50 ms (Windows: MFT + USN journal; fallback: directory walk).
- Content search via ripgrep internals: regex, `.gitignore`-aware, streamed results.
- Query language: `ext:ts size:>1mb modified:<7d in:~/code -node_modules`. Saved searches as sidebar virtual folders.

## 3. Keyboard-first

- Command palette (`Ctrl+Shift+P`): every action, setting and plugin command with its shortcut.
- Keymap presets: Default, Vim, Emacs-ish; fully remappable in TOML.
- Quick-jump hints (label visible items with letters).
- Marks: `m a` to set, `' a` to jump.

## 4. Developer integrations

- Git: status badges, branch in path bar, dimmed ignored files, "changed files only" filter (via `gix`).
- Integrated terminal pane (xterm.js + `portable-pty`) following the current folder; "Open terminal here" for external terminals (Windows Terminal first).
- Open in editor: VS Code, Cursor, Zed, JetBrains, Neovim.
- Project detection (`package.json`, `Cargo.toml`, `pyproject.toml`, `.sln`) with runnable scripts.
- Copy path variants: absolute, repo-relative, `file://`, POSIX vs Windows style.
- Hidden/dotfile toggle; `.gitignore` view mode.
- `vlt .` CLI to open Vault from a terminal; file-picker mode for other tools.

## 5. Automation

- Bulk rename with live preview: find/replace, regex, numbering, case, or "edit names in your editor".
- Custom actions in config (`ffmpeg -i {file} {stem}.mp4`), scoped by file type, shown in context menu and palette.
- Lua plugins: previewers, columns, actions, sidebar panels, themes ([ADR 0004](adr/0004-lua-plugins.md)).
- Folder rules (post-1.0).
