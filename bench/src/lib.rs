//! # vault-bench
//!
//! Benchmark harness for Vault's performance budgets (`docs/PERFORMANCE.md`).
//!
//! Rust-side budgets are measured with criterion benches in `benches/`. CI runs
//! them on every PR and compares against `main`; a regression of more than 10%
//! fails the build. Shared fixtures (synthetic directory trees, name corpora)
//! will live in this library so every bench uses the same inputs.
//!
//! Status: placeholder. The real harness is a Phase 0 backlog item
//! (`docs/backlog/phase-0.md`).

/// Number of entries in the "huge folder" fixture used by the scroll and
/// listing budgets.
pub const HUGE_FOLDER_ENTRIES: usize = 100_000;

#[cfg(test)]
mod tests {
    #[test]
    fn huge_folder_fixture_matches_budget() {
        assert_eq!(super::HUGE_FOLDER_ENTRIES, 100_000);
    }
}
