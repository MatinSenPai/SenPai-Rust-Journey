//! Solution for 2.1.1 — `Vec` in depth.

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
pub fn retain_unwatched(entries: &mut Vec<WatchEntry>) {
    entries.retain(|entry| !entry.watched);
}

/// Removes and returns the first `n` entries of `entries`, leaving the rest
/// behind in their original order.
pub fn drain_first_n(entries: &mut Vec<WatchEntry>, n: usize) -> Vec<WatchEntry> {
    let n = n.min(entries.len());
    let mut removed = Vec::new();
    for item in entries.drain(0..n) {
        removed.push(item);
    }
    removed
}

/// Removes an entry when its `title` exactly matches the title of the entry
/// immediately before it, keeping the first of each run.
pub fn dedup_adjacent_titles(entries: &mut Vec<WatchEntry>) {
    entries.dedup_by_key(|entry| entry.title.clone());
}

/// Returns a new `Vec` holding every entry from `entries`, sorted by
/// `rating` ascending, stably.
pub fn sorted_by_rating(entries: Vec<WatchEntry>) -> Vec<WatchEntry> {
    let mut entries = entries;
    entries.sort_by(|a, b| a.rating.total_cmp(&b.rating));
    entries
}

/// Binary-searches `entries` (already sorted by `rating` ascending) for an
/// entry whose `rating` equals `target`.
pub fn find_by_rating(entries: &[WatchEntry], target: f64) -> Option<usize> {
    entries
        .binary_search_by(|entry| entry.rating.total_cmp(&target))
        .ok()
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
