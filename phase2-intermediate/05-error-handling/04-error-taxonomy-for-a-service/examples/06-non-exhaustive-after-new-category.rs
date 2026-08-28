//! DELIBERATELY BROKEN — expected: E0004
//!
//! `describe` was written back when `ServiceError` only had two categories.
//! `Internal` was added to the taxonomy later, and every `match` that forgot
//! to grow with it is now a compile error, not a silent gap in behavior.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 06-non-exhaustive-after-new-category --features broken

#[derive(Debug, thiserror::Error)]
enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
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

fn describe(err: &ServiceError) -> &'static str {
    match err {
        ServiceError::Validation(_) => "bad input",
        ServiceError::NotFound { .. } => "not found",
    }
}

fn main() {
    println!("{}", describe(&ServiceError::NotFound { id: 1 }));
}
