# Vision

> Part of the [Vault plan](PLAN.md). If something here is wrong or outdated, fix it in the same change — don't silently diverge.

## Pitch

A cross-platform, open-source file explorer that opens instantly, finds anything in milliseconds, and gets out of your way — with a keyboard-first power layer for developers one keystroke below the surface.

## Audience (priority order)

1. **Developers** — git status, fuzzy jump, open in editor, no mouse required.
2. **Power users** — huge folders, fast search, bulk rename, previews, dual pane.
3. **Everyone else** — familiar on day one; never sees a feature they didn't ask for.

## Design principles

Use these to settle every argument.

- **Fast is a feature.** Every interaction has a time budget ([PERFORMANCE.md](PERFORMANCE.md)). A feature that breaks a budget doesn't ship.
- **Simple surface, deep core.** Default window looks familiar. Power lives in the command palette, keybindings and settings.
- **Keyboard and mouse are equals.** Everything clickable has a shortcut and vice versa.
- **Never lose a file.** Destructive actions are undoable or go to trash; long ops are resumable and transparent.
- **Plain-text config.** Settings, keymaps, themes are TOML files with a GUI on top.
- **Extensible, not bloated.** Core stays small; niche features are plugins.

## Non-goals for 1.0

Cloud sync, cross-device tag databases, mobile apps, AI features.

## Naming

- **Vault** is the app.
- **`vlt`** is the companion CLI — `vault` is taken by HashiCorp Vault. See [ADR 0006](adr/0006-cli-name-vlt.md).
