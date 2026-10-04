//! The real API plus `GET /api-docs/openapi.json`, on 127.0.0.1:3120.
//! Try: `curl -s http://127.0.0.1:3120/api-docs/openapi.json`

use std::sync::Arc;

use p3_03_03_api_contracts_and_openapi::{app, Catalog};

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3120")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app(Arc::new(Catalog::default())))
        .await
        .unwrap();
}
