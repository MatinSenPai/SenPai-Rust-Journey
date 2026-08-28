//! Implementing `TryFrom` for your own type: the newtype's constructor
//! *is* the validation. Because the field is private, holding a `Rating`
//! is proof the check already happened — nobody can build one another way.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 04-tryfrom-for-your-own-type

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
    let five_star = Rating::try_from(5);
    let zero_star = Rating::try_from(0);
    let off_scale = Rating::try_from(9);

    println!("Rating::try_from(5): {five_star:?}");
    println!("Rating::try_from(0): {zero_star:?}");
    println!("Rating::try_from(9): {off_scale:?}");
}
