//! Reference solution for 1.5.1 — Structs and methods.
//!
//! `Series` is `pub`; its four fields are not. The tests sit inside this
//! file, so they can read the fields directly — anything outside the file
//! only ever sees the methods. That split is the thing you are building.

/// A series on the watch list.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}

impl Series {
    /// A series just added to the list.
    ///
    /// `title` and `episodes` are stored as given, nothing has been watched
    /// yet, and it is not a favourite.
    ///
    /// # Examples
    ///
    /// `Series::new("Frieren".to_string(), 28)` holds the title `"Frieren"`,
    /// 28 episodes, 0 watched, and is not a favourite.
    pub fn new(title: String, episodes: u32) -> Self {
        Self {
            title,
            episodes,
            watched: 0,
            favourite: false,
        }
    }

    /// How many episodes are still unwatched.
    ///
    /// Never below zero: a series whose watched count has somehow passed its
    /// own length has `0` remaining, not a number wrapped around the bottom
    /// of a `u32`.
    ///
    /// # Examples
    ///
    /// 28 episodes with 3 watched leaves `25`.
    /// 28 episodes with 28 watched leaves `0`.
    /// 28 episodes with 30 watched also leaves `0`.
    pub fn remaining(&self) -> u32 {
        self.episodes.saturating_sub(self.watched)
    }

    /// Watch one more episode.
    ///
    /// When there was an episode left, the watched count goes up by exactly
    /// one and the answer is `true`. When the series was already finished,
    /// nothing changes at all and the answer is `false`.
    ///
    /// # Examples
    ///
    /// On a 2-episode series with 1 watched: `true`, and the count becomes 2.
    /// Calling it again: `false`, and the count stays at 2.
    pub fn watch_one(&mut self) -> bool {
        if self.watched < self.episodes {
            self.watched += 1;
            true
        } else {
            false
        }
    }

    /// Mark this series as a favourite.
    ///
    /// Marking one that is already a favourite is not an error and changes
    /// nothing.
    pub fn mark_favourite(&mut self) {
        self.favourite = true;
    }

    /// A one-line description: the title, a space, then the progress as
    /// `watched/episodes`. A favourite gets a trailing ` (favourite)`.
    ///
    /// # Examples
    ///
    /// A 28-episode `"Frieren"` with 3 watched, not a favourite, gives
    /// `"Frieren 3/28"`.
    /// The same series once marked gives `"Frieren 3/28 (favourite)"`.
    /// A brand new 12-episode `"Mushishi"` gives `"Mushishi 0/12"`.
    pub fn summary(&self) -> String {
        let mut line = format!("{} {}/{}", self.title, self.watched, self.episodes);
        if self.favourite {
            line.push_str(" (favourite)");
        }
        line
    }

    /// The title, taking the whole series with it.
    ///
    /// The caller cannot use the series afterwards — that is the point of
    /// the signature.
    ///
    /// # Examples
    ///
    /// `Series::new("Frieren".to_string(), 28).into_title()` is `"Frieren"`.
    pub fn into_title(self) -> String {
        self.title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frieren() -> Series {
        Series::new("Frieren".to_string(), 28)
    }

    #[test]
    fn a_new_series_starts_at_the_beginning() {
        let show = frieren();
        assert_eq!(show.title, "Frieren");
        assert_eq!(show.episodes, 28);
        assert_eq!(show.watched, 0);
        assert!(!show.favourite);
    }

    #[test]
    fn remaining_counts_down_and_stops_at_zero() {
        let mut show = frieren();
        assert_eq!(show.remaining(), 28);

        show.watched = 3;
        assert_eq!(show.remaining(), 25);

        show.watched = 28;
        assert_eq!(show.remaining(), 0);

        show.watched = 30;
        assert_eq!(
            show.remaining(),
            0,
            "never wraps around the bottom of a u32"
        );
    }

    #[test]
    fn watching_advances_by_one_and_reports_whether_it_did() {
        let mut show = Series::new("Short".to_string(), 2);
        show.watched = 1;

        assert!(show.watch_one());
        assert_eq!(show.watched, 2);

        assert!(!show.watch_one());
        assert_eq!(show.watched, 2, "a finished series does not move");
    }

    #[test]
    fn marking_a_favourite_is_idempotent() {
        let mut show = frieren();
        show.mark_favourite();
        assert!(show.favourite);

        show.mark_favourite();
        assert!(show.favourite);
    }

    #[test]
    fn the_summary_reads_like_a_line_of_a_list() {
        let mut show = frieren();
        show.watched = 3;
        assert_eq!(show.summary(), "Frieren 3/28");

        show.mark_favourite();
        assert_eq!(show.summary(), "Frieren 3/28 (favourite)");

        assert_eq!(
            Series::new("Mushishi".to_string(), 12).summary(),
            "Mushishi 0/12"
        );
    }

    #[test]
    fn the_title_can_be_taken_out() {
        assert_eq!(frieren().into_title(), "Frieren");
    }

    #[test]
    fn two_series_built_the_same_way_are_equal() {
        assert_eq!(frieren(), frieren());
        assert_ne!(frieren(), Series::new("Frieren".to_string(), 26));
    }
}
