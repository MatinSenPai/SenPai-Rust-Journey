//! DELIBERATELY BROKEN — expected: "future cannot be sent between threads safely" (this error has no code)
//!
//! `FromRequestParts` promises a future that is `Send`. An `async fn` keeps
//! every argument alive inside its future, including `_state: &S`, and a
//! `&S` can only move to another thread if `S: Sync`. This `impl<S>` makes
//! no promise about `S` at all.
//!
//!     cargo build -p p3-02-02-writing-your-own-extractor --example 07-missing-send-sync-bound-broken --features broken

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};

struct UserAgent(String);

impl<S> FromRequestParts<S> for UserAgent {
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

fn main() {}
