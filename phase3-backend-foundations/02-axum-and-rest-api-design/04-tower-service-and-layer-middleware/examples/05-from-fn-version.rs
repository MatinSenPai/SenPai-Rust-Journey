//! The same job as `Log`, written with `axum::middleware::from_fn`.
//!
//!     cargo run -p p3-02-04-tower-service-and-layer-middleware --example 05-from-fn-version

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

async fn log(request: Request, next: Next) -> Response {
    println!("fn: request in  ({} {})", request.method(), request.uri());
    let response = next.run(request).await;
    println!("fn: response out ({})", response.status());
    response
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "hello" }))
        .layer(from_fn(log));

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    app.oneshot(request).await.unwrap();
}
