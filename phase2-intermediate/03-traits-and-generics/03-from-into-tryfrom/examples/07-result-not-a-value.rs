//! DELIBERATELY BROKEN — expected: E0308.
//! `try_from`/`try_into` hand back a `Result`, not the target type itself
//! — unlike `from`/`into`, which hand back the target directly. Forgetting
//! this is the single most common `TryFrom` mistake.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 07-result-not-a-value --features broken

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rating(u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatingError {
    OutOfRange(u8),
}

impl TryFrom<u8> for Rating {
    type Error = RatingError;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        if (1..=5).contains(&raw) {
            Ok(Rating(raw))
        } else {
            Err(RatingError::OutOfRange(raw))
        }
    }
}

fn main() {
    let rating: Rating = 4u8.try_into();
    println!("{rating:?}");
}
