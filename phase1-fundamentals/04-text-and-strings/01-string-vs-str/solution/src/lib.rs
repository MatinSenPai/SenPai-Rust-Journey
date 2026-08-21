pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

pub fn owned_from_literal() -> String {
    "hello".to_string()
}

pub fn longest_word(text: &str) -> &str {
    text.split_whitespace()
        .max_by_key(|word| word.len())
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_from_literal_and_from_string() {
        assert_eq!(greet("Matin"), "Hello, Matin!");
        let owned = String::from("Rust");
        assert_eq!(greet(&owned), "Hello, Rust!");
    }

    #[test]
    fn converts_literal_to_owned() {
        let s: String = owned_from_literal();
        assert_eq!(s, "hello");
    }

    #[test]
    fn finds_longest_word() {
        assert_eq!(longest_word("the quickest brown fox"), "quickest");
        assert_eq!(longest_word("a bb ccc"), "ccc");
    }
}
