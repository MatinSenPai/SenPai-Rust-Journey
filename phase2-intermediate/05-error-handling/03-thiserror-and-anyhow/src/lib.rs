//! Exercises for 2.5.3 — thiserror-generated errors and the anyhow boundary.
//!
//! `WatchNoteError` is written for you already: filling in `#[error("...")]`
//! text is not something a `todo!()` can ask for, since an attribute is not
//! executable code — it is exactly the mechanical part `thiserror` exists to
//! save you from typing. What is yours to write is the two functions that
//! *use* it: a library-shaped parser returning the specific error, and a
//! binary-shaped loader that collapses everything into `anyhow::Result` with
//! one line of context.

use anyhow::Context;

/// One parsed watch note: which episode, and a short comment.
#[derive(Debug, PartialEq)]
pub struct WatchNote {
    pub episode: u32,
    pub note: String,
}

/// Everything that can go wrong turning one line of text into a
/// [`WatchNote`]. The `#[error("...")]` messages below are the exact
/// `Display` text — read them, don't guess.
#[derive(Debug, thiserror::Error)]
pub enum WatchNoteError {
    /// No `:` separator was found anywhere on the line.
    #[error("line {line}: missing ':' separator between episode and note")]
    MissingSeparator { line: usize },
    /// The text before `:` did not parse as a `u32`.
    #[error("line {line}: '{episode}' is not a valid episode number: {source}")]
    InvalidEpisode {
        line: usize,
        episode: String,
        #[source]
        source: std::num::ParseIntError,
    },
}

/// Parses one line shaped `"<episode>:<note>"` (e.g. `"12:great fight
/// scene"`) into a [`WatchNote`]. `line` is the 1-based line number,
/// used only to build error messages. Both sides of `:` are trimmed of
/// surrounding whitespace before use.
///
/// # Errors
///
/// - No `:` anywhere on the line -> `WatchNoteError::MissingSeparator { line }`.
/// - The trimmed text before `:` does not parse as `u32` ->
///   `WatchNoteError::InvalidEpisode { line, episode, source }`, where
///   `episode` is that trimmed text and `source` is the underlying
///   [`std::num::ParseIntError`].
pub fn parse_watch_note(line: usize, text: &str) -> Result<WatchNote, WatchNoteError> {
    todo!(
        "split `text` on the first ':'; missing it means MissingSeparator; the trimmed left \
         side must parse as u32 (InvalidEpisode on failure); the trimmed right side becomes `note`"
    )
}

/// Parses every line of `input` into a `WatchNote`, in order, skipping
/// lines that are blank after trimming — but line numbers passed to
/// [`parse_watch_note`] always match a line's real 1-based position in
/// `input`, blank lines included.
///
/// On the first failing line, returns `Err` whose `Display` text is
/// exactly `"failed to load watch notes"`, with the specific
/// [`WatchNoteError`] still reachable as its source (`anyhow::Context`).
pub fn load_watch_notes(input: &str) -> anyhow::Result<Vec<WatchNote>> {
    todo!(
        "parse every non-blank line of `input` with parse_watch_note, using its real 1-based \
         line position; stop at the first failure; attach the context message documented above"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_line() {
        let note = parse_watch_note(1, "12:great fight scene").unwrap();
        assert_eq!(
            note,
            WatchNote {
                episode: 12,
                note: "great fight scene".to_string(),
            }
        );
    }

    #[test]
    fn trims_whitespace_on_both_sides_of_the_separator() {
        let note = parse_watch_note(1, "  12  :  great fight scene  ").unwrap();
        assert_eq!(note.episode, 12);
        assert_eq!(note.note, "great fight scene");
    }

    #[test]
    fn missing_separator_reports_the_line_number() {
        match parse_watch_note(7, "no colon here") {
            Err(WatchNoteError::MissingSeparator { line }) => assert_eq!(line, 7),
            other => panic!("expected MissingSeparator, got {other:?}"),
        }
    }

    #[test]
    fn missing_separator_display_matches_the_documented_text() {
        let err = parse_watch_note(7, "no colon here").unwrap_err();
        assert_eq!(
            err.to_string(),
            "line 7: missing ':' separator between episode and note"
        );
    }

    #[test]
    fn invalid_episode_reports_line_and_the_offending_text() {
        match parse_watch_note(3, "oops:great scene") {
            Err(WatchNoteError::InvalidEpisode { line, episode, .. }) => {
                assert_eq!(line, 3);
                assert_eq!(episode, "oops");
            }
            other => panic!("expected InvalidEpisode, got {other:?}"),
        }
    }

    #[test]
    fn invalid_episode_display_matches_the_documented_text() {
        let err = parse_watch_note(3, "oops:great scene").unwrap_err();
        assert_eq!(
            err.to_string(),
            "line 3: 'oops' is not a valid episode number: invalid digit found in string"
        );
    }

    #[test]
    fn invalid_episode_keeps_the_parse_error_as_its_source() {
        use std::error::Error;
        let err = parse_watch_note(3, "oops:great scene").unwrap_err();
        let source = err.source().expect("InvalidEpisode must carry a source");
        assert_eq!(source.to_string(), "invalid digit found in string");
    }

    #[test]
    fn load_watch_notes_parses_every_line_in_order() {
        let input = "1:opening\n2:midpoint twist\n3:finale";
        let notes = load_watch_notes(input).unwrap();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].episode, 1);
        assert_eq!(notes[2].note, "finale");
    }

    #[test]
    fn load_watch_notes_skips_blank_lines_but_keeps_real_line_numbers() {
        let input = "1:opening\n\noops:broken line";
        let err = load_watch_notes(input).unwrap_err();
        // Line 2 is blank and skipped; the failure is really on line 3.
        assert!(err.to_string() == "failed to load watch notes");
        let source = err
            .source()
            .expect("context should preserve the original error");
        assert!(source.to_string().starts_with("line 3:"));
    }

    #[test]
    fn load_watch_notes_wraps_the_failure_with_context() {
        let err = load_watch_notes("no colon here").unwrap_err();
        assert_eq!(err.to_string(), "failed to load watch notes");
        let source = err
            .source()
            .expect("context should preserve the original error");
        assert_eq!(
            source.to_string(),
            "line 1: missing ':' separator between episode and note"
        );
    }
}
