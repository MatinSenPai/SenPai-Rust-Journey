//! The ready-made version: `SetRequestIdLayer`, `TraceLayer` and
//! `PropagateRequestIdLayer` from `tower-http`. Note what `SetRequestIdLayer`
//! does with an id the caller already sent: nothing, good or bad.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 05-ready-made-layers`

use axum::body::Body;
use axum::extract::Request;
use axum::http::Response;
use axum::routing::get;
use axum::Router;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

fn app() -> Router {
    let trace = TraceLayer::new_for_http()
        .make_span_with(|request: &Request| {
            let id = request.headers().get("x-request-id").and_then(|v| v.to_str().ok());
            tracing::span!(Level::INFO, "request", request_id = id.unwrap_or("-"), path = %request.uri().path())
        })
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));
    Router::new().route("/anime", get(|| async { "ok" })).layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(trace)
            .layer(PropagateRequestIdLayer::x_request_id()),
    )
}

async fn send(id: Option<&str>) -> Response<Body> {
    let mut builder = Request::builder().uri("/anime");
    if let Some(id) = id {
        builder = builder.header("x-request-id", id);
    }
    app()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .init();

    for id in [Some("abc-123"), None, Some("not valid!")] {
        let response = send(id).await;
        println!(
            "sent {:?} -> echoed {:?}",
            id,
            response.headers()["x-request-id"]
        );
    }
}
