//! Exercises for 1.1.6 — `Vec` and `String` basics.
//!
//! Both types are owned, growable buffers on the heap. These exercises build
//! them up, read them back, and make the byte-versus-character distinction
//! impossible to skip past.

/// Every even number from 0 up to and including `n`.
///
/// # Examples
///
/// `evens_up_to(10)` returns `[0, 2, 4, 6, 8, 10]`.
/// `evens_up_to(1)` returns `[0]`.
/// `evens_up_to(0)` returns `[0]`.
pub fn evens_up_to(n: u32) -> Vec<u32> {
    todo!("start with an empty Vec and put each even number on the end")
}

/// Every value added together.
///
/// An empty `values` totals `0`.
///
/// # Examples
///
/// `total(vec![1, 2, 3])` returns `6`.
/// `total(vec![-5, 5])` returns `0`.
/// `total(vec![])` returns `0`.
pub fn total(values: Vec<i64>) -> i64 {
    todo!("visit every value and keep a running sum")
}

/// The largest value.
///
/// `values` is never empty.
///
/// # Examples
///
/// `largest(vec![3, 9, 2])` returns `9`.
/// `largest(vec![3])` returns `3`.
/// `largest(vec![-3, -9])` returns `-3`.
pub fn largest(values: Vec<i32>) -> i32 {
    todo!("keep the best one seen so far as you go through them")
}

/// Every part run together, with `separator` between neighbours.
///
/// There is no separator before the first part or after the last one, so
/// three parts produce two separators. An empty `parts` produces an empty
/// `String`.
///
/// # Examples
///
/// `joined(vec!["a".to_string(), "b".to_string()], '-')` returns `"a-b"`.
/// `joined(vec!["only".to_string()], '-')` returns `"only"`.
/// `joined(vec![], '-')` returns `""`.
pub fn joined(parts: Vec<String>, separator: char) -> String {
    todo!("build up one String, adding the separator only between parts")
}

/// How many bytes `text` occupies, and how many characters it contains.
///
/// For English text the two are usually the same. For Persian they are not,
/// and that is the point of this exercise.
///
/// # Examples
///
/// `byte_and_char_count("hello".to_string())` returns `(5, 5)`.
/// `byte_and_char_count("سلام".to_string())` returns `(8, 4)`.
/// `byte_and_char_count("".to_string())` returns `(0, 0)`.
pub fn byte_and_char_count(text: String) -> (usize, usize) {
    todo!("ask the text for its size in bytes, then count its characters")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_vec_of_evens() {
        assert_eq!(evens_up_to(10), vec![0, 2, 4, 6, 8, 10]);
        assert_eq!(evens_up_to(9), vec![0, 2, 4, 6, 8]);
        assert_eq!(evens_up_to(1), vec![0]);
        assert_eq!(evens_up_to(0), vec![0]);
    }

    #[test]
    fn adds_up_a_vec() {
        assert_eq!(total(vec![1, 2, 3]), 6);
        assert_eq!(total(vec![-5, 5]), 0);
        assert_eq!(total(vec![]), 0);
        assert_eq!(total(vec![1_000_000_000_000]), 1_000_000_000_000);
    }

    #[test]
    fn finds_the_largest() {
        assert_eq!(largest(vec![3, 9, 2]), 9);
        assert_eq!(largest(vec![3]), 3);
        assert_eq!(largest(vec![-3, -9]), -3);
        assert_eq!(largest(vec![5, 5, 5]), 5);
    }

    #[test]
    fn joins_with_a_separator_between() {
        assert_eq!(
            joined(vec!["a".to_string(), "b".to_string(), "c".to_string()], '-'),
            "a-b-c"
        );
        assert_eq!(joined(vec!["only".to_string()], '-'), "only");
        assert_eq!(joined(vec![], '-'), "");
        assert_eq!(
            joined(vec!["alpha".to_string(), "beta".to_string()], ' '),
            "alpha beta"
        );
    }

    #[test]
    fn counts_bytes_and_characters_separately() {
        assert_eq!(byte_and_char_count("hello".to_string()), (5, 5));
        assert_eq!(byte_and_char_count("".to_string()), (0, 0));
        // Four Persian letters, two bytes each.
        assert_eq!(byte_and_char_count("سلام".to_string()), (8, 4));
        // Mixed: one ASCII byte, one two-byte letter, one ASCII byte.
        assert_eq!(byte_and_char_count("aسb".to_string()), (4, 3));
    }
}
