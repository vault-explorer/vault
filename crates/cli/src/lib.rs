//! # vault-cli
//!
//! Library behind `vlt`, the Vault companion command-line tool
//! (see `docs/adr/0006-cli-name-vlt.md`).
//!
//! Planned commands (not built yet):
//!
//! - `vlt [PATH]` — open Vault at `PATH` (default: current directory).
//! - `vlt pick` — file-picker mode: let the user choose files in Vault and print
//!   the chosen paths, for use from other tools.
//!
//! Like plugins, the CLI goes through `vault-core`'s APIs, operations queue and
//! undo log; it holds no business logic of its own. Argument parsing lives here
//! (not in `main.rs`) so it can be unit-tested.

/// What `vlt` should do, parsed from its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Print version information.
    Version,
    /// Print usage.
    Help,
    /// Open Vault at the given path (not implemented yet).
    Open(String),
}

/// Parses `vlt`'s arguments (excluding the program name).
#[must_use]
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Command {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("-V" | "--version") => Command::Version,
        Some("-h" | "--help") => Command::Help,
        Some(path) => Command::Open(path.to_owned()),
        None => Command::Open(".".to_owned()),
    }
}

/// Usage text for `vlt --help`.
#[must_use]
pub fn usage() -> String {
    format!(
        "vlt {} — companion CLI for the Vault file explorer\n\n\
         Usage:\n  vlt [PATH]     Open Vault at PATH (default: .)\n  \
         vlt --version  Print version\n  vlt --help     Print this help\n",
        vault_core::VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_version_help_and_paths() {
        assert_eq!(parse(args(&["--version"])), Command::Version);
        assert_eq!(parse(args(&["-h"])), Command::Help);
        assert_eq!(parse(args(&["C:\\code"])), Command::Open("C:\\code".into()));
        assert_eq!(parse(args(&[])), Command::Open(".".into()));
    }
}
