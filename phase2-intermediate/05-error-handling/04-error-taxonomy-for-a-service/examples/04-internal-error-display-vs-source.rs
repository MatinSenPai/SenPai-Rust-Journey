//! An `Internal` error's `Display` never shows the real `io::Error` text —
//! only a generic message safe to hand to an external caller. The real
//! detail is not thrown away: it is still reachable through `.source()`,
//! for whoever is reading the logs.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 04-internal-error-display-vs-source

use std::error::Error;

#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}

fn restore_from_file(path: &str) -> Result<String, ServiceError> {
    Ok(std::fs::read_to_string(path)?)
}

fn main() {
    let err = restore_from_file("definitely/does/not/exist.txt").unwrap_err();
    println!("shown to the caller: {err}");
    println!("logged for debugging: {}", err.source().unwrap());
}
