//! Solution for 2.5.4 — designing an error taxonomy for a service.

use std::collections::HashMap;

/// Everything wrong with input the caller gave us. One variant per distinct
/// validation failure, so callers (and this crate's own tests) can `match`
/// on *which* rule was broken instead of reading a message.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("rating {rating} is out of range 0..=10")]
    RatingOutOfRange { rating: u8 },
}

/// The service's one error type: one variant per *category*, not per
/// failure. `Validation` wraps the category-specific sub-error above;
/// `NotFound` carries the plain data a caller needs; `Internal` wraps a real
/// [`std::io::Error`] behind a message generic enough to show externally.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Bad input. `#[from]` makes `?` convert a [`ValidationError`]
    /// automatically; `#[error(transparent)]` makes this variant's
    /// `Display` — and its `source()` — exactly the wrapped error's own,
    /// with no extra wrapper text.
    #[error(transparent)]
    Validation(#[from] ValidationError),

    /// No entry with this id exists. Simple data — no sub-error type earns
    /// its keep for a category this small.
    #[error("no entry with id {id}")]
    NotFound { id: u64 },

    /// Something failed on our side, not the caller's. The `Display` text
    /// is deliberately generic; the real [`std::io::Error`] is still
    /// reachable through `.source()` for whoever reads the logs.
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}

/// One entry in the watchlist: a title and a rating out of 10.
pub struct Entry {
    pub title: String,
    pub rating: u8,
}

/// An in-memory store of [`Entry`] values, keyed by an id it assigns itself.
pub struct WatchlistStore {
    entries: HashMap<u64, Entry>,
    next_id: u64,
}

/// Checks that `title` (trimmed) is non-empty and `rating` is at most `10`.
/// The title is checked first: if both are invalid, the error is
/// [`ValidationError::EmptyTitle`].
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}

impl WatchlistStore {
    /// A store holding no entries yet, with its first id ready to hand out
    /// at `0`.
    pub fn new() -> Self {
        WatchlistStore {
            entries: HashMap::new(),
            next_id: 0,
        }
    }

    /// Validates `title` and `rating` (see [`validate`]'s rule, propagated
    /// as [`ServiceError::Validation`]), then stores a new [`Entry`] under
    /// the next unused id and returns that id. The id assigned to the very
    /// first entry this store ever accepts — through `add_entry` or
    /// [`WatchlistStore::restore_from_file`] — is `0`; each entry after that
    /// gets the next integer, in the order it was accepted.
    pub fn add_entry(&mut self, title: &str, rating: u8) -> Result<u64, ServiceError> {
        validate(title, rating)?;
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(
            id,
            Entry {
                title: title.to_string(),
                rating,
            },
        );
        Ok(id)
    }

    /// The rating stored under `id`, or [`ServiceError::NotFound`] carrying
    /// that same `id` if no entry has it.
    pub fn rating_of(&self, id: u64) -> Result<u8, ServiceError> {
        self.entries
            .get(&id)
            .map(|entry| entry.rating)
            .ok_or(ServiceError::NotFound { id })
    }

    /// Reads the file at `path` and adds one entry per line shaped
    /// `title=rating`, where `rating` parses as a `u8`. Blank lines (after
    /// trimming) and lines that are not shaped `title=rating` with a
    /// parseable `rating` are skipped without error — this function has
    /// exactly one way to fail, and a malformed backup line is not it.
    /// Reading the file itself failing is the one thing that becomes
    /// [`ServiceError::Internal`]. Returns how many entries were added.
    pub fn restore_from_file(&mut self, path: &str) -> Result<usize, ServiceError> {
        let contents = std::fs::read_to_string(path)?;
        let mut added = 0;
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((title, rating_text)) = line.split_once('=') else {
                continue;
            };
            let Ok(rating) = rating_text.trim().parse::<u8>() else {
                continue;
            };

            let id = self.next_id;
            self.next_id += 1;
            self.entries.insert(
                id,
                Entry {
                    title: title.trim().to_string(),
                    rating,
                },
            );
            added += 1;
        }
        Ok(added)
    }
}

impl Default for WatchlistStore {
    fn default() -> Self {
        WatchlistStore::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_title_and_rating_pass() {
        assert!(validate("Frieren", 10).is_ok());
    }

    #[test]
    fn empty_title_is_rejected() {
        assert!(matches!(
            validate("  ", 5),
            Err(ValidationError::EmptyTitle)
        ));
    }

    #[test]
    fn out_of_range_rating_is_rejected() {
        assert!(matches!(
            validate("Bocchi", 11),
            Err(ValidationError::RatingOutOfRange { rating: 11 })
        ));
    }

    #[test]
    fn empty_title_wins_over_bad_rating_when_both_are_wrong() {
        assert!(matches!(validate("", 20), Err(ValidationError::EmptyTitle)));
    }

    #[test]
    fn add_entry_assigns_ids_starting_at_zero() {
        let mut store = WatchlistStore::new();
        assert_eq!(store.add_entry("Frieren", 10).unwrap(), 0);
        assert_eq!(store.add_entry("Bocchi", 8).unwrap(), 1);
    }

    #[test]
    fn add_entry_rejects_bad_input_as_a_validation_error() {
        let mut store = WatchlistStore::new();
        assert!(matches!(
            store.add_entry("", 5),
            Err(ServiceError::Validation(ValidationError::EmptyTitle))
        ));
    }

    #[test]
    fn rating_of_a_stored_entry_matches_what_was_added() {
        let mut store = WatchlistStore::new();
        let id = store.add_entry("Frieren", 10).unwrap();
        assert_eq!(store.rating_of(id).unwrap(), 10);
    }

    #[test]
    fn rating_of_an_unknown_id_is_not_found_with_that_id() {
        let store = WatchlistStore::new();
        assert!(matches!(
            store.rating_of(99),
            Err(ServiceError::NotFound { id: 99 })
        ));
    }

    #[test]
    fn restore_from_file_adds_well_shaped_lines_and_skips_the_rest() {
        let mut store = WatchlistStore::new();
        let path = std::env::temp_dir().join("p2-05-04-solution-restore-well-shaped.txt");
        std::fs::write(
            &path,
            "Frieren=10\nBocchi=8\n\nnot-a-pair\nAlso=notanumber\n",
        )
        .unwrap();

        let added = store.restore_from_file(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(added, 2);
        assert_eq!(store.rating_of(0).unwrap(), 10);
        assert_eq!(store.rating_of(1).unwrap(), 8);
    }

    #[test]
    fn restore_from_file_reports_a_missing_file_as_internal() {
        let mut store = WatchlistStore::new();
        let err = store
            .restore_from_file("p2-05-04-solution-this-path-does-not-exist.txt")
            .unwrap_err();
        assert!(matches!(err, ServiceError::Internal(_)));
    }

    #[test]
    fn internal_display_is_generic_but_source_has_the_real_detail() {
        let mut store = WatchlistStore::new();
        let err = store
            .restore_from_file("p2-05-04-solution-another-missing-path.txt")
            .unwrap_err();

        assert_eq!(err.to_string(), "internal error");
        assert!(std::error::Error::source(&err).is_some());
    }

    #[test]
    fn validation_display_is_transparent_not_generic() {
        let err = ServiceError::from(ValidationError::EmptyTitle);
        assert_eq!(err.to_string(), "title must not be empty");
    }
}
