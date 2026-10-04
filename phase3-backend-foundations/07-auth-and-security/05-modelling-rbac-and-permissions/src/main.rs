//! The review API on `127.0.0.1:3210`, seeded with five users (see
//! `AppState::seeded`). Try it after the Implement and Build rungs:
//!
//!     cargo run -p p3-07-05-modelling-rbac-and-permissions

use p3_07_05_modelling_rbac_and_permissions::{app, AppState};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:3210";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on http://{addr}");
    axum::serve(listener, app(AppState::seeded()))
        .await
        .unwrap();
}
