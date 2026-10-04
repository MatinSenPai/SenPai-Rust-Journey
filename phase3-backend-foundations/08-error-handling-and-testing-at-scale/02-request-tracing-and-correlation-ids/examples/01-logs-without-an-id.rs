//! Two requests are in flight at once and both log. Without an id on each line
//! you cannot tell whose line is whose.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 01-logs-without-an-id`

use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

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

    let app = Router::new().route("/anime", get(slow_lookup));
    let request = || {
        Request::builder()
            .uri("/anime")
            .body(Body::empty())
            .unwrap()
    };

    let (a, b) = tokio::join!(
        app.clone().oneshot(request()),
        app.clone().oneshot(request())
    );
    println!("{} and {}", a.unwrap().status(), b.unwrap().status());
}
