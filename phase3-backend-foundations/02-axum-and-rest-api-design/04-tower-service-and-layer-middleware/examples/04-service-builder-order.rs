//! The same two layers through `ServiceBuilder`: it reads top to bottom, so
//! the FIRST layer listed is the outermost.
//!
//!     cargo run -p p3-02-04-tower-service-and-layer-middleware --example 04-service-builder-order

use axum::body::Body;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use p3_02_04_tower_service_and_layer_middleware::LogLayer;
use tower::{ServiceBuilder, ServiceExt};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "hello" })).layer(
        ServiceBuilder::new()
            .layer(LogLayer::new("A"))
            .layer(LogLayer::new("B")),
    );

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    app.oneshot(request).await.unwrap();
}
