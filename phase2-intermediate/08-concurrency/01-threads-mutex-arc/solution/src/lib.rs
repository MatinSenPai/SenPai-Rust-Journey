//! Threads, `Mutex`, `Arc` — reference solution.

use std::sync::{Arc, Mutex};
use std::thread;

pub fn sum_in_threads(nums: Vec<i32>, thread_count: usize) -> i32 {
    let thread_count = thread_count.max(1);
    let chunk_size = nums.len().div_ceil(thread_count).max(1);
    let chunks: Vec<Vec<i32>> = nums.chunks(chunk_size).map(|c| c.to_vec()).collect();

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| thread::spawn(move || chunk.iter().sum::<i32>()))
        .collect();

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

pub fn count_matching_in_threads(items: Vec<i32>, predicate: fn(i32) -> bool) -> i32 {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for item in items {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            if predicate(item) {
                *counter.lock().unwrap() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // NOT a bare tail expression `*counter.lock().unwrap()` — see 2.8.1's
    // "Errors you will meet" (E0597). Binding first drops the guard before
    // `counter` itself goes out of scope.
    let final_count = *counter.lock().unwrap();
    final_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn sums_across_multiple_threads() {
        assert_eq!(sum_in_threads(vec![1, 2, 3, 4, 5, 6], 3), 21);
    }

    #[test]
    fn sums_with_more_threads_than_elements() {
        assert_eq!(sum_in_threads(vec![1, 2], 10), 3);
    }

    #[test]
    fn sums_an_empty_vec() {
        assert_eq!(sum_in_threads(vec![], 4), 0);
    }

    #[test]
    fn thread_count_zero_is_treated_as_one() {
        assert_eq!(sum_in_threads(vec![1, 2, 3], 0), 6);
    }

    #[test]
    fn counts_matching_items_across_threads() {
        let count = count_matching_in_threads(vec![1, 2, 3, 4, 5, 6], |n| n % 2 == 0);
        assert_eq!(count, 3);
    }

    #[test]
    fn counts_zero_when_nothing_matches() {
        let count = count_matching_in_threads(vec![1, 3, 5], |n| n % 2 == 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn counts_an_empty_vec() {
        assert_eq!(count_matching_in_threads(vec![], |n| n > 0), 0);
    }

    // A same-shape "Build"-rung style check: a shared `HashMap` behind
    // `Arc<Mutex<_>>`, updated from several threads, ends up with every
    // update applied exactly once — run several times in review to catch
    // any flakiness a single green run would hide.
    #[test]
    fn shared_hashmap_across_threads_gets_every_update() {
        let tally: Arc<Mutex<HashMap<&'static str, u32>>> = Arc::new(Mutex::new(HashMap::new()));
        let words = [
            "frieren",
            "bocchi",
            "frieren",
            "made-in-abyss",
            "bocchi",
            "frieren",
        ];

        let handles: Vec<_> = words
            .into_iter()
            .map(|word| {
                let tally = Arc::clone(&tally);
                thread::spawn(move || {
                    *tally.lock().unwrap().entry(word).or_insert(0) += 1;
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }

        let tally = tally.lock().unwrap();
        assert_eq!(tally.get("frieren"), Some(&3));
        assert_eq!(tally.get("bocchi"), Some(&2));
        assert_eq!(tally.get("made-in-abyss"), Some(&1));
    }
}
