# Architecture Decision Records

An ADR records one significant decision: the context, what we chose, what we rejected, and the consequences. ADRs are how we avoid re-arguing settled questions — and how we notice when the context has changed enough to revisit one.

## When to write one

Write an ADR when a change:

- picks or replaces a framework, crate, or tool that is hard to swap out later;
- changes the layering or a rule in [ARCHITECTURE.md](../ARCHITECTURE.md);
- changes a supported platform, license, or naming;
- makes a trade-off against a [performance budget](../PERFORMANCE.md) or a design principle.

Changes to the **config format or plugin API** go through an [RFC](../rfcs/README.md) first; the accepted RFC is then summarized in an ADR.

## How

1. Copy [template.md](template.md) to `NNNN-short-title.md` using the next free number.
2. Fill it in. Keep it short — a page is plenty.
3. Open a PR. Status is `Proposed` until merged, then `Accepted`.
4. ADRs are immutable once accepted. To change a decision, write a new ADR that supersedes the old one, and update the old one's status line only.

## Index

| # | Title | Status |
| --- | --- | --- |
| [0001](0001-tauri-2-and-rust-core.md) | Use Tauri 2 with a Rust core | Accepted |
| [0002](0002-solidjs-ui-pending-spike.md) | Use SolidJS for the UI, pending the Phase 0 spike | Accepted (provisional) |
| [0003](0003-windows-first-mvp.md) | Target Windows 10 22H2+ and Windows 11 for the MVP | Accepted |
| [0004](0004-lua-plugins.md) | Use Lua 5.4 via mlua for 1.0 plugins | Accepted |
| [0005](0005-dual-mit-apache-license.md) | Dual-license under MIT and Apache-2.0 | Accepted (confirm before first release) |
| [0006](0006-cli-name-vlt.md) | Name the companion CLI `vlt` | Accepted |
