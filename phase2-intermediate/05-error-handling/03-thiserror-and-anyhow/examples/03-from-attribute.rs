//! `#[from]` only works on a field that is the *entire* variant — nothing
//! else to fill in. `Io` qualifies, so a bare `?` converts a real
//! `std::io::Error` for you. `InvalidScore` also needs a `line` number that
//! no `std::io::Error`/`ParseIntError` value could ever supply, so it keeps
//! `#[source]` without `#[from]` — `?` cannot use it, `.map_err(...)` still
//! can.

#[derive(Debug, thiserror::Error)]
pub enum RatingsError {
    #[error("could not read ratings file: {0}")]
    Io(#[from] std::io::Error),

    #[error("line {line}: invalid score: {source}")]
    InvalidScore {
        line: usize,
        #[source]
        source: std::num::ParseIntError,
    },
}

fn read_len(path: &str) -> Result<usize, RatingsError> {
    let text = std::fs::read_to_string(path)?; // bare `?` — `#[from]` covers it
    Ok(text.len())
}

fn parse_score(line: usize, raw: &str) -> Result<u8, RatingsError> {
    raw.trim()
        .parse()
        .map_err(|source| RatingsError::InvalidScore { line, source }) // `line` needs stating by hand
}

fn main() {
    match read_len("no-such-ratings-file.txt") {
        Ok(n) => println!("length: {n}"),
        Err(e) => println!("read_len error: {e}"),
    }
    match parse_score(3, "oops") {
        Ok(v) => println!("score: {v}"),
        Err(e) => println!("parse_score error: {e}"),
    }
}
