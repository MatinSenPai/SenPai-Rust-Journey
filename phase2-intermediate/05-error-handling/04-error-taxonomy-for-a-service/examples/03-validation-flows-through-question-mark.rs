//! `?` converts a `ValidationError` into a `ServiceError` on its own, because
//! `Validation` is marked `#[from]` — the same mechanism 2.5.1 taught, one
//! level deeper: a nested error converting into the taxonomy that wraps it.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 03-validation-flows-through-question-mark

#[derive(Debug, thiserror::Error)]
enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("rating {rating} is out of range 0..=10")]
    RatingOutOfRange { rating: u8 },
}

#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}

fn add_entry(title: &str, rating: u8) -> Result<(), ServiceError> {
    validate(title, rating)?;
    println!("stored: {title} ({rating}/10)");
    Ok(())
}

fn main() {
    if let Err(err) = add_entry("Frieren", 10) {
        println!("unexpected: {err}");
    }
    if let Err(err) = add_entry("", 5) {
        println!("rejected: {err}");
    }
    if let Err(err) = add_entry("Bocchi", 15) {
        println!("rejected: {err}");
    }
}
