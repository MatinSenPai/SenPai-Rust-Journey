//! DELIBERATELY BROKEN — expected: E0271
//!
//! `?` only converts an error type automatically when a matching `From`
//! impl exists. `ConfigError` has no `From<ParseIntError>`, so mixing a
//! `ParseIntError`-producing step into a function pinned to
//! `Result<_, ConfigError>` does not compile. This is the exact pain
//! `Box<dyn Error>` exists to remove.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 06-cannot-convert-without-box-broken --features broken

use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config problem: {}", self.0)
    }
}

impl Error for ConfigError {}

fn read_retries(text: &str) -> Result<u32, ConfigError> {
    let value: u32 = text.parse()?;
    Ok(value)
}

fn main() {
    match read_retries("not-a-number") {
        Ok(v) => println!("{v}"),
        Err(e) => println!("{e}"),
    }
}
