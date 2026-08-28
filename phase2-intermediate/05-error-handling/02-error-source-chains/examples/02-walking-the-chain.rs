//! Walk an error's cause chain: print the error, then keep calling
//! `.source()` and printing each link, until it returns `None`.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 02-walking-the-chain

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

fn load_config(path: &str) -> Result<String, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents)
}

fn main() {
    let Err(top) = load_config("definitely/does/not/exist/config.txt") else {
        unreachable!("this path is never created on purpose");
    };

    println!("error: {top}");
    let mut cause = top.source();
    while let Some(err) = cause {
        println!("caused by: {err}");
        cause = err.source();
    }
}
