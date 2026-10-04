// Provided for you — no `todo!()`s here, only in `lib.rs`.
//
// This is the one place the real dependencies are chosen: the real clock,
// the default settings, a new empty store. Every test chooses a fake clock
// instead, and the handlers cannot tell the difference.
use std::sync::Arc;

use p3_04_02_app_state_and_dependency_wiring::{app, AppState, Config, SystemClock};

#[tokio::main]
async fn main() {
    let state = AppState::new(Arc::new(SystemClock), Config::default());
    let router = app(state);

    let addr = "127.0.0.1:3150";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("watch log listening on {addr} — try: curl http://{addr}/watch/recent");

    axum::serve(listener, router).await.expect("server error");
}
