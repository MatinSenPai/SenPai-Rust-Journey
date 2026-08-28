//! Exercises for 2.2.3 — Consuming and collecting, including
//! `Result<Vec<_>, E>`.
//!
//! Every function below is meant to come out as a `.collect()` call (plus
//! whatever adapter feeds it) — no manual loop building up a `Vec` by hand,
//! no `HashMap::new()` plus a loop of `.insert()`s, and for the last one, no
//! hand-written `?` loop like the one you wrote in 1.6.3.

use std::collections::{HashMap, HashSet};

/// Doubles every episode count in `shows` that is at least `min_episodes`.
///
/// Walks `shows` in order and keeps only the counts that are `>=
/// min_episodes`; each kept count is then multiplied by two, in the same
/// relative order the counts appeared in `shows`. Counts below the minimum
/// are dropped entirely — not kept, not zeroed.
///
/// # Examples
///
/// `doubled_long_runs(&[12, 24, 6, 50], 12)` returns `vec![24, 48, 100]` —
/// `6` is dropped for being below the minimum, and each of the other three
/// is doubled.
pub fn doubled_long_runs(shows: &[u32], min_episodes: u32) -> Vec<u32> {
    todo!(
        "walk shows in order, keep only the counts that are at least min_episodes, double each \
         kept count, and gather the doubled counts into a Vec in the same relative order"
    )
}

/// The first letter of every title in `titles`, joined with nothing between them.
///
/// Walks `titles` in order and takes each one's first character, found the
/// way `chars().next()` finds it. An empty string contributes nothing to the
/// result — it has no first character to take. The characters are joined
/// directly, with no separator between them.
///
/// # Examples
///
/// `initials(&["Frieren", "Mob Psycho", "", "K-On!"])` returns
/// `"FMK".to_string()` — the empty string in the middle contributes nothing.
pub fn initials(titles: &[&str]) -> String {
    todo!(
        "walk titles in order, skip any empty string, take the first character of each \
         remaining title, and gather those characters directly into a String with nothing \
         between them"
    )
}

/// Builds a lookup from show title to episode count.
///
/// `entries` is a list of `(title, episode_count)` pairs. The result maps
/// each title to its episode count. If the same title appears more than
/// once, the value from whichever pair comes *last* in `entries` wins — the
/// same outcome you would get from inserting each pair in order.
///
/// # Examples
///
/// `episode_lookup(&[("Frieren".to_string(), 28), ("Nana".to_string(), 47)])`
/// returns a map with `"Frieren" -> 28` and `"Nana" -> 47`.
pub fn episode_lookup(entries: &[(String, u32)]) -> HashMap<String, u32> {
    todo!(
        "build a map from every title in entries to its episode count; if a title repeats, the \
         count from the pair that comes later in entries is the one that should survive"
    )
}

/// Every distinct genre mentioned across `shows`, with duplicates removed.
///
/// `shows` is a list of shows, each already given as a list of its own
/// genres. The result contains every genre that appears at least once, with
/// no particular order promised — it is a `HashSet`, so never rely on the
/// order you get back from it.
///
/// # Examples
///
/// `all_genres(&[vec!["comedy".to_string()], vec!["comedy".to_string(), "drama".to_string()]])`
/// returns a set containing exactly `"comedy"` and `"drama"` — `"comedy"`
/// only once even though it was listed twice.
pub fn all_genres(shows: &[Vec<String>]) -> HashSet<String> {
    todo!(
        "walk every genre across every show in shows, in any order, and gather them into a \
         HashSet so a repeated genre collapses into one entry"
    )
}

/// Parses every string in `inputs` as an `i32`, in order.
///
/// If every string parses successfully, returns `Ok` holding all the parsed
/// numbers, in the same order as `inputs`. If any string fails to parse,
/// returns `Err` holding that parse error's message (`.to_string()`) — the
/// message from the *first* string that fails, in order — and none of the
/// strings after it are parsed.
///
/// # Examples
///
/// `parse_all(&["1", "2", "3"])` returns `Ok(vec![1, 2, 3])`. `parse_all(&["1",
/// "x", "3"])` returns an `Err` holding `"x"`'s parse error message; `"3"` is
/// never parsed.
pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String> {
    todo!(
        "parse every string in inputs, in order, as an i32; if they all succeed, return Ok \
         holding all the parsed numbers in order; if one fails, return Err holding that parse \
         error's message and do not parse anything after it"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubled_long_runs_keeps_and_doubles_only_long_enough_runs() {
        assert_eq!(doubled_long_runs(&[12, 24, 6, 50], 12), vec![24, 48, 100]);
    }

    #[test]
    fn doubled_long_runs_threshold_is_inclusive() {
        assert_eq!(doubled_long_runs(&[5, 5, 5], 5), vec![10, 10, 10]);
    }

    #[test]
    fn doubled_long_runs_of_empty_input_is_empty() {
        assert_eq!(doubled_long_runs(&[], 5), Vec::<u32>::new());
    }

    #[test]
    fn doubled_long_runs_drops_everything_below_threshold() {
        assert_eq!(doubled_long_runs(&[1, 2, 3], 100), Vec::<u32>::new());
    }

    #[test]
    fn initials_takes_the_first_letter_of_each_title() {
        assert_eq!(initials(&["Frieren", "Mob Psycho", "", "K-On!"]), "FMK");
    }

    #[test]
    fn initials_of_empty_slice_is_empty_string() {
        assert_eq!(initials(&[]), "");
    }

    #[test]
    fn initials_skips_only_empty_strings() {
        assert_eq!(initials(&["", "ant", ""]), "a");
    }

    #[test]
    fn initials_of_single_title() {
        assert_eq!(initials(&["anime"]), "a");
    }

    #[test]
    fn episode_lookup_maps_every_pair() {
        let entries = [("Frieren".to_string(), 28), ("Nana".to_string(), 47)];
        let map = episode_lookup(&entries);
        assert_eq!(map.get("Frieren"), Some(&28));
        assert_eq!(map.get("Nana"), Some(&47));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn episode_lookup_last_write_wins_on_a_repeated_title() {
        let entries = [
            ("Frieren".to_string(), 1),
            ("Frieren".to_string(), 2),
            ("Frieren".to_string(), 3),
        ];
        let map = episode_lookup(&entries);
        assert_eq!(map.get("Frieren"), Some(&3));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn episode_lookup_of_empty_entries_is_empty_map() {
        assert!(episode_lookup(&[]).is_empty());
    }

    #[test]
    fn all_genres_collects_every_distinct_genre() {
        let shows = [
            vec!["comedy".to_string(), "slice of life".to_string()],
            vec!["comedy".to_string(), "drama".to_string()],
        ];
        let genres = all_genres(&shows);
        assert_eq!(genres.len(), 3);
        assert!(genres.contains("comedy"));
        assert!(genres.contains("slice of life"));
        assert!(genres.contains("drama"));
    }

    #[test]
    fn all_genres_of_no_shows_is_empty() {
        assert!(all_genres(&[]).is_empty());
    }

    #[test]
    fn all_genres_ignores_shows_with_no_genres() {
        let shows = [vec![], vec!["mecha".to_string()]];
        let genres = all_genres(&shows);
        assert_eq!(genres.len(), 1);
        assert!(genres.contains("mecha"));
    }

    #[test]
    fn parse_all_of_valid_input_returns_ok_in_order() {
        assert_eq!(parse_all(&["1", "2", "3"]), Ok(vec![1, 2, 3]));
    }

    #[test]
    fn parse_all_of_empty_input_is_ok_of_empty_vec() {
        assert_eq!(parse_all(&[]), Ok(vec![]));
    }

    #[test]
    fn parse_all_accepts_negative_numbers() {
        assert_eq!(parse_all(&["-5", "10"]), Ok(vec![-5, 10]));
    }

    #[test]
    fn parse_all_returns_the_first_parse_error() {
        let expected = "x".parse::<i32>().unwrap_err().to_string();
        assert_eq!(parse_all(&["1", "x", "3"]), Err(expected));
    }

    #[test]
    fn parse_all_reports_whichever_bad_string_comes_first() {
        let expected = "y".parse::<i32>().unwrap_err().to_string();
        assert_eq!(parse_all(&["y", "z"]), Err(expected));
    }
}
