pub fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() {
        a
    } else {
        b
    }
}

pub struct FirstSentence<'a> {
    pub text: &'a str,
}

impl<'a> FirstSentence<'a> {
    pub fn new(paragraph: &'a str) -> Self {
        FirstSentence {
            text: paragraph.split('.').next().unwrap_or(paragraph),
        }
    }

    pub fn as_str(&self) -> &str {
        self.text
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
