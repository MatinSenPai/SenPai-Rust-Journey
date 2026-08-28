//! Exercises for 2.4.3 — `Deref`, `AsRef`, `Borrow`, `ToOwned`.
//!
//! Four small pieces, one per trait. `Playlist`, `DisplayName`, and `Handle`
//! are deliberately different domains from the lesson's `Watchlist` — same
//! pattern, your own hands on it.

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

/// A newtype wrapper around `Vec<String>`.
///
/// Implement `Deref` (with `Target = Vec<String>`) and `DerefMut` for it, so
/// that a `Playlist` behaves like its inner `Vec<String>` at method-call
/// sites and through the `*` operator — exactly the pattern
/// `examples/01-deref-and-derefmut.rs` showed on `Watchlist`.
pub struct Playlist(pub Vec<String>);

impl Deref for Playlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        todo!("return a shared reference to the inner Vec<String>")
    }
}

impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        todo!("return a mutable reference to the inner Vec<String>")
    }
}

/// A newtype wrapper around `String`.
///
/// Implement `AsRef<str>` for it, so a `DisplayName` can be passed anywhere
/// a `impl AsRef<str>` parameter is expected, the same way `&str` and
/// `String` already can.
pub struct DisplayName(pub String);

impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        todo!("return a &str view of the inner String")
    }
}

/// A newtype wrapper around `String`, meant to be used as a `HashMap` key.
///
/// `Hash`, `Eq`, and `PartialEq` are already derived below, straight from the
/// inner `String` — so implement `Borrow<str>` the only way that stays
/// consistent with them: hand back a view of the exact same `String`, not a
/// transformed copy of it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub String);

impl Borrow<str> for Handle {
    fn borrow(&self) -> &str {
        todo!("return a &str view of the inner String, consistent with Handle's own Hash and Eq")
    }
}

/// The longest word in `text`, owned.
///
/// Words are split exactly as `str::split_whitespace` splits them — any run
/// of whitespace separates words, leading/trailing whitespace is ignored.
/// "Longest" means the greatest `.chars().count()`. When two or more words
/// tie for longest, the **last** one (in reading order) wins. `text` with no
/// words in it (empty, or all whitespace) returns an owned empty string.
///
/// # Examples
///
/// `longest_word_owned("a bb ccc")` returns `"ccc".to_string()`.
/// `longest_word_owned("cat dog owl bee")` returns `"bee".to_string()` — all
/// four words tie at 3 characters, and `"bee"` is the last of them.
pub fn longest_word_owned(text: &str) -> String {
    todo!(
        "find the longest word in text by character count, breaking a tie in favor of the last \
         tied word; return it as an owned String, or an owned empty String if text has no words"
    )
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
