//! The same kind of router, this time on a real socket. Runs until you stop
//! it with Ctrl+C; talk to it with `curl` from a second terminal.
//!
//!     cargo run -p p3-02-01-routing-handlers-extractors --example 05-serve-on-a-real-port

use axum::extract::Path;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ShoutRequest {
    message: String,
}

#[derive(Serialize)]
struct ShoutResponse {
    shouted: String,
}

async fn hello() -> &'static str {
    "Hello, world!"
}

async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}

async fn shout(Json(payload): Json<ShoutRequest>) -> Json<ShoutResponse> {
    Json(ShoutResponse {
        shouted: payload.message.to_uppercase(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/shout", post(shout));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3021")
        .await
        .expect("port 3021 should be free");
    println!("listening on http://127.0.0.1:3021 (Ctrl+C to stop)");
    axum::serve(listener, app).await.expect("server error");
}
