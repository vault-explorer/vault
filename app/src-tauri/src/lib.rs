//! # vault-app
//!
//! The Tauri 2 shell around `vault-core`.
//!
//! Responsibilities:
//!
//! - **IPC commands** — thin `#[tauri::command]` wrappers that translate UI
//!   requests into `vault-core` calls. No business logic and no disk access
//!   here: the UI never touches the disk, and neither does this glue layer.
//! - **Event stream** — forwarding core events (listing batches, metadata
//!   patches, progress) to the UI. Not built yet.
//! - **Window management** — creating, restoring and closing windows.
//! - **Readiness** — announcing on stdout when the UI is usable, which is how the
//!   cold-start bench measures startup without guessing.
//!
//! IPC payloads are DTOs defined here (`camelCase` over the wire) so core
//! types don't have to carry serialization concerns.

use serde::Serialize;

/// Line written to stdout once the UI has signalled it is usable.
const READY_LINE: &str = "vault:ready";

/// Wire format of [`vault_core::Pong`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PongDto {
    /// The message the UI sent.
    pub message: String,
    /// Version of `vault-core` that answered.
    pub core_version: &'static str,
    /// Name of the platform implementation in use.
    pub platform: &'static str,
}

impl From<vault_core::Pong> for PongDto {
    fn from(pong: vault_core::Pong) -> Self {
        Self {
            message: pong.message,
            core_version: pong.core_version,
            platform: pong.platform,
        }
    }
}

/// Example command: round-trips a message from the UI through `vault-core`.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri deserializes command args into owned values.
fn ping(message: String) -> PongDto {
    vault_core::ping(&message).into()
}

/// The UI calls this once, after its first paint with the core reachable.
#[tauri::command]
fn app_ready() {
    announce_ready(&mut std::io::stdout());
}

fn announce_ready(out: &mut impl std::io::Write) {
    // Nobody may be listening (or stdout may not exist), and that's fine.
    let _ = writeln!(out, "{READY_LINE}").and_then(|()| out.flush());
}

/// WebView2 honours this variable in any app that embeds it, which would let
/// anyone who controls our environment open a debugging port into the webview.
/// Only bench and debug builds keep it. See `docs/adr/0015`.
#[cfg(not(any(debug_assertions, feature = "bench-harness")))]
fn disable_webview_debugging_override() {
    // SAFETY: called first thing in `run`, before any other thread exists.
    unsafe { std::env::remove_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS") };
}

/// Builds and runs the Tauri application. Blocks until the app exits.
///
/// # Panics
///
/// Panics if the Tauri runtime fails to start (e.g. WebView2 is missing).
pub fn run() {
    #[cfg(not(any(debug_assertions, feature = "bench-harness")))]
    disable_webview_debugging_override();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping, app_ready])
        .run(tauri::generate_context!())
        .expect("failed to start the Tauri runtime");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_serializes_camel_case() {
        let json = serde_json::to_value(ping("hi".into())).unwrap();
        assert_eq!(json["message"], "hi");
        assert_eq!(json["coreVersion"], vault_core::VERSION);
        assert_eq!(json["platform"], std::env::consts::OS);
    }

    #[test]
    fn announce_ready_writes_one_line() {
        let mut out = Vec::new();
        announce_ready(&mut out);
        assert_eq!(out, b"vault:ready\n");
    }
}
