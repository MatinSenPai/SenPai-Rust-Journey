//! `Box<dyn Error>` as a type-erased "any error" container: one return
//! type that covers every concrete error type behind it, as long as each
//! one implements `std::error::Error`. This is the same `dyn Trait` /
//! type-erasure story 2.3.7 taught, applied to errors specifically.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 03-box-dyn-error-unifies-return-types

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
struct ConfigError(io::Error);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not read config file: {}", self.0)
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<io::Error> for ConfigError {
    fn from(source: io::Error) -> Self {
        ConfigError(source)
    }
}

/// How many bytes are in the file at `path`. Its only failure mode is
/// `ConfigError`, wrapping whatever `io::Error` reading it produced.
fn file_size(path: &str) -> Result<u64, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.len() as u64)
}

// Two genuinely unrelated concrete error types — `ConfigError` above, and
// `std::num::ParseIntError` straight from the standard library — both
// convert into `Box<dyn Error>` through the same bare `?`. Neither type
// knows the other exists.
fn run(path: &str, retry_count_text: &str) -> Result<(u64, u32), Box<dyn Error>> {
    let size = file_size(path)?;
    let retries: u32 = retry_count_text.parse()?;
    Ok((size, retries))
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join("senpai-2-5-2-example-03.cfg");
    std::fs::write(&path, "name=OnePieceTracker\n")?;

    let outcome = run(path.to_str().unwrap(), "5");
    let _ = std::fs::remove_file(&path);

    let (size, retries) = outcome?;
    println!("file was {size} bytes, retry count was {retries}");
    Ok(())
}
