//! Solution for 2.2.2 — iterator adapters.

/// One entry on a watchlist.
#[derive(Debug, Clone, PartialEq)]
pub struct Show {
    pub title: String,
    pub episodes: u32,
    pub completed: bool,
}

/// Returns a new `Vec` holding every show's title, uppercased, in the same
/// order as `shows`. Borrows `shows`; never consumes it.
pub fn uppercase_titles(shows: &[Show]) -> Vec<String> {
    shows.iter().fold(Vec::new(), |mut acc, show| {
        acc.push(show.title.to_uppercase());
        acc
    })
}

/// Returns the titles — borrowed, not cloned — of every show whose
/// `completed` field is `true`, in their original relative order.
pub fn completed_titles(shows: &[Show]) -> Vec<&str> {
    let mut out = Vec::new();
    for show in shows.iter().filter(|show| show.completed) {
        out.push(show.title.as_str());
    }
    out
}

/// Returns the sum of `episodes` across every completed show.
pub fn total_episodes_watched(shows: &[Show]) -> u32 {
    shows
        .iter()
        .filter(|show| show.completed)
        .fold(0, |acc, show| acc + show.episodes)
}

/// Returns the titles — borrowed, not cloned — of the first `n` shows (in
/// original order) whose `completed` field is `false`.
pub fn first_n_in_progress(shows: &[Show], n: usize) -> Vec<&str> {
    let mut out = Vec::new();
    for show in shows.iter().filter(|show| !show.completed).take(n) {
        out.push(show.title.as_str());
    }
    out
}

/// Pairs every show with its 1-based rank and clones its title into the
/// result.
pub fn ranked_titles(shows: &[Show]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (index, show) in shows.iter().enumerate() {
        out.push((index + 1, show.title.clone()));
    }
    out
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
