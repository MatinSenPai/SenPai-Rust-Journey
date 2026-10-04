//! The fix in miniature: three variants, one `IntoResponse`, and the `Json`
//! rejection routed through it. Two gaps are left on purpose: look at the
//! `/shows/abc` line and the `DELETE` line.
//! Run: `cargo run -p p3-08-01-consistent-error-envelopes --example 05-one-envelope`

use axum::body::{to_bytes, Body};
use axum::extract::Path;
use axum::http::{header, Request, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tower::ServiceExt;

enum ApiError {
    NotFound(String),
    Rejected(StatusCode, String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m),
            ApiError::Rejected(s, m) => (s, "rejected", m),
            ApiError::Internal(m) => {
                eprintln!("internal error: {m}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "something went wrong on our side".into(),
                )
            }
        };
        (
            status,
            Json(json!({"error": {"code": code, "message": message}})),
        )
            .into_response()
    }
}

#[derive(axum::extract::FromRequest)]
#[from_request(via(Json), rejection(ApiError))]
struct ApiJson<T>(T);

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(r: axum::extract::rejection::JsonRejection) -> Self {
        ApiError::Rejected(r.status(), r.body_text())
    }
}

#[derive(Deserialize)]
struct NewShow {
    #[allow(dead_code)]
    title: String,
}

async fn create(ApiJson(_): ApiJson<NewShow>) -> StatusCode {
    StatusCode::CREATED
}

async fn one(Path(id): Path<u64>) -> Result<(), ApiError> {
    Err(ApiError::NotFound(format!("show {id} not found")))
}

async fn boom() -> Result<(), ApiError> {
    Err(ApiError::Internal(
        "connect to postgres://anime:hunter2@db.internal refused".into(),
    ))
}

async fn fallback(uri: Uri) -> ApiError {
    ApiError::NotFound(format!("no route for {}", uri.path()))
}

async fn show(app: &Router, method: &str, uri: &str, ct: Option<&str>, body: &str) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(ct) = ct {
        req = req.header(header::CONTENT_TYPE, ct);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = res.status().as_u16();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    println!(
        "{method} {uri} -> {status} {}",
        String::from_utf8_lossy(&bytes)
    );
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/shows", post(create))
        .route("/shows/{id}", get(one))
        .route("/boom", get(boom))
        .fallback(fallback);
    let json = Some("application/json");
    show(&app, "GET", "/shows/9", None, "").await;
    show(&app, "GET", "/shows/abc", None, "").await;
    show(&app, "POST", "/shows", json, "{").await;
    show(&app, "POST", "/shows", json, "{}").await;
    show(&app, "POST", "/shows", None, "{}").await;
    show(&app, "GET", "/nope", None, "").await;
    show(&app, "DELETE", "/shows/1", None, "").await;
    show(&app, "GET", "/boom", None, "").await;
}
