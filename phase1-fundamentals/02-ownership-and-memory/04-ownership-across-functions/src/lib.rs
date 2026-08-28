//! Exercises for 1.2.4 — Ownership across functions.
//!
//! Four of these take ownership and one only borrows. The interesting part is
//! not writing them — it is noticing what each signature demands of whoever
//! calls it.

/// `base` with `extra` joined onto the end.
///
/// Both arguments are consumed.
///
/// # Examples
///
/// `appended("hello".to_string(), " world".to_string())` returns
/// `"hello world"`.
/// `appended(String::new(), "x".to_string())` returns `"x"`.
pub fn appended(base: String, extra: String) -> String {
    todo!("join the second onto the end of the first and hand the result back")
}

/// Whichever of the two is longer in bytes; `left` when they are equal.
///
/// Both arguments are consumed, and the one you do not return is dropped.
///
/// # Examples
///
/// `longer_of("aaa".to_string(), "b".to_string())` returns `"aaa"`.
/// `longer_of("a".to_string(), "bbb".to_string())` returns `"bbb"`.
/// `longer_of("ab".to_string(), "cd".to_string())` returns `"ab"`.
pub fn longer_of(left: String, right: String) -> String {
    todo!("compare their byte lengths and return the winner")
}

/// The byte length of `text`, together with `text` itself.
///
/// This is the shape a function has to take when it needs to read something
/// and the caller wants to keep it, using only what this module has taught.
/// Compare it with [`length_of`] below.
///
/// # Examples
///
/// `measure_and_return("hello".to_string())` returns `(5, "hello")`.
/// `measure_and_return(String::new())` returns `(0, "")`.
pub fn measure_and_return(text: String) -> (usize, String) {
    todo!("work out the length, then give both the length and the text back")
}

/// The byte length of `text`.
///
/// The same job as [`measure_and_return`], written the way it is written in
/// real code. Notice what the caller has to do differently — and what they no
/// longer have to do at all.
///
/// # Examples
///
/// `length_of("hello")` returns `5`.
/// `length_of("سلام")` returns `8`.
pub fn length_of(text: &str) -> usize {
    todo!("report how many bytes the text takes")
}

/// The first value, taken out, together with everything after it.
///
/// `values` is never empty.
///
/// # Examples
///
/// `split_off_first(vec![1, 2, 3])` returns `(1, [2, 3])`.
/// `split_off_first(vec![9])` returns `(9, [])`.
pub fn split_off_first(values: Vec<i32>) -> (i32, Vec<i32>) {
    todo!("take the first element out of the Vec and return it with the rest")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_two_strings() {
        assert_eq!(
            appended("hello".to_string(), " world".to_string()),
            "hello world"
        );
        assert_eq!(appended(String::new(), "x".to_string()), "x");
        assert_eq!(appended("x".to_string(), String::new()), "x");
    }

    #[test]
    fn picks_the_longer_one() {
        assert_eq!(longer_of("aaa".to_string(), "b".to_string()), "aaa");
        assert_eq!(longer_of("a".to_string(), "bbb".to_string()), "bbb");
        assert_eq!(longer_of("ab".to_string(), "cd".to_string()), "ab");
        // Bytes, not characters: one Persian letter outweighs one Latin one.
        assert_eq!(longer_of("a".to_string(), "س".to_string()), "س");
    }

    #[test]
    fn hands_the_string_back_with_its_length() {
        let (length, text) = measure_and_return("hello".to_string());
        assert_eq!(length, 5);
        assert_eq!(text, "hello");

        let (length, text) = measure_and_return(String::new());
        assert_eq!(length, 0);
        assert_eq!(text, "");
    }

    #[test]
    fn measures_without_taking_anything() {
        // The caller still owns this afterwards, which is the whole point.
        let owned = String::from("hello");
        assert_eq!(length_of(&owned), 5);
        assert_eq!(length_of(&owned), 5);
        assert_eq!(owned, "hello");

        // And a literal works with no conversion at all.
        assert_eq!(length_of("سلام"), 8);
        assert_eq!(length_of(""), 0);
    }

    #[test]
    fn takes_the_first_and_returns_the_rest() {
        assert_eq!(split_off_first(vec![1, 2, 3]), (1, vec![2, 3]));
        assert_eq!(split_off_first(vec![9]), (9, Vec::new()));
        assert_eq!(split_off_first(vec![-1, 0]), (-1, vec![0]));
    }
}
