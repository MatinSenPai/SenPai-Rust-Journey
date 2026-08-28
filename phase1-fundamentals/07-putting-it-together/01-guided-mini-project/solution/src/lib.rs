//! Reference solution for 1.7.1 — a guided mini-project: the watchlist.

use std::num::ParseIntError;

/// The largest rating a [`Rating`] can hold.
pub const MAX_RATING: u8 = 10;

/// A rating out of [`MAX_RATING`], guaranteed never to exceed it.
///
/// The only door in is [`Rating::new`], which clamps rather than fails —
/// nothing you pass it can produce a `Rating` greater than [`MAX_RATING`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rating(u8);

impl Rating {
    /// A `Rating` holding `value`, clamped to at most [`MAX_RATING`].
    ///
    /// `Rating::new(9)` holds `9`. `Rating::new(200)` holds [`MAX_RATING`].
    pub fn new(value: u8) -> Rating {
        if value > MAX_RATING {
            Rating(MAX_RATING)
        } else {
            Rating(value)
        }
    }

    /// The value inside, from `0` to [`MAX_RATING`].
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Where one watchlist entry stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Currently being watched, on this episode.
    Watching { episode: u32 },
    /// Finished, with a rating.
    Finished { rating: Rating },
    /// On the list, not started yet.
    Planned,
    /// Given up on, at this episode.
    Dropped { at: u32 },
}

/// One title in the watchlist, and where it stands.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub title: String,
    pub status: Status,
}

/// Everything that can go wrong asking a [`Watchlist`] to do something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchlistError {
    /// No entry has this title.
    NotFound(String),
    /// The rating text was not a whole number from 0 to 255.
    InvalidRating(String),
}

impl From<ParseIntError> for WatchlistError {
    fn from(err: ParseIntError) -> WatchlistError {
        WatchlistError::InvalidRating(err.to_string())
    }
}

/// An in-memory library of watchlist entries.
pub struct Watchlist {
    entries: Vec<Entry>,
}

impl Watchlist {
    /// An empty watchlist.
    pub fn new() -> Watchlist {
        Watchlist {
            entries: Vec::new(),
        }
    }

    /// Adds `entry` to the list. The list becomes its only owner.
    pub fn add(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// A look at the entry titled exactly `title`, or `None` if there is no
    /// such entry.
    pub fn find(&self, title: &str) -> Option<&Entry> {
        for entry in &self.entries {
            if entry.title == title {
                return Some(entry);
            }
        }
        None
    }

    /// The title of every entry, in the order they were added.
    pub fn titles(&self) -> Vec<&str> {
        let mut out = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            out.push(entry.title.as_str());
        }
        out
    }

    /// Marks the entry titled `title` as [`Status::Finished`] with `rating`.
    pub fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
        for entry in &mut self.entries {
            if entry.title == title {
                entry.status = Status::Finished { rating };
                return Ok(());
            }
        }
        Err(WatchlistError::NotFound(title.to_string()))
    }

    /// Parses `rating_text` as a whole number and rates the entry titled
    /// `title` with it, exactly like [`Watchlist::rate`].
    pub fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
        let raw: u8 = rating_text.trim().parse()?;
        self.rate(title, Rating::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planned(title: &str) -> Entry {
        Entry {
            title: title.to_string(),
            status: Status::Planned,
        }
    }

    #[test]
    fn a_new_list_has_no_titles() {
        let list = Watchlist::new();
        assert_eq!(list.titles(), Vec::<&str>::new());
        assert_eq!(list.find("Cowboy Bebop"), None);
    }

    #[test]
    fn add_makes_the_entry_findable_by_exact_title() {
        let mut list = Watchlist::new();
        list.add(planned("Cowboy Bebop"));

        let found = list.find("Cowboy Bebop").expect("just added");
        assert_eq!(found.title, "Cowboy Bebop");
        assert_eq!(found.status, Status::Planned);
        assert_eq!(
            list.find("cowboy bebop"),
            None,
            "titles are matched exactly"
        );
    }

    #[test]
    fn titles_come_back_in_insertion_order() {
        let mut list = Watchlist::new();
        list.add(planned("Cowboy Bebop"));
        list.add(planned("حمله به تایتان"));
        list.add(planned("Frieren"));

        assert_eq!(
            list.titles(),
            vec!["Cowboy Bebop", "حمله به تایتان", "Frieren"]
        );
    }

    #[test]
    fn rate_finishes_an_existing_entry() {
        let mut list = Watchlist::new();
        list.add(planned("Frieren"));

        let outcome = list.rate("Frieren", Rating::new(9));
        assert_eq!(outcome, Ok(()));
        assert_eq!(
            list.find("Frieren").unwrap().status,
            Status::Finished {
                rating: Rating::new(9)
            }
        );
    }

    #[test]
    fn rate_reports_a_missing_title_without_changing_the_list() {
        let mut list = Watchlist::new();
        list.add(planned("Frieren"));

        let outcome = list.rate("Cowboy Bebop", Rating::new(9));
        assert_eq!(
            outcome,
            Err(WatchlistError::NotFound("Cowboy Bebop".to_string()))
        );
        assert_eq!(list.titles(), vec!["Frieren"]);
    }

    #[test]
    fn rate_from_text_parses_then_clamps() {
        let mut list = Watchlist::new();
        list.add(planned("Frieren"));

        assert_eq!(list.rate_from_text("Frieren", "9"), Ok(()));
        assert_eq!(
            list.find("Frieren").unwrap().status,
            Status::Finished {
                rating: Rating::new(9)
            }
        );

        assert_eq!(list.rate_from_text("Frieren", "255"), Ok(()));
        assert_eq!(
            list.find("Frieren").unwrap().status,
            Status::Finished {
                rating: Rating::new(MAX_RATING)
            }
        );
    }

    #[test]
    fn rate_from_text_reports_bad_text_before_checking_the_title() {
        let mut list = Watchlist::new();
        let outcome = list.rate_from_text("Cowboy Bebop", "nine");
        assert!(matches!(outcome, Err(WatchlistError::InvalidRating(_))));
    }
}
