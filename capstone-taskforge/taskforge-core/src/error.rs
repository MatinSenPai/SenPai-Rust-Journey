use thiserror::Error;

/// Errors a `JobHandler` can return, and errors a `JobStore` implementation
/// can return. Deliberately one shared error type for both — a real system
/// would likely split these (storage errors and handler errors are
/// different concerns); TaskForge keeps one for simplicity, and says so
/// here rather than pretending it's an oversight.
#[derive(Debug, Error)]
pub enum JobError {
    #[error("handler failed: {0}")]
    HandlerFailed(String),

    #[error("job not found: {0}")]
    NotFound(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("job is not in a cancellable state: {0}")]
    NotCancellable(String),
}
