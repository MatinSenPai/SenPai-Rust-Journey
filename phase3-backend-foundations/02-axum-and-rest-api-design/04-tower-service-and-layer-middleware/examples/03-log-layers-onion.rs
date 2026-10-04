//! Two layers, added with repeated `.layer(...)` calls: the LAST one added
//! is the outermost, so it sees the request first.
//!
//!     cargo run -p p3-02-04-tower-service-and-layer-middleware --example 03-log-layers-onion

use axum::body::Body;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use p3_02_04_tower_service_and_layer_middleware::LogLayer;
use tower::ServiceExt;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "hello" }))
        .layer(LogLayer::new("A"))
        .layer(LogLayer::new("B"));

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    app.oneshot(request).await.unwrap();
}
