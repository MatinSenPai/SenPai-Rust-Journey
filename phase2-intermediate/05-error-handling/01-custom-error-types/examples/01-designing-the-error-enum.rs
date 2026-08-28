//! One enum, one variant per distinct failure kind — not a single
//! catch-all `String` message.

/// A single parsed review: a title and a score out of 100.
#[derive(Debug, PartialEq)]
pub struct Review {
    pub title: String,
    pub score: u8,
}

/// Everything that can go wrong turning a line of text into a [`Review`].
#[derive(Debug)]
pub enum ReviewError {
    /// The part before `:` was empty.
    MissingTitle,
    /// The part after `:` did not parse as a `u8` at all.
    InvalidScore(std::num::ParseIntError),
    /// It parsed fine, but is bigger than 100.
    ScoreOutOfRange(u8),
}

/// Parses `"title:score"`. Written the manual way, one `match` per
/// fallible step — 04 rewrites this with `?` once `From` exists.
fn parse_review(line: &str) -> Result<Review, ReviewError> {
    let (title, score_str) = line.split_once(':').unwrap_or((line, ""));
    if title.is_empty() {
        return Err(ReviewError::MissingTitle);
    }
    let score: u8 = match score_str.parse() {
        Ok(score) => score,
        Err(source) => return Err(ReviewError::InvalidScore(source)),
    };
    if score > 100 {
        return Err(ReviewError::ScoreOutOfRange(score));
    }
    Ok(Review {
        title: title.to_string(),
        score,
    })
}

fn main() {
    for line in ["Frieren:96", ":90", "Bocchi:oops", "Bocchi:150"] {
        println!("{line:?} -> {:?}", parse_review(line));
    }
}
