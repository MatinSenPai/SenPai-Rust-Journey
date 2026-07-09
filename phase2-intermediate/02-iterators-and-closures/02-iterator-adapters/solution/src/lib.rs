/// Returns a new `Vec` with every title uppercased. Borrows `titles`
/// (never consumes it) and produces owned `String`s.
pub fn titles_uppercased(titles: &[String]) -> Vec<String> {
    titles.iter().map(|t| t.to_uppercase()).collect()
}

/// Returns every title whose length is at least `min_len`, as borrowed
/// `&str` slices into the original `titles` -- no cloning needed since the
/// caller already owns `titles` and we're just handing back references
/// into it. (Lifetimes are covered properly in Phase 2 module 04; for now
/// just notice that `'a` ties the returned `&str`s to how long `titles`
/// itself stays alive.)
///
/// The `'a` is written out explicitly for teaching purposes -- Rust's
/// elision rules would actually infer it automatically here (one input
/// reference in, one reference out, so they're assumed to share a
/// lifetime), which is exactly why clippy flags it and why the allow below
/// is needed. Module 04 explains elision properly.
#[allow(clippy::needless_lifetimes)]
pub fn long_titles<'a>(titles: &'a [String], min_len: usize) -> Vec<&'a str> {
    titles
        .iter()
        .filter(|t| t.len() >= min_len)
        .map(|t| t.as_str())
        .collect()
}

/// Sums the episode count across every `(title, episode_count)` pair.
pub fn total_episodes(shows: &[(String, u32)]) -> u32 {
    shows.iter().map(|(_, episodes)| episodes).sum()
}

/// Returns the titles of the first `n` shows (in original order) whose
/// `bool` flag is `true` ("completed"). Stops scanning as soon as `n`
/// matches have been found.
///
/// Same note as `long_titles` above: `'a` is spelled out explicitly here
/// for practice, even though elision would infer it.
#[allow(clippy::needless_lifetimes)]
pub fn first_n_completed<'a>(shows: &'a [(String, bool)], n: usize) -> Vec<&'a str> {
    shows
        .iter()
        .filter(|(_, done)| *done)
        .map(|(title, _)| title.as_str())
        .take(n)
        .collect()
}

/// Pairs every title with its 1-based rank (its position in the input,
/// starting at 1, not 0) and clones each title into the result.
pub fn ranked_titles(titles: &[String]) -> Vec<(usize, String)> {
    titles
        .iter()
        .enumerate()
        .map(|(i, t)| (i + 1, t.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_titles() -> Vec<String> {
        vec![
            "frieren".to_string(),
            "mob psycho 100".to_string(),
            "aot".to_string(),
        ]
    }

    #[test]
    fn uppercases_every_title() {
        assert_eq!(
            titles_uppercased(&sample_titles()),
            vec!["FRIEREN", "MOB PSYCHO 100", "AOT"]
        );
    }

    #[test]
    fn uppercases_empty_slice() {
        let empty: Vec<String> = vec![];
        assert_eq!(titles_uppercased(&empty), Vec::<String>::new());
    }

    #[test]
    fn filters_long_titles() {
        assert_eq!(
            long_titles(&sample_titles(), 5),
            vec!["frieren", "mob psycho 100"]
        );
    }

    #[test]
    fn filters_long_titles_with_high_threshold() {
        let empty: Vec<&str> = vec![];
        assert_eq!(long_titles(&sample_titles(), 100), empty);
    }

    #[test]
    fn sums_episode_counts() {
        let shows = vec![
            ("frieren".to_string(), 28),
            ("mob psycho 100".to_string(), 37),
            ("aot".to_string(), 87),
        ];
        assert_eq!(total_episodes(&shows), 152);
    }

    #[test]
    fn sums_episode_counts_of_empty_slice() {
        assert_eq!(total_episodes(&[]), 0);
    }

    #[test]
    fn takes_first_n_completed() {
        let shows = vec![
            ("frieren".to_string(), true),
            ("mob psycho 100".to_string(), false),
            ("aot".to_string(), true),
            ("chainsaw man".to_string(), true),
        ];
        assert_eq!(first_n_completed(&shows, 2), vec!["frieren", "aot"]);
    }

    #[test]
    fn takes_first_n_completed_when_fewer_than_n_match() {
        let shows = vec![
            ("frieren".to_string(), true),
            ("mob psycho 100".to_string(), false),
        ];
        assert_eq!(first_n_completed(&shows, 5), vec!["frieren"]);
    }

    #[test]
    fn ranks_titles_starting_at_one() {
        assert_eq!(
            ranked_titles(&sample_titles()),
            vec![
                (1, "frieren".to_string()),
                (2, "mob psycho 100".to_string()),
                (3, "aot".to_string()),
            ]
        );
    }
}
