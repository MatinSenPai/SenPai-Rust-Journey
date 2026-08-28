//! DELIBERATELY BROKEN — expected: E0599.
//!
//!     cargo run -p p2-05-01-custom-error-types --example 07-source-needs-the-trait-in-scope --features broken
//!
//! No `use std::error::Error;` anywhere in this file — `source()` is a
//! trait method, and calling one needs the trait in scope.

#[derive(Debug)]
pub struct ReviewError;

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "review is missing a title")
    }
}

impl std::error::Error for ReviewError {}

fn main() {
    let err = ReviewError;
    println!("{:?}", err.source());
}
