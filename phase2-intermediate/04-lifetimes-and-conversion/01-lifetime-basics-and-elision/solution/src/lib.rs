//! Solution for 2.4.1 — lifetime basics and elision.

/// Returns whichever of `a`, `b` is shorter — `a` on a tie.
pub fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() <= b.len() {
        a
    } else {
        b
    }
}

/// Returns the portion of `text` before its first `'.'`, or all of `text`.
pub fn first_sentence(text: &str) -> &str {
    match text.split_once('.') {
        Some((before, _after)) => before,
        None => text,
    }
}

/// A raw `key=value` setting line, owned end to end — no borrowed field.
pub struct Setting {
    pub raw: String,
}

impl Setting {
    /// Returns the part of `raw` after its first `'='`, or all of `raw`.
    pub fn value(&self) -> &str {
        match self.raw.split_once('=') {
            Some((_key, value)) => value,
            None => &self.raw,
        }
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
