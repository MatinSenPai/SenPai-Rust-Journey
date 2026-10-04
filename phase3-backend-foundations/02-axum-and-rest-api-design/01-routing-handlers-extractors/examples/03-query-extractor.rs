//! `Query<T>`: the query string 3.1.2 split by hand, decoded into a struct.
//!
//!     cargo run -p p3-02-01-routing-handlers-extractors --example 03-query-extractor

use axum::body::{to_bytes, Body};
use axum::extract::Query;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Debug, Deserialize)]
struct AnimeFilter {
    status: String,
    page: Option<u32>,
}

async fn list_anime(Query(filter): Query<AnimeFilter>) -> String {
    format!("{filter:?}")
}

async fn send(app: &Router, uri: &str) {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("GET {uri}");
    println!("    -> {status}: {}", String::from_utf8_lossy(&bytes));
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/anime", get(list_anime));

    send(&app, "/anime?status=watching").await;
    send(&app, "/anime?status=watching&page=2").await;
    send(&app, "/anime?status=plan%20to%20watch").await;
    send(&app, "/anime?page=2").await;
    send(&app, "/anime?status=watching&page=two").await;
}
