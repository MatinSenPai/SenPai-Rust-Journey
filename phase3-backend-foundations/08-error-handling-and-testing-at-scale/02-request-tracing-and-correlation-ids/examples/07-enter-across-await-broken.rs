//! DELIBERATELY BROKEN — expected: E0277
//! `span.entered()` held across an `.await` makes the middleware's future not
//! `Send`, so `axum` refuses the layer.
//!
//! Run: `cargo build -p p3-08-02-request-tracing-and-correlation-ids --example 07-enter-across-await-broken --features broken`

use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;

async fn with_request_id(request: Request, next: Next) -> Response {
    let span = tracing::info_span!("request", request_id = "abc-123");
    let _guard = span.entered();
    next.run(request).await
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(from_fn(with_request_id));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3231")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
