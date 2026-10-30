//! Linux implementation (Phase 2: Ubuntu 22.04+).
//!
//! Empty stub: every operation returns [`PlatformError::Unsupported`].

use std::path::{Path, PathBuf};

use crate::{
    Indexer, Platform, PlatformError, Result, ShellIntegration, Thumbnailer, Trash, Watcher,
};

const NAME: &str = "linux";

fn unsupported<T>(operation: &'static str) -> Result<T> {
    Err(PlatformError::Unsupported {
        operation,
        platform: NAME,
    })
}

/// The Linux platform implementation.
#[derive(Debug, Default, Clone, Copy)]
pub struct LinuxPlatform;

impl Platform for LinuxPlatform {
    fn name(&self) -> &'static str {
        NAME
    }
}

impl Watcher for LinuxPlatform {
    fn watch(&self, _path: &Path) -> Result<()> {
        unsupported("Watcher::watch")
    }
}

impl Indexer for LinuxPlatform {
    fn has_fast_path(&self, _volume: &Path) -> bool {
        false
    }

    fn enumerate_volume(&self, _volume: &Path) -> Result<Vec<PathBuf>> {
        unsupported("Indexer::enumerate_volume")
    }
}

impl Thumbnailer for LinuxPlatform {
    fn thumbnail(&self, _path: &Path, _size: u32) -> Result<Vec<u8>> {
        unsupported("Thumbnailer::thumbnail")
    }
}

impl ShellIntegration for LinuxPlatform {
    fn open_default(&self, _path: &Path) -> Result<()> {
        unsupported("ShellIntegration::open_default")
    }

    fn open_terminal(&self, _dir: &Path) -> Result<()> {
        unsupported("ShellIntegration::open_terminal")
    }
}

impl Trash for LinuxPlatform {
    fn trash(&self, _path: &Path) -> Result<()> {
        unsupported("Trash::trash")
    }
}
