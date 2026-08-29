//! Threads, `Mutex`, `Arc` — exercise skeleton.
//!
//! Every function's doc comment is its full specification. The tests below
//! only check what those doc comments already describe — you should never
//! need to open this module's test block to know what to build.

use std::sync::{Arc, Mutex};
use std::thread;

/// Splits `nums` across up to `thread_count` threads, sums each thread's
/// own share on that thread, then joins every thread and adds the partial
/// sums into one grand total.
///
/// Treat a `thread_count` of `0` the same as `1`. An empty `nums` returns
/// `0`. No shared mutable state is needed here: each thread computes an
/// independent partial sum and hands it back through its own `JoinHandle`.
pub fn sum_in_threads(nums: Vec<i32>, thread_count: usize) -> i32 {
    todo!(
        "split nums into up to thread_count chunks, sum each chunk on its own spawned thread, \
         join every thread, and add the partial sums into one total"
    )
}

/// Spawns one thread per item in `items`. Each thread calls `predicate` on
/// its own item and, if it returns `true`, increments one count that is
/// shared live across every thread. Returns the final count once every
/// thread has finished.
///
/// An empty `items` returns `0`. Because more than one thread may try to
/// update the same count at the same moment, it needs to be wrapped so it
/// can be safely shared and mutated from several threads at once.
pub fn count_matching_in_threads(items: Vec<i32>, predicate: fn(i32) -> bool) -> i32 {
    todo!(
        "wrap a starting count of 0 so it can be shared and mutated across threads, spawn one \
         thread per item that checks predicate and increments the shared count when it matches, \
         join every thread, then return the final count"
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
}
