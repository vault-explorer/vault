//! # vault-index
//!
//! Whole-disk filename index and search.
//!
//! Responsibilities (only the core of fuzzy matching is built so far):
//!
//! - **Indexer** — builds the initial index (Windows: MFT enumeration through
//!   `vault-platform`'s `Indexer`; fallback: directory walk) in the background,
//!   pausable, then keeps it fresh from the USN journal / watcher events.
//! - **Store** — SQLite (`rusqlite`) on disk plus an in-memory FST for names.
//! - **Search** — fuzzy jump (`nucleo`, frecency-ranked), whole-disk filename
//!   search, and streamed content search (`grep-searcher`, `ignore`).
//! - **Query language** — parser for queries such as
//!   `ext:ts size:>1mb modified:<7d in:~/code -node_modules`.
//!
//! Budgets this crate owns: keystroke → fuzzy results < 16 ms, warm whole-disk
//! search < 50 ms, initial index of 1M files < 60 s (see `docs/PERFORMANCE.md`).

pub mod fuzzy;
