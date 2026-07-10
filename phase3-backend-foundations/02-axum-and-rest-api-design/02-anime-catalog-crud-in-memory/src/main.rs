// Provided for you — no `todo!()`s here, only in `lib.rs`.
use std::sync::Arc;

use p3_02_02_anime_catalog_crud_in_memory::{app, AnimeStore};

#[tokio::main]
async fn main() {
    let store = Arc::new(AnimeStore::default());
    let router = app(store);

    let addr = "127.0.0.1:3001";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("anime catalog listening on {addr} — try: curl http://{addr}/anime");

    axum::serve(listener, router).await.expect("server error");
}
