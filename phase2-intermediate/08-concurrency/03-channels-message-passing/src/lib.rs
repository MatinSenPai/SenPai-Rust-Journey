//! Exercises for 2.8.3 — channels and message passing.
//!
//! Both functions below use `std::sync::mpsc` the way the lesson built it up:
//! spawn threads, hand results back through a channel instead of a shared,
//! locked variable.

use std::sync::mpsc;
use std::thread;

/// Spawns one thread that computes the total of `nums` and sends that single
/// value back through a channel, then returns it.
///
/// # Examples
///
/// `sum_via_channel(vec![1, 2, 3, 4])` returns `10`.
/// `sum_via_channel(vec![])` returns `0`.
pub fn sum_via_channel(nums: Vec<i32>) -> i32 {
    todo!("spawn a thread that computes the total of nums and sends it back through a channel; return the value this thread receives")
}

/// Spawns `worker_count` threads sharing one channel. Worker `i` (counting
/// from `0`) sends every whole number in the range starting at
/// `i * values_per_worker` and running for `values_per_worker` numbers, one
/// `.send()` per number. Collects every number sent by every worker into a
/// single `Vec<i32>`, sorted ascending — sending order across threads is not
/// guaranteed, so the sort is what makes the return value deterministic.
///
/// # Examples
///
/// `collect_from_workers(3, 4)`, sorted, is
/// `[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]` — worker 0 sends `0..4`, worker 1
/// sends `4..8`, worker 2 sends `8..12`.
/// `collect_from_workers(0, 5)` returns an empty `Vec` — no workers, nothing
/// sent.
pub fn collect_from_workers(worker_count: usize, values_per_worker: usize) -> Vec<i32> {
    todo!(
        "spawn worker_count threads, each cloning the Sender; worker i sends every whole number \
         in its own range of values_per_worker numbers; drop the original Sender once every \
         clone has been handed off, then gather everything the Receiver yields into one sorted \
         Vec, and join every thread before returning"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_a_handful_of_numbers() {
        assert_eq!(sum_via_channel(vec![1, 2, 3, 4]), 10);
    }

    #[test]
    fn sums_an_empty_vec_to_zero() {
        assert_eq!(sum_via_channel(vec![]), 0);
    }

    #[test]
    fn collects_every_value_from_every_worker() {
        let mut result = collect_from_workers(3, 4);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn a_single_worker_still_works() {
        let mut result = collect_from_workers(1, 5);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn zero_workers_collects_nothing() {
        assert!(collect_from_workers(0, 5).is_empty());
    }

    #[test]
    fn zero_values_per_worker_collects_nothing() {
        assert!(collect_from_workers(4, 0).is_empty());
    }
}
