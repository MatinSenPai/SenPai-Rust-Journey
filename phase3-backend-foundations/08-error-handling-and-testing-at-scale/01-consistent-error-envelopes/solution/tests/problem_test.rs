//! The Challenge rung: the same `ApiError` as an RFC 9457 problem.

use axum::body::to_bytes;
use axum::http::{header, StatusCode};
use p3_08_01_consistent_error_envelopes_solution::{ApiError, FieldError};
use serde_json::{json, Value};

async fn parts(error: ApiError) -> (StatusCode, String, Value) {
    let response = error.problem_response("/shows/9");
    let status = response.status();
    let content_type = response.headers()[header::CONTENT_TYPE]
        .to_str()
        .unwrap()
        .to_string();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        content_type,
        serde_json::from_slice(&bytes).unwrap(),
    )
}

#[tokio::test]
async fn a_not_found_problem() {
    let (status, content_type, body) = parts(ApiError::NotFound("show 9 not found".into())).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(content_type, "application/problem+json");
    assert_eq!(
        body,
        json!({
            "type": "https://api.example.com/problems/not_found",
            "title": "Not Found",
            "status": 404,
            "detail": "show 9 not found",
            "instance": "/shows/9"
        })
    );
}

#[tokio::test]
async fn validation_adds_an_errors_extension() {
    let fields = vec![FieldError {
        field: "title",
        code: "length",
        message: "title must be 1 to 100 characters".into(),
    }];
    let (status, _, body) = parts(ApiError::Validation(fields)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "title");
    assert_eq!(body["detail"], "the request body has invalid fields");
}

#[tokio::test]
async fn an_internal_problem_leaks_nothing() {
    let (status, _, body) = parts(ApiError::Internal("password=hunter2".into())).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!body.to_string().contains("hunter2"));
}
