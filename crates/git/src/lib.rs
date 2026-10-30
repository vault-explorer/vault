//! # vault-git
//!
//! Git awareness for the file views.
//!
//! Responsibilities (not built yet):
//!
//! - Repository discovery for the current folder and branch name for the path bar.
//! - Per-file status (modified, added, untracked, ignored) for row badges, the
//!   "changed files only" filter and dimming ignored files.
//! - A status cache invalidated by watcher events, so status is never recomputed
//!   on every navigation.
//!
//! Built on `gix`, with `git2` as a fallback for anything `gix` can't do yet.
//! Status arrives in the UI as separate patch events and must never delay the
//! first rows of a listing.

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert_eq!(env!("CARGO_PKG_NAME"), "vault-git");
    }
}
