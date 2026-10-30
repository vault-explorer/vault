//! Placeholder criterion bench so the CI bench job has something to run.
//! Replace with real budget benches as part of the Phase 0 harness work.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn ping(c: &mut Criterion) {
    c.bench_function("core_ping", |b| {
        b.iter(|| vault_core::ping(black_box("bench")));
    });
}

criterion_group!(benches, ping);
criterion_main!(benches);
