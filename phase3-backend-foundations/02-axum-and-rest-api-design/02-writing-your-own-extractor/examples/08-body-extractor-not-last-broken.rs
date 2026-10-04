//! DELIBERATELY BROKEN — expected: E0277
//!
//! `String` reads the request body, so it implements `FromRequest`, and only
//! the last argument of a handler may do that. Here it comes first.
//!
//!     cargo build -p p3-02-02-writing-your-own-extractor --example 08-body-extractor-not-last-broken --features broken

use axum::http::Method;
use axum::routing::post;
use axum::Router;

async fn upload(body: String, method: Method) -> String {
    format!("{method} with {} bytes", body.len())
}

fn main() {
    let _app: Router = Router::new().route("/", post(upload));
}
