//! Making an extractor optional without touching it: ask for
//! `Result<T, T::Rejection>` instead of `T`. The extractor still runs, but
//! its rejection now comes to the handler as an `Err` instead of going
//! straight back to the client.
//!
//!     cargo run -p p3-02-02-writing-your-own-extractor --example 04-optional-with-result

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

async fn greet(agent: Result<UserAgent, (StatusCode, &'static str)>) -> String {
    match agent {
        Ok(UserAgent(agent)) => format!("hello, {agent}"),
        Err((_, reason)) => format!("hello, whoever you are ({reason})"),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(greet));

    let requests = [
        Request::builder()
            .uri("/")
            .header("user-agent", "curl/8.9.1")
            .body(Body::empty())
            .unwrap(),
        Request::builder().uri("/").body(Body::empty()).unwrap(),
    ];
    for request in requests {
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        println!("{status}: {}", String::from_utf8_lossy(&bytes));
    }
}
