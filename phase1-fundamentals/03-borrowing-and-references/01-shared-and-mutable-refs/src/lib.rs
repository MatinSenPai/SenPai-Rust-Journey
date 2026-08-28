//! Exercises for 1.3.1 — Shared and mutable references.
//!
//! Every signature here already tells you what to do. Two of them only look;
//! three of them write. None of them needs `.clone()`, and none of them
//! returns a value just to give it back.

/// The byte length of the longest string in `lines`.
///
/// `lines` is only read, so the caller still owns it afterwards. An empty
/// `lines` gives `0`.
///
/// # Examples
///
/// `longest_length(&vec!["a".to_string(), "abcd".to_string()])` is `4`.
/// `longest_length(&Vec::new())` is `0`.
pub fn longest_length(lines: &Vec<String>) -> usize {
    todo!("look at every string in `lines` and answer with the biggest byte length you saw")
}

/// How many numbers in `values` are **strictly greater than** `limit`.
///
/// Equal does not count. Both arguments are borrowed, so the caller keeps
/// both.
///
/// # Examples
///
/// `count_above(&vec![1, 5, 9], &5)` is `1` — only the `9`.
/// `count_above(&vec![1, 2], &10)` is `0`.
pub fn count_above(values: &Vec<i32>, limit: &i32) -> usize {
    todo!("count the numbers in `values` that sit strictly above `limit`")
}

/// Appends every string in `extras` to the end of `target`, in order, with
/// nothing inserted between them.
///
/// Returns nothing: `target` is the caller's own `String` and it is what
/// changes. `extras` is only read and comes back untouched.
///
/// # Examples
///
/// With `target` holding `"ab"` and `extras` holding `["c", "de"]`, `target`
/// ends up holding `"abcde"`.
/// An empty `extras` leaves `target` exactly as it was.
pub fn append_all(target: &mut String, extras: &Vec<String>) {
    todo!("add each of `extras` onto the end of `target`, keeping their order")
}

/// Lowers every number in `values` that is above `ceiling` down to
/// `ceiling`. Numbers at or below `ceiling` are left alone.
///
/// Returns nothing: the caller's own `Vec` is what changes, and its length
/// stays the same.
///
/// # Examples
///
/// `values` holding `[1, 7, 5, 9]` with `ceiling` of `5` becomes
/// `[1, 5, 5, 5]`.
pub fn clamp_all(values: &mut Vec<i32>, ceiling: &i32) {
    todo!("bring every number that sits above `ceiling` down to `ceiling`, in place")
}

/// Moves `amount` from `from` to `to`.
///
/// Afterwards `from` is smaller by `amount` and `to` is larger by `amount`.
/// Nothing is checked and nothing is refused: a negative `amount` simply
/// moves the other way, and `from` is allowed to go below zero.
///
/// # Examples
///
/// With `from` at `10`, `to` at `0` and `amount` of `3`, they end up at `7`
/// and `3`.
pub fn transfer(from: &mut i32, to: &mut i32, amount: &i32) {
    todo!("take `amount` out of `from` and put the same `amount` into `to`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_longest_and_leaves_the_vec_alone() {
        let lines = vec!["a".to_string(), "abcd".to_string(), "ab".to_string()];
        assert_eq!(longest_length(&lines), 4);
        // The caller still owns it — that is the whole point of `&`.
        assert_eq!(lines.len(), 3);

        assert_eq!(longest_length(&Vec::new()), 0);
        assert_eq!(longest_length(&vec!["only".to_string()]), 4);
    }

    #[test]
    fn counts_strictly_above_the_limit() {
        assert_eq!(count_above(&vec![1, 5, 9], &5), 1);
        assert_eq!(count_above(&vec![1, 2], &10), 0);
        assert_eq!(count_above(&vec![7, 7, 7], &6), 3);
        assert_eq!(count_above(&Vec::new(), &0), 0);

        let values = vec![-3, 0, 3];
        let limit = -1;
        assert_eq!(count_above(&values, &limit), 2);
        assert_eq!(values.len(), 3);
        assert_eq!(limit, -1);
    }

    #[test]
    fn appends_in_place_and_keeps_the_order() {
        let mut target = String::from("ab");
        let extras = vec!["c".to_string(), "de".to_string()];
        append_all(&mut target, &extras);
        assert_eq!(target, "abcde");
        assert_eq!(extras.len(), 2, "`extras` is only read");

        let mut untouched = String::from("same");
        append_all(&mut untouched, &Vec::new());
        assert_eq!(untouched, "same");
    }

    #[test]
    fn clamps_only_what_is_above_the_ceiling() {
        let mut values = vec![1, 7, 5, 9];
        clamp_all(&mut values, &5);
        assert_eq!(values, vec![1, 5, 5, 5]);

        let mut already_low = vec![-2, 0, 1];
        clamp_all(&mut already_low, &1);
        assert_eq!(already_low, vec![-2, 0, 1]);

        let mut empty: Vec<i32> = Vec::new();
        clamp_all(&mut empty, &3);
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn moves_the_amount_across() {
        let mut from = 10;
        let mut to = 0;
        transfer(&mut from, &mut to, &3);
        assert_eq!(from, 7);
        assert_eq!(to, 3);

        transfer(&mut from, &mut to, &7);
        assert_eq!(from, 0);
        assert_eq!(to, 10);

        // Nothing is refused: `from` may go below zero.
        transfer(&mut from, &mut to, &5);
        assert_eq!(from, -5);
        assert_eq!(to, 15);
    }
}
