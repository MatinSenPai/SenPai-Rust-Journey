//! Exercises for 1.4.3 — Building and transforming strings.
//!
//! Every one of these can be written with the pieces from this lesson and
//! nothing else. Two of them are one line. The other three are worth thinking
//! about before you type.

/// The parts, with `separator` between each neighbouring pair.
///
/// Nothing is added before the first part or after the last one. An empty
/// slice gives an empty `String`; a single part comes back on its own.
///
/// This is `joined` from 1.1.6, which you wrote by hand. The standard library
/// already had it.
///
/// # Examples
///
/// `joined(&["a".to_string(), "b".to_string()], "-")` returns `"a-b"`.
/// `joined(&["only".to_string()], "-")` returns `"only"`.
/// `joined(&[], "-")` returns `""`.
/// `joined(&["نام".to_string(), "شهر".to_string()], "، ")` returns `"نام، شهر"`.
pub fn joined(parts: &[String], separator: &str) -> String {
    todo!("put the separator between neighbouring parts and hand back the result")
}

/// One row of a two-column table.
///
/// `label` sits in a column of **10 characters**, padded on its right with
/// spaces. `amount` follows immediately in a column of **8 characters**,
/// padded on its left, written with **exactly two digits** after the point.
/// Nothing goes between the columns and nothing goes on either end, so when
/// both values fit the answer is 18 characters long.
///
/// A value too wide for its column is never cut: it pushes the row out.
///
/// # Examples
///
/// `aligned_row("tea", 3.5)` returns `"tea           3.50"` — three letters,
/// eleven spaces, then `3.50`.
/// `aligned_row("a-very-long-label", 1.0)` returns
/// `"a-very-long-label    1.00"`, which is 25 characters.
/// `aligned_row("", 0.0)` returns `"              0.00"`.
pub fn aligned_row(label: &str, amount: f64) -> String {
    todo!("lay the label and the amount out in the two fixed-width columns described above")
}

/// `text` with its outer whitespace gone and every inner run of whitespace
/// squeezed down to a single space.
///
/// Whitespace means any of it: spaces, tabs, newlines. Text that is nothing
/// but whitespace gives an empty `String`.
///
/// # Examples
///
/// `tidy("  دو   کلمه ")` returns `"دو کلمه"`.
/// `tidy("a\t\tb\nc")` returns `"a b c"`.
/// `tidy("   ")` returns `""`.
/// `tidy("one")` returns `"one"`.
pub fn tidy(text: &str) -> String {
    todo!("drop the outer whitespace and squeeze every inner run down to one space")
}

/// `text` trimmed, put into upper case, with a single `!` on the end.
///
/// Persian has no upper case, so Persian text comes back exactly as it went
/// in — plus the `!`. That is not a bug and it is worth seeing once.
///
/// # Examples
///
/// `shout("rust")` returns `"RUST!"`.
/// `shout("  hello ")` returns `"HELLO!"`.
/// `shout("سلام")` returns `"سلام!"`.
/// `shout("")` returns `"!"`.
pub fn shout(text: &str) -> String {
    todo!("trim it, raise the case, and add the exclamation mark")
}

/// At most `limit` characters of `text`.
///
/// When `text` is no longer than `limit`, it comes back untouched. When it is
/// longer, the answer is its first `limit` characters followed by a single `…`
/// (one character, U+2026), so the answer is `limit + 1` characters long.
///
/// The counting is in **characters**, never bytes — `preview("سلام", 2)` must
/// not panic and must not produce broken text.
///
/// # Examples
///
/// `preview("hello", 3)` returns `"hel…"`.
/// `preview("hello", 5)` returns `"hello"`.
/// `preview("hi", 5)` returns `"hi"`.
/// `preview("سلام", 2)` returns `"سل…"`.
/// `preview("سلام", 4)` returns `"سلام"`.
/// `preview("hello", 0)` returns `"…"`.
pub fn preview(text: &str, limit: usize) -> String {
    todo!("shorten the text to at most `limit` characters, and mark it when you cut")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(parts: &[&str]) -> Vec<String> {
        let mut out = Vec::with_capacity(parts.len());
        for part in parts {
            out.push(part.to_string());
        }
        out
    }

    #[test]
    fn puts_the_separator_between_neighbours() {
        assert_eq!(joined(&owned(&["a", "b"]), "-"), "a-b");
        assert_eq!(joined(&owned(&["a", "b", "c"]), "-"), "a-b-c");
        assert_eq!(joined(&owned(&["only"]), "-"), "only");
        assert_eq!(joined(&owned(&[]), "-"), "");
        assert_eq!(joined(&owned(&["نام", "شهر"]), "، "), "نام، شهر");
        assert_eq!(joined(&owned(&["a", "b"]), ""), "ab");
    }

    #[test]
    fn lays_out_two_fixed_width_columns() {
        assert_eq!(aligned_row("tea", 3.5), "tea           3.50");
        assert_eq!(aligned_row("tea", 3.5).chars().count(), 18);
        assert_eq!(aligned_row("", 0.0), "              0.00");
        assert_eq!(
            aligned_row("a-very-long-label", 1.0),
            "a-very-long-label    1.00"
        );
        // Two digits after the point, always — rounded, not cut.
        assert_eq!(aligned_row("x", 2.5), "x             2.50");
        assert_eq!(aligned_row("x", 12.3456), "x            12.35");
    }

    #[test]
    fn squeezes_whitespace() {
        assert_eq!(tidy("  دو   کلمه "), "دو کلمه");
        assert_eq!(tidy("a\t\tb\nc"), "a b c");
        assert_eq!(tidy("   "), "");
        assert_eq!(tidy(""), "");
        assert_eq!(tidy("one"), "one");
        assert_eq!(tidy(" one "), "one");
    }

    #[test]
    fn raises_the_case_where_there_is_one() {
        assert_eq!(shout("rust"), "RUST!");
        assert_eq!(shout("  hello "), "HELLO!");
        assert_eq!(shout(""), "!");
        // Persian has no case, so only the mark is added.
        assert_eq!(shout("سلام"), "سلام!");
        assert_eq!(shout(" سلام دنیا "), "سلام دنیا!");
    }

    #[test]
    fn shortens_by_characters_not_bytes() {
        assert_eq!(preview("hello", 3), "hel…");
        assert_eq!(preview("hello", 5), "hello");
        assert_eq!(preview("hello", 9), "hello");
        assert_eq!(preview("hi", 5), "hi");
        assert_eq!(preview("hello", 0), "…");
        assert_eq!(preview("", 3), "");

        assert_eq!(preview("سلام", 2), "سل…");
        assert_eq!(preview("سلام", 4), "سلام");
        assert_eq!(preview("سلام", 3).chars().count(), 4);
    }
}
