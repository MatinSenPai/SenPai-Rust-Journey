//! A `Router` is a `tower::Service`: `.oneshot(req)` is just
//! `poll_ready` followed by `call`, written out.
//!
//!     cargo run -p p3-02-04-tower-service-and-layer-middleware --example 01-router-is-a-service

use axum::body::Body;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() {
    let mut app = Router::new().route("/", get(|| async { "hello" }));

    // `ready()` waits until poll_ready says Ready, then hands the service back.
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let ready = ServiceExt::<Request<Body>>::ready(&mut app);
    let response = ready.await.unwrap().call(request).await.unwrap();
    println!("by hand:  {}", response.status());

    // `oneshot` is the same two steps in one call.
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    println!("oneshot:  {}", response.status());
}
