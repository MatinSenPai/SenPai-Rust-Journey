//! A `CorsLayer` answers a preflight by itself. The `OPTIONS` request never
//! reaches the router, so the handler behind it never runs: the counter
//! stays at zero until the browser sends the real request.
//!
//!     cargo run -p p3-02-05-cors-and-frontend-integration --example 02-preflight-never-reaches-the-handler

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{header, HeaderValue, Method, Request, StatusCode};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use tower::ServiceExt;
use tower_http::cors::{AllowOrigin, CorsLayer};

async fn create_anime(State(hits): State<Arc<AtomicUsize>>) -> StatusCode {
    hits.fetch_add(1, Ordering::SeqCst);
    StatusCode::CREATED
}

fn show(response: &Response) {
    for name in [
        "access-control-allow-origin",
        "access-control-allow-methods",
        "access-control-allow-headers",
    ] {
        if let Some(value) = response.headers().get(name) {
            println!("  {name}: {value:?}");
        }
    }
}

#[tokio::main]
async fn main() {
    let hits = Arc::new(AtomicUsize::new(0));
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([HeaderValue::from_static(
            "http://localhost:5173",
        )]))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);
    let app = Router::new()
        .route("/anime", post(create_anime))
        .with_state(hits.clone())
        .layer(cors);

    // 1. The preflight: what the browser sends before a JSON POST.
    let preflight = Request::builder()
        .method("OPTIONS")
        .uri("/anime")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(preflight).await.unwrap();
    println!("preflight -> {}", response.status());
    show(&response);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("  body: {} bytes", body.len());
    println!("  handler ran {} times", hits.load(Ordering::SeqCst));

    // 2. The real request, sent only because the preflight vouched for it.
    let real = Request::builder()
        .method("POST")
        .uri("/anime")
        .header("origin", "http://localhost:5173")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"title":"Frieren"}"#))
        .unwrap();
    let response = app.oneshot(real).await.unwrap();
    println!("real POST -> {}", response.status());
    show(&response);
    println!("  handler ran {} times", hits.load(Ordering::SeqCst));
}
