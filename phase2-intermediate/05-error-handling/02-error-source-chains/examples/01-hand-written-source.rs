//! `source()` for real: an error variant that wraps another error exposes
//! it here, so a caller can look one level past the message you handed
//! them. The signature is fixed by the trait — `Option<&(dyn Error +
//! 'static)>` — not something you get to reword.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 01-hand-written-source

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    MissingField(String),
    InvalidNumber {
        field: String,
        source: ParseIntError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "could not read config file: {e}"),
            ConfigError::MissingField(field) => write!(f, "missing required field: {field}"),
            ConfigError::InvalidNumber { field, source } => {
                write!(f, "invalid number for field '{field}': {source}")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::MissingField(_) => None,
            ConfigError::InvalidNumber { source, .. } => Some(source),
        }
    }
}

fn report(label: &str, err: &ConfigError) {
    println!("{label}: {err}");
    match err.source() {
        Some(source) => println!("  source: Some({source})"),
        None => println!("  source: None"),
    }
}

fn main() {
    let bad_path = ConfigError::Io(io::Error::new(io::ErrorKind::NotFound, "no such file"));
    let missing = ConfigError::MissingField("name".to_string());
    let invalid = ConfigError::InvalidNumber {
        field: "max_retries".to_string(),
        source: "not-a-number".parse::<u32>().unwrap_err(),
    };

    report("bad_path", &bad_path);
    report("missing", &missing);
    report("invalid", &invalid);
}
