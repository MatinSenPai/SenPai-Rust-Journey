//! Exercises for 1.4.2 — UTF-8: bytes, chars, graphemes.
//!
//! Every one of these is a question about the same text asked at a different
//! level: bytes, scalars, or the boundary between them. Run the tests with
//!
//!     cargo test -p p1-04-02-utf8-bytes-chars-graphemes

/// The byte length of `text` and the number of Unicode scalars in it, in that
/// order.
///
/// # Examples
///
/// `counts("hello")` returns `(5, 5)`.
/// `counts("سلام")` returns `(8, 4)`.
/// `counts("من 🌸")` returns `(9, 4)`.
/// `counts("")` returns `(0, 0)`.
pub fn counts(text: &str) -> (usize, usize) {
    todo!("return the byte length and the number of Unicode scalars, in that order")
}

/// How many bytes UTF-8 needs to store the single scalar `letter`.
///
/// The answer is always 1, 2, 3 or 4.
///
/// # Examples
///
/// `bytes_for('a')` returns `1`.
/// `bytes_for('س')` returns `2`.
/// `bytes_for('€')` returns `3`.
/// `bytes_for('🌸')` returns `4`.
pub fn bytes_for(letter: char) -> usize {
    todo!("return how many bytes UTF-8 needs for this one scalar")
}

/// The byte width of the widest single scalar in `text`.
///
/// Empty text has no scalars, so the answer for it is `0`.
///
/// # Examples
///
/// `bytes_of_widest("hello")` returns `1`.
/// `bytes_of_widest("سلام")` returns `2`.
/// `bytes_of_widest("Rust برای بک‌اند")` returns `3` — the ZWNJ is three bytes.
/// `bytes_of_widest("سلام 🌸")` returns `4`.
/// `bytes_of_widest("")` returns `0`.
pub fn bytes_of_widest(text: &str) -> usize {
    todo!("return the byte width of the widest scalar in the text")
}

/// How many bytes the first `n` scalars of `text` occupy.
///
/// This is the byte offset at which scalar number `n` starts. When `text` has
/// fewer than `n` scalars, the answer is the whole byte length of `text`.
///
/// # Examples
///
/// `bytes_of_first("hello", 2)` returns `2`.
/// `bytes_of_first("سلام", 2)` returns `4`.
/// `bytes_of_first("من 🌸", 3)` returns `5`.
/// `bytes_of_first("سلام", 0)` returns `0`.
/// `bytes_of_first("سلام", 9)` returns `8`.
pub fn bytes_of_first(text: &str, n: usize) -> usize {
    todo!("return how many bytes the first n scalars occupy")
}

/// How many bytes of `text` are UTF-8 **continuation bytes**.
///
/// A continuation byte is any byte whose value is between `0x80` and `0xBF`
/// inclusive: the second, third or fourth byte of a multi-byte scalar. The
/// first byte of every scalar is never one.
///
/// # Examples
///
/// `continuation_bytes("hello")` returns `0`.
/// `continuation_bytes("سلام")` returns `4`.
/// `continuation_bytes("🌸")` returns `3`.
/// `continuation_bytes("من 🌸")` returns `5`.
/// `continuation_bytes("")` returns `0`.
pub fn continuation_bytes(text: &str) -> usize {
    todo!("count the bytes that are continuation bytes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_bytes_and_scalars_separately() {
        assert_eq!(counts("hello"), (5, 5));
        assert_eq!(counts("Rust"), (4, 4));
        assert_eq!(counts("سلام"), (8, 4));
        assert_eq!(counts("متین"), (8, 4));
        assert_eq!(counts("سلام، من متین هستم."), (34, 19));
        assert_eq!(counts("Rust برای بک‌اند"), (27, 16));
        assert_eq!(counts("من 🌸"), (9, 4));
        assert_eq!(counts(""), (0, 0));
    }

    #[test]
    fn one_scalar_is_one_to_four_bytes() {
        assert_eq!(bytes_for('a'), 1);
        assert_eq!(bytes_for(' '), 1);
        assert_eq!(bytes_for('س'), 2);
        assert_eq!(bytes_for('ی'), 2);
        assert_eq!(bytes_for('۵'), 2);
        assert_eq!(bytes_for('€'), 3);
        assert_eq!(bytes_for('\u{200c}'), 3);
        assert_eq!(bytes_for('🌸'), 4);
    }

    #[test]
    fn finds_the_widest_scalar() {
        assert_eq!(bytes_of_widest("hello"), 1);
        assert_eq!(bytes_of_widest("سلام"), 2);
        assert_eq!(bytes_of_widest("Rust برای بک‌اند"), 3);
        assert_eq!(bytes_of_widest("سلام 🌸"), 4);
        assert_eq!(bytes_of_widest(""), 0);
    }

    #[test]
    fn measures_a_prefix_in_bytes() {
        assert_eq!(bytes_of_first("hello", 2), 2);
        assert_eq!(bytes_of_first("hello", 5), 5);
        assert_eq!(bytes_of_first("سلام", 1), 2);
        assert_eq!(bytes_of_first("سلام", 2), 4);
        assert_eq!(bytes_of_first("سلام", 4), 8);
        assert_eq!(bytes_of_first("من 🌸", 3), 5);
        assert_eq!(bytes_of_first("من 🌸", 4), 9);
        assert_eq!(bytes_of_first("سلام", 0), 0);
        assert_eq!(bytes_of_first("سلام", 9), 8);
        assert_eq!(bytes_of_first("", 3), 0);
    }

    #[test]
    fn counts_the_continuation_bytes() {
        assert_eq!(continuation_bytes("hello"), 0);
        assert_eq!(continuation_bytes("سلام"), 4);
        assert_eq!(continuation_bytes("🌸"), 3);
        assert_eq!(continuation_bytes("من 🌸"), 5);
        assert_eq!(continuation_bytes("Rust برای بک‌اند"), 11);
        assert_eq!(continuation_bytes(""), 0);
    }

    /// The two counts are not independent: every byte of `text` is either the
    /// start of a scalar or a continuation of one.
    #[test]
    fn every_byte_is_a_start_or_a_continuation() {
        for text in ["hello", "سلام", "من 🌸", "Rust برای بک‌اند", ""] {
            let (bytes, scalars) = counts(text);
            assert_eq!(bytes - continuation_bytes(text), scalars);
        }
    }
}
