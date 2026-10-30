# Windows (MVP platform)

> Part of the [Vault plan](PLAN.md). Target: **Windows 10 22H2+ and Windows 11, x64** (ARM64 later) — see [ADR 0003](adr/0003-windows-first-mvp.md).

## Platform notes

- WebView2 runtime is preinstalled on Windows 10 22H2 / 11; the installer should bootstrap it if missing.
- Windows 10 left mainstream support in October 2025 — test on both 10 and 11.
- Long paths (`\\?\`), UNC shares, drive letters, case-insensitive matching, reserved names (`CON`, `NUL`…).
- Raw MFT reads need elevation → optional helper service; **never require admin to run Vault.**
- Shell context menu extensions (IContextMenu) can crash hosts → load them out of process.
- Respect Windows shortcuts: `F2` rename, `Delete` to Recycle Bin, `Alt+Up` parent, `Ctrl+Shift+N` new folder.
- Code-sign builds early to build SmartScreen reputation.
- Keep all Windows code behind platform traits (`crates/platform`) so macOS/Linux ports in Phase 2 only add modules.

## Mapping to platform traits

| Trait (`vault-platform`) | Windows implementation (planned) |
| --- | --- |
| `Watcher` | `notify`, with a `ReadDirectoryChangesW` / USN journal fast path |
| `Indexer` | MFT enumeration + USN journal tailing via the `windows` crate; directory-walk fallback without elevation |
| `Thumbnailer` | `IShellItemImageFactory` |
| `ShellIntegration` | "Open with", context menus (out of process), Windows Terminal, Explorer reveal |
| `Trash` | Recycle Bin via the `trash` crate |

Today all of these are stubs that return `PlatformError::Unsupported`.
