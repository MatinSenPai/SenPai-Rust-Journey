use std::collections::{HashMap, HashSet};

/// Splits `text` on whitespace, lowercases each word, and counts how many
/// times each distinct word appears. Uses the entry API so each word is
/// looked up exactly once, whether it's the first time we've seen it or the
/// hundredth.
pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    todo!(
        "for word in text.split_whitespace() {{ *counts.entry(word.to_lowercase()).or_insert(0) += 1; }}"
    )
}

/// Returns the top-`n` (word, count) pairs from `freqs`, sorted by count
/// descending, then alphabetically ascending for ties. `HashMap` iteration
/// order is unspecified, so this function must collect and sort explicitly
/// rather than trusting whatever order `.iter()` happens to produce.
pub fn top_n(freqs: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    todo!(
        "collect freqs into a Vec<(String, usize)>, sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))), then truncate to n"
    )
}

/// Removes duplicates from `items` while preserving the order in which each
/// value was *first* seen. A `HashSet` tracks which values have already been
/// pushed to the output, so this runs in O(n) instead of the O(n^2) a
/// nested-loop "have I seen this before" check would cost.
///
/// Note this is deliberately *not* implemented as `items.sort(); items.dedup()`
/// — sorting would destroy the original order, which is the whole point here.
pub fn dedupe_preserve_order(items: Vec<i32>) -> Vec<i32> {
    todo!("let mut seen = HashSet::new(); keep items where seen.insert(item) returns true")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_word_frequency_case_insensitively() {
        let freqs = word_frequency("The the THE fox jumps");
        assert_eq!(freqs.get("the"), Some(&3));
        assert_eq!(freqs.get("fox"), Some(&1));
        assert_eq!(freqs.get("jumps"), Some(&1));
        assert_eq!(freqs.len(), 3);
    }

    #[test]
    fn word_frequency_of_empty_text_is_empty() {
        assert!(word_frequency("").is_empty());
    }

    #[test]
    fn top_n_sorts_by_count_descending_then_alphabetically() {
        let mut freqs = HashMap::new();
        freqs.insert("a".to_string(), 3);
        freqs.insert("b".to_string(), 5);
        freqs.insert("c".to_string(), 5);
        freqs.insert("d".to_string(), 1);

        assert_eq!(
            top_n(&freqs, 3),
            vec![
                ("b".to_string(), 5),
                ("c".to_string(), 5),
                ("a".to_string(), 3),
            ]
        );
    }

    #[test]
    fn top_n_returns_everything_if_n_exceeds_map_size() {
        let mut freqs = HashMap::new();
        freqs.insert("only".to_string(), 1);

        assert_eq!(top_n(&freqs, 10), vec![("only".to_string(), 1)]);
    }

    #[test]
    fn top_n_of_empty_map_is_empty() {
        let freqs: HashMap<String, usize> = HashMap::new();
        assert!(top_n(&freqs, 5).is_empty());
    }

    #[test]
    fn dedupe_preserve_order_keeps_first_occurrence_order() {
        assert_eq!(
            dedupe_preserve_order(vec![3, 1, 3, 2, 1, 4]),
            vec![3, 1, 2, 4]
        );
    }

    #[test]
    fn dedupe_preserve_order_of_empty_vec_is_empty() {
        assert!(dedupe_preserve_order(vec![]).is_empty());
    }

    #[test]
    fn dedupe_preserve_order_with_no_duplicates_is_unchanged() {
        assert_eq!(dedupe_preserve_order(vec![1, 2, 3]), vec![1, 2, 3]);
    }
}
