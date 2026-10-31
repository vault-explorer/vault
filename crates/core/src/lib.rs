//! # vault-core
//!
//! The heart of Vault. Owns all application logic and all disk access; the UI,
//! the `vlt` CLI and plugins are thin front ends over this crate.
//!
//! Responsibilities (most not built yet):
//!
//! - **File model** — entries, metadata, virtual file systems behind one trait
//!   (local first; archives, SFTP, S3 later).
//! - **Directory service** — streams listings in batches (~500 entries) and
//!   turns watcher events into diffs.
//! - **Scheduler** — one prioritized task scheduler (visible folder > selection
//!   preview > background index). Anything that can take more than 100 ms is a
//!   cancellable background task with progress.
//! - **Operations queue** — copy, move, rename, delete with pause, cancel and
//!   conflict resolution.
//! - **Journal and undo log** — every destructive operation is journaled before
//!   it runs, enabling crash recovery and undo across restarts.
//!
//! OS-specific behaviour comes from `vault-platform`; this crate never `cfg`s
//! on the target OS.

use vault_platform::Platform as _;

pub mod dir;

/// Version of the core crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reply to [`ping`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pong {
    /// The message that was sent.
    pub message: String,
    /// Version of `vault-core` that answered.
    pub core_version: &'static str,
    /// Name of the platform implementation in use.
    pub platform: &'static str,
}

/// Echoes `message` back with core and platform details.
///
/// Scaffold-only: proves the UI → Tauri → core round trip. Remove once real
/// commands exist.
#[must_use]
pub fn ping(message: &str) -> Pong {
    Pong {
        message: message.to_owned(),
        core_version: VERSION,
        platform: vault_platform::native().name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_echoes_message_and_reports_platform() {
        let pong = ping("hello");
        assert_eq!(pong.message, "hello");
        assert_eq!(pong.core_version, VERSION);
        assert_eq!(pong.platform, std::env::consts::OS);
    }
}
