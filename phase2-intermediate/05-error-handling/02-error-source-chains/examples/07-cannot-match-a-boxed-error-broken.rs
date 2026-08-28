//! DELIBERATELY BROKEN — expected: E0308
//!
//! Once an error is behind `Box<dyn Error>`, its concrete type is erased —
//! you cannot pattern-match it back into a specific enum's variants
//! without downcasting first. This is the real cost of the convenience
//! the previous examples showed.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 07-cannot-match-a-boxed-error-broken --features broken

use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    MissingField(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "could not read config file: {e}"),
            ConfigError::MissingField(field) => write!(f, "missing required field: {field}"),
        }
    }
}

impl Error for ConfigError {}

fn might_fail() -> Result<(), Box<dyn Error>> {
    Err(Box::new(ConfigError::MissingField("name".to_string())))
}

fn main() {
    let err = might_fail().unwrap_err();
    match err {
        ConfigError::Io(_) => println!("io"),
        ConfigError::MissingField(_) => println!("missing"),
    }
}
