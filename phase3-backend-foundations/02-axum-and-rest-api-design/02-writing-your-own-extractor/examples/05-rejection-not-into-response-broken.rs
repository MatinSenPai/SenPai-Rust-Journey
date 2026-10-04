//! DELIBERATELY BROKEN — expected: E0277
//!
//! A rejection is sent back to the client as the response, so its type has
//! to implement `IntoResponse`. This one is a plain enum that doesn't.
//!
//!     cargo build -p p3-02-02-writing-your-own-extractor --example 05-rejection-not-into-response-broken --features broken

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[derive(Debug)]
enum UserAgentError {
    Missing,
    Unreadable,
}

struct UserAgent(String);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = UserAgentError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("user-agent")
            .ok_or(UserAgentError::Missing)?;
        let text = value.to_str().map_err(|_| UserAgentError::Unreadable)?;
        Ok(UserAgent(text.to_string()))
    }
}

fn main() {}
