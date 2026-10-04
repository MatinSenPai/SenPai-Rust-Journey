//! A silent logic bug: a handler asks for `Extension<RequestId>`, but the router
//! was built without the middleware that inserts it. Compiles; every request
//! fails at run time with a 500.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 08-extension-without-middleware`

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use p3_08_02_request_tracing_and_correlation_ids::routes;
use tower::ServiceExt;

#[tokio::main]
async fn main() {
    // `routes()` is the app with no middleware; `app()` would add it.
    let request = Request::builder()
        .uri("/whoami")
        .body(Body::empty())
        .unwrap();
    let response = routes().oneshot(request).await.unwrap();
    println!("status: {}", response.status());
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("body: {}", String::from_utf8_lossy(&bytes));
}
