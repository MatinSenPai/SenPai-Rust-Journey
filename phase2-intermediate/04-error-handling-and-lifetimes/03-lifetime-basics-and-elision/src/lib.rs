/// Elision rule 1 applies (one input reference, its lifetime flows to the
/// elided output) — no explicit `<'a>` needed here at all.
pub fn first_word(s: &str) -> &str {
    todo!("s.split_whitespace().next().unwrap_or(\"\")")
}

/// Two input references, one output reference — elision rules 1/2 can't
/// pick which input the output borrows from (it might be either), so this
/// signature needs an explicit lifetime tying all three together.
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    todo!("if a.len() >= b.len() {{ a }} else {{ b }}")
}

/// A struct holding a reference always needs an explicit lifetime — no
/// elision rule covers struct fields.
pub struct FirstSentence<'a> {
    pub text: &'a str,
}

impl<'a> FirstSentence<'a> {
    /// Finds the text up to (excluding) the first `.`, or the whole
    /// paragraph if there's no `.` at all.
    pub fn new(paragraph: &'a str) -> Self {
        todo!("FirstSentence {{ text: paragraph.split('.').next().unwrap_or(paragraph) }}")
    }

    /// Elision rule 3 applies: this is a method (`&self`), so the elided
    /// output lifetime is `self`'s lifetime — no explicit annotation
    /// needed on this method, even though `FirstSentence` itself needed one.
    pub fn as_str(&self) -> &str {
        todo!("self.text")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_first_word() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("single"), "single");
    }

    #[test]
    fn finds_the_longer_string() {
        assert_eq!(longest("short", "much longer"), "much longer");
        assert_eq!(longest("equal", "sizes"), "equal");
    }

    #[test]
    fn extracts_first_sentence() {
        let excerpt = FirstSentence::new("Ownership is central. Borrowing comes next.");
        assert_eq!(excerpt.as_str(), "Ownership is central");
    }

    #[test]
    fn whole_paragraph_if_no_period() {
        let excerpt = FirstSentence::new("no period here");
        assert_eq!(excerpt.as_str(), "no period here");
    }
}
