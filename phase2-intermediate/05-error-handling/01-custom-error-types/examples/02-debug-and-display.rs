//! `Debug` is mechanical and derived; `Display` is a decision you write by
//! hand — the same split 2.3.4 taught, now applied to an error type.

#[derive(Debug)]
pub enum ReviewError {
    MissingTitle,
    InvalidScore(std::num::ParseIntError),
    ScoreOutOfRange(u8),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewError::MissingTitle => write!(f, "review is missing a title"),
            ReviewError::InvalidScore(source) => write!(f, "invalid score: {source}"),
            ReviewError::ScoreOutOfRange(score) => {
                write!(f, "score {score} is out of range (must be 0-100)")
            }
        }
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
        println!("Display: {err}");
        println!("Debug:   {err:?}");
    }
}
