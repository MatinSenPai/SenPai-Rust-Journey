//! Exercises for 2.2.2 — iterator adapters.
//!
//! Everything here is buildable from `.map()`, `.filter()`, `.fold()`,
//! `.take()` and `.enumerate()` — the adapters the lesson covers. No
//! `.collect()` anywhere: that is 2.2.3's job, so build each `Vec` yourself,
//! one push (or one `.fold()`) at a time.

/// One entry on a watchlist.
#[derive(Debug, Clone, PartialEq)]
pub struct Show {
    pub title: String,
    pub episodes: u32,
    pub completed: bool,
}

/// Returns a new `Vec` holding every show's title, uppercased, in the same
/// order as `shows`. Borrows `shows`; never consumes it.
///
/// # Examples
///
/// Given titles `"frieren"` and `"aot"`, returns `"FRIEREN"` and `"AOT"`, in
/// that order.
pub fn uppercase_titles(shows: &[Show]) -> Vec<String> {
    todo!("build a new Vec holding every show's title uppercased, in the same order as shows")
}

/// Returns the titles — borrowed, not cloned — of every show whose
/// `completed` field is `true`, in their original relative order. Shows
/// that are not completed are left out entirely.
///
/// # Examples
///
/// Given shows titled `"A"`, `"B"`, `"C"` with `completed` values `true`,
/// `false`, `true`, returns `"A"` and `"C"`, in that order.
pub fn completed_titles(shows: &[Show]) -> Vec<&str> {
    todo!("keep only the shows whose completed field is true, then gather their borrowed titles, in order")
}

/// Returns the sum of `episodes` across every completed show. A show whose
/// `completed` field is `false` never contributes to the total, no matter
/// how large its own `episodes` count is.
///
/// # Examples
///
/// Given episode counts `12`, `24`, `6` with `completed` values `true`,
/// `false`, `true`, returns `18` — the `24` is skipped.
pub fn total_episodes_watched(shows: &[Show]) -> u32 {
    todo!("add up the episodes field, counting only the shows whose completed field is true")
}

/// Returns the titles — borrowed, not cloned — of the first `n` shows (in
/// original order) whose `completed` field is `false`. Stops scanning as
/// soon as `n` such shows have been found; if fewer than `n` qualify,
/// returns every one that does.
///
/// # Examples
///
/// Given shows titled `"A"`, `"B"`, `"C"`, `"D"` with `completed` values
/// `false`, `true`, `false`, `false` and `n = 2`, returns `"A"` and `"C"` —
/// the first two not-yet-completed shows. `"D"` is never reached.
pub fn first_n_in_progress(shows: &[Show], n: usize) -> Vec<&str> {
    todo!(
        "keep only the shows whose completed field is false, then stop as soon as n of their \
         titles have been gathered"
    )
}

/// Pairs every show with its 1-based rank — its position in `shows`,
/// starting at 1, not 0 — and clones its title into the result.
///
/// # Examples
///
/// Given titles `"A"`, `"B"`, `"C"`, returns `(1, "A")`, `(2, "B")`,
/// `(3, "C")`.
pub fn ranked_titles(shows: &[Show]) -> Vec<(usize, String)> {
    todo!("pair each show with its 1-based position in shows, and clone its title into that pair")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(title: &str, episodes: u32, completed: bool) -> Show {
        Show {
            title: title.to_string(),
            episodes,
            completed,
        }
    }

    fn sample() -> Vec<Show> {
        vec![
            show("Frieren", 28, true),
            show("Bocchi the Rock!", 12, false),
            show("Mushoku Tensei", 24, true),
        ]
    }

    #[test]
    fn uppercase_titles_uppercases_every_title_in_order() {
        assert_eq!(
            uppercase_titles(&sample()),
            vec!["FRIEREN", "BOCCHI THE ROCK!", "MUSHOKU TENSEI"]
        );
    }

    #[test]
    fn uppercase_titles_of_empty_slice_is_empty() {
        assert_eq!(uppercase_titles(&[]), Vec::<String>::new());
    }

    #[test]
    fn completed_titles_keeps_only_completed_in_order() {
        assert_eq!(
            completed_titles(&sample()),
            vec!["Frieren", "Mushoku Tensei"]
        );
    }

    #[test]
    fn completed_titles_of_none_completed_is_empty() {
        let shows = vec![show("A", 1, false), show("B", 2, false)];
        let empty: Vec<&str> = vec![];
        assert_eq!(completed_titles(&shows), empty);
    }

    #[test]
    fn total_episodes_watched_sums_only_completed() {
        assert_eq!(total_episodes_watched(&sample()), 52);
    }

    #[test]
    fn total_episodes_watched_of_empty_slice_is_zero() {
        assert_eq!(total_episodes_watched(&[]), 0);
    }

    #[test]
    fn first_n_in_progress_stops_at_n_matches() {
        let shows = vec![
            show("A", 1, false),
            show("B", 2, true),
            show("C", 3, false),
            show("D", 4, false),
        ];
        assert_eq!(first_n_in_progress(&shows, 2), vec!["A", "C"]);
    }

    #[test]
    fn first_n_in_progress_with_fewer_matches_than_n_returns_all() {
        let shows = vec![show("A", 1, false), show("B", 2, true)];
        assert_eq!(first_n_in_progress(&shows, 5), vec!["A"]);
    }

    #[test]
    fn ranked_titles_starts_at_one() {
        let shows = vec![
            show("A", 1, false),
            show("B", 2, false),
            show("C", 3, false),
        ];
        assert_eq!(
            ranked_titles(&shows),
            vec![
                (1, "A".to_string()),
                (2, "B".to_string()),
                (3, "C".to_string()),
            ]
        );
    }

    #[test]
    fn ranked_titles_of_empty_slice_is_empty() {
        assert_eq!(ranked_titles(&[]), Vec::new());
    }
}
