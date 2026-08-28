//! Exercises for 2.5.1 — custom error types and `std::error::Error`.
//!
//! One struct, one error enum, three things to write: `Display`, a `From`
//! impl, and the parsing function that ties them together. `impl Error for
//! EntryError {}` is given below, already complete — see why in the lesson.

use std::num::ParseIntError;

/// One parsed leaderboard entry: a player name and a score.
#[derive(Debug, PartialEq)]
pub struct LeaderboardEntry {
    pub name: String,
    pub score: u32,
}

/// Everything that can go wrong turning a line of text into a
/// [`LeaderboardEntry`]. One variant per distinct failure kind.
#[derive(Debug)]
pub enum EntryError {
    /// The name half, after trimming, was empty.
    BlankName,
    /// The score half, after trimming, did not parse as a `u32` at all.
    BadScore(ParseIntError),
    /// The score parsed fine but is bigger than the 9999 cap.
    ScoreTooHigh(u32),
}

/// The message shown to whoever submitted the line. States exactly:
/// - [`EntryError::BlankName`] -> `"entry is missing a name"`
/// - [`EntryError::BadScore`] -> `"invalid score: "` followed by the
///   wrapped [`ParseIntError`]'s own `Display` text
/// - [`EntryError::ScoreTooHigh`] -> `"score "`, the number, then
///   `" is above the maximum of 9999"`
impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!(
            "match self and write!(f, ...) the three exact messages documented above, one per variant"
        )
    }
}

/// `Debug` (derived above) and `Display` (just written) are both required
/// by `Error`, and both already exist — so the entire implementation is an
/// empty body. `source()` keeps its default, which returns `None`.
impl std::error::Error for EntryError {}

/// Lets a bare `?` on a `.parse::<u32>()` call become an
/// [`EntryError::BadScore`] automatically — the same mechanism 1.6.5 taught.
impl From<ParseIntError> for EntryError {
    fn from(source: ParseIntError) -> Self {
        todo!("wrap `source` in EntryError's score-parsing variant")
    }
}

/// Parses `line`, formatted as `"name:score"` (for example `"Matin:9001"`):
/// everything before the first `:` is the name, everything after is the
/// score. Both halves are trimmed of surrounding whitespace first; a line
/// with no `:` is treated as having an empty score half.
///
/// Checked in this order:
/// 1. The trimmed name must not be empty, or this returns
///    `Err(EntryError::BlankName)`.
/// 2. The trimmed score half must parse as a `u32`, or this returns
///    `Err(EntryError::BadScore(the_parse_error))`.
/// 3. The parsed score must be at most `9999`, or this returns
///    `Err(EntryError::ScoreTooHigh(the_parsed_score))`.
///
/// Otherwise, returns `Ok(LeaderboardEntry { name, score })` with the
/// trimmed name and the parsed score.
pub fn parse_entry(line: &str) -> Result<LeaderboardEntry, EntryError> {
    todo!("split on the first ':', trim both halves, then apply the three checks documented above")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_entry() {
        let entry = parse_entry("Matin:9001").unwrap();
        assert_eq!(entry.name, "Matin");
        assert_eq!(entry.score, 9001);
    }

    #[test]
    fn trims_whitespace_around_both_halves() {
        let entry = parse_entry(" Matin : 9001 ").unwrap();
        assert_eq!(entry.name, "Matin");
        assert_eq!(entry.score, 9001);
    }

    #[test]
    fn blank_name_is_rejected() {
        for line in [":9001", "   :9001"] {
            match parse_entry(line) {
                Err(EntryError::BlankName) => {}
                other => panic!("expected BlankName for {line:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn bad_score_is_reported_via_automatic_from() {
        match parse_entry("Matin:oops") {
            Err(EntryError::BadScore(_)) => {}
            other => panic!("expected BadScore, got {other:?}"),
        }
    }

    #[test]
    fn line_with_no_colon_has_an_empty_score_half() {
        match parse_entry("JustAName") {
            Err(EntryError::BadScore(_)) => {}
            other => panic!("expected BadScore, got {other:?}"),
        }
    }

    #[test]
    fn score_above_the_cap_is_rejected() {
        match parse_entry("Matin:10000") {
            Err(EntryError::ScoreTooHigh(10000)) => {}
            other => panic!("expected ScoreTooHigh(10000), got {other:?}"),
        }
    }

    #[test]
    fn nine_thousand_nine_hundred_ninety_nine_is_still_in_range() {
        let entry = parse_entry("Matin:9999").unwrap();
        assert_eq!(entry.score, 9999);
    }

    #[test]
    fn display_messages_are_exact() {
        assert_eq!(EntryError::BlankName.to_string(), "entry is missing a name");

        let source = "oops".parse::<u32>().unwrap_err();
        assert_eq!(
            EntryError::BadScore(source).to_string(),
            format!("invalid score: {}", "oops".parse::<u32>().unwrap_err())
        );

        assert_eq!(
            EntryError::ScoreTooHigh(10000).to_string(),
            "score 10000 is above the maximum of 9999"
        );
    }

    #[test]
    fn entry_error_is_a_proper_std_error() {
        fn assert_is_error<E: std::error::Error>() {}
        assert_is_error::<EntryError>();
    }
}
