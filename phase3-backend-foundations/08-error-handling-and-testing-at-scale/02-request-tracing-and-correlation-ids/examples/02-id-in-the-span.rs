//! The same two concurrent requests, now through a `from_fn` middleware that
//! gives each one an id and opens a span carrying it. A last request then shows
//! the cost of trusting the caller's id without checking it.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 02-id-in-the-span`

use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;
use tracing::Instrument;

async fn with_request_id(request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("none")
        .to_string();
    let span = tracing::info_span!("request", request_id = %id);
    let mut response = next.run(request).instrument(span).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&id).unwrap());
    response
}

async fn slow_lookup() -> &'static str {
    tracing::info!("loading from the database");
    tokio::time::sleep(Duration::from_millis(20)).await;
    tracing::warn!("the query was slow");
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    let app = Router::new()
        .route("/anime", get(slow_lookup))
        .layer(from_fn(with_request_id));
    let request = |id: &str| {
        Request::builder()
            .uri("/anime")
            .header("x-request-id", id)
            .body(Body::empty())
            .unwrap()
    };

    let (a, b) = tokio::join!(
        app.clone().oneshot(request("req-a")),
        app.clone().oneshot(request("req-b"))
    );
    println!(
        "echoed: {:?} {:?}",
        a.unwrap().headers()["x-request-id"],
        b.unwrap().headers()["x-request-id"]
    );

    // This middleware trusts whatever the caller sent. Watch what that costs.
    println!("--- a caller who sends a crafted id");
    app.oneshot(request("x level=ERROR forged=true"))
        .await
        .unwrap();
}
