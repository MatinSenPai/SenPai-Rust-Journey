//! Reference solution for 2.7.5 — benchmarking with `criterion`.

use std::collections::HashSet;

/// Returns `true` if `needle` appears anywhere in `haystack`, checking one
/// element at a time from the start. O(n) in the length of `haystack`.
#[allow(clippy::manual_contains)] // the point of this lesson: contrast this scan against contains_hashset's O(1)
pub fn contains_linear(haystack: &[u32], needle: u32) -> bool {
    haystack.iter().any(|&item| item == needle)
}

/// Returns `true` if `needle` is a member of `haystack`. O(1) on average, no
/// matter how many elements `haystack` holds.
pub fn contains_hashset(haystack: &HashSet<u32>, needle: u32) -> bool {
    haystack.contains(&needle)
}

/// The sum of the squares of every integer from `1` to `n` inclusive
/// (`0` when `n == 0`), computed with a hand-written `for` loop.
pub fn sum_of_squares_loop(n: u32) -> u64 {
    let mut total: u64 = 0;
    for i in 1..=n {
        total += (i as u64) * (i as u64);
    }
    total
}

/// The same total as [`sum_of_squares_loop`], computed with an iterator
/// chain instead of a hand-written loop.
pub fn sum_of_squares_iter(n: u32) -> u64 {
    (1..=n).map(|i| (i as u64) * (i as u64)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_linear_finds_present_values_and_rejects_absent_ones() {
        assert!(contains_linear(&[1, 2, 3], 2));
        assert!(!contains_linear(&[1, 2, 3], 9));
        assert!(!contains_linear(&[], 1));
    }

    #[test]
    fn contains_hashset_finds_present_values_and_rejects_absent_ones() {
        let set: HashSet<u32> = [1, 2, 3].into_iter().collect();
        assert!(contains_hashset(&set, 2));
        assert!(!contains_hashset(&set, 9));
    }

    #[test]
    fn sum_of_squares_loop_matches_known_totals() {
        assert_eq!(sum_of_squares_loop(0), 0);
        assert_eq!(sum_of_squares_loop(1), 1);
        assert_eq!(sum_of_squares_loop(3), 14);
        assert_eq!(sum_of_squares_loop(10), 385);
    }

    #[test]
    fn sum_of_squares_iter_matches_known_totals() {
        assert_eq!(sum_of_squares_iter(0), 0);
        assert_eq!(sum_of_squares_iter(1), 1);
        assert_eq!(sum_of_squares_iter(3), 14);
        assert_eq!(sum_of_squares_iter(10), 385);
    }

    #[test]
    fn both_implementations_agree_across_a_range() {
        for n in 0..50u32 {
            assert_eq!(
                sum_of_squares_loop(n),
                sum_of_squares_iter(n),
                "disagreement at n = {n}"
            );
        }
    }
}
