//! Machine-readable benchmark results.
//!
//! One JSON file per suite, the same format `bench/scripts/app-bench.mjs` writes
//! and `bench/scripts/compare.mjs` reads (documented in `bench/README.md`). Unit
//! and budget come from `bench/budgets.json`, never from the bench itself, so a
//! bench can't quietly grade itself against a different number.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use serde::{Deserialize, Serialize};

const BUDGETS_JSON: &str = include_str!("../budgets.json");

/// Version of the result-file format.
pub const SCHEMA_VERSION: u32 = 1;

/// A metric's entry in `bench/budgets.json`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BudgetEntry {
    /// Unit of the metric's values.
    pub unit: String,
    /// The limit from `docs/PERFORMANCE.md`, if this metric has one.
    pub budget: Option<f64>,
    /// How the value compares to the budget: `<` or `<=`.
    pub budget_op: Option<String>,
    /// What part of the budget this metric covers, if not all of it.
    pub scope: Option<String>,
}

#[derive(Deserialize)]
struct BudgetsFile {
    metrics: BTreeMap<String, BudgetEntry>,
}

/// Returns every entry of `bench/budgets.json` by metric name.
///
/// # Panics
///
/// Panics if the embedded `bench/budgets.json` is malformed, which the tests in
/// this module rule out.
#[must_use]
pub fn budgets() -> BTreeMap<String, BudgetEntry> {
    let file: BudgetsFile =
        serde_json::from_str(BUDGETS_JSON).expect("bench/budgets.json is checked by tests");
    file.metrics
}

/// One measured metric.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Metric {
    /// Name of the metric; a key in `bench/budgets.json`.
    pub name: String,
    /// Median over all samples.
    pub value: f64,
    /// Unit of `value` and `p95`.
    pub unit: String,
    /// 95th percentile over all samples.
    pub p95: f64,
    /// Number of samples.
    pub samples: usize,
    /// The limit from `docs/PERFORMANCE.md`, if this metric has one.
    pub budget: Option<f64>,
    /// How `value` compares to `budget`: `<` or `<=`.
    pub budget_op: Option<String>,
    /// What part of the budget this metric covers, if not all of it.
    pub scope: Option<String>,
    /// File-system cache state during the measurement.
    pub cache: &'static str,
}

/// Machine details recorded next to every number, as `docs/PERFORMANCE.md` asks.
/// `bench/scripts/run.mjs` adds CPU model, RAM, disk and OS build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Machine {
    /// Operating system family.
    pub os: &'static str,
    /// CPU architecture.
    pub arch: &'static str,
    /// Logical CPU cores.
    pub logical_cores: usize,
}

/// Contents of one result file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResultFile {
    /// [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Which bench suite wrote this file.
    pub suite: String,
    /// Commit that was measured.
    pub commit: Option<String>,
    /// Where it was measured.
    pub machine: Machine,
    /// The measurements.
    pub metrics: Vec<Metric>,
}

/// Errors from writing a result file.
#[derive(Debug)]
#[non_exhaustive]
pub enum ResultError {
    /// The metric has no entry in `bench/budgets.json`.
    UnknownMetric(String),
    /// A metric had no samples.
    NoSamples(String),
    /// Writing the file failed.
    Write {
        /// The file being written.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
}

impl fmt::Display for ResultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMetric(name) => write!(f, "{name} is not in bench/budgets.json"),
            Self::NoSamples(name) => write!(f, "{name} has no samples"),
            Self::Write { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for ResultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write { source, .. } => Some(source),
            Self::UnknownMetric(_) | Self::NoSamples(_) => None,
        }
    }
}

/// Builds a [`Metric`] from raw samples, looking up its unit and budget.
///
/// # Errors
///
/// Returns an error if `name` isn't in `bench/budgets.json` or `samples` is empty.
pub fn metric(name: &str, samples: &[f64], cache: &'static str) -> Result<Metric, ResultError> {
    let entry = budgets()
        .remove(name)
        .ok_or_else(|| ResultError::UnknownMetric(name.to_owned()))?;
    if samples.is_empty() {
        return Err(ResultError::NoSamples(name.to_owned()));
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(Metric {
        name: name.to_owned(),
        value: median(&sorted),
        unit: entry.unit,
        p95: percentile(&sorted, 95),
        samples: sorted.len(),
        budget: entry.budget,
        budget_op: entry.budget_op,
        scope: entry.scope,
        cache,
    })
}

/// Median of sorted values. `sorted` must not be empty.
fn median(sorted: &[f64]) -> f64 {
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        f64::midpoint(sorted[mid - 1], sorted[mid])
    } else {
        sorted[mid]
    }
}

/// Nearest-rank percentile of sorted values. `sorted` must not be empty.
fn percentile(sorted: &[f64], percent: usize) -> f64 {
    let rank = (sorted.len() * percent).div_ceil(100).max(1);
    sorted[rank - 1]
}

/// `$VAULT_BENCH_OUT`, or `bench/results` (gitignored).
#[must_use]
pub fn default_out_dir() -> PathBuf {
    std::env::var_os("VAULT_BENCH_OUT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("results"),
        PathBuf::from,
    )
}

/// Writes `<out_dir>/<suite>.json` and returns its path.
///
/// # Errors
///
/// Returns [`ResultError::Write`] if the file can't be written.
pub fn write(out_dir: &Path, suite: &str, metrics: Vec<Metric>) -> Result<PathBuf, ResultError> {
    let path = out_dir.join(format!("{suite}.json"));
    let write_err = |source| ResultError::Write {
        path: path.clone(),
        source,
    };
    let file = ResultFile {
        schema_version: SCHEMA_VERSION,
        suite: suite.to_owned(),
        commit: current_commit(),
        machine: Machine {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            logical_cores: thread::available_parallelism().map_or(0, usize::from),
        },
        metrics,
    };
    let json = serde_json::to_string_pretty(&file).map_err(|e| write_err(e.into()))?;
    fs::create_dir_all(out_dir).map_err(write_err)?;
    fs::write(&path, json + "\n").map_err(write_err)?;
    Ok(path)
}

fn current_commit() -> Option<String> {
    if let Ok(commit) = std::env::var("VAULT_BENCH_COMMIT") {
        return Some(commit);
    }
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // These values are exact, not computed approximations.
mod tests {
    use super::*;

    const PERFORMANCE_MD: &str = include_str!("../../docs/PERFORMANCE.md");

    /// `bench/budgets.json` must agree with the coverage table in
    /// `docs/PERFORMANCE.md`, so neither can change without the other.
    #[test]
    fn every_budget_matches_the_performance_doc() {
        for (name, entry) in budgets() {
            let row = PERFORMANCE_MD
                .lines()
                .find(|line| line.starts_with(&format!("| `{name}` |")))
                .unwrap_or_else(|| panic!("{name} has no row in docs/PERFORMANCE.md"));
            match (entry.budget, entry.budget_op.as_deref()) {
                (Some(budget), Some(op)) => {
                    let op = if op == "<=" { "≤" } else { op };
                    let expected = format!("{op} {budget} {}", entry.unit);
                    assert!(row.contains(&expected), "{name}: {expected} not in: {row}");
                }
                (None, None) => {}
                _ => panic!("{name}: budget and budget_op must be set together"),
            }
        }
    }

    #[test]
    fn metric_looks_up_budget_and_computes_stats() {
        let samples = [5.0, 1.0, 3.0, 2.0, 4.0];
        let m = metric("dir_list.10k.full", &samples, "warm").unwrap();
        assert_eq!(m.value, 3.0);
        assert_eq!(m.p95, 5.0);
        assert_eq!(m.samples, 5);
        assert_eq!(m.unit, "ms");
        assert_eq!(m.budget, Some(300.0));
        assert_eq!(m.budget_op.as_deref(), Some("<"));
    }

    #[test]
    fn metric_without_budget_has_none() {
        let m = metric("dir_list.100k.full", &[1.0], "warm").unwrap();
        assert_eq!(m.budget, None);
        assert_eq!(m.budget_op, None);
    }

    #[test]
    fn median_of_even_count_averages_the_middle() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 10.0]), 2.5);
    }

    #[test]
    fn percentile_is_nearest_rank() {
        let values: Vec<_> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile(&values, 95), 95.0);
        assert_eq!(percentile(&[7.0], 95), 7.0);
    }

    #[test]
    fn unknown_metric_and_empty_samples_are_errors() {
        assert!(matches!(
            metric("nope", &[1.0], "warm"),
            Err(ResultError::UnknownMetric(_))
        ));
        assert!(matches!(
            metric("cold_start", &[], "warm"),
            Err(ResultError::NoSamples(_))
        ));
    }
}
