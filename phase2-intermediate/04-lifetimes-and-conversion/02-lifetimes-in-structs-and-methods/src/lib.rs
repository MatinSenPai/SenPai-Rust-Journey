//! Exercises for 2.4.2 — lifetimes in structs and methods.
//!
//! `Excerpt` and its `first_line` constructor are already written below,
//! exactly as covered in the lesson. Your job is one more constructor, two
//! methods, and one plain function: fill in every `todo!()`.

pub struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    /// The first line of `source` (up to the first `\n`, or the whole
    /// string if there is none), without copying it.
    pub fn first_line(source: &'a str) -> Self {
        let text = source.lines().next().unwrap_or(source);
        Excerpt { text }
    }

    /// Wraps the entire `source` as the excerpt — no line-splitting.
    pub fn whole(source: &'a str) -> Self {
        todo!("wrap the entire source string as this excerpt's text")
    }

    /// The excerpt's own text. Decide for yourself whether the return type
    /// needs an explicit lifetime here.
    pub fn text(&self) -> &str {
        todo!("return the excerpt's own text")
    }

    /// How many whitespace-separated words are in the excerpt.
    pub fn word_count(&self) -> usize {
        todo!("split the excerpt's text on whitespace and count how many pieces result")
    }
}

/// The sum of `word_count()` across every excerpt in `excerpts`.
pub fn total_words(excerpts: &[Excerpt<'_>]) -> usize {
    todo!("add up word_count() across every excerpt in the slice")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_wraps_the_entire_source() {
        let source = String::from("no newlines here");
        let excerpt = Excerpt::whole(&source);
        assert_eq!(excerpt.text(), "no newlines here");
    }

    #[test]
    fn first_line_stops_at_the_first_newline() {
        let source = String::from("first line\nsecond line");
        let excerpt = Excerpt::first_line(&source);
        assert_eq!(excerpt.text(), "first line");
    }

    #[test]
    fn word_count_counts_whitespace_separated_words() {
        let source = String::from("four little words here");
        let excerpt = Excerpt::whole(&source);
        assert_eq!(excerpt.word_count(), 4);
    }

    #[test]
    fn word_count_is_zero_for_an_empty_excerpt() {
        let source = String::new();
        let excerpt = Excerpt::whole(&source);
        assert_eq!(excerpt.word_count(), 0);
    }

    #[test]
    fn total_words_sums_every_excerpt() {
        let a = String::from("two words");
        let b = String::from("three little words");
        let excerpts = vec![Excerpt::whole(&a), Excerpt::whole(&b)];
        assert_eq!(total_words(&excerpts), 5);
    }

    #[test]
    fn total_words_of_an_empty_slice_is_zero() {
        assert_eq!(total_words(&[]), 0);
    }
}
