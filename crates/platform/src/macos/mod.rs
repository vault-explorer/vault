//! macOS implementation (Phase 2: macOS 13+).
//!
//! Empty stub: every operation returns [`PlatformError::Unsupported`].

use std::path::{Path, PathBuf};

use crate::{
    Indexer, Platform, PlatformError, Result, ShellIntegration, Thumbnailer, Trash, Watcher,
};

const NAME: &str = "macos";

fn unsupported<T>(operation: &'static str) -> Result<T> {
    Err(PlatformError::Unsupported {
        operation,
        platform: NAME,
    })
}

/// The macOS platform implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct MacPlatform;

impl Platform for MacPlatform {
    fn name(&self) -> &'static str {
        NAME
    }
}

impl Watcher for MacPlatform {
    fn watch(&self, _path: &Path) -> Result<()> {
        unsupported("Watcher::watch")
    }
}

impl Indexer for MacPlatform {
    fn has_fast_path(&self, _volume: &Path) -> bool {
        false
    }

    fn enumerate_volume(&self, _volume: &Path) -> Result<Vec<PathBuf>> {
        unsupported("Indexer::enumerate_volume")
    }
}

impl Thumbnailer for MacPlatform {
    fn thumbnail(&self, _path: &Path, _size: u32) -> Result<Vec<u8>> {
        unsupported("Thumbnailer::thumbnail")
    }
}

impl ShellIntegration for MacPlatform {
    fn open_default(&self, _path: &Path) -> Result<()> {
        unsupported("ShellIntegration::open_default")
    }

    fn open_terminal(&self, _dir: &Path) -> Result<()> {
        unsupported("ShellIntegration::open_terminal")
    }
}

impl Trash for MacPlatform {
    fn trash(&self, _path: &Path) -> Result<()> {
        unsupported("Trash::trash")
    }
}
