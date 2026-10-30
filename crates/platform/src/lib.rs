//! # vault-platform
//!
//! The only crate in Vault that contains operating-system-specific code.
//!
//! Everything the rest of Vault needs from the OS goes through one of five
//! traits:
//!
//! - [`Watcher`] — file-system change notifications.
//! - [`Indexer`] — fast whole-volume enumeration (MFT + USN journal on Windows).
//! - [`Thumbnailer`] — OS-provided thumbnails and icons.
//! - [`ShellIntegration`] — "Open with", context menus, reveal, external terminals.
//! - [`Trash`] — moving files to the Recycle Bin / Trash.
//!
//! Each supported OS has one module (`windows`, `macos`, `linux`) with a type
//! that implements all five traits. Exactly one module is compiled, selected by
//! `target_os`, and exposed as [`Native`]. Callers use [`native()`] and the
//! traits; they never `cfg` on the OS themselves.
//!
//! Windows is the MVP platform (see `docs/adr/0003-windows-first-mvp.md`).
//! Every implementation is currently a stub returning
//! [`PlatformError::Unsupported`]. The trait signatures are provisional and
//! will be designed properly as Phase 0/1 work lands.

use std::fmt;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as Native;
#[cfg(target_os = "macos")]
pub use macos::MacPlatform as Native;
#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as Native;

/// Errors returned by platform implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlatformError {
    /// The operation is not implemented on this platform (yet).
    Unsupported {
        /// The operation that was requested, e.g. `"Trash::trash"`.
        operation: &'static str,
        /// The platform it was requested on, e.g. `"windows"`.
        platform: &'static str,
    },
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported {
                operation,
                platform,
            } => write!(f, "{operation} is not supported on {platform} yet"),
        }
    }
}

impl std::error::Error for PlatformError {}

/// Result alias for platform operations.
pub type Result<T> = std::result::Result<T, PlatformError>;

/// Identifies the running platform implementation.
pub trait Platform: Watcher + Indexer + Thumbnailer + ShellIntegration + Trash {
    /// Short lowercase name of the platform: `"windows"`, `"macos"` or `"linux"`.
    fn name(&self) -> &'static str;
}

/// File-system change notifications for a directory.
///
/// Changes are delivered as diff events, never as "reload everything"
/// (see `docs/ARCHITECTURE.md`).
pub trait Watcher {
    /// Starts watching `path` (non-recursively) for changes.
    fn watch(&self, path: &Path) -> Result<()>;
}

/// Fast whole-volume file enumeration for the search index.
pub trait Indexer {
    /// Returns `true` if a fast path (e.g. MFT read) is available for `volume`
    /// without elevation. When `false`, callers fall back to a directory walk.
    fn has_fast_path(&self, volume: &Path) -> bool;

    /// Enumerates every file path on `volume`.
    fn enumerate_volume(&self, volume: &Path) -> Result<Vec<PathBuf>>;
}

/// OS-provided thumbnails.
pub trait Thumbnailer {
    /// Returns an encoded image (PNG) of at most `size`×`size` pixels for `path`.
    fn thumbnail(&self, path: &Path, size: u32) -> Result<Vec<u8>>;
}

/// Integration with the desktop shell.
pub trait ShellIntegration {
    /// Opens `path` with the user's default application.
    fn open_default(&self, path: &Path) -> Result<()>;

    /// Opens an external terminal in `dir`.
    fn open_terminal(&self, dir: &Path) -> Result<()>;
}

/// Moving files to the OS trash (Recycle Bin on Windows).
pub trait Trash {
    /// Moves `path` to the trash. Never deletes permanently.
    fn trash(&self, path: &Path) -> Result<()>;
}

/// Returns the platform implementation for the OS Vault was compiled for.
#[must_use]
pub fn native() -> Native {
    Native
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_platform_reports_its_name() {
        assert_eq!(native().name(), std::env::consts::OS);
    }

    #[test]
    fn stubs_return_unsupported() {
        let err = native().trash(Path::new("does-not-matter")).unwrap_err();
        assert!(matches!(
            err,
            PlatformError::Unsupported {
                operation: "Trash::trash",
                ..
            }
        ));
    }
}
