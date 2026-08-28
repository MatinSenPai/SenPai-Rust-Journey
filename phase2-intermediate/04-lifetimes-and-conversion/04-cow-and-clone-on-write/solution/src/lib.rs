//! Exercises for 2.4.4 — `Cow<'_, str>` and copy-on-write.
//!
//! Three functions, each exercising one piece of this lesson: reading a
//! `Cow`'s state without consuming it, building one that stays cheap in the
//! common case, and mutating one only when mutation is actually needed.

use std::borrow::Cow;

/// Whether `value` currently holds a borrowed view or an owned copy.
///
/// Returns the literal string `"borrowed"` for `Cow::Borrowed`, or
/// `"owned"` for `Cow::Owned`.
// clippy's `ptr_arg` lint wants `&str` here — good advice when a function
// only reads text, wrong when the whole point is to inspect which variant
// the `Cow` is, which a plain `&str` could never answer.
#[allow(clippy::ptr_arg)]
pub fn describe(value: &Cow<'_, str>) -> &'static str {
    match value {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}

/// Collapses every run of two or more consecutive ASCII space characters
/// (`' '`, U+0020) in `input` down to a single space. Every other
/// character — tabs, newlines, and any lone space — is left exactly as it
/// was.
///
/// Returns `Cow::Borrowed(input)`, completely unchanged, when `input`
/// contains no run of two or more spaces. Returns `Cow::Owned` holding the
/// collapsed text otherwise.
///
/// # Examples
///
/// `collapse_spaces("Trigun: 26 episodes")` borrows its input unchanged —
/// there is no run of spaces to collapse.
/// `collapse_spaces("Trigun:    26   episodes")` returns an owned
/// `"Trigun: 26 episodes"`.
pub fn collapse_spaces(input: &str) -> Cow<'_, str> {
    if !input.contains("  ") {
        return Cow::Borrowed(input);
    }
    let mut collapsed = String::with_capacity(input.len());
    let mut previous_was_space = false;
    for ch in input.chars() {
        if ch == ' ' && previous_was_space {
            continue;
        }
        previous_was_space = ch == ' ';
        collapsed.push(ch);
    }
    Cow::Owned(collapsed)
}

/// Makes sure `value` ends with `'!'`, appending one only if it is
/// missing. If `value` already ends with `'!'`, it is returned completely
/// unchanged — still `Borrowed` if it started that way.
///
/// # Examples
///
/// `ensure_exclaimed(Cow::Borrowed("Trigun"))` returns an owned
/// `"Trigun!"`. `ensure_exclaimed(Cow::Borrowed("Trigun!"))` returns the
/// exact same `Cow::Borrowed("Trigun!")` it was given.
pub fn ensure_exclaimed(mut value: Cow<'_, str>) -> Cow<'_, str> {
    if !value.ends_with('!') {
        value.to_mut().push('!');
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_names_the_borrowed_variant() {
        let value: Cow<str> = Cow::Borrowed("Trigun");
        assert_eq!(describe(&value), "borrowed");
    }

    #[test]
    fn describe_names_the_owned_variant() {
        let value: Cow<str> = Cow::Owned(String::from("Trigun"));
        assert_eq!(describe(&value), "owned");
    }

    #[test]
    fn collapse_spaces_borrows_input_with_no_double_space() {
        let input = "Trigun: 26 episodes";
        let result = collapse_spaces(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result.as_ptr(), input.as_ptr());
    }

    #[test]
    fn collapse_spaces_borrows_a_single_leading_or_trailing_space_too() {
        let input = " Trigun ";
        let result = collapse_spaces(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(&result, input);
    }

    #[test]
    fn collapse_spaces_leaves_tabs_and_newlines_alone() {
        let input = "Trigun:\t26\nepisodes";
        let result = collapse_spaces(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(&result, input);
    }

    #[test]
    fn collapse_spaces_owns_and_collapses_internal_runs() {
        let result = collapse_spaces("Trigun:    26   episodes");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&result, "Trigun: 26 episodes");
    }

    #[test]
    fn collapse_spaces_collapses_a_leading_run_too() {
        let result = collapse_spaces("   Trigun");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&result, " Trigun");
    }

    #[test]
    fn collapse_spaces_on_empty_input_borrows_it() {
        let input = "";
        let result = collapse_spaces(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(&result, "");
    }

    #[test]
    fn ensure_exclaimed_appends_when_missing() {
        let result = ensure_exclaimed(Cow::Borrowed("Trigun"));
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&result, "Trigun!");
    }

    #[test]
    fn ensure_exclaimed_leaves_a_borrowed_value_untouched_when_already_present() {
        let input = "Trigun!";
        let result = ensure_exclaimed(Cow::Borrowed(input));
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result.as_ptr(), input.as_ptr());
    }

    #[test]
    fn ensure_exclaimed_leaves_an_owned_value_untouched_when_already_present() {
        let result = ensure_exclaimed(Cow::Owned(String::from("Trigun!")));
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(&result, "Trigun!");
    }
}
