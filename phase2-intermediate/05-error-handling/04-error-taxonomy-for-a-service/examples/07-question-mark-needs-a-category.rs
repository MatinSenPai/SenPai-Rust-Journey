//! DELIBERATELY BROKEN — expected: E0277
//!
//! `restore_from_file` reaches for a bare `?` on an `io::Error`, but this
//! draft of `ServiceError` has no `Internal` category yet — no
//! `From<std::io::Error>` exists for the compiler to call. `?` cannot invent
//! a category on your behalf; you have to design one in.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 07-question-mark-needs-a-category --features broken

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
}

fn restore_from_file(path: &str) -> Result<String, ServiceError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents)
}

fn main() {
    println!("{:?}", restore_from_file("nope.txt"));
}
