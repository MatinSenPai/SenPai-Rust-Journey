//! What `axum` sends back when an extractor refuses a request — before the
//! handler body ever runs. Compare each line with 3.1.2's `HttpParseError`
//! variants and 3.1.3's `400`-vs-`422` rule.
//!
//!     cargo run -p p3-02-01-routing-handlers-extractors --example 02-what-axum-answers-for-you

use axum::body::{to_bytes, Body};
use axum::extract::Path;
use axum::http::{header, Request};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Deserialize)]
struct EchoRequest {
    message: String,
}

async fn echo(Json(payload): Json<EchoRequest>) -> String {
    format!("echo: {}", payload.message)
}

async fn anime(Path(id): Path<u32>) -> String {
    format!("anime #{id}")
}

async fn send(app: &Router, method: &str, uri: &str, json: bool, body: &'static str) {
    let mut request = Request::builder().method(method).uri(uri);
    if json {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    let request = request.body(Body::from(body)).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let note = if method == "POST" && !json {
        "   (no Content-Type header)"
    } else {
        ""
    };
    let line = format!("{method} {uri} {body}{note}");
    println!("{}", line.trim_end());
    println!("    -> {status}: {}", String::from_utf8_lossy(&bytes));
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/echo", post(echo))
        .route("/anime/{id}", get(anime));

    send(&app, "POST", "/echo", true, r#"{"message":"hi"}"#).await;
    send(&app, "POST", "/echo", true, "not json").await;
    send(&app, "POST", "/echo", true, r#"{"msg":"hi"}"#).await;
    send(&app, "POST", "/echo", true, r#"{"message":42}"#).await;
    send(&app, "POST", "/echo", false, r#"{"message":"hi"}"#).await;
    send(&app, "GET", "/anime/7", false, "").await;
    send(&app, "GET", "/anime/seven", false, "").await;
}
