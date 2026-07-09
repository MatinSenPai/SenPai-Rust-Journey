/// Borrows `name` (works for a `String`, a literal, or any `&str`), returns
/// a brand new, owned greeting.
pub fn greet(name: &str) -> String {
    todo!("format!(\"Hello, {{name}}!\")")
}

/// Converts a string literal into an owned `String`.
pub fn owned_from_literal() -> String {
    todo!("\"hello\".to_string(), or String::from(\"hello\")")
}

/// Returns the longest whitespace-separated word in `text`, **borrowed**
/// directly from `text` (no allocation) — not an owned copy.
pub fn longest_word(text: &str) -> &str {
    todo!("text.split_whitespace(), find the one with the max .len()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_from_literal_and_from_string() {
        assert_eq!(greet("Matin"), "Hello, Matin!");
        let owned = String::from("Rust");
        assert_eq!(greet(&owned), "Hello, Rust!"); // &String coerces to &str
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
