//! DELIBERATELY BROKEN — expected: E0004
//! Run `cargo run -p p3-08-01-consistent-error-envelopes --example 02-missing-arm-broken --features broken`
//! and read the error. A new variant was added; `status` was not updated.

#[allow(dead_code)]
enum ApiError {
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl ApiError {
    fn status(&self) -> u16 {
        match self {
            ApiError::NotFound(_) => 404,
            ApiError::Internal(_) => 500,
        }
    }
}

fn main() {
    println!("{}", ApiError::NotFound("x".into()).status());
}
