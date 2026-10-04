//! A silent bug: this compiles, runs, and has an envelope, but the 500 hands
//! the client the server's own error text. No gate needed.
//! Run: `cargo run -p p3-08-01-consistent-error-envelopes --example 04-leaky-500-trap`

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

enum ApiError {
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let ApiError::Internal(text) = self;
        let body = json!({"error": {"code": "internal_error", "message": text}});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

#[tokio::main]
async fn main() {
    let failure =
        ApiError::Internal("connect to postgres://anime:hunter2@db.internal:5432 refused".into());
    let response = failure.into_response();
    println!("{}", response.status());
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("{}", String::from_utf8_lossy(&bytes));
}
