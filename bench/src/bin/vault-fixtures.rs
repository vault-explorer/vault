//! Generates, verifies, lists and removes benchmark fixtures. See `bench/README.md`.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use vault_bench::fixtures::{
    self, ALL_SPECS, DEFAULT_SEED, DEFAULT_SPECS, FixtureError, NESTED_1M, TreeSpec,
};

const USAGE: &str = "\
usage: vault-fixtures <command> [options]

commands:
  generate   write fixtures that are missing or out of date
  verify     check fixtures on disk against their manifests
  list       print each fixture's planned size (reads nothing from disk)
  clean      delete fixtures

options:
  --include-1m    also act on the 1M-file tree (opt-in: large and slow)
  --only <name>   act only on this tree: flat-10k, flat-100k or nested-1m
  --seed <n>      seed (default: the seed the benches expect)
  --root <path>   fixtures directory (default: $VAULT_BENCH_FIXTURES or bench/fixtures)
  --force         with generate: rewrite fixtures even if up to date";

struct Args {
    command: String,
    specs: Vec<TreeSpec>,
    seed: u64,
    root: PathBuf,
    force: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("missing command")?;
    let mut specs = DEFAULT_SPECS.to_vec();
    let mut only = None;
    let mut seed = DEFAULT_SEED;
    let mut root = fixtures::default_root();
    let mut force = false;
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--include-1m" => specs.push(NESTED_1M),
            "--only" => {
                let name = value()?;
                let spec = ALL_SPECS.iter().find(|spec| spec.name == name);
                only = Some(*spec.ok_or_else(|| format!("unknown tree {name}"))?);
            }
            "--seed" => {
                let text = value()?;
                seed = text.parse().map_err(|_| format!("bad seed {text}"))?;
            }
            "--root" => root = PathBuf::from(value()?),
            "--force" => force = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(Args {
        command,
        specs: only.map_or(specs, |spec| vec![spec]),
        seed,
        root,
        force,
    })
}

#[allow(clippy::cast_precision_loss)] // Display only.
fn mib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn run(args: &Args) -> Result<(), FixtureError> {
    for spec in &args.specs {
        match args.command.as_str() {
            "generate" => {
                let start = Instant::now();
                let manifest = fixtures::generate(&args.root, spec, args.seed, args.force)?;
                println!(
                    "{}: {} files, {} dirs, {:.0} MiB, digest {} ({:.1} s) at {}",
                    spec.name,
                    manifest.files,
                    manifest.dirs,
                    mib(manifest.bytes),
                    manifest.digest,
                    start.elapsed().as_secs_f64(),
                    args.root.join(spec.name).display(),
                );
            }
            "verify" => {
                let manifest = fixtures::verify(&args.root, spec, args.seed)?;
                println!("{}: ok, digest {}", spec.name, manifest.digest);
            }
            "list" => {
                let plan = fixtures::plan(spec, args.seed);
                println!(
                    "{}: {} files, {} dirs, {:.0} MiB of file contents",
                    spec.name,
                    plan.files.len(),
                    plan.dirs.len(),
                    mib(plan.total_bytes()),
                );
            }
            "clean" => {
                fixtures::remove(&args.root, spec)?;
                println!("{}: removed", spec.name);
            }
            _ => unreachable!("command is validated in main"),
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) if ["generate", "verify", "list", "clean"].contains(&args.command.as_str()) => {
            args
        }
        Ok(args) => {
            eprintln!("unknown command {}\n\n{USAGE}", args.command);
            return ExitCode::from(2);
        }
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
