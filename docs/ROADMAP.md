# Roadmap

> Part of the [Vault plan](PLAN.md). Current progress is tracked in [STATUS.md](STATUS.md); the active phase's work items are in [backlog/](backlog/).

Suggested pace: part-time solo.

**Each phase ends with a pass/fail gate. If a phase runs long, move features out — never relax the gate.**

| Phase | Timing | Scope | Gate |
| --- | --- | --- | --- |
| 0 · Spikes | Weeks 1–4 | 100k-row virtual list in SolidJS vs Svelte; Tauri IPC streaming test; MFT read prototype; monorepo, CI, benchmark harness | 100k rows scroll at 60 fps on Windows 10 and 11 |
| 1 · MVP daily driver (Windows) | Months 1–3 | Sidebar, smart path bar, details/grid/columns, tabs, filter; file ops with undo + operations queue; basic previews; TOML settings | You use it as your main explorer for two weeks |
| 2 · Developer alpha | Months 4–6 | Command palette, fuzzy jump, Vim keymap, git badges, open in editor, terminal pane; whole-disk index; `vlt` CLI; macOS + Linux ports; public installers | Public alpha on Windows, macOS, Linux |
| 3 · Power beta | Months 7–9 | Content search + query language, saved searches, bulk rename, custom actions, Lua plugin API, archives, dual pane, PDF/media previews | Plugin API frozen; zero open data-loss bugs |
| 4 · 1.0 | Months 10–12 | Accessibility, translations, onboarding tour, signed installers, auto-update, Winget/Homebrew/Flathub, docs site | Every performance budget green on all three OSes |

**Post-1.0:** WASM plugins, SFTP/S3 virtual file systems, folder rules, plugin registry.

## Platforms by phase

- **Phase 0–1:** Windows 10 22H2+ and Windows 11 (x64). macOS and Linux get compile checks in CI only.
- **Phase 2:** macOS 13+ and Ubuntu 22.04+ get full tests and public installers. ARM64 Windows later.

## Risks

| Risk | Mitigation |
| --- | --- |
| Scope creep | Phase gates; cut features, never budgets |
| Webview performance on huge folders | Virtualize from day one; CI benchmarks; logic in Rust so UI is swappable |
| MFT needs admin | Optional elevated helper; walk fallback |
| Data-loss bug | Journal, undo log, fuzz/property tests on copy/move, beta channel |
| Shell extension crashes | Out-of-process loading |
| Maintainer burnout | Ship small, recruit early, stable plugin API |
