//! The problem: one API, several error shapes. Nothing here has an envelope.
//! Run: `cargo run -p p3-08-01-consistent-error-envelopes --example 01-four-shapes`

use axum::body::{to_bytes, Body};
use axum::extract::Path;
use axum::http::{header, Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower::ServiceExt;

#[derive(Deserialize)]
struct NewShow {
    #[allow(dead_code)]
    title: String,
}

async fn create(Json(_): Json<NewShow>) -> StatusCode {
    StatusCode::CREATED
}

async fn one(Path(id): Path<u64>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({"error": format!("show {id} not found")})),
    ))
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
    let kind = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map_or("-", |v| v.to_str().unwrap())
        .to_string();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8_lossy(&bytes);
    let text = if text.is_empty() {
        "(empty body)"
    } else {
        &text
    };
    println!(
        "{method} {uri} -> {status} [{kind}]
    {text}"
    );
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/shows", post(create))
        .route("/shows/{id}", get(one));
    let json = Some("application/json");
    show(&app, "GET", "/shows/9", None, "").await;
    show(&app, "GET", "/shows/abc", None, "").await;
    show(&app, "POST", "/shows", json, "{").await;
    show(&app, "POST", "/shows", json, "{}").await;
    show(&app, "POST", "/shows", None, "{}").await;
    show(&app, "GET", "/nope", None, "").await;
    show(&app, "DELETE", "/shows/1", None, "").await;
}
