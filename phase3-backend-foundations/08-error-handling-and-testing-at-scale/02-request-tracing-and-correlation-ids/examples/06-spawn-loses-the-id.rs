//! A silent logic bug: `tokio::spawn` starts a new task, and a new task does not
//! inherit the span of the code that spawned it. Compiles, runs, and the second
//! line is missing its request id.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 06-spawn-loses-the-id`

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;
use tracing::Instrument;

async fn with_request_id(request: Request, next: Next) -> Response {
    let span = tracing::info_span!("request", request_id = "abc-123");
    next.run(request).instrument(span).await
}

async fn send_email() -> &'static str {
    tracing::info!("handler: queueing the welcome email");
    let worker = tokio::spawn(async {
        tracing::info!("worker: sending the welcome email");
    });
    worker.await.unwrap();
    "queued"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    let app = Router::new()
        .route("/welcome", get(send_email))
        .layer(from_fn(with_request_id));
    let request = Request::builder()
        .uri("/welcome")
        .body(Body::empty())
        .unwrap();
    app.oneshot(request).await.unwrap();
}
