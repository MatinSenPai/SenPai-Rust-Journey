//! The same shape of error type 2.5.1 and 2.5.2 taught you to hand-write:
//! one plain variant, one variant wrapping another error with `#[source]`
//! wired up by hand, and one `impl From<...>` for the clean case. Every line
//! here is code you already know how to write. Compare it to
//! `02-thiserror-derive.rs`.

use std::fmt;

#[derive(Debug)]
pub enum RatingsError {
    Io(std::io::Error),
    MissingScore {
        line: usize,
    },
    InvalidScore {
        line: usize,
        source: std::num::ParseIntError,
    },
}

impl fmt::Display for RatingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RatingsError::Io(source) => write!(f, "could not read ratings file: {source}"),
            RatingsError::MissingScore { line } => write!(f, "line {line}: missing score"),
            RatingsError::InvalidScore { line, source } => {
                write!(f, "line {line}: invalid score: {source}")
            }
        }
    }
}

impl std::error::Error for RatingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RatingsError::Io(source) => Some(source),
            RatingsError::MissingScore { .. } => None,
            RatingsError::InvalidScore { source, .. } => Some(source),
        }
    }
}

impl From<std::io::Error> for RatingsError {
    fn from(source: std::io::Error) -> Self {
        RatingsError::Io(source)
    }
}

fn print_with_source(err: &RatingsError) {
    println!("{err}");
    if let Some(source) = std::error::Error::source(err) {
        println!("  caused by: {source}");
    }
}

fn main() {
    print_with_source(&RatingsError::MissingScore { line: 2 });

    let source = "oops".parse::<u8>().unwrap_err();
    print_with_source(&RatingsError::InvalidScore { line: 5, source });

    let io_source = std::fs::read_to_string("no-such-ratings-file.txt").unwrap_err();
    print_with_source(&RatingsError::Io(io_source));
}
