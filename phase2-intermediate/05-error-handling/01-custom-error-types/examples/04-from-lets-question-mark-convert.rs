//! 1.6.5 already taught this mechanism: `?` calls `From::from` on an `Err`
//! before returning it. One `impl From` here turns a bare `?` into the
//! whole `InvalidScore` conversion — no `.map_err(...)` needed.

#[derive(Debug, PartialEq)]
pub struct Review {
    pub title: String,
    pub score: u8,
}

#[derive(Debug)]
pub enum ReviewError {
    MissingTitle,
    InvalidScore(std::num::ParseIntError),
    ScoreOutOfRange(u8),
}

impl From<std::num::ParseIntError> for ReviewError {
    fn from(source: std::num::ParseIntError) -> Self {
        ReviewError::InvalidScore(source)
    }
}

/// Same behaviour as 01's `parse_review`, this time with `?`.
fn parse_review(line: &str) -> Result<Review, ReviewError> {
    let (title, score_str) = line.split_once(':').unwrap_or((line, ""));
    if title.is_empty() {
        return Err(ReviewError::MissingTitle);
    }
    let score: u8 = score_str.parse()?;
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
