// Provided for you — no `todo!()`s here, only in `lib.rs`.
use std::sync::Arc;

use p3_08_01_consistent_error_envelopes::{app, ShowStore};

#[tokio::main]
async fn main() {
    let router = app(Arc::new(ShowStore::default()));

    let addr = "127.0.0.1:3220";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("show API listening on {addr}");

    axum::serve(listener, router).await.expect("server error");
}
