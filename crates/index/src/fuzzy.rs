//! Fuzzy matching of file names, built on `nucleo`.
//!
//! This is the smallest real matcher: one query against one in-memory list, on
//! the calling thread. Frecency ranking and incremental matching across
//! keystrokes come with the search work in a later phase.

use nucleo::Matcher;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};

/// A name that matched a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyMatch<'a> {
    /// The matching name.
    pub name: &'a str,
    /// Match quality; higher is better.
    pub score: u32,
}

/// Reusable fuzzy matcher. Holds scratch memory, so keep one around rather than
/// creating one per query.
pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    /// Creates a matcher tuned for file names.
    #[must_use]
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo::Config::DEFAULT.match_paths()),
        }
    }

    /// Returns the `limit` best matches for `query` in `names`, best first.
    ///
    /// Matching is smart-case: a query with an uppercase letter is case-sensitive.
    pub fn best_matches<'a, S: AsRef<str>>(
        &mut self,
        names: &'a [S],
        query: &str,
        limit: usize,
    ) -> Vec<FuzzyMatch<'a>> {
        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut matches: Vec<_> = pattern
            .match_list(names.iter().map(AsRef::as_ref), &mut self.matcher)
            .into_iter()
            .map(|(name, score)| FuzzyMatch { name, score })
            .collect();
        matches.truncate(limit);
        matches
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: [&str; 4] = [
        "annual_report_2024.pdf",
        "report.txt",
        "holiday photo.jpg",
        "Rep_Old.doc",
    ];

    #[test]
    fn ranks_better_matches_first_and_skips_non_matches() {
        let matches = FuzzyMatcher::new().best_matches(&NAMES, "report", 10);
        let names: Vec<_> = matches.iter().map(|m| m.name).collect();
        assert_eq!(names, ["report.txt", "annual_report_2024.pdf"]);
        assert!(matches[0].score > matches[1].score);
    }

    #[test]
    fn matches_non_contiguous_characters() {
        let matches = FuzzyMatcher::new().best_matches(&NAMES, "hdyjpg", 10);
        assert_eq!(matches[0].name, "holiday photo.jpg");
    }

    #[test]
    fn respects_limit() {
        assert_eq!(FuzzyMatcher::new().best_matches(&NAMES, "e", 2).len(), 2);
    }

    #[test]
    fn uppercase_query_is_case_sensitive() {
        let matches = FuzzyMatcher::new().best_matches(&NAMES, "Rep", 10);
        let names: Vec<_> = matches.iter().map(|m| m.name).collect();
        assert_eq!(names, ["Rep_Old.doc"]);
    }
}
