//! A request is two things: `Parts` (method, URI, headers: plain data you
//! can look at as often as you like) and a `Body` (a stream of bytes you can
//! read exactly once).
//!
//!     cargo run -p p3-02-02-writing-your-own-extractor --example 01-parts-and-body

use axum::body::{to_bytes, Body};
use axum::http::Request;

#[tokio::main]
async fn main() {
    let request = Request::builder()
        .method("POST")
        .uri("/anime?page=2")
        .header("x-api-key", "secret-123")
        .body(Body::from(r#"{"title":"Frieren"}"#))
        .unwrap();

    let (parts, body) = request.into_parts();

    // `Parts` is plain data: read it as many times as you like.
    println!("method:        {}", parts.method);
    println!("path:          {}", parts.uri.path());
    println!("query:         {:?}", parts.uri.query());
    println!("api key:       {:?}", parts.headers.get("x-api-key"));
    println!("api key again: {:?}", parts.headers.get("x-api-key"));

    // The body is a stream: reading it takes it by value, and consumes it.
    let bytes = to_bytes(body, 1024).await.unwrap();
    println!("body:          {}", String::from_utf8_lossy(&bytes));
}
