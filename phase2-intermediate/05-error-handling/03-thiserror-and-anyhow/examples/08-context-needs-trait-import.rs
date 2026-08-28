//! DELIBERATELY BROKEN — expected: E0599
//!
//! `.context(...)` is not a method on `Result` itself — it is a method on
//! the `anyhow::Context` trait, added to `Result<T, E>` for any
//! `E: std::error::Error`. Without `use anyhow::Context;` in scope, the
//! method genuinely does not exist yet.
//!
//!     cargo run -p p2-05-03-thiserror-and-anyhow --example 08-context-needs-trait-import --features broken

fn parse_score(raw: &str) -> Result<u8, std::num::ParseIntError> {
    raw.trim().parse()
}

fn load_score(raw: &str) -> anyhow::Result<u8> {
    parse_score(raw).context("failed to load score")
}

fn main() {
    println!("{:?}", load_score("87"));
}
