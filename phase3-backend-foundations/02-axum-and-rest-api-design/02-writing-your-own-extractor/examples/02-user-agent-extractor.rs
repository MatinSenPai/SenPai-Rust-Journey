//! Your first extractor: `UserAgent`, built from the `user-agent` header.
//! The handler only names it in its signature; when the header is missing,
//! the extractor rejects the request and the handler never runs.
//!
//!     cargo run -p p3-02-02-writing-your-own-extractor --example 02-user-agent-extractor

use axum::body::{to_bytes, Body};
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

struct UserAgent(String);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let rejection = (
            StatusCode::BAD_REQUEST,
            "missing or unreadable user-agent header",
        );
        let value = parts.headers.get("user-agent").ok_or(rejection)?;
        let text = value.to_str().map_err(|_| rejection)?;
        Ok(UserAgent(text.to_string()))
    }
}

async fn hello(UserAgent(agent): UserAgent) -> String {
    println!("  (the handler ran)");
    format!("hello, {agent}")
}

async fn send(app: &Router, label: &str, request: Request<Body>) {
    println!("{label}");
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    println!("  -> {status}: {}", String::from_utf8_lossy(&bytes));
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));

    let with_header = Request::builder()
        .uri("/")
        .header("user-agent", "curl/8.9.1")
        .body(Body::empty())
        .unwrap();
    send(&app, "GET / with a user-agent header", with_header).await;

    let without_header = Request::builder().uri("/").body(Body::empty()).unwrap();
    send(&app, "GET / without one", without_header).await;
}
