//! An extractor built from another one. `SearchTerm` asks axum's own `Query`
//! to parse the query string, then adds its own rule on top: the `q`
//! parameter must be there and must not be blank.
//!
//!     cargo run -p p3-02-02-writing-your-own-extractor --example 03-composing-query

use axum::body::{to_bytes, Body};
use axum::extract::{FromRequestParts, Query};
use axum::http::{request::Parts, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

struct SearchTerm(String);

impl<S: Send + Sync> FromRequestParts<S> for SearchTerm {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<SearchParams>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid query string"))?;
        let term = params.q.unwrap_or_default().trim().to_string();
        if term.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "missing search term: add ?q=..."));
        }
        Ok(SearchTerm(term))
    }
}

async fn search(SearchTerm(term): SearchTerm) -> String {
    format!("searching for {term:?}")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/search", get(search));

    for uri in [
        "/search?q=frieren",
        "/search?q=%20%20",
        "/search",
        "/search?q=a&q=b",
    ] {
        let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        println!("{uri:<20} -> {status}: {}", String::from_utf8_lossy(&bytes));
    }
}
