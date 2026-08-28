//! Exercises for 1.7.1 — a guided mini-project: the watchlist.
//!
//! The lesson builds every type below in full — `Rating`, `Status`,
//! `Entry`, `WatchlistError` — because they are "the data": nothing about
//! them is left for you here. What is left is `Watchlist` itself: the
//! store that holds a `Vec<Entry>` and answers questions about it. Each
//! method's signature already states an ownership decision; the doc
//! comment above it states the rest.

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
    ///
    /// # Examples
    ///
    /// After `list.add(entry)`, `list.titles()` contains `entry`'s title.
    pub fn add(&mut self, entry: Entry) {
        todo!("store `entry` at the end of the list")
    }

    /// A look at the entry titled exactly `title`, or `None` if there is no
    /// such entry.
    ///
    /// # Examples
    ///
    /// On an empty list, `find("Cowboy Bebop")` is `None`.
    pub fn find(&self, title: &str) -> Option<&Entry> {
        todo!("search the entries for one whose title matches exactly, and hand back a reference to it")
    }

    /// The title of every entry, in the order they were added.
    ///
    /// # Examples
    ///
    /// On an empty list, `titles()` is `[]`.
    pub fn titles(&self) -> Vec<&str> {
        todo!("build a Vec of borrowed string slices, one per entry, in order")
    }

    /// Marks the entry titled `title` as [`Status::Finished`] with `rating`.
    ///
    /// On success, returns `Ok(())`. If no entry has that title, the list
    /// is unchanged and this returns
    /// `Err(WatchlistError::NotFound(title.to_string()))`.
    ///
    /// # Examples
    ///
    /// Rating a title that was just added succeeds. Rating a title that
    /// was never added returns `Err(WatchlistError::NotFound(_))`.
    pub fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
        todo!(
            "find the entry with this title and set its status to Finished with `rating`; if there is none, return the NotFound error instead"
        )
    }

    /// Parses `rating_text` as a whole number and rates the entry titled
    /// `title` with it, exactly like [`Watchlist::rate`] (so the value is
    /// clamped through [`Rating::new`]).
    ///
    /// If `rating_text` cannot be parsed as a `u8`, this returns
    /// `Err(WatchlistError::InvalidRating(_))` and never looks `title` up
    /// at all — a bad number is reported as a bad number, not as a missing
    /// title.
    ///
    /// # Examples
    ///
    /// `rate_from_text(title, "9")` succeeds if `title` exists.
    /// `rate_from_text(title, "nine")` is `Err(WatchlistError::InvalidRating(_))`,
    /// whether or not `title` exists.
    pub fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
        todo!(
            "parse rating_text as a u8, letting the ? operator convert a parse failure into a WatchlistError, then delegate to rate"
        )
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
        // "Cowboy Bebop" was never added — if the title were checked first
        // this would be NotFound instead.
        let outcome = list.rate_from_text("Cowboy Bebop", "nine");
        assert!(matches!(outcome, Err(WatchlistError::InvalidRating(_))));
    }
}
