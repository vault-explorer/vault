//! Criterion benches for the Rust-side budgets in `docs/PERFORMANCE.md`. Each one
//! calls the real crate API; results also go to `bench/results/core.json`.
//!
//! Needs the fixtures from `vault-fixtures generate` (see `bench/README.md`).

#![allow(missing_docs)]

use std::cell::RefCell;
use std::hint::black_box;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, SamplingMode};
use vault_bench::fixtures::{self, DEFAULT_SEED, FLAT_10K, FLAT_100K, TreeSpec};
use vault_bench::results::{self, Metric};
use vault_core::dir::{DEFAULT_BATCH_SIZE, list_dir};
use vault_index::fuzzy::FuzzyMatcher;

const SAMPLE_SIZE: usize = 20;
/// The criterion warm-up keeps the file-system cache warm for every sample.
const CACHE_STATE: &str = "warm";

const FUZZY_CORPUS_NAMES: usize = 100_000;
const FUZZY_QUERY: &str = "rep24pdf";
const FUZZY_LIMIT: usize = 100;

/// Runs `once` as a criterion bench and adds its timings to `metrics`.
///
/// `once` returns the time of one run, so setup and teardown can stay untimed.
/// Samples come from our own timing instead of criterion's output files, whose
/// layout isn't a public API.
fn timed_bench(
    group: &mut BenchmarkGroup<'_, WallTime>,
    metrics: &mut Vec<Metric>,
    name: &str,
    mut once: impl FnMut() -> Duration,
) {
    let per_iter_ms = RefCell::new(Vec::new());
    group.bench_function(name, |b| {
        b.iter_custom(|iters| {
            let total: Duration = (0..iters).map(|_| once()).sum();
            #[allow(clippy::cast_precision_loss)] // Iteration counts are far below 2^52.
            per_iter_ms
                .borrow_mut()
                .push(total.as_secs_f64() * 1000.0 / iters as f64);
            total
        });
    });
    // Warm-up also goes through iter_custom; the measurement is the last samples.
    let per_iter_ms = per_iter_ms.into_inner();
    if per_iter_ms.is_empty() {
        return; // Filtered out on the command line.
    }
    let measured = &per_iter_ms[per_iter_ms.len().saturating_sub(SAMPLE_SIZE)..];
    let metric = results::metric(name, measured, CACHE_STATE).expect("metric is in budgets.json");
    metrics.push(metric);
}

fn bench_listing(
    c: &mut Criterion,
    metrics: &mut Vec<Metric>,
    spec: &TreeSpec,
    label: &str,
) -> Result<(), fixtures::FixtureError> {
    let root = fixtures::default_root();
    fixtures::require(&root, spec, DEFAULT_SEED)?;
    let path = root.join(spec.name);

    let mut group = c.benchmark_group("dir_list");
    group
        .sampling_mode(SamplingMode::Flat)
        .sample_size(SAMPLE_SIZE);

    timed_bench(
        &mut group,
        metrics,
        &format!("dir_list.{label}.first_batch"),
        || {
            let start = Instant::now();
            let mut batches = list_dir(&path, DEFAULT_BATCH_SIZE).expect("fixture is readable");
            let first = batches.next().expect("fixture isn't empty");
            let elapsed = start.elapsed();
            assert_eq!(
                first.expect("fixture is readable").len(),
                DEFAULT_BATCH_SIZE
            );
            elapsed
        },
    );

    timed_bench(
        &mut group,
        metrics,
        &format!("dir_list.{label}.full"),
        || {
            let start = Instant::now();
            let mut entries = 0;
            for batch in list_dir(&path, DEFAULT_BATCH_SIZE).expect("fixture is readable") {
                entries += black_box(batch.expect("fixture is readable")).len();
            }
            let elapsed = start.elapsed();
            assert_eq!(entries, spec.files);
            elapsed
        },
    );

    group.finish();
    Ok(())
}

/// Placeholder so the search work has a home: one query over a fixed corpus.
fn bench_fuzzy(c: &mut Criterion, metrics: &mut Vec<Metric>) {
    let names = fixtures::name_corpus(FUZZY_CORPUS_NAMES, DEFAULT_SEED);
    let mut matcher = FuzzyMatcher::new();
    assert!(
        !matcher.best_matches(&names, FUZZY_QUERY, 1).is_empty(),
        "query must match something, or this only measures rejection"
    );

    let mut group = c.benchmark_group("fuzzy");
    group
        .sampling_mode(SamplingMode::Flat)
        .sample_size(SAMPLE_SIZE);
    timed_bench(&mut group, metrics, "fuzzy.100k.one_query", || {
        let start = Instant::now();
        black_box(matcher.best_matches(&names, black_box(FUZZY_QUERY), FUZZY_LIMIT));
        start.elapsed()
    });
    group.finish();
}

fn main() -> ExitCode {
    let mut criterion = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5))
        .configure_from_args();
    let mut metrics = Vec::new();

    let listed = bench_listing(&mut criterion, &mut metrics, &FLAT_10K, "10k")
        .and_then(|()| bench_listing(&mut criterion, &mut metrics, &FLAT_100K, "100k"));
    if let Err(err) = listed {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }
    bench_fuzzy(&mut criterion, &mut metrics);
    criterion.final_summary();

    // `cargo bench -- --test` only checks that benches run; its timings mean nothing.
    if std::env::args().any(|arg| arg == "--test") {
        return ExitCode::SUCCESS;
    }
    match results::write(&results::default_out_dir(), "core", metrics) {
        Ok(path) => {
            println!("results: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
