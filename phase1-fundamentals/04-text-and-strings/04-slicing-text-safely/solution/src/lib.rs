//! Reference solution for 1.4.4 — Slicing text safely.
//!
//! Every one of these must work for Persian, for English, for a mixture of
//! the two, and for the empty string. None of them may panic on any input.

/// Every byte index of `text` that is a legal place to cut, in ascending
/// order.
///
/// The list always starts at `0` and always ends at `text.len()`, so the
/// empty string gives `[0]`.
///
/// # Examples
///
/// `char_boundaries("abc")` returns `[0, 1, 2, 3]`.
/// `char_boundaries("سلام")` returns `[0, 2, 4, 6, 8]` — four two-byte
/// letters.
/// `char_boundaries("")` returns `[0]`.
pub fn char_boundaries(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for index in 0..=text.len() {
        if text.is_char_boundary(index) {
            found.push(index);
        }
    }
    found
}

/// The longest prefix of `text` that takes at most `max_bytes` bytes and
/// still ends on a character boundary.
///
/// When `max_bytes` is at least `text.len()`, the answer is all of `text`.
/// When the budget lands in the middle of a character, the answer stops
/// *before* that character rather than including it.
///
/// # Examples
///
/// `safe_prefix("سلام", 3)` returns `"س"` — byte 3 is inside the second
/// letter, so the cut moves back to byte 2.
/// `safe_prefix("سلام", 4)` returns `"سل"`.
/// `safe_prefix("hello", 3)` returns `"hel"`.
/// `safe_prefix("سلام", 99)` returns `"سلام"`.
/// `safe_prefix("سلام", 0)` returns `""`.
pub fn safe_prefix(text: &str, max_bytes: usize) -> &str {
    &text[..text.floor_char_boundary(max_bytes)]
}

/// The first `max_chars` characters of `text` — characters, not bytes.
///
/// When `text` has `max_chars` characters or fewer, the answer is all of
/// `text`.
///
/// # Examples
///
/// `truncate_to_chars("سلام دنیا", 4)` returns `"سلام"`.
/// `truncate_to_chars("hello world", 5)` returns `"hello"`.
/// `truncate_to_chars("سلام", 10)` returns `"سلام"`.
/// `truncate_to_chars("سلام", 0)` returns `""`.
/// `truncate_to_chars("", 5)` returns `""`.
pub fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}

/// `text` shortened to `max_chars` characters, with a single `…` (U+2026)
/// added **only when something was actually removed**.
///
/// So a `text` of `max_chars` characters or fewer comes back unchanged and
/// without an ellipsis, and a longer one comes back as its first `max_chars`
/// characters followed by `…` — at most `max_chars + 1` characters in total.
///
/// # Examples
///
/// `truncated_with_ellipsis("سلام دنیا", 4)` returns `"سلام…"`.
/// `truncated_with_ellipsis("سلام", 4)` returns `"سلام"` — exactly the limit,
/// so nothing was cut and no ellipsis is added.
/// `truncated_with_ellipsis("hello world", 5)` returns `"hello…"`.
/// `truncated_with_ellipsis("", 3)` returns `""`.
/// `truncated_with_ellipsis("سلام", 0)` returns `"…"`.
pub fn truncated_with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let kept = truncate_to_chars(text, max_chars);
    format!("{kept}…")
}

/// `text` cut in two at character number `char_index`: the first
/// `char_index` characters, then everything after them.
///
/// When `text` has fewer than `char_index` characters, the whole of `text`
/// comes back as the first half and the second half is empty.
///
/// # Examples
///
/// `split_at_char("سلام دنیا", 4)` returns `("سلام", " دنیا")`.
/// `split_at_char("hello", 2)` returns `("he", "llo")`.
/// `split_at_char("سلام", 0)` returns `("", "سلام")`.
/// `split_at_char("سلام", 99)` returns `("سلام", "")`.
pub fn split_at_char(text: &str, char_index: usize) -> (&str, &str) {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == char_index {
            return text.split_at(index);
        }
        seen += 1;
    }
    (text, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    // "سلام" is four two-byte letters: 8 bytes, boundaries at 0 2 4 6 8.
    // "می‌روم" has a three-byte zero-width non-joiner at bytes 4..7.

    #[test]
    fn boundaries_of_latin_persian_mixed_and_empty() {
        assert_eq!(char_boundaries("abc"), vec![0, 1, 2, 3]);
        assert_eq!(char_boundaries("سلام"), vec![0, 2, 4, 6, 8]);
        assert_eq!(char_boundaries("می‌روم"), vec![0, 2, 4, 7, 9, 11, 13]);
        assert_eq!(char_boundaries("aسb"), vec![0, 1, 3, 4]);
        assert_eq!(char_boundaries(""), vec![0]);
    }

    #[test]
    fn boundaries_always_start_at_zero_and_end_at_len() {
        for sample in ["", "a", "سلام", "Rust برای همه"] {
            let found = char_boundaries(sample);
            assert_eq!(found[0], 0, "the list must start at 0 for {sample:?}");
            assert_eq!(
                found[found.len() - 1],
                sample.len(),
                "the list must end at len() for {sample:?}"
            );
        }
    }

    #[test]
    fn a_prefix_never_lands_mid_character() {
        assert_eq!(safe_prefix("سلام", 3), "س");
        assert_eq!(safe_prefix("سلام", 4), "سل");
        assert_eq!(safe_prefix("سلام", 0), "");
        assert_eq!(safe_prefix("hello", 3), "hel");
        assert_eq!(safe_prefix("Rust برای همه", 6), "Rust ");
        assert_eq!(safe_prefix("Rust برای همه", 7), "Rust ب");
        assert_eq!(safe_prefix("می‌روم", 6), "می");
        assert_eq!(safe_prefix("می‌روم", 7), "می\u{200c}");
    }

    #[test]
    fn a_prefix_bigger_than_the_text_is_the_whole_text() {
        assert_eq!(safe_prefix("سلام", 8), "سلام");
        assert_eq!(safe_prefix("سلام", 99), "سلام");
        assert_eq!(safe_prefix("", 5), "");
        assert_eq!(safe_prefix("", 0), "");
    }

    #[test]
    fn truncating_counts_characters_not_bytes() {
        assert_eq!(truncate_to_chars("سلام دنیا", 4), "سلام");
        assert_eq!(truncate_to_chars("hello world", 5), "hello");
        assert_eq!(truncate_to_chars("Rust برای همه", 6), "Rust ب");
        assert_eq!(truncate_to_chars("می‌روم", 3), "می\u{200c}");
    }

    #[test]
    fn truncating_handles_the_ends_of_the_range() {
        assert_eq!(truncate_to_chars("سلام", 4), "سلام");
        assert_eq!(truncate_to_chars("سلام", 10), "سلام");
        assert_eq!(truncate_to_chars("سلام", 0), "");
        assert_eq!(truncate_to_chars("", 5), "");
        assert_eq!(truncate_to_chars("", 0), "");
    }

    #[test]
    fn the_ellipsis_appears_only_when_something_was_removed() {
        assert_eq!(truncated_with_ellipsis("سلام دنیا", 4), "سلام…");
        assert_eq!(truncated_with_ellipsis("hello world", 5), "hello…");
        assert_eq!(truncated_with_ellipsis("Rust برای همه", 6), "Rust ب…");

        // Exactly the limit: nothing was cut, so nothing is added.
        assert_eq!(truncated_with_ellipsis("سلام", 4), "سلام");
        assert_eq!(truncated_with_ellipsis("hello", 5), "hello");

        // Under the limit, and empty.
        assert_eq!(truncated_with_ellipsis("سلام", 10), "سلام");
        assert_eq!(truncated_with_ellipsis("", 3), "");
        assert_eq!(truncated_with_ellipsis("", 0), "");

        // A budget of zero on non-empty text: everything went, so the
        // ellipsis stays.
        assert_eq!(truncated_with_ellipsis("سلام", 0), "…");
    }

    #[test]
    fn the_answer_is_never_longer_than_the_budget_plus_one() {
        let samples = ["سلام دنیا", "hello world", "Rust برای همه", "می‌روم", ""];
        for sample in samples {
            for budget in 0..12 {
                let answer = truncated_with_ellipsis(sample, budget);
                assert!(
                    answer.chars().count() <= budget + 1,
                    "{sample:?} at {budget} gave {answer:?}"
                );
            }
        }
    }

    #[test]
    fn splitting_at_a_character_gives_both_halves_back() {
        assert_eq!(split_at_char("سلام دنیا", 4), ("سلام", " دنیا"));
        assert_eq!(split_at_char("hello", 2), ("he", "llo"));
        assert_eq!(split_at_char("Rust برای همه", 5), ("Rust ", "برای همه"));
        assert_eq!(split_at_char("می‌روم", 3), ("می\u{200c}", "روم"));
    }

    #[test]
    fn splitting_at_the_ends_of_the_range() {
        assert_eq!(split_at_char("سلام", 0), ("", "سلام"));
        assert_eq!(split_at_char("سلام", 4), ("سلام", ""));
        assert_eq!(split_at_char("سلام", 99), ("سلام", ""));
        assert_eq!(split_at_char("", 0), ("", ""));
        assert_eq!(split_at_char("", 3), ("", ""));
    }

    #[test]
    fn the_two_halves_always_rebuild_the_original() {
        let samples = ["سلام دنیا", "hello", "Rust برای همه", "می‌روم", ""];
        for sample in samples {
            for cut in 0..10 {
                let (head, tail) = split_at_char(sample, cut);
                assert_eq!(
                    format!("{head}{tail}"),
                    sample,
                    "{sample:?} split at {cut} lost something"
                );
            }
        }
    }
}
