// Provided for you — no `todo!()`s here, only in `lib.rs`.
use std::sync::Arc;

use p3_07_01_consistent_error_envelopes::{app, WidgetStore};

#[tokio::main]
async fn main() {
    let store = Arc::new(WidgetStore::default());
    let router = app(store);

    let addr = "127.0.0.1:3002";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("widget API listening on {addr} — try: curl http://{addr}/widgets/1");

    axum::serve(listener, router).await.expect("server error");
}
