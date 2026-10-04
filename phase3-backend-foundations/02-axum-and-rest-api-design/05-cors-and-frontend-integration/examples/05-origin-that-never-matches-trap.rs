//! No compiler error, no panic, no log line — and no browser request ever
//! gets through. Both allowed origins below were typed into a config file by
//! hand, and neither is what a browser actually sends in its `Origin` header:
//! one has a trailing slash, the other the wrong scheme. The policy compares
//! bytes, so it never matches.
//!
//!     cargo run -p p3-02-05-cors-and-frontend-integration --example 05-origin-that-never-matches-trap

use axum::body::Body;
use axum::http::{HeaderValue, Request};
use tower::ServiceExt;
use tower_http::cors::{AllowOrigin, CorsLayer};

use p3_02_05_cors_and_frontend_integration::app;

// What a page served from https://anime.example.com/ sends, every time.
const BROWSER_SENDS: &str = "https://anime.example.com";

#[tokio::main]
async fn main() {
    for configured in ["https://anime.example.com/", "http://anime.example.com"] {
        let allowed = AllowOrigin::list([HeaderValue::from_static(configured)]);
        let cors = CorsLayer::new().allow_origin(allowed);

        let request = Request::builder()
            .uri("/anime")
            .header("origin", BROWSER_SENDS)
            .body(Body::empty())
            .unwrap();
        let response = app(cors).oneshot(request).await.unwrap();

        let allow = response.headers().get("access-control-allow-origin");
        println!("allowed {configured:?}, Origin {BROWSER_SENDS:?}");
        println!(
            "  -> {}, access-control-allow-origin: {allow:?}",
            response.status()
        );
    }
}
