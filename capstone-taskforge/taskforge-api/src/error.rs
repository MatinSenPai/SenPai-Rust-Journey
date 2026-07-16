use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use taskforge_core::JobError;

/// A single, consistent JSON error shape for every failure this API can
/// return: `{"error": "human-readable message"}`. Real systems often want
/// a machine-readable `code` field too (a stretch extension noted in the
/// README) — this keeps it simple on purpose, matching
/// `phase3-backend-foundations/07-error-handling-and-testing-at-scale/
/// 01-consistent-error-envelopes`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

/// Maps a domain-level `JobError` (from `taskforge-core`, storage-agnostic)
/// onto the right HTTP status — this mapping is exactly the kind of thing
/// that belongs at the API boundary, not inside `JobStore` implementations
/// themselves.
impl From<JobError> for ApiError {
    fn from(error: JobError) -> Self {
        match error {
            JobError::NotFound(id) => ApiError::not_found(format!("job not found: {id}")),
            JobError::NotCancellable(status) => {
                ApiError::conflict(format!("job cannot be cancelled in status {status}"))
            }
            JobError::Storage(message) => ApiError::internal(message),
            JobError::HandlerFailed(message) => ApiError::internal(message),
        }
    }
}
