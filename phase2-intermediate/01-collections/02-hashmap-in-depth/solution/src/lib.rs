//! Solution — 2.1.2 `HashMap` in depth.

use std::collections::HashMap;

/// The value stored under `key` in `counts`, or `0` if `key` is not present.
///
/// # Examples
///
/// With `counts` containing `"a" -> 5`, `get_or_zero(counts, "a")` returns
/// `5`, and `get_or_zero(counts, "missing")` returns `0`.
pub fn get_or_zero(counts: &HashMap<String, u32>, key: &str) -> u32 {
    counts.get(key).copied().unwrap_or(0)
}

/// Counts how many times each word in `text` appears.
///
/// `text` is split exactly as `str::split_whitespace` splits it — any run of
/// whitespace separates words, and leading/trailing whitespace is ignored.
/// Each distinct token becomes a key; its value is how many times that exact
/// token occurs. Comparison is case-sensitive: `"Cat"` and `"cat"` are two
/// different keys. `text` with no words in it (empty, or all whitespace)
/// returns an empty map.
///
/// # Examples
///
/// `word_counts("the cat sat on the mat and the cat ate")` returns a map
/// with `"the" -> 3`, `"cat" -> 2`, and `"sat"`, `"on"`, `"mat"`, `"and"`,
/// `"ate"` each mapped to `1`.
pub fn word_counts(text: &str) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for word in text.split_whitespace() {
        counts
            .entry(word.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    counts
}

/// Groups `words` by their first character.
///
/// For every word in `words`, in order, an owned copy of that word is
/// appended to the list stored under its first character (found the way
/// `chars().next()` finds it). A key's list holds words in the same
/// relative order they appeared in `words`. An empty string contributes to
/// no key at all — it has no first character to group by.
///
/// # Examples
///
/// `group_by_first_letter(&["apple", "ant", "banana", "avocado", "berry"])`
/// returns a map with `'a' -> ["apple", "ant", "avocado"]` and
/// `'b' -> ["banana", "berry"]`.
pub fn group_by_first_letter(words: &[&str]) -> HashMap<char, Vec<String>> {
    let mut groups: HashMap<char, Vec<String>> = HashMap::new();
    for word in words {
        if let Some(letter) = word.chars().next() {
            groups
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(word.to_string());
        }
    }
    groups
}

/// Adjusts `key`'s value in `counts` by `amount`, in place.
///
/// If `key` is already present, `amount` is added to its current value. If
/// it is not, `key` is inserted with `amount` as its starting value. Nothing
/// is returned — `counts` is changed directly.
///
/// # Examples
///
/// Starting from a map with `"x" -> 5`: calling `bump_or_init(counts, "x", 3)`
/// leaves `"x" -> 8`. Calling `bump_or_init(counts, "y", -2)` on a map
/// without a `"y"` key inserts `"y" -> -2`.
pub fn bump_or_init(counts: &mut HashMap<String, i32>, key: &str, amount: i32) {
    counts
        .entry(key.to_string())
        .and_modify(|value| *value += amount)
        .or_insert(amount);
}

/// The key/value pair with the highest value in `counts`.
///
/// Returns `None` if `counts` is empty. When two or more keys share the
/// highest value, the one that is alphabetically first (by the ordering
/// `String`'s own comparison gives) is returned — a fixed rule, not
/// whichever one iteration happens to reach first, because `HashMap`
/// iteration order is not something you can rely on for that.
///
/// # Examples
///
/// `most_common` on a map with `"a" -> 3`, `"b" -> 5`, `"c" -> 5` returns
/// `Some(("b".to_string(), 5))` — `5` is the highest value, and between the
/// tied `"b"` and `"c"`, `"b"` sorts first.
pub fn most_common(counts: &HashMap<String, u32>) -> Option<(String, u32)> {
    let mut best: Option<(&String, &u32)> = None;
    for (key, value) in counts {
        let replace = match best {
            None => true,
            Some((best_key, best_value)) => {
                value > best_value || (value == best_value && key < best_key)
            }
        };
        if replace {
            best = Some((key, value));
        }
    }
    best.map(|(key, value)| (key.clone(), *value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_counts() -> HashMap<String, u32> {
        let mut counts = HashMap::new();
        counts.insert("a".to_string(), 5);
        counts
    }

    #[test]
    fn get_or_zero_finds_a_present_key() {
        assert_eq!(get_or_zero(&sample_counts(), "a"), 5);
    }

    #[test]
    fn get_or_zero_defaults_a_missing_key() {
        assert_eq!(get_or_zero(&sample_counts(), "missing"), 0);
    }

    #[test]
    fn word_counts_counts_repeats() {
        let counts = word_counts("the cat sat on the mat and the cat ate");
        assert_eq!(counts.get("the"), Some(&3));
        assert_eq!(counts.get("cat"), Some(&2));
        assert_eq!(counts.get("sat"), Some(&1));
        assert_eq!(counts.get("on"), Some(&1));
        assert_eq!(counts.get("mat"), Some(&1));
        assert_eq!(counts.get("and"), Some(&1));
        assert_eq!(counts.get("ate"), Some(&1));
        assert_eq!(counts.len(), 7);
    }

    #[test]
    fn word_counts_is_case_sensitive() {
        let counts = word_counts("Cat cat CAT");
        assert_eq!(counts.get("Cat"), Some(&1));
        assert_eq!(counts.get("cat"), Some(&1));
        assert_eq!(counts.get("CAT"), Some(&1));
    }

    #[test]
    fn word_counts_of_blank_text_is_empty() {
        assert!(word_counts("   ").is_empty());
        assert!(word_counts("").is_empty());
    }

    #[test]
    fn group_by_first_letter_buckets_and_preserves_order() {
        let words = ["apple", "ant", "banana", "avocado", "berry"];
        let groups = group_by_first_letter(&words);
        assert_eq!(
            groups.get(&'a'),
            Some(&vec![
                "apple".to_string(),
                "ant".to_string(),
                "avocado".to_string(),
            ])
        );
        assert_eq!(
            groups.get(&'b'),
            Some(&vec!["banana".to_string(), "berry".to_string()])
        );
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn group_by_first_letter_skips_empty_strings() {
        let words = ["", "ant", ""];
        let groups = group_by_first_letter(&words);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups.get(&'a'), Some(&vec!["ant".to_string()]));
    }

    #[test]
    fn bump_or_init_adds_to_an_existing_key() {
        let mut counts = HashMap::new();
        counts.insert("x".to_string(), 5);
        bump_or_init(&mut counts, "x", 3);
        assert_eq!(counts.get("x"), Some(&8));
    }

    #[test]
    fn bump_or_init_creates_a_missing_key() {
        let mut counts: HashMap<String, i32> = HashMap::new();
        bump_or_init(&mut counts, "y", -2);
        assert_eq!(counts.get("y"), Some(&-2));
    }

    #[test]
    fn most_common_breaks_ties_alphabetically() {
        let mut counts = HashMap::new();
        counts.insert("a".to_string(), 3);
        counts.insert("b".to_string(), 5);
        counts.insert("c".to_string(), 5);
        assert_eq!(most_common(&counts), Some(("b".to_string(), 5)));
    }

    #[test]
    fn most_common_finds_a_unique_maximum() {
        let mut counts = HashMap::new();
        counts.insert("x".to_string(), 1);
        counts.insert("y".to_string(), 9);
        counts.insert("z".to_string(), 2);
        assert_eq!(most_common(&counts), Some(("y".to_string(), 9)));
    }

    #[test]
    fn most_common_of_empty_map_is_none() {
        let counts: HashMap<String, u32> = HashMap::new();
        assert_eq!(most_common(&counts), None);
    }
}
