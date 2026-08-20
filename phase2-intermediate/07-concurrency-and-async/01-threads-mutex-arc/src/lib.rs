use std::sync::{Arc, Mutex};
use std::thread;

/// Splits `nums` into up to `thread_count` chunks, sums each chunk in its
/// own thread, and returns the total. No shared *mutable* state — each
/// thread computes an independent partial sum and hands it back through
/// its `JoinHandle`; nothing here needs a `Mutex`.
pub fn sum_in_threads(nums: Vec<i32>, thread_count: usize) -> i32 {
    todo!(
        "chunk nums into thread_count pieces, thread::spawn(move || chunk.iter().sum()) per piece, join and sum the results"
    )
}

/// Spawns one thread per item in `items`. Each thread checks `predicate`
/// against its item and, if it matches, increments a count *shared across
/// all threads* — this DOES need `Arc<Mutex<i32>>`, since multiple threads
/// may try to increment the same counter at the same time.
pub fn count_matching_in_threads(items: Vec<i32>, predicate: fn(i32) -> bool) -> i32 {
    todo!(
        "Arc::new(Mutex::new(0)); per item, clone the Arc, spawn a thread that locks and increments if predicate(item); join all; \
         then `let final_count = *counter.lock().unwrap(); final_count` — NOT a bare tail-expression lock().unwrap(), \
         which trips a real borrow-checker/drop-order error here (see recall questions)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn counts_matching_items_across_threads() {
        let count = count_matching_in_threads(vec![1, 2, 3, 4, 5, 6], |n| n % 2 == 0);
        assert_eq!(count, 3);
    }

    #[test]
    fn counts_zero_when_nothing_matches() {
        let count = count_matching_in_threads(vec![1, 3, 5], |n| n % 2 == 0);
        assert_eq!(count, 0);
    }
}
