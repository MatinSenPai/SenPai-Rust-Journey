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

    let final_count = *counter.lock().unwrap();
    final_count
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
