use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// Converts a `HashMap` into a `BTreeMap` with the same entries. Once
/// converted, iterating the result always yields entries in ascending key
/// order — no explicit sort required, unlike `top_n` from the previous
/// lesson.
pub fn sorted_by_key(counts: HashMap<String, u32>) -> BTreeMap<String, u32> {
    todo!("counts.into_iter().collect()")
}

/// Returns the genres present in *both* `a` and `b` (set intersection).
pub fn shared_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    todo!("a.intersection(b).cloned().collect()")
}

/// Returns the genres present in exactly *one* of `a` or `b`, but not both
/// (set symmetric difference).
pub fn exclusive_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    todo!("a.symmetric_difference(b).cloned().collect()")
}

/// A simple "up next" queue of show titles. Wraps `VecDeque` instead of
/// `Vec` specifically because titles need to be added and removed from
/// *both* ends in O(1): normal additions go to the back, but
/// `watch_next_priority` needs to jump a title to the front without
/// shifting every other element over (which `Vec::insert(0, _)` would do,
/// in O(n)).
pub struct WatchQueue {
    queue: VecDeque<String>,
}

impl Default for WatchQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl WatchQueue {
    /// Creates an empty queue.
    pub fn new() -> Self {
        todo!("Self {{ queue: VecDeque::new() }}")
    }

    /// Adds `title` to the back of the queue — the normal way a title
    /// joins the "up next" list.
    pub fn enqueue(&mut self, title: String) {
        todo!("self.queue.push_back(title)")
    }

    /// Removes and returns the title at the front of the queue, or `None`
    /// if the queue is empty.
    pub fn watch_next(&mut self) -> Option<String> {
        todo!("self.queue.pop_front()")
    }

    /// Jumps `title` straight to the front of the queue, ahead of
    /// everything already waiting.
    pub fn watch_next_priority(&mut self, title: String) {
        todo!("self.queue.push_front(title)")
    }

    /// Number of titles currently queued.
    #[must_use]
    pub fn len(&self) -> usize {
        todo!("self.queue.len()")
    }

    /// Whether the queue has no titles left.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!("self.queue.is_empty()")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_by_key_orders_entries_by_key_ascending() {
        let counts: HashMap<String, u32> = HashMap::from([
            ("charlie".to_string(), 1),
            ("alice".to_string(), 2),
            ("bob".to_string(), 3),
        ]);

        let sorted = sorted_by_key(counts);
        let ordered_keys: Vec<&String> = sorted.keys().collect();
        assert_eq!(ordered_keys, vec!["alice", "bob", "charlie"]);
    }

    #[test]
    fn sorted_by_key_of_empty_map_is_empty() {
        let empty: HashMap<String, u32> = HashMap::new();
        assert!(sorted_by_key(empty).is_empty());
    }

    #[test]
    fn shared_genres_returns_the_intersection() {
        let a: HashSet<String> = ["action", "comedy", "isekai"]
            .into_iter()
            .map(String::from)
            .collect();
        let b: HashSet<String> = ["comedy", "drama"].into_iter().map(String::from).collect();

        let shared = shared_genres(&a, &b);
        assert_eq!(shared, HashSet::from(["comedy".to_string()]));
    }

    #[test]
    fn shared_genres_of_disjoint_sets_is_empty() {
        let a: HashSet<String> = HashSet::from(["action".to_string()]);
        let b: HashSet<String> = HashSet::from(["drama".to_string()]);
        assert!(shared_genres(&a, &b).is_empty());
    }

    #[test]
    fn exclusive_genres_returns_the_symmetric_difference() {
        let a: HashSet<String> = ["action", "comedy", "isekai"]
            .into_iter()
            .map(String::from)
            .collect();
        let b: HashSet<String> = ["comedy", "drama"].into_iter().map(String::from).collect();

        let exclusive = exclusive_genres(&a, &b);
        assert_eq!(
            exclusive,
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
    fn enqueue_and_watch_next_is_fifo() {
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
}
