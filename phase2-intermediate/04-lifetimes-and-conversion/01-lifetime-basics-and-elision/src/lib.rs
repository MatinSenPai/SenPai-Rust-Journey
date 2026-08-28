//! Exercises for 2.4.1 — lifetime basics and elision.
//!
//! Three function signatures. One needs an explicit lifetime because it has
//! two plausible inputs the return value could borrow from. Two need none
//! at all — elision rules 2 and 3 already cover them — but say, in your own
//! head, which rule applies before you check `solution/SOLUTION.md`.

/// Returns whichever of `a`, `b` is shorter — `a` on a tie.
///
/// Both parameters and the return value share one lifetime, `'a`: the
/// returned reference is only ever as valid as the shorter-lived of the two
/// inputs, exactly like `longest` in the lesson body.
///
/// # Examples
///
/// `shorter("hi", "hello")` returns `"hi"`.
/// `shorter("cat", "dog")` returns `"cat"` (equal length, `a` wins).
pub fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str {
    todo!("compare a.len() and b.len(); return the shorter one, or `a` if they are equal")
}

/// Returns the portion of `text` before its first `'.'`.
///
/// If `text` contains no `'.'` at all, returns the whole of `text`
/// unchanged. There is exactly one reference parameter, so elision rule 2
/// covers the return type — no `<'a>` belongs anywhere on this signature.
///
/// # Examples
///
/// `first_sentence("Ownership is central. Borrowing comes next.")` returns
/// `"Ownership is central"`.
/// `first_sentence("no period here")` returns `"no period here"`.
pub fn first_sentence(text: &str) -> &str {
    todo!("return everything in `text` before the first '.', or all of `text` if there is none")
}

/// A raw `key=value` setting line, owned end to end — no borrowed field.
pub struct Setting {
    pub raw: String,
}

impl Setting {
    /// Returns the part of `raw` after its first `'='`.
    ///
    /// If `raw` contains no `'='`, returns the whole of `raw` unchanged.
    /// `&self` is the only reference here, so elision rule 3 assigns its
    /// lifetime to the return type automatically.
    ///
    /// # Examples
    ///
    /// `Setting { raw: "timeout=30".to_string() }.value()` returns `"30"`.
    /// `Setting { raw: "verbose".to_string() }.value()` returns `"verbose"`.
    pub fn value(&self) -> &str {
        todo!("return the part of `self.raw` after its first '=', or all of it if there is none")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorter_picks_the_shorter_string() {
        assert_eq!(shorter("hi", "hello"), "hi");
        assert_eq!(shorter("hello", "hi"), "hi");
    }

    #[test]
    fn shorter_favors_the_first_argument_on_a_tie() {
        assert_eq!(shorter("cat", "dog"), "cat");
    }

    #[test]
    fn first_sentence_stops_before_the_first_period() {
        assert_eq!(
            first_sentence("Ownership is central. Borrowing comes next."),
            "Ownership is central"
        );
    }

    #[test]
    fn first_sentence_returns_everything_if_no_period() {
        assert_eq!(first_sentence("no period here"), "no period here");
    }

    #[test]
    fn setting_value_reads_after_the_equals_sign() {
        let setting = Setting {
            raw: "timeout=30".to_string(),
        };
        assert_eq!(setting.value(), "30");
    }

    #[test]
    fn setting_value_falls_back_to_the_whole_raw_string() {
        let setting = Setting {
            raw: "verbose".to_string(),
        };
        assert_eq!(setting.value(), "verbose");
    }
}
