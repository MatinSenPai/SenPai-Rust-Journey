//! DELIBERATELY BROKEN — expected: impl signature mismatch (rustc gives no
//! error code for this one)
//!
//! `source()`'s return type is fixed by the trait as
//! `Option<&(dyn Error + 'static)>`. Writing `Option<&dyn Error>` instead
//! elides a *different* lifetime — tied to `&self` — which is not the same
//! signature, and the compiler rejects the `impl` outright.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 05-source-wrong-lifetime-broken --features broken

use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ConfigError {
    inner: std::io::Error,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not read config file: {}", self.inner)
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&dyn Error> {
        Some(&self.inner)
    }
}

fn main() {
    let err = ConfigError {
        inner: std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
    };
    println!("{err}");
}
