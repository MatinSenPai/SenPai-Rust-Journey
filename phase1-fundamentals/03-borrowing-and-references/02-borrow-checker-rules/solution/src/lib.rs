pub fn first_word_len_then_clear(s: &mut String) -> usize {
    let len = s.split_whitespace().next().unwrap_or("").len();
    s.clear();
    len
}

pub fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

pub fn describe_and_grow(s: &mut String) -> String {
    let description = {
        let first_char = s.chars().next().unwrap_or(' ');
        format!("{} chars, starts with '{}'", s.len(), first_char)
    };
    s.push_str(" (grown)");
    description
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_then_clears() {
        let mut s = String::from("hello world");
        assert_eq!(first_word_len_then_clear(&mut s), 5);
        assert_eq!(s, "");
    }

    #[test]
    fn greets() {
        assert_eq!(make_greeting("Matin"), "Hello, Matin!");
    }

    #[test]
    fn describes_then_grows() {
        let mut s = String::from("rust");
        let description = describe_and_grow(&mut s);
        assert_eq!(description, "4 chars, starts with 'r'");
        assert_eq!(s, "rust (grown)");
    }
}
