//! # vault-plugins
//!
//! The plugin host and the plugin API.
//!
//! Responsibilities (not built yet):
//!
//! - Hosting Lua 5.4 plugins via `mlua` (see `docs/adr/0004-lua-plugins.md`).
//! - The plugin API: previewers, columns, actions, sidebar panels, themes.
//! - Isolation: plugins get no direct file-system access. Every file operation a
//!   plugin performs goes through `vault-core`'s operations queue and undo log,
//!   exactly like a user action.
//!
//! Changes to the plugin API go through the RFC process (`docs/rfcs/`). The API
//! freezes at the Phase 3 gate.

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert_eq!(env!("CARGO_PKG_NAME"), "vault-plugins");
    }
}
