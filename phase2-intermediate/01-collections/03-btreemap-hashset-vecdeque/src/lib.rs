//! Exercises for 2.1.3 — `BTreeMap`, `HashSet`, `VecDeque`, `BinaryHeap`.
//!
//! Every collection here is one this lesson just covered. `.collect()` is not
//! needed anywhere below — a plain `for` loop, pushing into a fresh
//! collection, is always enough.

use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};

/// Converts `counts` into a `BTreeMap` with the same entries, so iterating
/// the result afterward always yields titles in ascending alphabetical
/// order — no explicit sort needed.
///
/// # Examples
///
/// `sorted_by_title` on a map containing `"naruto" -> 4` and `"bleach" -> 2`
/// produces a `BTreeMap` that iterates as `[("bleach", 2), ("naruto", 4)]`.
pub fn sorted_by_title(counts: HashMap<String, u32>) -> BTreeMap<String, u32> {
    todo!("build a new, empty BTreeMap and move every title/count pair out of `counts` into it")
}

/// The titles in `releases` whose year falls within `start..=end` (both
/// ends included), in ascending year order.
///
/// If `start` is greater than `end`, returns an empty `Vec` without
/// querying `releases` — see "Errors you will meet" for what happens if you
/// skip this guard.
///
/// # Examples
///
/// With `releases` containing `2019 -> "demon-slayer"`, `2022 -> "bocchi"`
/// and `2023 -> "frieren"`:
/// `shows_in_year_range(&releases, 2019, 2022)` returns
/// `vec!["demon-slayer".to_string(), "bocchi".to_string()]`.
/// `shows_in_year_range(&releases, 2030, 2000)` returns `vec![]`.
pub fn shows_in_year_range(releases: &BTreeMap<u32, String>, start: u32, end: u32) -> Vec<String> {
    todo!(
        "handle the start-after-end case first by returning an empty Vec; otherwise walk the \
         entries whose year falls between start and end inclusive, in ascending order, and push \
         each title into a new Vec"
    )
}

/// The genres present in both `a` and `b`.
///
/// # Examples
///
/// With `a` containing `"action"`, `"comedy"`, `"isekai"` and `b` containing
/// `"comedy"`, `"drama"`: `shared_genres(&a, &b)` returns a set containing
/// only `"comedy"`.
pub fn shared_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    todo!(
        "look at every genre a and b have in common, and place an owned copy of each into a new \
         HashSet"
    )
}

/// The genres present in exactly one of `a` or `b`, never both.
///
/// # Examples
///
/// With `a` containing `"action"`, `"comedy"`, `"isekai"` and `b` containing
/// `"comedy"`, `"drama"`: `exclusive_genres(&a, &b)` returns a set
/// containing `"action"`, `"isekai"` and `"drama"`.
pub fn exclusive_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    todo!(
        "look at every genre that belongs to exactly one of a or b, and place an owned copy of \
         each into a new HashSet"
    )
}

/// A simple "up next" queue of show titles. Wraps a `VecDeque` instead of a
/// `Vec` specifically because titles need to be added and removed from
/// *both* ends in O(1): normal arrivals join the back, but
/// `watch_next_priority` has to jump a title to the front without shifting
/// every other element over — which `Vec::insert(0, _)` would do, in O(n).
pub struct WatchQueue {
    queue: VecDeque<String>,
}

impl WatchQueue {
    /// Creates an empty queue.
    pub fn new() -> Self {
        todo!("wrap a fresh, empty VecDeque")
    }

    /// Adds `title` to the back of the queue — the normal way a title joins
    /// the "up next" list.
    pub fn enqueue(&mut self, title: String) {
        todo!("add title to the back of the queue")
    }

    /// Removes and returns the title at the front of the queue, or `None`
    /// if the queue is empty.
    pub fn watch_next(&mut self) -> Option<String> {
        todo!("remove and return the title at the front of the queue, if there is one")
    }

    /// Jumps `title` straight to the front of the queue, ahead of
    /// everything already waiting.
    pub fn watch_next_priority(&mut self, title: String) {
        todo!("add title to the front of the queue")
    }

    /// Number of titles currently queued.
    #[must_use]
    pub fn len(&self) -> usize {
        todo!("report how many titles are currently queued")
    }

    /// Whether the queue has no titles left.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!("report whether the queue has no titles queued")
    }
}

/// A priority "up next" queue. Whichever queued title has the highest
/// `priority` is always the one `watch_highest_priority` hands back next,
/// no matter what order titles were added in. When two titles share a
/// priority, the one that is greater in lexicographic (dictionary) order
/// comes back first — the same rule a `(u32, String)` tuple already
/// compares by, which is exactly what this type stores.
pub struct WatchPriorityQueue {
    queue: BinaryHeap<(u32, String)>,
}

impl WatchPriorityQueue {
    /// Creates an empty priority queue.
    pub fn new() -> Self {
        todo!("wrap a fresh, empty BinaryHeap")
    }

    /// Adds `title` with the given `priority`. Higher numbers are watched
    /// sooner.
    pub fn add(&mut self, priority: u32, title: String) {
        todo!("push the priority and title onto the heap as a pair, priority first")
    }

    /// Removes and returns the title with the highest priority currently
    /// queued, discarding the priority itself, or `None` if the queue is
    /// empty.
    pub fn watch_highest_priority(&mut self) -> Option<String> {
        todo!(
            "remove the highest-priority pair from the heap, if there is one, and hand back only \
             its title"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_by_title_orders_entries_alphabetically() {
        let counts: HashMap<String, u32> = HashMap::from([
            ("charlie".to_string(), 1),
            ("alice".to_string(), 2),
            ("bob".to_string(), 3),
        ]);

        let sorted = sorted_by_title(counts);
        let mut ordered_titles: Vec<&String> = Vec::new();
        for title in sorted.keys() {
            ordered_titles.push(title);
        }
        assert_eq!(ordered_titles, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn sorted_by_title_of_empty_map_is_empty() {
        let empty: HashMap<String, u32> = HashMap::new();
        assert!(sorted_by_title(empty).is_empty());
    }

    #[test]
    fn shows_in_year_range_returns_titles_in_ascending_year_order() {
        let releases: BTreeMap<u32, String> = BTreeMap::from([
            (2019, "demon-slayer".to_string()),
            (2022, "bocchi".to_string()),
            (2023, "frieren".to_string()),
            (2013, "attack-on-titan".to_string()),
        ]);

        assert_eq!(
            shows_in_year_range(&releases, 2019, 2022),
            vec!["demon-slayer".to_string(), "bocchi".to_string()]
        );
    }

    #[test]
    fn shows_in_year_range_includes_both_endpoints() {
        let releases: BTreeMap<u32, String> = BTreeMap::from([
            (2019, "demon-slayer".to_string()),
            (2023, "frieren".to_string()),
        ]);

        assert_eq!(
            shows_in_year_range(&releases, 2019, 2023),
            vec!["demon-slayer".to_string(), "frieren".to_string()]
        );
    }

    #[test]
    fn shows_in_year_range_is_empty_when_start_is_after_end() {
        let releases: BTreeMap<u32, String> = BTreeMap::from([(2019, "demon-slayer".to_string())]);

        assert_eq!(
            shows_in_year_range(&releases, 2030, 2000),
            Vec::<String>::new()
        );
    }

    #[test]
    fn shared_genres_returns_the_intersection() {
        let a: HashSet<String> = HashSet::from([
            "action".to_string(),
            "comedy".to_string(),
            "isekai".to_string(),
        ]);
        let b: HashSet<String> = HashSet::from(["comedy".to_string(), "drama".to_string()]);

        assert_eq!(shared_genres(&a, &b), HashSet::from(["comedy".to_string()]));
    }

    #[test]
    fn shared_genres_of_disjoint_sets_is_empty() {
        let a: HashSet<String> = HashSet::from(["action".to_string()]);
        let b: HashSet<String> = HashSet::from(["drama".to_string()]);
        assert!(shared_genres(&a, &b).is_empty());
    }

    #[test]
    fn exclusive_genres_returns_the_symmetric_difference() {
        let a: HashSet<String> = HashSet::from([
            "action".to_string(),
            "comedy".to_string(),
            "isekai".to_string(),
        ]);
        let b: HashSet<String> = HashSet::from(["comedy".to_string(), "drama".to_string()]);

        assert_eq!(
            exclusive_genres(&a, &b),
            HashSet::from([
                "action".to_string(),
                "isekai".to_string(),
                "drama".to_string(),
            ])
        );
    }

    #[test]
    fn new_queue_is_empty() {
        let q = WatchQueue::new();
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn enqueue_and_watch_next_is_first_in_first_out() {
        let mut q = WatchQueue::new();
        q.enqueue("Frieren".to_string());
        q.enqueue("Bocchi".to_string());
        assert_eq!(q.len(), 2);
        assert_eq!(q.watch_next(), Some("Frieren".to_string()));
        assert_eq!(q.watch_next(), Some("Bocchi".to_string()));
        assert_eq!(q.watch_next(), None);
    }

    #[test]
    fn watch_next_priority_jumps_the_queue() {
        let mut q = WatchQueue::new();
        q.enqueue("Frieren".to_string());
        q.enqueue("Bocchi".to_string());
        q.watch_next_priority("Bleach".to_string());

        assert_eq!(q.watch_next(), Some("Bleach".to_string()));
        assert_eq!(q.watch_next(), Some("Frieren".to_string()));
        assert_eq!(q.watch_next(), Some("Bocchi".to_string()));
    }

    #[test]
    fn new_priority_queue_watches_nothing() {
        let mut q = WatchPriorityQueue::new();
        assert_eq!(q.watch_highest_priority(), None);
    }

    #[test]
    fn watch_highest_priority_ignores_arrival_order() {
        let mut q = WatchPriorityQueue::new();
        q.add(2, "Bocchi".to_string());
        q.add(5, "Frieren".to_string());
        q.add(1, "Bleach".to_string());

        assert_eq!(q.watch_highest_priority(), Some("Frieren".to_string()));
        assert_eq!(q.watch_highest_priority(), Some("Bocchi".to_string()));
        assert_eq!(q.watch_highest_priority(), Some("Bleach".to_string()));
        assert_eq!(q.watch_highest_priority(), None);
    }

    #[test]
    fn watch_highest_priority_breaks_ties_by_greater_title() {
        let mut q = WatchPriorityQueue::new();
        q.add(5, "Bleach".to_string());
        q.add(5, "Frieren".to_string());

        assert_eq!(q.watch_highest_priority(), Some("Frieren".to_string()));
        assert_eq!(q.watch_highest_priority(), Some("Bleach".to_string()));
    }
}
