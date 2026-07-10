//! The binary the `Dockerfile` in this directory actually builds and runs.
//! Not `todo!()`-gated — unlike `health`/`app` in `src/lib.rs`, there's no
//! exercise here, this is just enough plumbing to have a real process that
//! listens on a port, so the Dockerfile has something concrete to package.

use p4_07_01_docker_compose_and_ci::app;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind port 3000");

    println!("listening on http://{addr}");

    axum::serve(listener, app())
        .await
        .expect("server exited unexpectedly");
}
