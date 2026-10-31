//! Directory listing: the first piece of the directory service.
//!
//! [`list_dir`] returns a pull-based iterator of batches, so a caller can show
//! the first rows long before a huge folder has been read, and can cancel by
//! simply not asking for the next batch. Scheduling it as a background task is
//! the caller's job until the core's scheduler exists.
//!
//! `std::fs` already abstracts the OS here, so no `vault-platform` trait is
//! involved.

use std::ffi::OsString;
use std::fmt;
use std::fs::{self, ReadDir};
use std::io;
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Entries per batch when streaming a listing (`docs/ARCHITECTURE.md`).
pub const DEFAULT_BATCH_SIZE: usize = 500;

/// What a directory entry is. Symlinks are reported as such, not followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Dir,
    /// A symbolic link or other reparse point the OS reports as a link.
    Symlink,
    /// Anything else (sockets, devices, ...).
    Other,
}

/// One row of a directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    /// File name, without the parent path. Not necessarily valid Unicode.
    pub name: OsString,
    /// What the entry is.
    pub kind: EntryKind,
    /// Size in bytes, as reported by the file system.
    pub size: u64,
    /// Last-modified time, if the file system records one.
    pub modified: Option<SystemTime>,
}

/// Errors from [`list_dir`] and the iterator it returns.
#[derive(Debug)]
#[non_exhaustive]
pub enum ListError {
    /// The directory could not be opened.
    Open {
        /// The directory that was requested.
        path: PathBuf,
        /// The underlying OS error.
        source: io::Error,
    },
    /// Reading an entry failed part-way through the listing.
    Read {
        /// The directory being listed.
        path: PathBuf,
        /// The underlying OS error.
        source: io::Error,
    },
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { path, source } => {
                write!(f, "cannot open directory {}: {source}", path.display())
            }
            Self::Read { path, source } => {
                write!(f, "cannot read directory {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Open { source, .. } | Self::Read { source, .. } => Some(source),
        }
    }
}

/// Starts listing `path`, yielding batches of up to `batch_size` entries.
///
/// A `batch_size` of 0 is treated as 1. Entries are in the order the OS returns
/// them; sorting is a view concern. Entries deleted while the listing runs are
/// skipped. The iterator ends after the first error.
///
/// # Errors
///
/// Returns [`ListError::Open`] if the directory can't be opened.
pub fn list_dir(path: &Path, batch_size: usize) -> Result<DirBatches, ListError> {
    let entries = fs::read_dir(path).map_err(|source| ListError::Open {
        path: path.to_owned(),
        source,
    })?;
    Ok(DirBatches {
        path: path.to_owned(),
        entries: Some(entries),
        batch_size: batch_size.max(1),
    })
}

/// Iterator over the batches of a listing. See [`list_dir`].
#[derive(Debug)]
pub struct DirBatches {
    path: PathBuf,
    entries: Option<ReadDir>,
    batch_size: usize,
}

impl DirBatches {
    fn read_error(&mut self, source: io::Error) -> ListError {
        self.entries = None;
        ListError::Read {
            path: self.path.clone(),
            source,
        }
    }
}

impl Iterator for DirBatches {
    type Item = Result<Vec<DirEntry>, ListError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(self.batch_size);
        while batch.len() < self.batch_size {
            let Some(next) = self.entries.as_mut()?.next() else {
                self.entries = None;
                break;
            };
            let converted = next.and_then(|entry| to_dir_entry(&entry));
            match converted {
                Ok(entry) => batch.push(entry),
                // Deleted between the directory read and the metadata read.
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Some(Err(self.read_error(err))),
            }
        }
        (!batch.is_empty()).then_some(Ok(batch))
    }
}

impl FusedIterator for DirBatches {}

fn to_dir_entry(entry: &fs::DirEntry) -> io::Result<DirEntry> {
    let metadata = entry.metadata()?;
    let file_type = metadata.file_type();
    let kind = if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Dir
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    };
    Ok(DirEntry {
        name: entry.file_name(),
        kind,
        size: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Unique scratch directory, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(test: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("vault-core-{test}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn with_files(test: &str, count: usize) -> TempDir {
        let dir = TempDir::new(test);
        for i in 0..count {
            fs::write(dir.0.join(format!("f{i}.txt")), "x".repeat(i)).unwrap();
        }
        dir
    }

    #[test]
    fn batches_are_full_except_the_last() {
        let dir = with_files("batches", 7);
        let sizes: Vec<_> = list_dir(&dir.0, 3)
            .unwrap()
            .map(|batch| batch.unwrap().len())
            .collect();
        assert_eq!(sizes, [3, 3, 1]);
    }

    #[test]
    fn lists_every_entry_with_metadata() {
        let dir = with_files("metadata", 5);
        fs::create_dir(dir.0.join("sub")).unwrap();

        let entries: Vec<_> = list_dir(&dir.0, DEFAULT_BATCH_SIZE)
            .unwrap()
            .flat_map(Result::unwrap)
            .collect();

        let names: BTreeSet<_> = entries
            .iter()
            .map(|e| e.name.to_string_lossy().into_owned())
            .collect();
        let expected: BTreeSet<_> = (0..5)
            .map(|i| format!("f{i}.txt"))
            .chain(["sub".to_owned()])
            .collect();
        assert_eq!(names, expected);

        let f3 = entries.iter().find(|e| e.name == "f3.txt").unwrap();
        assert_eq!((f3.kind, f3.size), (EntryKind::File, 3));
        assert!(f3.modified.is_some());
        let sub = entries.iter().find(|e| e.name == "sub").unwrap();
        assert_eq!(sub.kind, EntryKind::Dir);
    }

    #[test]
    fn empty_directory_yields_no_batches() {
        let dir = TempDir::new("empty");
        assert_eq!(list_dir(&dir.0, 10).unwrap().count(), 0);
    }

    #[test]
    fn batch_size_zero_is_treated_as_one() {
        let dir = with_files("zero", 2);
        assert_eq!(list_dir(&dir.0, 0).unwrap().count(), 2);
    }

    #[test]
    fn missing_directory_is_an_open_error() {
        let dir = TempDir::new("missing");
        let err = list_dir(&dir.0.join("nope"), 10).unwrap_err();
        assert!(matches!(err, ListError::Open { .. }));
        assert!(err.to_string().contains("nope"));
    }
}
