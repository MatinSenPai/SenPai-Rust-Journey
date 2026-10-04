// Provided for you — no `todo!()`s here, only in `lib.rs`.
use std::sync::Arc;

use p3_03_04_api_versioning_and_evolution::{app, AnimeStore};

#[tokio::main]
async fn main() {
    let router = app(Arc::new(AnimeStore::seeded()));

    let addr = "127.0.0.1:3130";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("versioned anime API listening on {addr} — try: curl -si http://{addr}/v1/anime");

    axum::serve(listener, router).await.expect("server error");
}
