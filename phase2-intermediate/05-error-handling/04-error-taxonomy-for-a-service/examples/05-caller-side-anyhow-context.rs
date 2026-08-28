//! At the binary boundary, `anyhow::Context` adds a human-readable message
//! on top of whatever `ServiceError` category actually happened — the
//! `thiserror`-in-the-library / `anyhow`-at-the-boundary split 2.5.3 taught,
//! applied to this lesson's taxonomy instead of a single flat error type.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 05-caller-side-anyhow-context

use anyhow::Context;

#[derive(Debug, thiserror::Error)]
enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
}

#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

fn add_entry(title: &str) -> Result<(), ServiceError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle.into());
    }
    Ok(())
}

fn seed_startup_data() -> anyhow::Result<()> {
    add_entry("").context("failed to seed the watchlist on startup")?;
    Ok(())
}

fn main() {
    if let Err(err) = seed_startup_data() {
        println!("{err:?}");
    }
}
