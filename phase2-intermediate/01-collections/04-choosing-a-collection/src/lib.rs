//! Exercises for 2.1.4 — choosing a collection.
//!
//! Every function and method below is specified by what it must do, not by
//! which of the six collections to reach for inside it. That choice is the
//! point of this lesson — decide it the way `## مفهوم`/`## The concept`
//! taught you to, then write the body.

use std::collections::VecDeque;

/// How many distinct viewer IDs appear in `viewer_ids`. An ID may repeat any
/// number of times in the slice; it still counts once.
///
/// # Examples
///
/// `unique_viewer_count(&[1, 2, 2, 3, 1])` returns `3`.
/// `unique_viewer_count(&[])` returns `0`.
/// `unique_viewer_count(&[5, 5, 5])` returns `1`.
pub fn unique_viewer_count(viewer_ids: &[u32]) -> usize {
    todo!(
        "count how many distinct values appear in `viewer_ids`, so that repeats of the same id \
         are counted once"
    )
}

/// Keeps track of the most recently recorded events, oldest dropped first
/// once more than `capacity` have been recorded.
pub struct RecentEvents {
    capacity: usize,
    events: VecDeque<String>,
}

impl RecentEvents {
    /// A tracker that holds at most `capacity` events at a time. A
    /// `capacity` of `0` holds none, ever.
    pub fn new(capacity: usize) -> Self {
        todo!("store `capacity` and start out holding nothing")
    }

    /// Records `event` as the newest one seen. If the tracker is already
    /// holding `capacity` events, the oldest one is dropped first so the
    /// count never exceeds `capacity`.
    pub fn record(&mut self, event: &str) {
        todo!(
            "add `event` as the newest entry, then drop the oldest entry, if any, for as long \
             as more than `capacity` entries are held"
        )
    }

    /// The events currently held, oldest first.
    ///
    /// # Examples
    ///
    /// Recording `"a"`, `"b"`, `"c"`, `"d"` in that order into a tracker
    /// built with `RecentEvents::new(3)` leaves `oldest_to_newest()` equal
    /// to `vec!["b".to_string(), "c".to_string(), "d".to_string()]` — `"a"`
    /// was dropped the moment `"d"` arrived.
    pub fn oldest_to_newest(&self) -> Vec<String> {
        todo!("return the events currently held, ordered oldest first, as owned strings")
    }
}

/// Builds a report of event counts by day from `entries` of `(day, count)`
/// pairs. `entries` may list the same day more than once and in any order;
/// when a day repeats, the entry that appears **later** in the slice is the
/// one that counts (it is not summed with the earlier one).
///
/// The report has exactly one line per distinct day, each formatted
/// `"day {day}: {count}\n"`, in ascending order of day. An empty `entries`
/// produces an empty report.
///
/// # Examples
///
/// `daily_report(&[(3, 40), (1, 12), (2, 25)])` returns
/// `"day 1: 12\nday 2: 25\nday 3: 40\n"`.
/// `daily_report(&[(1, 5), (1, 9)])` returns `"day 1: 9\n"`.
/// `daily_report(&[])` returns `""`.
pub fn daily_report(entries: &[(u32, u32)]) -> String {
    todo!(
        "keep only the latest count seen for each day, then build one line per distinct day in \
         ascending order of day, in the exact format the doc comment states"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_each_distinct_viewer_once() {
        assert_eq!(unique_viewer_count(&[1, 2, 2, 3, 1]), 3);
        assert_eq!(unique_viewer_count(&[5, 5, 5]), 1);
        assert_eq!(unique_viewer_count(&[1, 2, 3, 4]), 4);
    }

    #[test]
    fn empty_input_has_no_distinct_viewers() {
        assert_eq!(unique_viewer_count(&[]), 0);
    }

    #[test]
    fn recent_events_keeps_only_the_newest_capacity_entries() {
        let mut recent = RecentEvents::new(3);
        recent.record("a");
        recent.record("b");
        recent.record("c");
        recent.record("d");
        assert_eq!(
            recent.oldest_to_newest(),
            vec!["b".to_string(), "c".to_string(), "d".to_string()]
        );
    }

    #[test]
    fn recent_events_holds_everything_under_capacity() {
        let mut recent = RecentEvents::new(5);
        recent.record("only");
        recent.record("two");
        assert_eq!(
            recent.oldest_to_newest(),
            vec!["only".to_string(), "two".to_string()]
        );
    }

    #[test]
    fn recent_events_with_zero_capacity_holds_nothing() {
        let mut recent = RecentEvents::new(0);
        recent.record("gone immediately");
        assert_eq!(recent.oldest_to_newest(), Vec::<String>::new());
    }

    #[test]
    fn daily_report_sorts_by_day_ascending() {
        assert_eq!(
            daily_report(&[(3, 40), (1, 12), (2, 25)]),
            "day 1: 12\nday 2: 25\nday 3: 40\n"
        );
    }

    #[test]
    fn daily_report_keeps_the_later_entry_for_a_repeated_day() {
        assert_eq!(daily_report(&[(1, 5), (1, 9)]), "day 1: 9\n");
    }

    #[test]
    fn daily_report_of_no_entries_is_empty() {
        assert_eq!(daily_report(&[]), "");
    }
}
