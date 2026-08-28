//! Reference solution for 1.3.2 — the rules of the borrow checker.
//!
//! Every one of these takes a `&mut Vec<...>`, so for the length of the call
//! you are the single writer. The work is arranging each body so that you
//! never need a second borrow while the first one is still alive.
//!
//! None of them needs `.clone()` except the one whose specification says a
//! copy of a `String` comes out.

/// Appends everything already in `values` to the end of `values`, in the same
/// order.
///
/// An empty `Vec` is left empty.
///
/// # Examples
///
/// `[1, 2, 3]` becomes `[1, 2, 3, 1, 2, 3]`.
/// `[7]` becomes `[7, 7]`.
pub fn duplicate_in_place(values: &mut Vec<i32>) {
    let original = values.len();
    for index in 0..original {
        let value = values[index];
        values.push(value);
    }
}

/// Moves the first element of `values` to the end.
///
/// Everything else shifts down one position and keeps its order. A `Vec` with
/// fewer than two elements is left exactly as it was.
///
/// # Examples
///
/// `[1, 2, 3]` becomes `[2, 3, 1]`.
/// `[1, 2]` becomes `[2, 1]`.
/// `[9]` stays `[9]`.
pub fn move_first_to_last(values: &mut Vec<i32>) {
    if values.len() < 2 {
        return;
    }
    let front = values.remove(0);
    values.push(front);
}

/// Appends a copy of the longest string in `lines` to the end of `lines`.
///
/// Length is what `len()` reports — bytes, not characters. If two strings are
/// equally long, the earlier one wins. An empty `Vec` is left empty.
///
/// # Examples
///
/// `["ab", "cdef", "gh"]` becomes `["ab", "cdef", "gh", "cdef"]`.
/// `["aa", "bb"]` becomes `["aa", "bb", "aa"]`.
pub fn append_longest(lines: &mut Vec<String>) {
    if lines.is_empty() {
        return;
    }
    let mut best = 0;
    for index in 1..lines.len() {
        if lines[index].len() > lines[best].len() {
            best = index;
        }
    }
    let copy = lines[best].clone();
    lines.push(copy);
}

/// Keeps only the first occurrence of each value in `values`.
///
/// The survivors stay in the order they first appeared. An empty `Vec` is
/// left empty.
///
/// # Examples
///
/// `[3, 1, 3, 2, 1]` becomes `[3, 1, 2]`.
/// `[5, 5, 5]` becomes `[5]`.
pub fn drop_duplicates(values: &mut Vec<i32>) {
    let mut kept: Vec<i32> = Vec::new();
    for value in values.iter() {
        if !kept.contains(value) {
            kept.push(*value);
        }
    }
    *values = kept;
}

/// Adds `bonus` to every element of `scores`, then appends the total of the
/// changed elements to the end of `scores`, and returns that same total.
///
/// If `scores` is empty, nothing is appended and the answer is `0`. `bonus`
/// may be negative.
///
/// # Examples
///
/// `[1, 2, 3]` with a bonus of `10` becomes `[11, 12, 13, 36]`, and `36` is
/// returned.
/// `[10, 20]` with a bonus of `-5` becomes `[5, 15, 20]`, and `20` is
/// returned.
pub fn apply_bonus(scores: &mut Vec<i32>, bonus: i32) -> i32 {
    if scores.is_empty() {
        return 0;
    }
    let mut total = 0;
    for score in scores.iter_mut() {
        *score += bonus;
        total += *score;
    }
    scores.push(total);
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicates_the_whole_vec_in_order() {
        let mut values = vec![1, 2, 3];
        duplicate_in_place(&mut values);
        assert_eq!(values, vec![1, 2, 3, 1, 2, 3]);

        let mut single = vec![7];
        duplicate_in_place(&mut single);
        assert_eq!(single, vec![7, 7]);

        let mut empty: Vec<i32> = Vec::new();
        duplicate_in_place(&mut empty);
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn moves_the_front_element_to_the_back() {
        let mut values = vec![1, 2, 3];
        move_first_to_last(&mut values);
        assert_eq!(values, vec![2, 3, 1]);

        let mut pair = vec![1, 2];
        move_first_to_last(&mut pair);
        assert_eq!(pair, vec![2, 1]);

        let mut longer = vec![1, 2, 3, 4, 5];
        move_first_to_last(&mut longer);
        assert_eq!(longer, vec![2, 3, 4, 5, 1]);
    }

    #[test]
    fn a_short_vec_is_left_alone() {
        let mut single = vec![9];
        move_first_to_last(&mut single);
        assert_eq!(single, vec![9]);

        let mut empty: Vec<i32> = Vec::new();
        move_first_to_last(&mut empty);
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn appends_a_copy_of_the_longest_line() {
        let mut lines = vec![String::from("ab"), String::from("cdef"), String::from("gh")];
        append_longest(&mut lines);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "ab");
        assert_eq!(lines[1], "cdef");
        assert_eq!(lines[2], "gh");
        assert_eq!(lines[3], "cdef");

        let mut empty: Vec<String> = Vec::new();
        append_longest(&mut empty);
        assert_eq!(empty, Vec::<String>::new());
    }

    #[test]
    fn the_earlier_of_two_equal_lines_wins() {
        let mut lines = vec![String::from("aa"), String::from("bb")];
        append_longest(&mut lines);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[2], "aa");
    }

    #[test]
    fn the_appended_line_owns_its_own_buffer() {
        let mut lines = vec![String::from("hello")];
        append_longest(&mut lines);
        lines[1].push('!');
        assert_eq!(lines[0], "hello", "the copy must not share a buffer");
        assert_eq!(lines[1], "hello!");
    }

    #[test]
    fn keeps_the_first_of_each_value() {
        let mut values = vec![3, 1, 3, 2, 1];
        drop_duplicates(&mut values);
        assert_eq!(values, vec![3, 1, 2]);

        let mut all_same = vec![5, 5, 5];
        drop_duplicates(&mut all_same);
        assert_eq!(all_same, vec![5]);

        let mut already_unique = vec![4, 0, -2];
        drop_duplicates(&mut already_unique);
        assert_eq!(already_unique, vec![4, 0, -2]);

        let mut empty: Vec<i32> = Vec::new();
        drop_duplicates(&mut empty);
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn raises_every_score_and_records_the_total() {
        let mut scores = vec![1, 2, 3];
        assert_eq!(apply_bonus(&mut scores, 10), 36);
        assert_eq!(scores, vec![11, 12, 13, 36]);

        let mut penalised = vec![10, 20];
        assert_eq!(apply_bonus(&mut penalised, -5), 20);
        assert_eq!(penalised, vec![5, 15, 20]);
    }

    #[test]
    fn an_empty_score_list_gains_nothing() {
        let mut empty: Vec<i32> = Vec::new();
        assert_eq!(apply_bonus(&mut empty, 10), 0);
        assert_eq!(empty, Vec::<i32>::new());
    }
}
