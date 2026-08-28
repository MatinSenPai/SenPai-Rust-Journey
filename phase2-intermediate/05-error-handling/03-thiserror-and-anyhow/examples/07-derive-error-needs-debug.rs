//! DELIBERATELY BROKEN — expected: E0277
//!
//! `std::error::Error` requires `Debug` (2.5.1 made you derive it by hand
//! for exactly this reason). `#[derive(thiserror::Error)]` generates the
//! `impl Error`, but it does not also derive `Debug` for you — that one is
//! still your own `#[derive(Debug)]` to add.
//!
//!     cargo run -p p2-05-03-thiserror-and-anyhow --example 07-derive-error-needs-debug --features broken

#[derive(thiserror::Error)]
pub enum RatingsError {
    #[error("line {0}: missing score")]
    MissingScore(usize),
}

fn main() {
    let err = RatingsError::MissingScore(3);
    println!("{err}");
}
