// Provided for you — no `todo!()`s here, only in `lib.rs`. Runs the API
// with the permissive dev layer so you can poke at preflights with curl
// (see "Try it for real" in README.md).
use p3_02_03_cors_and_frontend_integration::{app, dev_cors};

#[tokio::main]
async fn main() {
    let router = app(dev_cors());

    let addr = "127.0.0.1:3002";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("anime API with CORS listening on {addr} — try the preflight curl from README.md");

    axum::serve(listener, router).await.expect("server error");
}
