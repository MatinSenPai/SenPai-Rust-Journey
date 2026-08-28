//! Reference solution for 1.2.2 — Move semantics.
//!
//! Every function here takes ownership of what it is given and hands back
//! something new. That shape is not an accident: it is what code looks like
//! when values move, and getting comfortable with it now saves a lot of
//! arguing with the compiler later.

/// `text` with `suffix` added on the end.
///
/// The caller gives up `text` and gets the finished string back.
///
/// # Examples
///
/// `extended("hello".to_string(), '!')` returns `"hello!"`.
/// `extended(String::new(), 'x')` returns `"x"`.
pub fn extended(mut text: String, suffix: char) -> String {
    text.push(suffix);
    text
}

/// `values` in the opposite order.
///
/// # Examples
///
/// `reversed(vec![1, 2, 3])` returns `[3, 2, 1]`.
/// `reversed(vec![])` returns `[]`.
/// `reversed(vec![7])` returns `[7]`.
pub fn reversed(values: Vec<i32>) -> Vec<i32> {
    let mut out = Vec::with_capacity(values.len());
    for offset in 0..values.len() {
        out.push(values[values.len() - 1 - offset]);
    }
    out
}

/// The total number of bytes across every string in `values`.
///
/// Bytes, not characters — the distinction from 1.1.6.
///
/// # Examples
///
/// `total_bytes_of(vec!["ab".to_string(), "c".to_string()])` returns `3`.
/// `total_bytes_of(vec!["سلام".to_string()])` returns `8`.
/// `total_bytes_of(vec![])` returns `0`.
pub fn total_bytes_of(values: Vec<String>) -> usize {
    let mut bytes = 0;
    for value in &values {
        bytes += value.len();
    }
    bytes
}

/// The first string, taken out of `values` and handed back on its own.
///
/// `values` is never empty. The rest of it is discarded.
///
/// Note that `values[0]` on its own will not compile — a `Vec` cannot be left
/// with a hole in it. You need the method that removes an element *and* gives
/// it to you.
///
/// # Examples
///
/// `take_first(vec!["a".to_string(), "b".to_string()])` returns `"a"`.
pub fn take_first(mut values: Vec<String>) -> String {
    values.remove(0)
}

/// `left` and `right` run together into one `Vec`, `left` first.
///
/// Both arguments are consumed.
///
/// # Examples
///
/// `merged(vec![1, 2], vec![3])` returns `[1, 2, 3]`.
/// `merged(vec![], vec![1])` returns `[1]`.
/// `merged(vec![], vec![])` returns `[]`.
pub fn merged(left: Vec<i32>, right: Vec<i32>) -> Vec<i32> {
    let mut out = left;
    for value in right {
        out.push(value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_a_character_to_the_end() {
        assert_eq!(extended("hello".to_string(), '!'), "hello!");
        assert_eq!(extended(String::new(), 'x'), "x");
        assert_eq!(extended("سلام".to_string(), '!'), "سلام!");
    }

    #[test]
    fn turns_a_vec_round() {
        assert_eq!(reversed(vec![1, 2, 3]), vec![3, 2, 1]);
        assert_eq!(reversed(vec![7]), vec![7]);
        assert_eq!(reversed(vec![]), Vec::<i32>::new());
        assert_eq!(reversed(vec![1, 2, 3, 4]), vec![4, 3, 2, 1]);
    }

    #[test]
    fn adds_up_byte_lengths() {
        assert_eq!(total_bytes_of(vec!["ab".to_string(), "c".to_string()]), 3);
        assert_eq!(total_bytes_of(vec!["سلام".to_string()]), 8);
        assert_eq!(total_bytes_of(vec![]), 0);
    }

    #[test]
    fn takes_the_first_one_out() {
        assert_eq!(take_first(vec!["a".to_string(), "b".to_string()]), "a");
        assert_eq!(take_first(vec!["only".to_string()]), "only");
    }

    #[test]
    fn runs_two_vecs_together() {
        assert_eq!(merged(vec![1, 2], vec![3]), vec![1, 2, 3]);
        assert_eq!(merged(vec![], vec![1]), vec![1]);
        assert_eq!(merged(vec![1], vec![]), vec![1]);
        assert_eq!(merged(vec![], vec![]), Vec::<i32>::new());
    }
}
