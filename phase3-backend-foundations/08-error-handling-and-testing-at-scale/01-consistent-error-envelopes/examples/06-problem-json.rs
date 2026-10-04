//! The same failure in two shapes: this lesson's envelope, and the RFC 9457
//! "problem details" format (`application/problem+json`).
//! Run: `cargo run -p p3-08-01-consistent-error-envelopes --example 06-problem-json`

use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};

fn envelope(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({"error": {"code": code, "message": message}})),
    )
}

fn problem(
    status: StatusCode,
    code: &str,
    title: &str,
    detail: &str,
    instance: &str,
) -> impl IntoResponse {
    let body = json!({
        "type": format!("https://api.example.com/problems/{code}"),
        "title": title,
        "status": status.as_u16(),
        "detail": detail,
        "instance": instance,
    });
    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        body.to_string(),
    )
}

async fn dump(label: &str, response: impl IntoResponse) {
    let response = response.into_response();
    let status = response.status().as_u16();
    let ct = response.headers()[header::CONTENT_TYPE]
        .to_str()
        .unwrap()
        .to_string();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    println!(
        "{label}: {status} {ct}\n  {}",
        String::from_utf8_lossy(&bytes)
    );
}

#[tokio::main]
async fn main() {
    let status = StatusCode::NOT_FOUND;
    dump(
        "envelope",
        envelope(status, "not_found", "show 9 not found"),
    )
    .await;
    dump(
        "problem ",
        problem(
            status,
            "not_found",
            "Not Found",
            "show 9 not found",
            "/shows/9",
        ),
    )
    .await;
}
