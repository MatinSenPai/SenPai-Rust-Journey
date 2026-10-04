//! No compiler error, no panic: this create handler returns a bare `Json`,
//! so the answer is `200 OK` where a `POST` that makes something should say
//! `201 Created`. It compiles, runs, and returns the right body, and is
//! still wrong. Only a test that checks the status code catches it.
//!
//!     cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 05-created-answers-200-trap

use axum::body::Body;
use axum::http::Request;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn create() -> Json<Value> {
    Json(json!({"id": 1, "title": "Frieren"})) // bug: should be (CREATED, ..)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/anime", post(create));
    let request = Request::post("/anime").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    println!(
        "POST /anime -> {}  (should be 201 Created)",
        response.status()
    );
}
