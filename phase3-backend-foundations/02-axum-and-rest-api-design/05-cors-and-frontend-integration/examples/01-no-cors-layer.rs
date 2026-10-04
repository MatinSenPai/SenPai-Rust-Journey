//! The bug that isn't a bug. A router with no CORS layer at all answers a
//! cross-origin `GET` perfectly well: status, body, everything. The only
//! thing missing is the header a browser needs before it hands the response
//! to the page. And the preflight a browser sends before a JSON `POST` gets
//! a `405`, because nobody ever wrote an `OPTIONS` handler.
//!
//!     cargo run -p p3-02-05-cors-and-frontend-integration --example 01-no-cors-layer

use axum::body::{to_bytes, Body};
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

async fn list_anime() -> &'static str {
    r#"[{"id":1,"title":"Frieren"}]"#
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/anime", get(list_anime));

    // What a page served from http://localhost:5173 sends when it calls fetch().
    let get_request = Request::builder()
        .uri("/anime")
        .header("origin", "http://localhost:5173")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(get_request).await.unwrap();
    let allow_origin = response.headers().get("access-control-allow-origin");
    println!("GET /anime -> {}", response.status());
    println!("  access-control-allow-origin: {allow_origin:?}");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("  body: {}", String::from_utf8_lossy(&body));

    // What the same page sends first, before a JSON POST.
    let preflight = Request::builder()
        .method("OPTIONS")
        .uri("/anime")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(preflight).await.unwrap();
    println!("OPTIONS /anime -> {}", response.status());
}
