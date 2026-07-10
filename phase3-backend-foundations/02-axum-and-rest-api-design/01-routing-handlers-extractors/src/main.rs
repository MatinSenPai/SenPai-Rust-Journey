// Provided for you — no `todo!()`s here, only in `lib.rs`. This is the
// "I/O at the edges" shell: build the shared state, build the router, bind
// a real socket, serve forever.
use p3_02_01_routing_handlers_extractors::{app, AppState};

#[tokio::main]
async fn main() {
    let state = AppState::default();
    let router = app(state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("axum server listening on {addr} — try: curl http://{addr}/");

    axum::serve(listener, router).await.expect("server error");
}
