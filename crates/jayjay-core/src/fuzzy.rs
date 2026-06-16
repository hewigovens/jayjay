use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Rank `candidates` against `query`, best match first; returns their indices.
/// Whitespace splits the query into atoms that must all match. An empty query
/// keeps every candidate in original order.
pub fn rank(query: &str, candidates: &[String]) -> Vec<u32> {
    let query = query.trim();
    if query.is_empty() {
        return (0..candidates.len() as u32).collect();
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut buf = Vec::new();
    let mut scored: Vec<(u32, u32)> = candidates
        .iter()
        .enumerate()
        .filter_map(|(ix, candidate)| {
            pattern
                .score(Utf32Str::new(candidate, &mut buf), &mut matcher)
                .map(|score| (score, ix as u32))
        })
        .collect();
    // Ties keep declaration order.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, ix)| ix).collect()
}

#[cfg(test)]
mod tests {
    use super::rank;

    fn candidates(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn empty_query_keeps_original_order() {
        let items = candidates(&["b", "a"]);
        assert_eq!(rank("", &items), vec![0, 1]);
        assert_eq!(rank("  ", &items), vec![0, 1]);
    }

    #[test]
    fn matches_subsequences_case_insensitively() {
        let items = candidates(&["Toggle Side-by-side Diff", "Open Settings"]);
        assert_eq!(rank("tsd", &items), vec![0]);
        assert_eq!(rank("SETT", &items), vec![1]);
        assert!(rank("bookmark", &items).is_empty());
    }

    #[test]
    fn all_query_words_must_match() {
        let items = candidates(&["Toggle Tree File List tree file folder list", "Refresh"]);
        assert_eq!(rank("tree list", &items), vec![0]);
        assert!(rank("tree bookmark", &items).is_empty());
    }

    #[test]
    fn closer_matches_rank_first() {
        let items = candidates(&["The Memo Editor", "Theme: Dark", "Theme: Light"]);
        let ranked = rank("theme", &items);
        assert_eq!(ranked.len(), 3);
        // Contiguous "Theme" beats the scattered match; ties keep order.
        assert_eq!(&ranked[..2], &[1, 2]);
    }
}
