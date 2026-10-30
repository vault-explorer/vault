//! Windows implementation (MVP platform: Windows 10 22H2+ and Windows 11).
//!
//! All methods are stubs for now. Planned implementations are noted on each
//! method; see `docs/WINDOWS.md`.

use std::path::{Path, PathBuf};

use crate::{
    Indexer, Platform, PlatformError, Result, ShellIntegration, Thumbnailer, Trash, Watcher,
};

const NAME: &str = "windows";

fn unsupported<T>(operation: &'static str) -> Result<T> {
    Err(PlatformError::Unsupported {
        operation,
        platform: NAME,
    })
}

/// The Windows platform implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn name(&self) -> &'static str {
        NAME
    }
}

impl Watcher for WindowsPlatform {
    /// Planned: `notify`, with a `ReadDirectoryChangesW` / USN journal fast path.
    fn watch(&self, _path: &Path) -> Result<()> {
        unsupported("Watcher::watch")
    }
}

impl Indexer for WindowsPlatform {
    /// Planned: `true` for NTFS volumes when the elevated helper is available.
    fn has_fast_path(&self, _volume: &Path) -> bool {
        false
    }

    /// Planned: MFT enumeration via `FSCTL_ENUM_USN_DATA`, then USN journal tailing.
    fn enumerate_volume(&self, _volume: &Path) -> Result<Vec<PathBuf>> {
        unsupported("Indexer::enumerate_volume")
    }
}

impl Thumbnailer for WindowsPlatform {
    /// Planned: `IShellItemImageFactory::GetImage`.
    fn thumbnail(&self, _path: &Path, _size: u32) -> Result<Vec<u8>> {
        unsupported("Thumbnailer::thumbnail")
    }
}

impl ShellIntegration for WindowsPlatform {
    /// Planned: `ShellExecuteExW` with the `open` verb.
    fn open_default(&self, _path: &Path) -> Result<()> {
        unsupported("ShellIntegration::open_default")
    }

    /// Planned: Windows Terminal (`wt.exe -d`), falling back to `cmd.exe`.
    fn open_terminal(&self, _dir: &Path) -> Result<()> {
        unsupported("ShellIntegration::open_terminal")
    }
}

impl Trash for WindowsPlatform {
    /// Planned: Recycle Bin via the `trash` crate.
    fn trash(&self, _path: &Path) -> Result<()> {
        unsupported("Trash::trash")
    }
}
