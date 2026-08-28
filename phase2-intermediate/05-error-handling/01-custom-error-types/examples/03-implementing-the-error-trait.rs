//! `Error` needs `Debug` + `Display` — both already exist below — and
//! provides one method, `source()`, with a default that returns `None`.

use std::error::Error;

#[derive(Debug)]
pub enum ReviewError {
    MissingTitle,
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "review is missing a title")
    }
}

// The whole implementation: an empty body. `Debug` and `Display` are
// already satisfied above, and `source()` already has a default.
impl Error for ReviewError {}

/// Compiles only if `E` really implements `std::error::Error`.
fn assert_is_error<E: Error>() {}

fn main() {
    assert_is_error::<ReviewError>();
    let err = ReviewError::MissingTitle;
    println!("err.source(): {:?}", err.source());
}
