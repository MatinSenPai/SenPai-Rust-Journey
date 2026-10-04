//! DELIBERATELY BROKEN — expected: E0277
//!
//! In axum 0.7, `Option<T>` worked for every extractor `T`. In axum 0.8 it
//! needs `T` to implement a second trait, `OptionalFromRequestParts`, and
//! `UserAgent` only implements `FromRequestParts`.
//!
//!     cargo build -p p3-02-02-writing-your-own-extractor --example 06-option-extractor-broken --features broken

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::routing::get;
use axum::Router;

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

async fn greet(agent: Option<UserAgent>) -> String {
    match agent {
        Some(UserAgent(agent)) => format!("hello, {agent}"),
        None => "hello, whoever you are".to_string(),
    }
}

fn main() {
    let _app: Router = Router::new().route("/", get(greet));
}
