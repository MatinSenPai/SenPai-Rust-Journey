//! The direction of a conversion decides whether it can fail. Turning a
//! raw `u8` into a `Rating` might be invalid input: `TryFrom`. Turning an
//! already-valid `Rating` back into a `u8` can't go wrong — it already
//! passed the check, so there is nothing left to reject: `From`.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 06-choosing-from-or-tryfrom

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

// The other direction never fails: every `Rating` that exists has already
// been checked, so unwrapping it back to a `u8` cannot reject anything.
impl From<Rating> for u8 {
    fn from(value: Rating) -> Self {
        value.0
    }
}

fn main() {
    let rating = Rating::try_from(4).unwrap();
    let raw: u8 = rating.into();
    println!("Rating::try_from(4).unwrap().into(): {raw}");
}
