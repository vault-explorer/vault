//! # vault-bench
//!
//! Benchmark harness for Vault's performance budgets (`docs/PERFORMANCE.md`).
//!
//! - [`fixtures`] generates deterministic test trees and name corpora, so every
//!   bench and every machine uses the same inputs.
//! - [`results`] writes the machine-readable result files that
//!   `bench/scripts/compare.mjs` compares.
//! - `benches/core.rs` holds the criterion benches for Rust-side budgets. The
//!   app-level benches (cold start, idle memory, scroll) are in
//!   `bench/scripts/app-bench.mjs`.
//!
//! How to run all of it: `bench/README.md`.

pub mod fixtures;
pub mod results;
