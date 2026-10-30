//! `vlt` binary entry point. All logic lives in the `vault_cli` library.

use std::process::ExitCode;

use vault_cli::{Command, parse, usage};

fn main() -> ExitCode {
    match parse(std::env::args().skip(1)) {
        Command::Version => println!("vlt {}", vault_core::VERSION),
        Command::Help => print!("{}", usage()),
        Command::Open(path) => {
            eprintln!("vlt: opening Vault at {path:?} is not implemented yet");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
