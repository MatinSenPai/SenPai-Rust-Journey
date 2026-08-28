//! Putting both crates together, at the boundary described in "The
//! concept": `parse_line` is library-shaped — it returns the specific
//! `RatingsError` so a caller *could* `match` on which line failed and how.
//! `load_ratings` is binary-shaped — nothing calls it in turn, so it
//! collapses everything into `anyhow::Result` and adds one line of context.

use anyhow::Context;

#[derive(Debug, thiserror::Error)]
enum RatingsError {
    #[error("line {line}: missing score")]
    MissingScore { line: usize },
    #[error("line {line}: invalid score: {source}")]
    InvalidScore {
        line: usize,
        #[source]
        source: std::num::ParseIntError,
    },
}

fn parse_line(line: usize, text: &str) -> Result<(String, u8), RatingsError> {
    let (title, score) = text
        .split_once(',')
        .ok_or(RatingsError::MissingScore { line })?;
    let score = score
        .trim()
        .parse()
        .map_err(|source| RatingsError::InvalidScore { line, source })?;
    Ok((title.to_string(), score))
}

fn load_ratings(input: &str) -> anyhow::Result<Vec<(String, u8)>> {
    input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_line(index + 1, line))
        .collect::<Result<Vec<_>, RatingsError>>()
        .context("failed to load ratings")
}

fn main() {
    let good = "Frieren,96\nBocchi the Rock!,90";
    println!("{:?}", load_ratings(good));

    let bad = "Frieren,96\nBocchi the Rock!,oops";
    match load_ratings(bad) {
        Ok(ratings) => println!("{ratings:?}"),
        Err(err) => {
            println!("Display : {err}");
            println!("Debug   : {err:?}");
        }
    }
}
