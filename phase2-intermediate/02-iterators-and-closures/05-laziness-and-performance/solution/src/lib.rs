//! Solution for 2.2.5 — Laziness and iterator performance.

/// Returns the first `n` powers of two, starting at `2^0 = 1`.
pub fn powers_of_two(n: usize) -> Vec<u64> {
    std::iter::successors(Some(1u64), |&x| Some(x * 2))
        .take(n)
        .collect()
}

/// Repeats `pattern` end-to-end to produce a `Vec` of exactly `total_len`
/// elements, cutting the final repetition short if needed.
pub fn cycle_to_length(pattern: &[i32], total_len: usize) -> Vec<i32> {
    pattern.iter().copied().cycle().take(total_len).collect()
}

/// Returns the first `n` positive multiples of `k`, starting with `k`
/// itself.
pub fn first_n_multiples_of(k: u32, n: usize) -> Vec<u32> {
    (1..).map(|i| i * k).take(n).collect()
}

/// Returns the positive multiples of `k` that are strictly less than
/// `limit`.
pub fn multiples_of_k_below(k: u32, limit: u32) -> Vec<u32> {
    if k == 0 {
        return Vec::new();
    }
    (1..).map(|i| i * k).take_while(|&m| m < limit).collect()
}

/// Scans `numbers` from the start for the first `n` even values, returning
/// them together with how many elements were looked at to find them.
pub fn first_n_even_with_scan_count(numbers: &[i32], n: usize) -> (Vec<i32>, usize) {
    let mut found = Vec::new();
    let mut scanned = 0;
    for &value in numbers {
        if found.len() >= n {
            break;
        }
        scanned += 1;
        if value % 2 == 0 {
            found.push(value);
        }
    }
    (found, scanned)
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
