use std::sync::mpsc;
use std::thread;

/// Spawns one thread that sums `nums` and sends the single result back
/// through a channel — an alternative to reading a `JoinHandle`'s return
/// value, useful once you have more than one kind of message a thread
/// might send back over its lifetime (this exercise only sends one).
pub fn compute_async_sum(nums: Vec<i32>) -> i32 {
    todo!(
        "let (tx, rx) = mpsc::channel(); spawn a thread that tx.send(nums.iter().sum()); rx.recv().unwrap()"
    )
}

/// Spawns `producer_count` threads, each sending its own unique range of
/// `values_per_producer` numbers through a shared channel. Collects every
/// value sent by every producer, sorted (sending order across threads
/// isn't deterministic, so sorting is what makes this testable).
pub fn collect_from_producers(producer_count: usize, values_per_producer: usize) -> Vec<i32> {
    todo!(
        "mpsc::channel(); per producer i, clone tx, spawn a thread sending (i*values_per_producer)..(i*values_per_producer + values_per_producer); \
         drop the original tx; collect from `for v in rx`; join every handle; sort and return"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_sum_via_channel() {
        assert_eq!(compute_async_sum(vec![1, 2, 3, 4]), 10);
    }

    #[test]
    fn computes_sum_of_empty_vec() {
        assert_eq!(compute_async_sum(vec![]), 0);
    }

    #[test]
    fn collects_every_value_from_every_producer() {
        let mut result = collect_from_producers(3, 4);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn single_producer_still_works() {
        let mut result = collect_from_producers(1, 5);
        result.sort();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }
}
