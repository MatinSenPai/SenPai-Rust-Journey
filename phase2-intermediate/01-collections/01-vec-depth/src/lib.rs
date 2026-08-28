//! Exercises for 2.1.1 — `Vec` in depth.
//!
//! `.retain()`, `.drain()`, `.dedup_by_key()`, `.sort_by()` and
//! `.binary_search_by()` are the tools; no `.iter().collect()` chains
//! needed anywhere here — those are Phase 2's iterators module.

/// A single row of watch history: a title, this viewer's rating out of 10,
/// and whether the episode has been watched yet.
#[derive(Debug, Clone, PartialEq)]
pub struct WatchEntry {
    pub title: String,
    pub rating: f64,
    pub watched: bool,
}

/// Keeps only the entries that have not been watched yet, in place, in
/// their original relative order. Watched entries are dropped.
///
/// # Examples
///
/// Given entries titled `"A", "B", "C", "D"` with `watched` values `true,
/// false, true, false`, only `"B"` and `"D"` remain afterward, in that
/// order.
pub fn retain_unwatched(entries: &mut Vec<WatchEntry>) {
    todo!("keep only the entries whose `watched` field is false; drop the rest, in place")
}

/// Removes and returns the first `n` entries of `entries`, leaving the rest
/// behind in their original order. If `n` is greater than or equal to
/// `entries.len()`, every entry is removed and returned, and `entries` ends
/// up empty.
///
/// # Examples
///
/// `drain_first_n` on entries titled `"A", "B", "C", "D"` with `n = 2`
/// returns the `"A"`, `"B"` entries and leaves `"C"`, `"D"` behind.
pub fn drain_first_n(entries: &mut Vec<WatchEntry>, n: usize) -> Vec<WatchEntry> {
    todo!(
        "remove the first `n` entries from `entries` and gather exactly those into the Vec you \
         return, leaving whatever is left behind in `entries`; if `n` reaches past the end, that \
         just means everything gets removed"
    )
}

/// Removes an entry when its `title` is an exact match for the title of the
/// entry immediately before it, keeping the first of each run. Entries are
/// compared by `title` only — a repeated title with a different `rating` or
/// `watched` value still counts as a duplicate, and it's the *earlier*
/// entry (with its own rating and watched flag) that survives.
///
/// This only ever looks at *neighboring* entries. It does not sort first,
/// so a duplicate title that isn't already adjacent in `entries` is left
/// alone — sorting first (or not) is the caller's decision, not this
/// function's.
///
/// # Examples
///
/// Given titles `"Frieren", "Frieren", "Bocchi", "Frieren"` in that order,
/// the result keeps titles `"Frieren", "Bocchi", "Frieren"` — the third
/// `"Frieren"` survives because it is not adjacent to the first two.
pub fn dedup_adjacent_titles(entries: &mut Vec<WatchEntry>) {
    todo!(
        "remove an entry whenever its title exactly matches the title of the entry right before \
         it in the Vec; leave titles that repeat non-adjacently untouched"
    )
}

/// Returns a new `Vec` holding every entry from `entries`, sorted by
/// `rating` ascending. When two entries share the same rating, the one that
/// appeared earlier in `entries` must still appear earlier in the result —
/// sorting by rating alone must never reorder entries that tie.
///
/// # Examples
///
/// Two entries both rated `7.5`, one titled `"A"` appearing before one
/// titled `"B"` in the input, appear as `"A"` then `"B"` in the output too,
/// even though nothing but position distinguishes them.
pub fn sorted_by_rating(entries: Vec<WatchEntry>) -> Vec<WatchEntry> {
    todo!(
        "sort a copy of `entries` by the `rating` field only, ascending, using a comparison that \
         keeps entries with equal ratings in their original relative order"
    )
}

/// Binary-searches `entries` — which the caller guarantees is already
/// sorted by `rating` ascending, exactly as `sorted_by_rating` produces —
/// for an entry whose `rating` equals `target`. Returns the index of a
/// match if one exists, `None` otherwise.
///
/// Calling this on an `entries` slice that is not sorted by rating is a
/// caller error: the result is unspecified. It will not panic, but it may
/// answer `None` for a rating that is genuinely present.
///
/// # Examples
///
/// `find_by_rating` on ratings, in order, `3.0, 5.5, 7.5, 9.0` for
/// `target = 7.5` returns `Some(2)`. For `target = 6.0` (absent) it returns
/// `None`.
pub fn find_by_rating(entries: &[WatchEntry], target: f64) -> Option<usize> {
    todo!(
        "binary-search `entries` by comparing each entry's `rating` field against `target`; on a \
         match return its index wrapped in Some, otherwise None"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str, rating: f64, watched: bool) -> WatchEntry {
        WatchEntry {
            title: title.to_string(),
            rating,
            watched,
        }
    }

    fn titles_of(entries: &[WatchEntry]) -> Vec<String> {
        let mut out = Vec::new();
        for item in entries {
            out.push(item.title.clone());
        }
        out
    }

    #[test]
    fn retain_unwatched_keeps_only_unwatched_in_order() {
        let mut entries = vec![
            entry("A", 8.0, true),
            entry("B", 7.0, false),
            entry("C", 9.0, true),
            entry("D", 6.0, false),
        ];
        retain_unwatched(&mut entries);
        assert_eq!(titles_of(&entries), vec!["B", "D"]);
    }

    #[test]
    fn retain_unwatched_of_all_watched_is_empty() {
        let mut entries = vec![entry("A", 8.0, true), entry("B", 7.0, true)];
        retain_unwatched(&mut entries);
        assert!(entries.is_empty());
    }

    #[test]
    fn drain_first_n_splits_the_list() {
        let mut entries = vec![
            entry("A", 1.0, false),
            entry("B", 2.0, false),
            entry("C", 3.0, false),
            entry("D", 4.0, false),
        ];
        let removed = drain_first_n(&mut entries, 2);
        assert_eq!(titles_of(&removed), vec!["A", "B"]);
        assert_eq!(titles_of(&entries), vec!["C", "D"]);
    }

    #[test]
    fn drain_first_n_past_the_end_takes_everything() {
        let mut entries = vec![entry("A", 1.0, false), entry("B", 2.0, false)];
        let removed = drain_first_n(&mut entries, 10);
        assert_eq!(titles_of(&removed), vec!["A", "B"]);
        assert!(entries.is_empty());
    }

    #[test]
    fn drain_first_n_of_zero_takes_nothing() {
        let mut entries = vec![entry("A", 1.0, false)];
        let removed = drain_first_n(&mut entries, 0);
        assert!(removed.is_empty());
        assert_eq!(titles_of(&entries), vec!["A"]);
    }

    #[test]
    fn dedup_adjacent_titles_removes_only_neighboring_repeats() {
        let mut entries = vec![
            entry("Frieren", 9.0, false),
            entry("Frieren", 8.0, true),
            entry("Bocchi", 7.5, false),
            entry("Frieren", 9.0, false),
        ];
        dedup_adjacent_titles(&mut entries);
        assert_eq!(titles_of(&entries), vec!["Frieren", "Bocchi", "Frieren"]);
        assert_eq!(entries[0].rating, 9.0);
        assert!(!entries[0].watched);
    }

    #[test]
    fn dedup_adjacent_titles_with_no_neighbors_is_unchanged() {
        let mut entries = vec![entry("Frieren", 9.0, false), entry("Bocchi", 7.5, false)];
        let before = entries.clone();
        dedup_adjacent_titles(&mut entries);
        assert_eq!(entries, before);
    }

    #[test]
    fn sorted_by_rating_orders_ascending_and_keeps_ties_stable() {
        let entries = vec![
            entry("A", 7.5, false),
            entry("B", 9.0, false),
            entry("C", 7.5, false),
            entry("D", 6.0, false),
        ];
        let sorted = sorted_by_rating(entries);
        assert_eq!(titles_of(&sorted), vec!["D", "A", "C", "B"]);
    }

    #[test]
    fn sorted_by_rating_of_empty_is_empty() {
        assert!(sorted_by_rating(Vec::new()).is_empty());
    }

    #[test]
    fn find_by_rating_locates_an_existing_value() {
        let entries = vec![
            entry("D", 6.0, false),
            entry("A", 7.5, false),
            entry("B", 9.0, false),
        ];
        assert_eq!(find_by_rating(&entries, 7.5), Some(1));
        assert_eq!(find_by_rating(&entries, 6.0), Some(0));
    }

    #[test]
    fn find_by_rating_reports_a_missing_value() {
        let entries = vec![
            entry("D", 6.0, false),
            entry("A", 7.5, false),
            entry("B", 9.0, false),
        ];
        assert_eq!(find_by_rating(&entries, 8.0), None);
    }

    #[test]
    fn find_by_rating_of_empty_slice_is_none() {
        assert_eq!(find_by_rating(&[], 5.0), None);
    }
}
