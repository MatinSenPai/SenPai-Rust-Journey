//! Whatever a handler returns, axum calls `.into_response()` on it. This
//! program makes that call by hand for the three shapes this lesson's
//! handlers return, and prints each response the way 3.1.2 put one on the
//! wire: status line, headers, blank line, body.
//!
//!     cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 01-what-a-handler-return-becomes

use axum::body::to_bytes;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

async fn show(label: &str, response: Response) {
    println!("--- {label}");
    println!("HTTP/1.1 {}", response.status());
    for (name, value) in response.headers() {
        println!("{name}: {}", value.to_str().unwrap());
    }
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!();
    println!("{}", String::from_utf8_lossy(&body));
}

#[tokio::main]
async fn main() {
    let anime = json!({"id": 1, "title": "Frieren"});

    show("Json(anime)", Json(anime.clone()).into_response()).await;

    let created = (
        StatusCode::CREATED,
        [(header::LOCATION, "/anime/1".to_string())],
        Json(anime),
    );
    show(
        "(CREATED, [(LOCATION, ..)], Json(anime))",
        created.into_response(),
    )
    .await;

    show(
        "StatusCode::NO_CONTENT",
        StatusCode::NO_CONTENT.into_response(),
    )
    .await;
}
