//! The payoff of a structured enum: a caller can `match` on *which*
//! variant happened and react differently — not just print a string.

#[derive(Debug)]
pub enum ReviewError {
    MissingTitle,
    InvalidScore(std::num::ParseIntError),
    ScoreOutOfRange(u8),
}

/// What to tell the person who submitted the review, per failure kind.
fn guidance(err: &ReviewError) -> &'static str {
    match err {
        ReviewError::MissingTitle => "ask them to add a title",
        ReviewError::InvalidScore(_) => "ask them to type digits only",
        ReviewError::ScoreOutOfRange(_) => "ask them for a score between 0 and 100",
    }
}

fn main() {
    let bad_score = "oops".parse::<u8>().unwrap_err();
    let errors = [
        ReviewError::MissingTitle,
        ReviewError::InvalidScore(bad_score),
        ReviewError::ScoreOutOfRange(150),
    ];
    for err in &errors {
        println!("{err:?} -> {}", guidance(err));
    }
}
