//! Solution for 2.4.3 — `Deref`, `AsRef`, `Borrow`, `ToOwned`.

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

pub struct Playlist(pub Vec<String>);

impl Deref for Playlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

pub struct DisplayName(pub String);

impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub String);

impl Borrow<str> for Handle {
    fn borrow(&self) -> &str {
        &self.0
    }
}

pub fn longest_word_owned(text: &str) -> String {
    text.split_whitespace()
        .max_by_key(|word| word.chars().count())
        .unwrap_or("")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn playlist_len_via_deref() {
        let list = Playlist(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn playlist_push_via_deref_mut() {
        let mut list = Playlist(vec!["a".to_string()]);
        list.push("b".to_string());
        assert_eq!(list.0, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn playlist_index_via_deref() {
        let list = Playlist(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(list[1], "b");
    }

    #[test]
    fn display_name_as_ref() {
        let name = DisplayName("Frieren".to_string());
        assert_eq!(name.as_ref(), "Frieren");
    }

    #[test]
    fn handle_looks_up_by_str_in_hashmap() {
        let mut seen: HashMap<Handle, i32> = HashMap::new();
        seen.insert(Handle("frieren".to_string()), 10);

        assert_eq!(seen.get("frieren"), Some(&10));
        assert_eq!(seen.get("someone-else"), None);
    }

    #[test]
    fn longest_word_owned_finds_the_longest() {
        assert_eq!(longest_word_owned("a bb ccc"), "ccc".to_string());
    }

    #[test]
    fn longest_word_owned_breaks_ties_with_the_last_one() {
        assert_eq!(longest_word_owned("cat dog owl bee"), "bee".to_string());
    }

    #[test]
    fn longest_word_owned_of_blank_text_is_empty() {
        assert_eq!(longest_word_owned(""), String::new());
        assert_eq!(longest_word_owned("   "), String::new());
    }
}
