//! DELIBERATELY BROKEN — expected: E0046.
//! `TryFrom` has two required items, an `Error` type and a `try_from`
//! method — writing only the method is a genuinely common slip when you
//! hand-write the impl for a brand-new type instead of filling in a given
//! skeleton.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 09-missing-error-type --features broken

pub struct Rating(u8);

#[derive(Debug)]
pub enum RatingError {
    OutOfRange(u8),
}

impl TryFrom<u8> for Rating {
    fn try_from(raw: u8) -> Result<Self, RatingError> {
        if (1..=5).contains(&raw) {
            Ok(Rating(raw))
        } else {
            Err(RatingError::OutOfRange(raw))
        }
    }
}

fn main() {
    println!("{:?}", Rating::try_from(3).is_ok());
}
