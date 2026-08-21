//! Exercises for 1.3.4 — Slices.
//!
//! Every one of these takes a view rather than a collection, and three of
//! them hand a view back. None of them allocates anything.

/// How many elements of `values` are strictly greater than `threshold`.
///
/// "Strictly" means an element equal to `threshold` does not count.
///
/// # Examples
///
/// `count_above(&[1, 5, 3, 9], 3)` returns `2` — the 5 and the 9.
/// `count_above(&[1, 2, 3], 3)` returns `0`.
/// `count_above(&[], 0)` returns `0`.
pub fn count_above(values: &[i32], threshold: i32) -> usize {
    todo!("count the elements bigger than `threshold`")
}

/// Every element of `values` except the first and the last, as a view.
///
/// Nothing is copied — the answer points into the caller's own numbers.
/// When `values` has fewer than three elements there is no middle, and the
/// answer is the empty slice.
///
/// # Examples
///
/// `middle(&[1, 2, 3, 4, 5])` returns `[2, 3, 4]`.
/// `middle(&[1, 2, 3])` returns `[2]`.
/// `middle(&[1, 2])` returns `[]`.
/// `middle(&[7])` returns `[]`.
/// `middle(&[])` returns `[]`.
pub fn middle(values: &[i32]) -> &[i32] {
    todo!("a view of everything but the two ends, and the empty view when there is no middle")
}

/// `values` cut into two views at its midpoint.
///
/// The two answers together cover the whole of `values`, in order, and
/// neither of them copies anything. When the length is odd, the extra
/// element belongs to the second half.
///
/// # Examples
///
/// `halves(&[1, 2, 3, 4])` returns `([1, 2], [3, 4])`.
/// `halves(&[1, 2, 3])` returns `([1], [2, 3])`.
/// `halves(&[9])` returns `([], [9])`.
/// `halves(&[])` returns `([], [])`.
pub fn halves(values: &[i32]) -> (&[i32], &[i32]) {
    todo!("two views of `values`, cut at the middle")
}

/// The sum of `length` elements of `values`, starting at index `start`.
///
/// A `length` of `0` sums nothing and gives `0`.
///
/// A window running past the end of `values` is a mistake in the caller,
/// and this function panics on it. You do not have to write that check
/// yourself: taking the slice already does it.
///
/// # Examples
///
/// `window_sum(&[10, 20, 30, 40], 1, 2)` returns `50`.
/// `window_sum(&[10, 20, 30, 40], 0, 4)` returns `100`.
/// `window_sum(&[10, 20, 30, 40], 2, 0)` returns `0`.
/// `window_sum(&[10, 20, 30], 2, 5)` panics.
pub fn window_sum(values: &[i32], start: usize, length: usize) -> i32 {
    todo!("add up the window of `length` elements that begins at `start`")
}

/// Doubles every element of `values`, in place.
///
/// Nothing is returned and nothing is allocated: the caller's own numbers
/// change. An empty slice is left alone.
///
/// # Examples
///
/// After `let mut v = vec![1, 2, 3]; double_in_place(&mut v);` the Vec
/// holds `[2, 4, 6]`.
/// After the same call on an empty Vec, it is still empty.
pub fn double_in_place(values: &mut [i32]) {
    todo!("replace every element with twice itself, writing through the view")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_only_what_is_strictly_bigger() {
        assert_eq!(count_above(&[1, 5, 3, 9], 3), 2);
        assert_eq!(count_above(&[1, 2, 3], 3), 0);
        assert_eq!(count_above(&[], 0), 0);
        assert_eq!(count_above(&[-5, 0, 5], -1), 2);
    }

    #[test]
    fn one_function_takes_an_array_a_vec_and_a_piece_of_either() {
        let fixed = [1, 5, 3, 9];
        let grown = vec![1, 5, 3, 9];

        assert_eq!(count_above(&fixed, 3), 2);
        assert_eq!(count_above(&grown, 3), 2);
        assert_eq!(count_above(&grown[..2], 3), 1);
        assert_eq!(count_above(&fixed[2..], 3), 1);
    }

    #[test]
    fn the_middle_is_everything_but_the_ends() {
        assert_eq!(middle(&[1, 2, 3, 4, 5]), &[2, 3, 4]);
        assert_eq!(middle(&[1, 2, 3]), &[2]);
        assert_eq!(middle(&[1, 2]), &[] as &[i32]);
        assert_eq!(middle(&[7]), &[] as &[i32]);
        assert_eq!(middle(&[]), &[] as &[i32]);
    }

    #[test]
    fn the_middle_is_a_view_of_the_original() {
        let values = vec![1, 2, 3, 4, 5];
        let view = middle(&values);
        assert!(std::ptr::eq(&view[0], &values[1]));
    }

    #[test]
    fn halves_cover_everything_and_the_odd_one_goes_right() {
        assert_eq!(halves(&[1, 2, 3, 4]), (&[1, 2][..], &[3, 4][..]));
        assert_eq!(halves(&[1, 2, 3]), (&[1][..], &[2, 3][..]));
        assert_eq!(halves(&[9]), (&[][..], &[9][..]));
        assert_eq!(halves(&[]), (&[][..], &[][..]));
    }

    #[test]
    fn sums_the_window_that_was_asked_for() {
        assert_eq!(window_sum(&[10, 20, 30, 40], 1, 2), 50);
        assert_eq!(window_sum(&[10, 20, 30, 40], 0, 4), 100);
        assert_eq!(window_sum(&[10, 20, 30, 40], 2, 0), 0);
        assert_eq!(window_sum(&[10, 20, 30, 40], 4, 0), 0);
    }

    #[test]
    #[should_panic]
    fn a_window_past_the_end_panics() {
        window_sum(&[10, 20, 30], 2, 5);
    }

    #[test]
    fn doubling_changes_the_callers_numbers() {
        let mut values = vec![1, 2, 3];
        double_in_place(&mut values);
        assert_eq!(values, vec![2, 4, 6]);

        let mut empty: Vec<i32> = Vec::new();
        double_in_place(&mut empty);
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn doubling_a_window_leaves_the_rest_alone() {
        let mut values = vec![1, 2, 3, 4, 5];
        double_in_place(&mut values[1..4]);
        assert_eq!(values, vec![1, 4, 6, 8, 5]);
    }
}
