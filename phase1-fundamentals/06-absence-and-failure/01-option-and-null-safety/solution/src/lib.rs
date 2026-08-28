//! Reference solution for 1.6.1 — `Option` and null safety.
//!
//! `.map()`, `.and_then()`, `.unwrap_or()` and `.ok_or()` are not here on
//! purpose — they're 1.6.2. `match` and `if let` are enough for all five.

/// The index of the first negative number in `readings`, or `None` if every
/// reading is zero or positive.
///
/// This is 1.1.5's `index_of_first_negative`, fixed. There it returned
/// `readings.len()` to mean "not found" — a plain `usize` that a caller
/// could index with and panic on, with nothing in the signature to warn
/// them. Here, "not found" is a value of a different type than "found at
/// position N", so a caller cannot use one as if it were the other without
/// the compiler stopping them first.
///
/// # Examples
///
/// `index_of_first_negative([5, 3, -2, 8, -9, 1])` returns `Some(2)`.
/// `index_of_first_negative([-1, 0, 0, 0, 0, 0])` returns `Some(0)`.
/// `index_of_first_negative([1, 2, 3, 4, 5, 6])` returns `None`.
pub fn index_of_first_negative(readings: [i32; 6]) -> Option<usize> {
    for position in 0..readings.len() {
        if readings[position] < 0 {
            return Some(position);
        }
    }
    None
}

/// The average of `total` split across `count` items, or `None` if `count`
/// is zero — avoiding a division by zero rather than panicking on one.
///
/// # Examples
///
/// `safe_average(10, 4)` returns `Some(2.5)`.
/// `safe_average(-9, 3)` returns `Some(-3.0)`.
/// `safe_average(10, 0)` returns `None`.
pub fn safe_average(total: i32, count: u32) -> Option<f64> {
    if count == 0 {
        return None;
    }
    Some(total as f64 / count as f64)
}

/// The number of bytes in `nickname`, or `None` if there isn't one.
///
/// `nickname` is a reference — this function only looks, it never takes
/// ownership of what the caller has.
///
/// # Examples
///
/// `nickname_len(&Some("Matin".to_string()))` returns `Some(5)`.
/// `nickname_len(&None)` returns `None`.
pub fn nickname_len(nickname: &Option<String>) -> Option<usize> {
    match nickname.as_ref() {
        Some(name) => Some(name.len()),
        None => None,
    }
}

/// The first entry in `words`, upper-cased, or `None` if `words` is empty.
///
/// # Examples
///
/// `first_word_upper(&["hello".to_string(), "world".to_string()])` returns
/// `Some("HELLO".to_string())`.
/// `first_word_upper(&[])` returns `None`.
pub fn first_word_upper(words: &[String]) -> Option<String> {
    match words.first() {
        Some(word) => Some(word.to_uppercase()),
        None => None,
    }
}

/// A user record where the nickname might never have been set.
pub struct Profile {
    pub nickname: Option<String>,
}

/// A one-line greeting for `profile`.
///
/// Exactly `"Hey, {nickname}!"` when there is one, and exactly
/// `"Hey, stranger!"` when there isn't.
///
/// # Examples
///
/// A profile with `nickname: Some("Yui".to_string())` greets `"Hey, Yui!"`.
/// A profile with `nickname: None` greets `"Hey, stranger!"`.
pub fn greeting(profile: &Profile) -> String {
    match &profile.nickname {
        Some(name) => format!("Hey, {name}!"),
        None => "Hey, stranger!".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_first_negative_position() {
        assert_eq!(index_of_first_negative([5, 3, -2, 8, -9, 1]), Some(2));
        assert_eq!(index_of_first_negative([-1, 0, 0, 0, 0, 0]), Some(0));
        assert_eq!(index_of_first_negative([0, 0, 0, 0, 0, -1]), Some(5));
    }

    #[test]
    fn none_when_nothing_is_negative() {
        assert_eq!(index_of_first_negative([1, 2, 3, 4, 5, 6]), None);
        assert_eq!(index_of_first_negative([0, 0, 0, 0, 0, 0]), None);
    }

    #[test]
    fn averages_a_nonzero_count() {
        assert_eq!(safe_average(10, 4), Some(2.5));
        assert_eq!(safe_average(-9, 3), Some(-3.0));
        assert_eq!(safe_average(0, 5), Some(0.0));
    }

    #[test]
    fn refuses_to_divide_by_zero() {
        assert_eq!(safe_average(10, 0), None);
        assert_eq!(safe_average(0, 0), None);
    }

    #[test]
    fn reports_the_nickname_length() {
        assert_eq!(nickname_len(&Some("Matin".to_string())), Some(5));
        assert_eq!(nickname_len(&Some(String::new())), Some(0));
    }

    #[test]
    fn reports_none_without_a_nickname() {
        let missing: Option<String> = None;
        assert_eq!(nickname_len(&missing), None);
        // The reference must still be usable — nothing was moved out of it.
        assert_eq!(nickname_len(&missing), None);
    }

    #[test]
    fn upper_cases_the_first_word() {
        assert_eq!(
            first_word_upper(&["hello".to_string(), "world".to_string()]),
            Some("HELLO".to_string())
        );
        assert_eq!(
            first_word_upper(&["single".to_string()]),
            Some("SINGLE".to_string())
        );
    }

    #[test]
    fn none_for_an_empty_slice() {
        assert_eq!(first_word_upper(&[]), None);
    }

    #[test]
    fn greets_by_nickname() {
        let matin = Profile {
            nickname: Some("Yui".to_string()),
        };
        assert_eq!(greeting(&matin), "Hey, Yui!");
    }

    #[test]
    fn greets_a_stranger() {
        let anon = Profile { nickname: None };
        assert_eq!(greeting(&anon), "Hey, stranger!");
    }
}
