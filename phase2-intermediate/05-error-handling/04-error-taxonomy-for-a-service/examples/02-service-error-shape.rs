//! The taxonomy itself: one `ServiceError` variant per *category*, not per
//! failure. `Validation` wraps its own sub-error, `NotFound` carries plain
//! data, `Internal` wraps a real `std::io::Error`.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 02-service-error-shape

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
    #[error("no entry with id {id}")]
    NotFound { id: u64 },
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}

fn main() {
    let bad_title = ServiceError::from(ValidationError::EmptyTitle);
    let bad_rating = ServiceError::from(ValidationError::RatingOutOfRange { rating: 15 });
    let missing = ServiceError::NotFound { id: 7 };

    println!("{bad_title}");
    println!("{bad_rating}");
    println!("{missing}");
}
