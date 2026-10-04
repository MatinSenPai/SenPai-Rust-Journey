use std::sync::Arc;

use p3_03_02_validation_solution::{app, ReviewStore};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:3111";
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    println!("reviews listening on {addr}");
    axum::serve(listener, app(Arc::new(ReviewStore::default())))
        .await
        .expect("server error");
}
