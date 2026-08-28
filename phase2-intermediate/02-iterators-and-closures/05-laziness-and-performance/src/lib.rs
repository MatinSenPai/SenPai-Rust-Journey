//! Exercises for 2.2.5 — Laziness and iterator performance.
//!
//! Every function here is naturally suited to an unbounded source plus a
//! stopping condition (`.take()`, `.take_while()`, or a hand-written loop
//! that breaks) — the same shape as "The concept". No generics, no `Box`,
//! no `impl Trait` return types: everything works on the concrete types
//! this module has used all along.

/// Returns the first `n` powers of two, starting at `2^0 = 1`, in
/// increasing order: `powers_of_two(5)` is `[1, 2, 4, 8, 16]`.
///
/// If `n` is `0`, returns an empty `Vec`.
///
/// # Examples
///
/// `powers_of_two(0)` is `[]`. `powers_of_two(1)` is `[1]`.
pub fn powers_of_two(n: usize) -> Vec<u64> {
    todo!(
        "generate the sequence 1, 2, 4, 8, ... by doubling the previous value each step, take \
         the first `n` of them, and gather them into the Vec you return"
    )
}

/// Repeats `pattern` end-to-end as many times as needed to produce a `Vec`
/// of exactly `total_len` elements — the final repetition may be cut short.
///
/// If `pattern` is empty, returns an empty `Vec` regardless of `total_len`.
///
/// # Examples
///
/// `cycle_to_length(&[1, 2, 3], 7)` is `[1, 2, 3, 1, 2, 3, 1]`.
/// `cycle_to_length(&[9], 4)` is `[9, 9, 9, 9]`.
pub fn cycle_to_length(pattern: &[i32], total_len: usize) -> Vec<i32> {
    todo!(
        "repeat `pattern` from its start as many times as it takes to produce exactly \
         `total_len` values, cutting the final repetition short if needed, and gather the \
         result into the Vec you return"
    )
}

/// Returns the first `n` positive multiples of `k`, in increasing order,
/// starting with `k` itself: `first_n_multiples_of(3, 4)` is
/// `[3, 6, 9, 12]`.
///
/// If `n` is `0`, returns an empty `Vec`. `k` is never `0` in the inputs
/// this function is called with.
pub fn first_n_multiples_of(k: u32, n: usize) -> Vec<u32> {
    todo!(
        "produce the positive multiples of `k` in increasing order — `k`, then `k * 2`, then \
         `k * 3`, and so on — stop after the first `n` of them, and gather the result into the \
         Vec you return"
    )
}

/// Returns the positive multiples of `k`, in increasing order, that are
/// strictly less than `limit`: `multiples_of_k_below(3, 10)` is
/// `[3, 6, 9]`.
///
/// Returns an empty `Vec` when `k` is `0` (there is no well-defined
/// sequence of "multiples of zero" to stop), and also whenever the very
/// first multiple (`k` itself) is already not less than `limit`.
///
/// # Examples
///
/// `multiples_of_k_below(4, 4)` is `[]`. `multiples_of_k_below(1, 5)` is
/// `[1, 2, 3, 4]`.
pub fn multiples_of_k_below(k: u32, limit: u32) -> Vec<u32> {
    todo!(
        "produce the positive multiples of `k` in increasing order and keep only the ones that \
         stay strictly below `limit`, stopping as soon as one does not; handle `k` equal to 0 \
         as its own case first, since there is no stopping point to find otherwise"
    )
}

/// Scans `numbers` from the start and collects the first `n` even numbers
/// it finds, in order, together with how many elements of `numbers` were
/// looked at to produce them — one more than the index of the last element
/// examined.
///
/// If `numbers` runs out before `n` even numbers are found, the returned
/// `Vec` holds fewer than `n` elements and the count equals
/// `numbers.len()`. If `n` is `0`, returns `(vec![], 0)` without looking at
/// any element.
///
/// # Examples
///
/// `first_n_even_with_scan_count(&[1, 3, 4, 5, 6, 7], 2)` is
/// `(vec![4, 6], 5)`: it has to look at `1, 3, 4, 5, 6` (5 elements) before
/// it has found two even numbers.
pub fn first_n_even_with_scan_count(numbers: &[i32], n: usize) -> (Vec<i32>, usize) {
    todo!(
        "walk `numbers` from the start, stopping the moment you have found `n` even values; \
         return the even values you found together with how many elements you had to look at \
         to find them"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powers_of_two_of_zero_is_empty() {
        assert_eq!(powers_of_two(0), Vec::<u64>::new());
    }

    #[test]
    fn powers_of_two_starts_at_one() {
        assert_eq!(powers_of_two(1), vec![1]);
    }

    #[test]
    fn powers_of_two_doubles_each_step() {
        assert_eq!(powers_of_two(5), vec![1, 2, 4, 8, 16]);
    }

    #[test]
    fn cycle_to_length_wraps_and_cuts_short() {
        assert_eq!(cycle_to_length(&[1, 2, 3], 7), vec![1, 2, 3, 1, 2, 3, 1]);
    }

    #[test]
    fn cycle_to_length_of_zero_is_empty() {
        assert_eq!(cycle_to_length(&[1, 2, 3], 0), Vec::<i32>::new());
    }

    #[test]
    fn cycle_to_length_of_empty_pattern_is_empty() {
        assert_eq!(cycle_to_length(&[], 5), Vec::<i32>::new());
    }

    #[test]
    fn cycle_to_length_single_element_pattern_repeats() {
        assert_eq!(cycle_to_length(&[9], 4), vec![9, 9, 9, 9]);
    }

    #[test]
    fn first_n_multiples_of_three_starts_at_three() {
        assert_eq!(first_n_multiples_of(3, 4), vec![3, 6, 9, 12]);
    }

    #[test]
    fn first_n_multiples_of_with_n_one() {
        assert_eq!(first_n_multiples_of(5, 1), vec![5]);
    }

    #[test]
    fn first_n_multiples_of_with_n_zero_is_empty() {
        assert_eq!(first_n_multiples_of(7, 0), Vec::<u32>::new());
    }

    #[test]
    fn multiples_of_k_below_ordinary_case() {
        assert_eq!(multiples_of_k_below(3, 10), vec![3, 6, 9]);
    }

    #[test]
    fn multiples_of_k_below_where_k_equals_limit_is_empty() {
        assert_eq!(multiples_of_k_below(4, 4), Vec::<u32>::new());
    }

    #[test]
    fn multiples_of_k_below_with_k_one() {
        assert_eq!(multiples_of_k_below(1, 5), vec![1, 2, 3, 4]);
    }

    #[test]
    fn multiples_of_k_below_with_k_zero_is_empty() {
        assert_eq!(multiples_of_k_below(0, 100), Vec::<u32>::new());
    }

    #[test]
    fn multiples_of_k_below_with_limit_below_k_is_empty() {
        assert_eq!(multiples_of_k_below(5, 1), Vec::<u32>::new());
    }

    #[test]
    fn first_n_even_with_scan_count_finds_two() {
        assert_eq!(
            first_n_even_with_scan_count(&[1, 3, 4, 5, 6, 7], 2),
            (vec![4, 6], 5)
        );
    }

    #[test]
    fn first_n_even_with_scan_count_runs_out_early() {
        assert_eq!(
            first_n_even_with_scan_count(&[2, 4, 6], 5),
            (vec![2, 4, 6], 3)
        );
    }

    #[test]
    fn first_n_even_with_scan_count_finds_none() {
        assert_eq!(first_n_even_with_scan_count(&[1, 3, 5], 1), (vec![], 3));
    }

    #[test]
    fn first_n_even_with_scan_count_of_empty_slice() {
        assert_eq!(first_n_even_with_scan_count(&[], 3), (vec![], 0));
    }

    #[test]
    fn first_n_even_with_scan_count_of_n_zero_scans_nothing() {
        assert_eq!(first_n_even_with_scan_count(&[2, 4, 6, 8], 0), (vec![], 0));
    }
}
