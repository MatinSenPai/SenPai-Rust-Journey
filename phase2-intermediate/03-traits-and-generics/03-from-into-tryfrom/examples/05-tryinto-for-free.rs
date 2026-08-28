//! Implementing `TryFrom<u8> for Rating` (see 04) is the only thing we
//! wrote. `TryInto` arrived on its own — the mirror of the blanket `Into`
//! impl 1.6.5 showed: `impl<T, U> TryInto<U> for T where U: TryFrom<T>`.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 05-tryinto-for-free

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

fn build(raw: u8) -> Result<Rating, RatingError> {
    // `?` already knew how to do this from 1.6.3/1.6.5 — nothing new here
    // except which trait it is now converting through.
    let rating: Rating = raw.try_into()?;
    Ok(rating)
}

fn main() {
    let from_call: Result<Rating, RatingError> = Rating::try_from(4);
    let from_method: Result<Rating, RatingError> = 4u8.try_into();
    println!("Rating::try_from(4): {from_call:?}");
    println!("4u8.try_into():      {from_method:?}");

    println!("build(3): {:?}", build(3));
    println!("build(7): {:?}", build(7));
}
