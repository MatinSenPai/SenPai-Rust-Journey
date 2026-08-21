//! Exercises for 1.7.2 — Phase review.
//!
//! Six functions, one per module of Phase 1 (excluding this one). Each needs
//! a *different* idea, on purpose: if one of them won't come right, that
//! tells you precisely which lesson to reread, not just "something's wrong."

/// The letter grade for `score`, a percentage from 0 to 100.
///
/// `90..=100` is `'A'`, `80..=89` is `'B'`, `70..=79` is `'C'`,
/// `60..=69` is `'D'`, and anything below `60` is `'F'`.
///
/// Write it as a single expression — a chain of `if`/`else if`/`else` that
/// *is* the return value, not a chain that assigns into a variable and
/// returns it afterward. (Stuck? Reread 1.1.5 — control flow.)
///
/// # Examples
///
/// `grade_letter(95)` is `'A'`. `grade_letter(60)` is `'D'`.
/// `grade_letter(59)` is `'F'`.
pub fn grade_letter(score: u32) -> char {
    todo!("return the letter for score's range, as one if/else expression")
}

/// `base`, extended with every entry from `extra` that is not already
/// present in `base` — comparing whole strings, not prefixes.
///
/// `base`'s own order comes first, unchanged; new entries from `extra` are
/// appended in the order they appear in `extra`. `base` is consumed: this
/// function owns it and hands back the extended version, not a copy.
/// `extra` is only read.
///
/// The only strings this function should ever clone are the ones it
/// actually inserts. Every string already in `base`, and every string from
/// `extra` that turns out to be a duplicate, must reach the end without a
/// clone. (Stuck? Reread 1.2.4 — ownership across functions — and 1.2.3 —
/// `Clone` and `Copy`.)
///
/// # Examples
///
/// `merge_unique(vec!["a".into(), "b".into()], &["b".into(), "c".into()])`
/// returns `["a", "b", "c"]`.
pub fn merge_unique(base: Vec<String>, extra: &[String]) -> Vec<String> {
    todo!("extend base with the extra entries it doesn't already contain")
}

/// `values` with its first and last element removed — as a borrowed view,
/// with no new `Vec` allocated.
///
/// If `values` has fewer than two elements, the answer is an empty slice.
/// (Stuck? Reread 1.3.4 — slices.)
///
/// # Examples
///
/// `interior(&[1, 2, 3, 4])` is `&[2, 3]`. `interior(&[1, 2])` is `&[]`.
/// `interior(&[1])` is `&[]`. `interior(&[])` is `&[]`.
pub fn interior(values: &[i32]) -> &[i32] {
    todo!("slice off the first and last element, or return &[] if that's not possible")
}

/// The first `max_chars` **characters** of `text`, as a slice into `text` —
/// never a byte count, and never a cut that lands inside a character.
///
/// If `text` has `max_chars` characters or fewer, the whole thing comes
/// back unchanged. (Stuck? Reread 1.4.2 — UTF-8: bytes, chars, graphemes.)
///
/// # Examples
///
/// `shorten("سلام", 2)` is `"سل"`. `shorten("hi", 10)` is `"hi"`.
/// `shorten("", 3)` is `""`.
pub fn shorten(text: &str, max_chars: usize) -> &str {
    todo!("find the byte offset of the max_chars-th character and slice up to it")
}

/// The status of an order, exactly as wide as the shapes it can take.
pub enum Status {
    Pending,
    Shipped { tracking: String },
    Cancelled { reason: String },
}

/// A one-line description of `status`.
///
/// `Status::Pending` is `"pending"`.
/// `Status::Shipped { tracking }` is `"shipped, tracking {tracking}"` — for
/// example `"shipped, tracking RS100"`.
/// `Status::Cancelled { reason }` is `"cancelled: {reason}"` — for example
/// `"cancelled: out of stock"`.
///
/// (Stuck? Reread 1.5.3 — enums as data — and 1.5.4 — `match` in depth.)
pub fn describe_status(status: &Status) -> String {
    todo!("match on status and format the line for each variant")
}

/// The arithmetic mean of `values`, or `None` if `values` is empty —
/// never a divide-by-zero panic.
///
/// (Stuck? Reread 1.6.1 — `Option` and null safety.)
///
/// # Examples
///
/// `safe_average(&[2, 4, 6])` is `Some(4.0)`. `safe_average(&[])` is `None`.
pub fn safe_average(values: &[i32]) -> Option<f64> {
    todo!("return None for an empty slice, otherwise Some(mean)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_letter_covers_every_band() {
        assert_eq!(grade_letter(100), 'A');
        assert_eq!(grade_letter(95), 'A');
        assert_eq!(grade_letter(90), 'A');
        assert_eq!(grade_letter(89), 'B');
        assert_eq!(grade_letter(80), 'B');
        assert_eq!(grade_letter(79), 'C');
        assert_eq!(grade_letter(70), 'C');
        assert_eq!(grade_letter(69), 'D');
        assert_eq!(grade_letter(60), 'D');
        assert_eq!(grade_letter(59), 'F');
        assert_eq!(grade_letter(0), 'F');
    }

    #[test]
    fn merge_unique_appends_only_the_new_entries() {
        let base = vec!["a".to_string(), "b".to_string()];
        let extra = vec!["b".to_string(), "c".to_string()];
        assert_eq!(merge_unique(base, &extra), vec!["a", "b", "c"]);

        let base = vec!["only".to_string()];
        assert_eq!(merge_unique(base, &[]), vec!["only"]);

        let base: Vec<String> = vec![];
        let extra = vec!["x".to_string(), "x".to_string()];
        assert_eq!(merge_unique(base, &extra), vec!["x"]);
    }

    #[test]
    fn interior_drops_the_ends_without_allocating() {
        let values = [1, 2, 3, 4];
        assert_eq!(interior(&values), &[2, 3]);
        assert_eq!(interior(&[1, 2]), &[] as &[i32]);
        assert_eq!(interior(&[1]), &[] as &[i32]);
        assert_eq!(interior(&[]), &[] as &[i32]);
    }

    #[test]
    fn shorten_counts_characters_not_bytes() {
        assert_eq!(shorten("سلام", 2), "سل");
        assert_eq!(shorten("سلام", 0), "");
        assert_eq!(shorten("سلام", 100), "سلام");
        assert_eq!(shorten("hi", 10), "hi");
        assert_eq!(shorten("", 3), "");
    }

    #[test]
    fn describe_status_covers_every_variant() {
        assert_eq!(describe_status(&Status::Pending), "pending");
        assert_eq!(
            describe_status(&Status::Shipped {
                tracking: "RS100".to_string()
            }),
            "shipped, tracking RS100"
        );
        assert_eq!(
            describe_status(&Status::Cancelled {
                reason: "out of stock".to_string()
            }),
            "cancelled: out of stock"
        );
    }

    #[test]
    fn safe_average_handles_the_empty_case() {
        assert_eq!(safe_average(&[2, 4, 6]), Some(4.0));
        assert_eq!(safe_average(&[1, 2]), Some(1.5));
        assert_eq!(safe_average(&[5]), Some(5.0));
        assert_eq!(safe_average(&[]), None);
    }
}
